// Copyright 2024 Gigi Team.
//
// Gigi DNS Behaviour - libp2p integration with if-watch for per-interface support

use crate::interface::{handle_if_event, InterfaceEvent, InterfaceTask};
use crate::types::*;
use futures::stream::StreamExt;
use if_watch::tokio::IfWatcher;
use libp2p::core::transport::PortUse;
use libp2p::core::Endpoint;
use libp2p::Multiaddr;
use libp2p_identity::PeerId;
use libp2p_swarm::{
    behaviour::FromSwarm, dummy, ConnectionDenied, ConnectionId, ListenAddresses, NetworkBehaviour,
    THandler, THandlerInEvent, THandlerOutEvent, ToSwarm,
};
use std::{
    collections::{HashMap, VecDeque},
    convert::Infallible,
    net::IpAddr,
    sync::{Arc, RwLock},
    task::{Context, Poll},
};
use tokio::task::JoinHandle;

pub struct GigiDnsBehaviour {
    config: GigiDnsConfig,
    #[allow(dead_code)]
    local_peer_id: libp2p_identity::PeerId,
    pending_events: VecDeque<GigiDnsEvent>,
    listen_addresses: Arc<RwLock<ListenAddresses>>,
    // if-watch for monitoring network interfaces
    if_watcher: IfWatcher,
    // Per-interface tasks
    if_tasks: HashMap<IpAddr, JoinHandle<()>>,
    // Channel for receiving events from interface tasks
    interface_rx: tokio::sync::mpsc::UnboundedReceiver<InterfaceEvent>,
    // Sender that will be passed to all interface tasks
    interface_tx: tokio::sync::mpsc::UnboundedSender<InterfaceEvent>,
    // Senders for address updates to interface tasks
    address_update_txs: HashMap<IpAddr, tokio::sync::mpsc::UnboundedSender<Vec<Multiaddr>>>,
    // Track discovered peers by peer_id for outbound connections (single source of truth)
    discovered_peers: HashMap<PeerId, GigiPeerInfo>,
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
        // Create if-watcher for monitoring network interfaces
        let if_watcher = IfWatcher::new()?;

        // Create channel for interface events (shared by all interface tasks)
        let (interface_tx, interface_rx) = tokio::sync::mpsc::unbounded_channel();

        // Spawn interface tasks for existing interfaces
        let if_tasks = HashMap::new();

        // Note: IfWatcher will emit events for existing interfaces during first poll
        // We'll handle them in the poll() method

        tracing::debug!("Gigi DNS behaviour initialized with if-watch support");

        Ok(Self {
            config,
            local_peer_id,
            pending_events: VecDeque::new(),
            listen_addresses: Arc::new(RwLock::new(ListenAddresses::default())),
            if_watcher,
            if_tasks,
            interface_rx,
            interface_tx,
            address_update_txs: HashMap::new(),
            discovered_peers: HashMap::new(),
        })
    }

    fn spawn_interface_task(&mut self, interface_ip: IpAddr) -> std::io::Result<()> {
        tracing::debug!("Spawning task for interface {}", interface_ip);

        // If task already exists, stop it first to avoid stale tasks
        if self.if_tasks.contains_key(&interface_ip) {
            tracing::debug!("Restarting task for interface {}", interface_ip);
            self.stop_interface_task(interface_ip);
        }

        // Create channel for address updates
        let (address_update_tx, address_update_rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = InterfaceTask::spawn(
            interface_ip,
            self.config.clone(),
            self.local_peer_id,
            self.interface_tx.clone(),
            address_update_rx,
        )?;

        // Store the task handle and address update sender
        self.if_tasks.insert(interface_ip, handle);
        self.address_update_txs
            .insert(interface_ip, address_update_tx);

        // Send current addresses if available
        let addrs = match self.listen_addresses.read() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::warn!(
                    "Failed to acquire lock on listen_addresses: {}, using inner state",
                    e
                );
                e.into_inner()
            }
        };
        let libp2p_addrs: Vec<Multiaddr> = addrs.iter().cloned().collect();
        if !libp2p_addrs.is_empty() {
            let _ = self
                .address_update_txs
                .get(&interface_ip)
                .unwrap()
                .send(libp2p_addrs);
        }

        tracing::debug!("Task spawned for interface {}", interface_ip);
        Ok(())
    }

    fn stop_interface_task(&mut self, interface_ip: IpAddr) {
        if let Some(handle) = self.if_tasks.remove(&interface_ip) {
            tracing::debug!("Stopping task for interface {}", interface_ip);
            handle.abort();
        }
        self.address_update_txs.remove(&interface_ip);
    }
}

impl Drop for GigiDnsBehaviour {
    fn drop(&mut self) {
        tracing::debug!(
            "GigiDnsBehaviour dropping, cleaning up {} tasks",
            self.if_tasks.len()
        );
        // Abort all interface tasks
        for (ip, handle) in self.if_tasks.drain() {
            tracing::debug!("Aborting task for interface {}", ip);
            handle.abort();
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
        maybe_peer: Option<PeerId>,
        _addresses: &[libp2p::Multiaddr],
        _effective_role: Endpoint,
    ) -> Result<Vec<libp2p::Multiaddr>, ConnectionDenied> {
        let Some(peer_id) = maybe_peer else {
            return Ok(vec![]);
        };

        // Look up the peer from our discovered peers map
        Ok(self
            .discovered_peers
            .get(&peer_id)
            .map(|info| info.multiaddr.clone())
            .into_iter()
            .collect())
    }

    fn handle_established_outbound_connection(
        &mut self,
        _: ConnectionId,
        _: PeerId,
        _: &libp2p::Multiaddr,
        _: Endpoint,
        _: PortUse,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }

    fn on_connection_handler_event(
        &mut self,
        _: PeerId,
        _: ConnectionId,
        _ev: THandlerOutEvent<Self>,
    ) {
    }

    fn on_swarm_event(&mut self, event: FromSwarm) {
        self.listen_addresses
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .on_swarm_event(&event);

        // Broadcast listen address changes to all interface tasks
        let addrs = match self.listen_addresses.read() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::warn!(
                    "Failed to acquire lock on listen_addresses: {}, using inner state",
                    e
                );
                e.into_inner()
            }
        };
        let libp2p_addrs: Vec<Multiaddr> = addrs.iter().cloned().collect();

        if !libp2p_addrs.is_empty() {
            for (interface_ip, tx) in &self.address_update_txs {
                tracing::debug!(
                    "Sending address update to interface {} ({} addresses)",
                    interface_ip,
                    libp2p_addrs.len()
                );
                let _ = tx.send(libp2p_addrs.clone());
            }
        }
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        loop {
            // Process if-watch events
            while let Poll::Ready(Some(event)) = self.if_watcher.poll_next_unpin(cx) {
                if let Some((interface_ip, is_up)) = handle_if_event(event, &self.config) {
                    if is_up {
                        // Interface came up - spawn task
                        if !self.if_tasks.contains_key(&interface_ip) {
                            let _ = self.spawn_interface_task(interface_ip);
                        }
                    } else {
                        // Interface went down - stop task
                        self.stop_interface_task(interface_ip);
                    }
                }
            }

            // Process interface task events
            while let Poll::Ready(Some(event)) = self.interface_rx.poll_recv(cx) {
                let should_emit = match &event {
                    InterfaceEvent::PeerDiscovered(GigiDnsEvent::Discovered(peer_info)) => {
                        let peer_id = peer_info.peer_id.clone();
                        // Check if this is a new peer or an update
                        if let Some(old_info) = self.discovered_peers.get(&peer_id) {
                            if old_info.nickname != peer_info.nickname
                                || old_info.multiaddr != peer_info.multiaddr
                            {
                                // Generate Updated event
                                self.pending_events.push_back(GigiDnsEvent::Updated {
                                    peer_id,
                                    old_info: old_info.clone(),
                                    new_info: peer_info.clone(),
                                });
                                self.discovered_peers.insert(peer_id, peer_info.clone());
                                false // Don't emit the original Discovered event
                            } else {
                                // Update TTL
                                self.discovered_peers.insert(peer_id, peer_info.clone());
                                true
                            }
                        } else {
                            // New peer
                            self.discovered_peers.insert(peer_id, peer_info.clone());
                            true
                        }
                    }
                    InterfaceEvent::PeerExpired(GigiDnsEvent::Expired { peer_id, info: _ }) => {
                        self.discovered_peers.remove(peer_id);
                        true
                    }
                    _ => true,
                };

                if should_emit {
                    self.pending_events.push_back(event.into());
                }
            }

            // Return any pending GigiDnsEvents
            if let Some(event) = self.pending_events.pop_front() {
                return Poll::Ready(ToSwarm::GenerateEvent(event));
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
