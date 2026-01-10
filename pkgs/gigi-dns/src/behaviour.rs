// Copyright 2024 Gigi Team.
//
// Gigi DNS Behaviour - libp2p integration with async UDP

use crate::protocol::GigiDnsProtocol;
use crate::types::*;
use libp2p::core::transport::PortUse;
use libp2p::core::Endpoint;
use libp2p_swarm::{
    behaviour::FromSwarm, dummy, ConnectionDenied, ConnectionId, ListenAddresses, NetworkBehaviour,
    THandler, THandlerInEvent, THandlerOutEvent, ToSwarm,
};
use std::{
    collections::VecDeque,
    convert::Infallible,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, RwLock},
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::interval;

// Internal message type for communication between background task and behaviour
enum MulticastMessage {
    PacketReceived(Vec<u8>, SocketAddr),
    Tick,
}

pub struct GigiDnsBehaviour {
    config: GigiDnsConfig,
    #[allow(dead_code)]
    local_peer_id: libp2p_identity::PeerId,
    protocol: GigiDnsProtocol,
    pending_events: VecDeque<GigiDnsEvent>,
    query_timer: Instant,
    announce_timer: Instant,
    cleanup_timer: Instant,
    listen_addresses: Arc<RwLock<ListenAddresses>>,
    // Async channels for background task communication
    multicast_rx: UnboundedReceiver<MulticastMessage>,
    send_tx: UnboundedSender<(Vec<u8>, SocketAddr)>,
    _task_handle: tokio::task::JoinHandle<()>,
    _timer_handle: tokio::task::JoinHandle<()>,
}

pub enum GigiDnsCommand {
    UpdateNickname(String),
    UpdateCapabilities(Vec<String>),
    UpdateMetadata(String, String),
}

impl GigiDnsBehaviour {
    pub fn new(
        local_peer_id: libp2p_identity::PeerId,
        config: GigiDnsConfig,
    ) -> std::io::Result<Self> {
        // Create tokio runtime if not already present
        let handle = tokio::runtime::Handle::try_current().unwrap_or_else(|_| {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");
            rt.handle().clone()
        });

        // Create receive socket - binds to fixed port 5354
        let recv_socket = Self::create_recv_socket(&config)?;
        let recv_local = recv_socket.local_addr()?;
        tracing::info!("Receive socket bound to {}", recv_local);

        // Create send socket - binds to random port
        let send_socket = Self::create_send_socket(&config)?;
        let send_local = send_socket.local_addr()?;
        tracing::info!("Send socket bound to {}", send_local);

        // Create channels for background task communication
        let (event_tx, event_rx) = unbounded_channel();
        let (send_tx, mut send_rx) = unbounded_channel::<(Vec<u8>, SocketAddr)>();

        // Spawn timer task to wake up the poll periodically
        let event_tx_clone = event_tx.clone();
        let timer_handle = handle.spawn(async move {
            let mut timer = interval(Duration::from_secs(1));
            loop {
                timer.tick().await;
                let _ = event_tx_clone.send(MulticastMessage::Tick);
            }
        });

        // Spawn background task
        let task_handle = handle.spawn(async move {
            let mut buffer = vec![0u8; 4096];
            tracing::info!("UDP background task started");

            loop {
                tokio::select! {
                    // Handle incoming packets on recv socket
                    result = recv_socket.recv_from(&mut buffer) => {
                        match result {
                            Ok((len, src)) => {
                                tracing::debug!("UDP recv: {} bytes from {}", len, src);
                                let packet = buffer[..len].to_vec();
                                let _ = event_tx.send(MulticastMessage::PacketReceived(packet, src));
                            }
                            Err(e) => {
                                tracing::error!("UDP recv error: {}", e);
                            }
                        }
                    }
                    // Handle outgoing packets on send socket
                    Some((data, addr)) = send_rx.recv() => {
                        match send_socket.send_to(&data, addr).await {
                            Ok(n) => {
                                tracing::debug!("UDP sent {} bytes to {}", n, addr);
                            }
                            Err(e) => {
                                tracing::error!("UDP send error: {}", e);
                            }
                        }
                    }
                }
            }
        });

        let protocol = GigiDnsProtocol::new(local_peer_id, config.clone());
        let now = Instant::now();

        Ok(Self {
            config,
            local_peer_id,
            protocol,
            pending_events: VecDeque::new(),
            query_timer: now,
            announce_timer: now,
            cleanup_timer: now,
            listen_addresses: Arc::new(RwLock::new(ListenAddresses::default())),
            multicast_rx: event_rx,
            send_tx,
            _task_handle: task_handle,
            _timer_handle: timer_handle,
        })
    }

    fn create_recv_socket(config: &GigiDnsConfig) -> std::io::Result<UdpSocket> {
        let multicast_ip = if config.enable_ipv6 {
            std::net::IpAddr::V6(IPV6_MDNS_MULTICAST_ADDRESS)
        } else {
            std::net::IpAddr::V4(IPV4_MDNS_MULTICAST_ADDRESS)
        };

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

        // Enable SO_REUSEADDR and SO_REUSEPORT to allow multiple instances
        socket.set_reuse_address(true)?;
        #[cfg(unix)]
        socket.set_reuse_port(true)?;

        socket.bind(&bind_addr.into())?;

        if config.use_broadcast {
            socket.set_broadcast(true)?;
        } else {
            if let IpAddr::V4(ipv4) = multicast_ip {
                socket.join_multicast_v4(&ipv4, &Ipv4Addr::UNSPECIFIED)?;
                socket.set_multicast_loop_v4(true)?;
                socket.set_multicast_ttl_v4(1)?;
            } else if let IpAddr::V6(ipv6) = multicast_ip {
                let _ = socket.join_multicast_v6(&ipv6, 0);
                let _ = socket.set_multicast_loop_v6(true);
            }
        }

        let std_socket: std::net::UdpSocket = socket.into();
        std_socket.set_nonblocking(true)?;
        UdpSocket::from_std(std_socket)
    }

    fn create_send_socket(config: &GigiDnsConfig) -> std::io::Result<UdpSocket> {
        let bind_addr = SocketAddr::new(
            if config.enable_ipv6 {
                std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED)
            } else {
                std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
            },
            0, // Random port
        );

        let socket = std::net::UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;
        UdpSocket::from_std(socket)
    }

    fn send_query(&mut self) -> std::io::Result<()> {
        let query = self.protocol.build_query();
        let addr = if self.config.use_broadcast {
            SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(255, 255, 255, 255)),
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

        self.query_timer = Instant::now();
        tracing::info!("Sent Gigi DNS query to {}", addr);
        Ok(())
    }

    fn send_announcement(&mut self) -> std::io::Result<()> {
        let responses = self
            .protocol
            .build_response()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let addr = if self.config.use_broadcast {
            SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(255, 255, 255, 255)),
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

        self.announce_timer = Instant::now();
        tracing::info!("Sent Gigi DNS announcement to {}", addr);
        Ok(())
    }

    fn process_packet(&mut self, packet: &[u8], src: SocketAddr) {
        tracing::info!("Received {} bytes from {}", packet.len(), src);

        // Check if it's a query and respond if needed
        if self.protocol.is_query(packet) {
            tracing::info!("Detected query packet from {}", src);
            match self.protocol.build_response() {
                Ok(packets) => {
                    for response in packets {
                        let _ = self.send_tx.send((response, src));
                    }
                    tracing::info!("Responded to Gigi DNS query from {}", src);
                }
                Err(e) => {
                    tracing::warn!("Failed to build response: {}", e);
                }
            }
            return;
        }

        // Handle response packet
        tracing::info!("Processing response packet from {}", src);
        match self.protocol.handle_packet(packet) {
            Ok(Some(event)) => {
                tracing::info!("Processed Gigi DNS packet successfully");
                self.pending_events.push_back(event);
            }
            Ok(None) => {
                tracing::info!("No event generated from packet");
            }
            Err(e) => {
                if e != "Self-discovery" {
                    tracing::warn!("Failed to process DNS packet: {}", e);
                } else {
                    tracing::debug!("Ignoring self-discovery");
                }
            }
        }
    }
}

impl NetworkBehaviour for GigiDnsBehaviour {
    type ConnectionHandler = dummy::ConnectionHandler;
    type ToSwarm = GigiDnsEvent;

    fn handle_pending_inbound_connection(
        &mut self,
        _: ConnectionId,
        _: &libp2p::Multiaddr,
        _: &libp2p::Multiaddr,
    ) -> Result<(), ConnectionDenied> {
        Ok(())
    }

    fn handle_established_inbound_connection(
        &mut self,
        _: ConnectionId,
        _: libp2p_identity::PeerId,
        _: &libp2p::Multiaddr,
        _: &libp2p::Multiaddr,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }

    fn handle_pending_outbound_connection(
        &mut self,
        _connection_id: ConnectionId,
        maybe_peer: Option<libp2p_identity::PeerId>,
        _addresses: &[libp2p::Multiaddr],
        _effective_role: Endpoint,
    ) -> Result<Vec<libp2p::Multiaddr>, ConnectionDenied> {
        let Some(peer_id) = maybe_peer else {
            return Ok(vec![]);
        };

        Ok(self
            .protocol
            .find_peer_by_id(&peer_id)
            .map(|info| info.multiaddr.clone())
            .into_iter()
            .collect())
    }

    fn handle_established_outbound_connection(
        &mut self,
        _: ConnectionId,
        _: libp2p_identity::PeerId,
        _: &libp2p::Multiaddr,
        _: Endpoint,
        _: PortUse,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }

    fn on_connection_handler_event(
        &mut self,
        _: libp2p_identity::PeerId,
        _: ConnectionId,
        _ev: THandlerOutEvent<Self>,
    ) {
    }

    fn on_swarm_event(&mut self, event: FromSwarm) {
        self.listen_addresses
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .on_swarm_event(&event);

        let addrs = self.listen_addresses.read().unwrap();
        let libp2p_addrs: Vec<libp2p::Multiaddr> = addrs.iter().cloned().collect();
        self.protocol.update_listen_addresses(libp2p_addrs);
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        loop {
            // Process any pending multicast events
            while let Poll::Ready(Some(msg)) = self.multicast_rx.poll_recv(cx) {
                match msg {
                    MulticastMessage::PacketReceived(packet, src) => {
                        self.process_packet(&packet, src);
                    }
                    MulticastMessage::Tick => {
                        // Timer tick received, continue to check timers below
                        tracing::trace!("Timer tick received");
                    }
                }
            }

            // Return any pending GigiDnsEvents
            if let Some(event) = self.pending_events.pop_front() {
                return Poll::Ready(ToSwarm::GenerateEvent(event));
            }

            let now = Instant::now();

            // Send periodic announcements
            if now.duration_since(self.announce_timer) >= self.config.announce_interval {
                tracing::debug!("Announcement timer triggered");
                let _ = self.send_announcement();
            }

            // Send periodic queries
            if now.duration_since(self.query_timer) >= self.config.query_interval {
                tracing::debug!("Query timer triggered");
                let _ = self.send_query();
            }

            // Cleanup expired peers periodically
            if now.duration_since(self.cleanup_timer) >= self.config.cleanup_interval {
                for event in self.protocol.cleanup_expired() {
                    self.pending_events.push_back(event);
                }
                self.cleanup_timer = now;
            }

            return Poll::Pending;
        }
    }
}

impl From<GigiDnsEvent> for ToSwarm<GigiDnsEvent, Infallible> {
    fn from(event: GigiDnsEvent) -> Self {
        ToSwarm::GenerateEvent(event)
    }
}
