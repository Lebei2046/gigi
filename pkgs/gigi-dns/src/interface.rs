// Copyright 2024 Gigi Team.
//
// Per-interface mDNS handler for gigi-dns
//
// This module implements per-interface mDNS communication for the Gigi DNS protocol.
// Each network interface gets its own background task that handles:
// - Sending periodic queries for peer discovery
// - Responding to queries from other peers
// - Processing responses and extracting peer information
// - Adaptive probing for faster peer discovery
// - Rate limiting to prevent DoS attacks
//
// Key features:
// - Separate UDP sockets for sending and receiving
// - Multi-segment TXT record support (RFC 1035)
// - Adaptive query intervals (exponential backoff)
// - Query response rate limiting (10 per second)
// - Graceful cleanup on interface down events

use crate::protocol::GigiDnsProtocol;
use crate::types::*;
use if_watch::IfEvent;
use std::collections::VecDeque;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Adaptive probing state for faster peer discovery
///
/// When no peers are discovered, the interface uses shorter query intervals
/// to rapidly probe the network. Once a peer is found, it switches to the
/// normal (longer) query interval to reduce network traffic.
#[derive(Debug, Clone, Copy)]
enum ProbeState {
    /// Currently probing with the given interval
    Probing(Duration),
    /// Probing complete, use normal interval
    Finished,
}

/// Events sent from interface tasks back to the main behaviour
///
/// These events inform the main behaviour about peer lifecycle changes.
#[derive(Debug)]
pub enum InterfaceEvent {
    /// A new peer was discovered
    PeerDiscovered(GigiDnsEvent),
    /// A peer's information expired
    PeerExpired(GigiDnsEvent),
}

impl From<InterfaceEvent> for GigiDnsEvent {
    fn from(event: InterfaceEvent) -> Self {
        match event {
            InterfaceEvent::PeerDiscovered(ev) => ev,
            InterfaceEvent::PeerExpired(ev) => ev,
        }
    }
}

/// Per-interface state that runs as a background task
///
/// Each InterfaceTask manages DNS communication on a single network interface.
/// It runs in its own async task and communicates with the main behaviour via channels.
pub struct InterfaceTask {
    /// Configuration for this interface
    config: GigiDnsConfig,
    /// IP address of the interface
    interface_ip: IpAddr,
    /// DNS protocol handler
    protocol: GigiDnsProtocol,
    /// Channel for sending events to main behaviour
    event_tx: UnboundedSender<InterfaceEvent>,
    /// Channel for receiving packets from I/O task
    multicast_rx: UnboundedReceiver<InterfacePacket>,
    /// Channel for sending packets via I/O task
    send_tx: UnboundedSender<(Vec<u8>, SocketAddr)>,
    /// Deadline for next query
    query_deadline: Instant,
    /// Deadline for next announcement
    announce_deadline: Instant,
    /// Deadline for next cleanup
    cleanup_deadline: Instant,
    /// Current adaptive probing state
    probe_state: ProbeState,
    /// Whether we've discovered any peers yet
    has_discovered_peers: bool,
    /// Whether this is the first run (send immediate query/announcement)
    first_run: bool,
    /// Handle for background I/O task
    _io_handle: tokio::task::JoinHandle<()>,
    /// Channel to receive address updates from main behaviour
    address_update_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<libp2p::Multiaddr>>,
    /// Recent query response timestamps for rate limiting
    recent_query_responses: std::collections::VecDeque<std::time::Instant>,
}

/// Internal packet type for communication between I/O task and main task
pub enum InterfacePacket {
    /// Received packet with source address
    Received(Vec<u8>, SocketAddr),
}

impl InterfaceTask {
    /// Spawns a new interface task for the given IP address
    ///
    /// Creates separate UDP sockets for sending and receiving, spawns a background
    /// I/O task, and starts the main async task that handles protocol logic.
    ///
    /// # Arguments
    /// * `interface_ip` - IP address of the network interface
    /// * `config` - Configuration for DNS behavior
    /// * `local_peer_id` - Our libp2p peer ID
    /// * `event_tx` - Channel for sending events to main behaviour
    /// * `address_update_rx` - Channel for receiving address updates from main behaviour
    ///
    /// # Returns
    /// - `Ok(JoinHandle<()>)` - Handle for the spawned task
    /// - `Err(std::io::Error)` - Failed to create sockets
    pub fn spawn(
        interface_ip: IpAddr,
        config: GigiDnsConfig,
        local_peer_id: libp2p_identity::PeerId,
        event_tx: UnboundedSender<InterfaceEvent>,
        address_update_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<libp2p::Multiaddr>>,
    ) -> std::io::Result<tokio::task::JoinHandle<()>> {
        // Create receive socket bound to this interface
        let recv_socket = Self::create_recv_socket(&interface_ip, &config)?;
        let recv_local = recv_socket.local_addr()?;
        tracing::debug!(
            "Interface {} recv socket bound to {}",
            interface_ip,
            recv_local
        );

        // Create send socket
        let send_socket = Self::create_send_socket(&interface_ip, &config)?;
        let send_local = send_socket.local_addr()?;
        tracing::debug!(
            "Interface {} send socket bound to {}",
            interface_ip,
            send_local
        );

        // Create channels
        let (packet_tx, packet_rx) = unbounded_channel();
        let (send_tx, mut send_rx) = unbounded_channel::<(Vec<u8>, SocketAddr)>();

        // Spawn background I/O task with adaptive buffer size
        let io_handle = tokio::spawn(async move {
            // Start with reasonable buffer size to avoid truncation
            const INITIAL_BUFFER_SIZE: usize = 4096; // 4KB to accommodate typical DNS packets
            const MAX_BUFFER_SIZE: usize = 65536; // 64KB max (UDP packet limit)
            let mut buffer = vec![0u8; INITIAL_BUFFER_SIZE];
            tracing::debug!(
                "Interface {} I/O task started with buffer size {}",
                interface_ip,
                buffer.len()
            );

            loop {
                tokio::select! {
                    result = recv_socket.recv_from(&mut buffer) => {
                        match result {
                            Ok((len, src)) => {
                                // Check if buffer was too small (packet truncated)
                                if len == buffer.len() && buffer.len() < MAX_BUFFER_SIZE {
                                    tracing::warn!(
                                        "Interface {} buffer too small ({}), packet may be truncated. Growing to {}",
                                        interface_ip,
                                        buffer.len(),
                                        buffer.len() * 2
                                    );
                                    let new_size = (buffer.len() * 2).min(MAX_BUFFER_SIZE);
                                    buffer.resize(new_size, 0);
                                    // Drop this truncated packet and continue
                                    continue;
                                }
                                let packet = buffer[..len].to_vec();
                                let _ = packet_tx.send(InterfacePacket::Received(packet, src));
                            }
                            Err(e) => {
                                tracing::error!("Interface {} recv error: {}", interface_ip, e);
                            }
                        }
                    }
                    Some((data, addr)) = send_rx.recv() => {
                        match send_socket.send_to(&data, addr).await {
                            Ok(n) => {
                                tracing::debug!("Interface {} sent {} bytes to {}", interface_ip, n, addr);
                            }
                            Err(e) => {
                                tracing::error!("Interface {} send error: {}", interface_ip, e);
                            }
                        }
                    }
                }
            }
        });

        let protocol = GigiDnsProtocol::new(local_peer_id, config.clone());
        let now = Instant::now();

        let task = Self {
            config,
            interface_ip,
            protocol,
            event_tx,
            multicast_rx: packet_rx,
            send_tx,
            query_deadline: now,
            announce_deadline: now,
            cleanup_deadline: now,
            probe_state: ProbeState::Probing(Duration::from_millis(500)),
            has_discovered_peers: false,
            first_run: true,
            _io_handle: io_handle,
            address_update_rx,
            recent_query_responses: VecDeque::new(),
        };

        // Spawn the task that runs the async logic
        let handle = tokio::spawn(async move {
            task.run().await;
        });

        Ok(handle)
    }

    /// Creates a UDP socket for receiving multicast packets
    ///
    /// This socket is bound to the multicast address and joins the multicast group
    /// on the specified interface.
    ///
    /// # Arguments
    /// * `interface_ip` - IP address of the network interface
    /// * `config` - Configuration (for IPv4/IPv6 selection)
    ///
    /// # Returns
    /// - `Ok(UdpSocket)` - Configured receive socket
    /// - `Err(std::io::Error)` - Failed to create or configure socket
    fn create_recv_socket(
        interface_ip: &IpAddr,
        config: &GigiDnsConfig,
    ) -> std::io::Result<UdpSocket> {
        let multicast_ip = if config.enable_ipv6 {
            std::net::IpAddr::V6(IPV6_MDNS_MULTICAST_ADDRESS)
        } else {
            std::net::IpAddr::V4(IPV4_MDNS_MULTICAST_ADDRESS)
        };

        // Bind to INADDR_ANY (0.0.0.0) for receiving multicast
        let bind_addr = SocketAddr::new(
            if config.enable_ipv6 {
                std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED)
            } else {
                std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
            },
            GIGI_DNS_PORT,
        );

        let domain = if config.enable_ipv6 {
            socket2::Domain::IPV6
        } else {
            socket2::Domain::IPV4
        };

        let socket = socket2::Socket::new(domain, socket2::Type::DGRAM, None)?;
        socket.set_reuse_address(true)?;
        #[cfg(unix)]
        socket.set_reuse_port(true)?;

        socket.bind(&bind_addr.into())?;

        // Join multicast group on specific interface
        if let IpAddr::V4(ipv4) = multicast_ip {
            if let IpAddr::V4(interface_ip) = interface_ip {
                socket.join_multicast_v4(&ipv4, interface_ip)?;
            }
            socket.set_multicast_loop_v4(true)?;
            socket.set_multicast_ttl_v4(1)?;
        } else if let IpAddr::V6(ipv6) = multicast_ip {
            let _ = socket.join_multicast_v6(&ipv6, 0);
            let _ = socket.set_multicast_loop_v6(true);
        }

        let std_socket: std::net::UdpSocket = socket.into();
        std_socket.set_nonblocking(true)?;
        UdpSocket::from_std(std_socket)
    }

    /// Creates a UDP socket for sending multicast packets
    ///
    /// This socket is bound to the interface IP with a random port.
    ///
    /// # Arguments
    /// * `interface_ip` - IP address of the network interface
    /// * `_config` - Configuration (unused but kept for consistency)
    ///
    /// # Returns
    /// - `Ok(UdpSocket)` - Configured send socket
    /// - `Err(std::io::Error)` - Failed to create socket
    fn create_send_socket(
        interface_ip: &IpAddr,
        _config: &GigiDnsConfig,
    ) -> std::io::Result<UdpSocket> {
        let bind_addr = SocketAddr::new(*interface_ip, 0); // Random port

        let socket = std::net::UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;
        UdpSocket::from_std(socket)
    }

    /// Gets the current query interval based on probing state
    ///
    /// # Returns
    /// Current query interval (shorter during probing, normal after)
    fn get_current_query_interval(&self) -> Duration {
        match self.probe_state {
            ProbeState::Probing(interval) => interval,
            ProbeState::Finished => self.config.query_interval,
        }
    }

    /// Advances the adaptive probing state
    ///
    /// Doubles the probing interval until it reaches the normal interval,
    /// then switches to Finished state.
    fn advance_probe_state(&mut self) {
        if let ProbeState::Probing(interval) = self.probe_state {
            let next_interval = (interval * 2).min(self.config.query_interval);

            if next_interval >= self.config.query_interval {
                self.probe_state = ProbeState::Finished;
                tracing::debug!("Interface {} adaptive probing finished", self.interface_ip);
            } else {
                self.probe_state = ProbeState::Probing(next_interval);
                tracing::debug!(
                    "Interface {} adaptive probing: {:?} -> {:?}",
                    self.interface_ip,
                    interval,
                    next_interval
                );
            }
        }
    }

    /// Called when a peer is discovered
    ///
    /// Stops adaptive probing since we've found at least one peer.
    fn on_peer_discovered(&mut self) {
        if !self.has_discovered_peers {
            self.has_discovered_peers = true;
            self.probe_state = ProbeState::Finished;
            tracing::debug!(
                "Interface {} discovered peer, stopping adaptive probing",
                self.interface_ip
            );
        }
    }

    /// Sends a DNS query packet to the multicast address
    ///
    /// # Returns
    /// - `Ok(())` - Query sent successfully
    /// - `Err(std::io::Error)` - Failed to send
    fn send_query(&mut self) -> std::io::Result<()> {
        let query = self.protocol.build_query();

        // Add random jitter to avoid synchronized queries across peers
        let jitter = Duration::from_millis(rand::random::<u64>() % 100);
        let query_interval = self.get_current_query_interval() + jitter;

        let addr = if self.config.enable_ipv6 {
            SocketAddr::new(
                std::net::IpAddr::V6(IPV6_MDNS_MULTICAST_ADDRESS),
                GIGI_DNS_PORT,
            )
        } else {
            SocketAddr::new(
                std::net::IpAddr::V4(IPV4_MDNS_MULTICAST_ADDRESS),
                GIGI_DNS_PORT,
            )
        };

        self.send_tx
            .send((query, addr))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        self.query_deadline = Instant::now() + query_interval;
        tracing::debug!("Interface {} sent query to {}", self.interface_ip, addr);
        Ok(())
    }

    /// Sends an announcement packet to the multicast address
    ///
    /// Announcements are sent periodically to refresh our presence on the network.
    ///
    /// # Returns
    /// - `Ok(())` - Announcement sent successfully
    /// - `Err(std::io::Error)` - Failed to send
    fn send_announcement(&mut self) -> std::io::Result<()> {
        let responses = self
            .protocol
            .build_response()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let addr = if self.config.enable_ipv6 {
            SocketAddr::new(
                std::net::IpAddr::V6(IPV6_MDNS_MULTICAST_ADDRESS),
                GIGI_DNS_PORT,
            )
        } else {
            SocketAddr::new(
                std::net::IpAddr::V4(IPV4_MDNS_MULTICAST_ADDRESS),
                GIGI_DNS_PORT,
            )
        };

        for response in responses {
            self.send_tx
                .send((response, addr))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }

        self.announce_deadline = Instant::now() + self.config.announce_interval;
        tracing::debug!("Interface {} sent announcement", self.interface_ip);
        Ok(())
    }

    /// Processes an incoming DNS packet
    ///
    /// Handles both queries (responds with our info) and responses (discovers peers).
    ///
    /// # Arguments
    /// * `packet` - Raw DNS packet bytes
    /// * `src` - Source address of the packet
    fn process_packet(&mut self, packet: &[u8], src: SocketAddr) {
        tracing::debug!(
            "Interface {} received {} bytes from {}",
            self.interface_ip,
            packet.len(),
            src
        );

        // Check if it's a query and respond
        if self.protocol.is_query(packet) {
            tracing::debug!(
                "Interface {} detected query from {}",
                self.interface_ip,
                src
            );

            // Rate limiting: check if we're responding too frequently
            if self.is_query_response_rate_limited() {
                tracing::debug!(
                    "Interface {} rate limited query response to {}",
                    self.interface_ip,
                    src
                );
                return;
            }

            match self.protocol.build_response() {
                Ok(packets) => {
                    for response in packets {
                        let _ = self.send_tx.send((response, src));
                    }
                    tracing::debug!(
                        "Interface {} responded to query from {}",
                        self.interface_ip,
                        src
                    );
                }
                Err(e) => {
                    tracing::debug!(
                        "Interface {} failed to build response: {}",
                        self.interface_ip,
                        e
                    );
                }
            }
            return;
        }

        // Handle response packet
        tracing::debug!(
            "Interface {} processing response from {}",
            self.interface_ip,
            src
        );
        match self.protocol.handle_packet(packet) {
            Ok(Some(event)) => {
                self.on_peer_discovered();
                let _ = self.event_tx.send(InterfaceEvent::PeerDiscovered(event));
                tracing::debug!(
                    "Interface {} discovered peer from {}",
                    self.interface_ip,
                    src
                );
            }
            Ok(None) => {}
            Err(e) => {
                if e != "Self-discovery" {
                    tracing::debug!(
                        "Interface {} failed to process packet: {}",
                        self.interface_ip,
                        e
                    );
                }
            }
        }
    }

    /// Checks if query responses are rate-limited
    ///
    /// Prevents DoS attacks by limiting responses to 10 per second.
    ///
    /// # Returns
    /// `true` if rate limited (should not respond), `false` otherwise
    fn is_query_response_rate_limited(&mut self) -> bool {
        const MAX_RESPONSES_PER_SECOND: usize = 10;
        const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(1);

        let now = Instant::now();

        // Cleanup old response records
        while let Some(&timestamp) = self.recent_query_responses.front() {
            if now.duration_since(timestamp) > RATE_LIMIT_WINDOW {
                self.recent_query_responses.pop_front();
            } else {
                break;
            }
        }

        // Check if we've exceeded the limit
        if self.recent_query_responses.len() >= MAX_RESPONSES_PER_SECOND {
            return true;
        }

        // Record this response
        self.recent_query_responses.push_back(now);
        false
    }

    /// Main async loop for the interface task
    ///
    /// This loop handles:
    /// - Address updates from main behaviour
    /// - Incoming packets from I/O task
    /// - Periodic queries, announcements, and cleanup
    ///
    /// The loop runs until the channel is closed or an error occurs.
    pub async fn run(mut self) {
        tracing::debug!("Interface {} task starting", self.interface_ip);

        // Send initial query and announcement immediately on startup
        if self.first_run {
            tracing::debug!(
                "Interface {} sending initial query and announcement",
                self.interface_ip
            );
            let _ = self.send_query();
            let _ = self.send_announcement();
            self.first_run = false;
        }

        loop {
            // Calculate the next deadline among all timers
            let next_deadline = *[
                self.query_deadline,
                self.announce_deadline,
                self.cleanup_deadline,
            ]
            .iter()
            .min()
            .unwrap();

            // Calculate delay to next deadline
            let now = Instant::now();
            let delay = if next_deadline > now {
                Some(next_deadline - now)
            } else {
                // All deadlines are in the past, process immediately
                Some(Duration::ZERO)
            };

            tokio::select! {
                // Process address updates - highest priority
                Some(addresses) = self.address_update_rx.recv() => {
                    tracing::debug!("Interface {} received address update with {} addresses", self.interface_ip, addresses.len());
                    self.protocol.update_listen_addresses(addresses);
                }
                // Process packets from I/O task - highest priority
                result = self.multicast_rx.recv() => {
                    match result {
                        Some(InterfacePacket::Received(packet, src)) => {
                            self.process_packet(&packet, src);
                        }
                        None => {
                            tracing::debug!("Interface {} channel closed, stopping", self.interface_ip);
                            break;
                        }
                    }
                }
                // Wait until next deadline
                _ = tokio::time::sleep(delay.unwrap_or(Duration::ZERO)) => {
                    // Deadline reached or all deadlines were in the past
                }
            }

            let now = Instant::now();

            // Send periodic announcements
            if now >= self.announce_deadline {
                let _ = self.send_announcement();
            }

            // Send periodic queries with adaptive probing
            if now >= self.query_deadline {
                let _ = self.send_query();
                self.advance_probe_state();
            }

            // Cleanup expired peers
            if now >= self.cleanup_deadline {
                for event in self.protocol.cleanup_expired() {
                    let _ = self.event_tx.send(InterfaceEvent::PeerExpired(event));
                }
                self.cleanup_deadline = now + self.config.cleanup_interval;
            }
        }

        tracing::debug!("Interface {} task stopped", self.interface_ip);
    }
}

/// Handles if-watch events and converts them to interface changes
///
/// # Arguments
/// * `event` - The if-watch event
/// * `config` - Configuration (for IPv4/IPv6 filtering)
///
/// # Returns
/// - `Some((ip, true))` - Interface came up
/// - `Some((ip, false))` - Interface went down
/// - `None` - Event should be ignored (loopback, wrong IP version, error)
pub fn handle_if_event(
    event: std::io::Result<IfEvent>,
    config: &GigiDnsConfig,
) -> Option<(IpAddr, bool)> {
    match event {
        Ok(IfEvent::Up(inet)) => {
            let addr = inet.addr();

            // Skip loopback
            if addr.is_loopback() {
                return None;
            }

            // Filter by IPv4/IPv6 config
            if addr.is_ipv4() && config.enable_ipv6 || addr.is_ipv6() && !config.enable_ipv6 {
                return None;
            }

            tracing::debug!("Interface {} came up", addr);
            Some((addr, true)) // true = up
        }
        Ok(IfEvent::Down(inet)) => {
            let addr = inet.addr();
            tracing::debug!("Interface {} went down", addr);
            Some((addr, false)) // false = down
        }
        Err(e) => {
            tracing::error!("if-watch error: {}", e);
            None
        }
    }
}
