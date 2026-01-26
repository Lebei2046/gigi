// Copyright 2024 Gigi Team.
//
// Per-interface mDNS handler for gigi-dns

use crate::protocol::GigiDnsProtocol;
use crate::types::*;
use if_watch::IfEvent;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

// Adaptive probing state
#[derive(Debug, Clone, Copy)]
enum ProbeState {
    Probing(Duration), // Current probing interval
    Finished,          // Probing complete, use normal interval
}

// Events sent from interface tasks back to the main behaviour
#[derive(Debug)]
pub enum InterfaceEvent {
    PeerDiscovered(GigiDnsEvent),
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

// Per-interface state that runs as a background task
pub struct InterfaceTask {
    config: GigiDnsConfig,
    interface_ip: IpAddr,
    protocol: GigiDnsProtocol,
    event_tx: UnboundedSender<InterfaceEvent>,
    multicast_rx: UnboundedReceiver<InterfacePacket>,
    send_tx: UnboundedSender<(Vec<u8>, SocketAddr)>,
    query_deadline: Instant,
    announce_deadline: Instant,
    cleanup_deadline: Instant,
    probe_state: ProbeState,
    has_discovered_peers: bool,
    first_run: bool,
    _io_handle: tokio::task::JoinHandle<()>,
    // Channel to receive address updates from main behaviour
    address_update_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<libp2p::Multiaddr>>,
}

// Internal packet type
pub enum InterfacePacket {
    Received(Vec<u8>, SocketAddr),
}

impl InterfaceTask {
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

        // Spawn background I/O task
        let io_handle = tokio::spawn(async move {
            let mut buffer = vec![0u8; 4096];
            tracing::debug!("Interface {} I/O task started", interface_ip);

            loop {
                tokio::select! {
                    result = recv_socket.recv_from(&mut buffer) => {
                        match result {
                            Ok((len, src)) => {
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
        };

        // Spawn the task that runs the async logic
        let handle = tokio::spawn(async move {
            task.run().await;
        });

        Ok(handle)
    }

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

    fn create_send_socket(
        interface_ip: &IpAddr,
        _config: &GigiDnsConfig,
    ) -> std::io::Result<UdpSocket> {
        let bind_addr = SocketAddr::new(*interface_ip, 0); // Random port

        let socket = std::net::UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;
        UdpSocket::from_std(socket)
    }

    fn get_current_query_interval(&self) -> Duration {
        match self.probe_state {
            ProbeState::Probing(interval) => interval,
            ProbeState::Finished => self.config.query_interval,
        }
    }

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

    fn send_query(&mut self) -> std::io::Result<()> {
        let query = self.protocol.build_query();

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

            // Calculate delay to next deadline, or sleep indefinitely if all deadlines are past
            let delay = next_deadline.checked_duration_since(Instant::now());

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
                _ = tokio::time::sleep_until(tokio::time::Instant::from_std(next_deadline)), if delay.is_some() => {
                    // Deadline reached, check and execute tasks below
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

// Handle if-watch events
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
