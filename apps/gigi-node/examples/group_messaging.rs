//! Group messaging test across a 3-node gigi-node network (in-process).
//!
//! Topology
//! --------
//!   node-1  Bootstrap node A  (DHT entry point for Alice)
//!   node-2  Bootstrap node B  (DHT entry point for Bob)
//!   alice   Client – dials node-1, joins group, sends messages
//!   bob     Client – dials node-2, joins group, receives & replies
//!
//! node-1 and node-2 connect to each other, forming the backbone.
//! Alice and Bob reach each other through the DHT + GossipSub mesh
//! without ever dialing each other directly.
//!
//! Run
//! ---
//!   RUST_LOG=info cargo run -p gigi-node --example group_messaging

use anyhow::{bail, Result};
use futures::StreamExt;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::Boxed},
    gossipsub, identify, identity, kad, noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{sync::oneshot, time::timeout};
use tracing::{error, info, warn};

// ── NodeBehaviour (identical to main.rs) ─────────────────────────────────────

#[derive(NetworkBehaviour)]
struct NodeBehaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    relay: relay::Behaviour,
}

// ── Test parameters ───────────────────────────────────────────────────────────

const GROUP: &str = "gigi-general";

const ALICE_MSG_1: &str = "alice: hello group!";
const ALICE_MSG_2: &str = "alice: anyone here?";
const BOB_REPLY: &str = "bob: yes, loud and clear!";

const T_CONNECT: Duration = Duration::from_secs(20);
const T_MESH: Duration = Duration::from_secs(4);
const T_MSG: Duration = Duration::from_secs(25);

// ── Shared state written by receiver tasks ────────────────────────────────────

#[derive(Default, Debug)]
struct Results {
    bob_got_msg1: bool,
    bob_got_msg2: bool,
    alice_got_reply: bool,
}

// ─────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let results = Arc::new(Mutex::new(Results::default()));

    // Channels: each bootstrap task sends its (PeerId, Multiaddr) when ready
    let (n1_tx, n1_rx) = oneshot::channel::<(PeerId, Multiaddr)>();
    let (n2_tx, n2_rx) = oneshot::channel::<(PeerId, Multiaddr)>();

    // ── Start the two bootstrap nodes ────────────────────────────────────────
    // node-1 starts first; node-2 dials node-1 so the backbone is connected.

    let n1_handle = tokio::spawn(run_bootstrap("node-1", n1_tx, None));

    // Wait for node-1 before starting node-2
    let (n1_peer_id, n1_addr) = timeout(T_CONNECT, n1_rx)
        .await
        .map_err(|_| anyhow::anyhow!("timed out waiting for node-1"))?
        .map_err(|_| anyhow::anyhow!("node-1 channel closed"))?;

    info!("[main] node-1 ready: {} @ {}", n1_peer_id, n1_addr);

    // node-2 knows about node-1 so the two bootstrap nodes form a DHT backbone
    let n2_handle = tokio::spawn(run_bootstrap(
        "node-2",
        n2_tx,
        Some((n1_peer_id, n1_addr.clone())),
    ));

    let (n2_peer_id, n2_addr) = timeout(T_CONNECT, n2_rx)
        .await
        .map_err(|_| anyhow::anyhow!("timed out waiting for node-2"))?
        .map_err(|_| anyhow::anyhow!("node-2 channel closed"))?;

    info!("[main] node-2 ready: {} @ {}", n2_peer_id, n2_addr);

    // ── Start Bob first so he is subscribed before Alice publishes ────────────
    let results_bob = Arc::clone(&results);
    let bob_handle = tokio::spawn(run_bob(n2_peer_id, n2_addr.clone(), results_bob));

    // Small pause to let Bob subscribe and propagate through the mesh
    tokio::time::sleep(Duration::from_millis(500)).await;

    // ── Start Alice ───────────────────────────────────────────────────────────
    let results_alice = Arc::clone(&results);
    let alice_handle = tokio::spawn(run_alice(n1_peer_id, n1_addr.clone(), results_alice));

    // Wait for both clients to finish
    alice_handle.await??;
    bob_handle.await??;

    // Cancel the forever-running bootstrap nodes
    n1_handle.abort();
    n2_handle.abort();

    // ── Print results ─────────────────────────────────────────────────────────
    let r = results.lock().unwrap();
    println!();
    println!("══════════════════════════ RESULTS ══════════════════════════");
    println!("  Bob   received alice_msg_1 : {}", pass(r.bob_got_msg1));
    println!("  Bob   received alice_msg_2 : {}", pass(r.bob_got_msg2));
    println!("  Alice received bob reply   : {}", pass(r.alice_got_reply));
    println!("═════════════════════════════════════════════════════════════");
    println!();

    if !r.bob_got_msg1 || !r.bob_got_msg2 || !r.alice_got_reply {
        bail!("One or more tests FAILED");
    }
    info!("All group messaging tests PASSED ✓");
    Ok(())
}

// ── Bootstrap node ────────────────────────────────────────────────────────────
// Runs forever (aborted by main after the test completes).
// Subscribes to GROUP so it participates in the GossipSub mesh.

async fn run_bootstrap(
    tag: &'static str,
    ready_tx: oneshot::Sender<(PeerId, Multiaddr)>,
    peer: Option<(PeerId, Multiaddr)>,
) -> Result<()> {
    let key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(key.public());
    info!("[{tag}] peer_id = {peer_id}");

    let mut swarm = build_swarm(&key)?;
    swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?;

    // Join the GossipSub mesh so messages can propagate through this node
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&gossipsub::IdentTopic::new(GROUP))
        .map_err(|e| anyhow::anyhow!("[{tag}] subscribe: {e}"))?;

    // If a sibling bootstrap is known, add it to the DHT and connect
    if let Some((other_id, other_addr)) = peer {
        swarm
            .behaviour_mut()
            .kademlia
            .add_address(&other_id, other_addr.clone());
        let _ = swarm.behaviour_mut().kademlia.bootstrap();
        swarm.dial(other_addr)?;
    }

    // Wrap in Option so it can be consumed exactly once inside the loop
    let mut ready_tx = Some(ready_tx);
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("[{tag}] listening on {address}");
                if let Some(tx) = ready_tx.take() {
                    let _ = tx.send((peer_id, address));
                }
            }
            SwarmEvent::ConnectionEstablished { peer_id: p, .. } => {
                info!("[{tag}] connected to {p}");
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Kademlia(kad::Event::RoutingUpdated {
                peer,
                ..
            })) => {
                info!("[{tag}] DHT routing updated: {peer}");
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(
                gossipsub::Event::Subscribed { peer_id, topic },
            )) => {
                info!("[{tag}] peer {peer_id} subscribed to {topic}");
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                message,
                ..
            })) => {
                let text = String::from_utf8_lossy(&message.data);
                info!("[{tag}] relayed on {}: \"{text}\"", message.topic);
            }
            _ => {}
        }
    }
}

// ── Alice ─────────────────────────────────────────────────────────────────────
// Dials node-1, subscribes to GROUP, publishes two messages, waits for reply.

async fn run_alice(
    n1_peer_id: PeerId,
    n1_addr: Multiaddr,
    results: Arc<Mutex<Results>>,
) -> Result<()> {
    let key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(key.public());
    info!("[alice] peer_id = {peer_id}");

    let mut swarm = build_swarm(&key)?;
    swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?;

    let topic = gossipsub::IdentTopic::new(GROUP);
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic)
        .map_err(|e| anyhow::anyhow!("alice subscribe: {e}"))?;

    // Bootstrap via node-1
    swarm
        .behaviour_mut()
        .kademlia
        .add_address(&n1_peer_id, n1_addr.clone());
    let _ = swarm.behaviour_mut().kademlia.bootstrap();
    swarm.dial(n1_addr)?;

    // Wait until connected to node-1
    info!("[alice] connecting to node-1…");
    timeout(T_CONNECT, async {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id: p, .. } => {
                    info!("[alice] connected to {p}");
                    break;
                }
                SwarmEvent::OutgoingConnectionError { error, .. } => {
                    error!("[alice] connection error: {error}");
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("[alice] listening on {address}");
                }
                _ => {}
            }
        }
    })
    .await
    .map_err(|_| anyhow::anyhow!("alice: timed out connecting to node-1"))?;

    // Let the GossipSub mesh form across the backbone
    info!("[alice] waiting {}s for GossipSub mesh…", T_MESH.as_secs());
    drain(&mut swarm, T_MESH).await;

    // ── Publish two messages ──────────────────────────────────────────────────
    for msg in [ALICE_MSG_1, ALICE_MSG_2] {
        info!("[alice] → \"{msg}\"");
        swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), msg.as_bytes())
            .map_err(|e| anyhow::anyhow!("alice publish \"{msg}\": {:?}", e))?;
        // Small gap so the two messages arrive as separate events at Bob
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // ── Wait for Bob's reply ──────────────────────────────────────────────────
    info!("[alice] waiting for Bob's reply…");
    let _ = timeout(T_MSG, async {
        loop {
            if results.lock().unwrap().alice_got_reply {
                break;
            }
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message { message, .. },
                )) => {
                    let text = String::from_utf8_lossy(&message.data).to_string();
                    info!("[alice] ← received on {}: \"{text}\"", message.topic);
                    if text == BOB_REPLY {
                        info!("[alice] ✓ got Bob's reply");
                        results.lock().unwrap().alice_got_reply = true;
                        break;
                    }
                }
                _ => {}
            }
        }
    })
    .await;

    if !results.lock().unwrap().alice_got_reply {
        warn!("[alice] did not receive Bob's reply within timeout");
    }
    Ok(())
}

// ── Bob ───────────────────────────────────────────────────────────────────────
// Dials node-2 (different bootstrap than Alice), subscribes to GROUP,
// receives both of Alice's messages, sends one reply.

async fn run_bob(
    n2_peer_id: PeerId,
    n2_addr: Multiaddr,
    results: Arc<Mutex<Results>>,
) -> Result<()> {
    let key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(key.public());
    info!("[bob]   peer_id = {peer_id}");

    let mut swarm = build_swarm(&key)?;
    swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?;

    let topic = gossipsub::IdentTopic::new(GROUP);
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic)
        .map_err(|e| anyhow::anyhow!("bob subscribe: {e}"))?;

    // Bootstrap via node-2 (different entry point than Alice)
    swarm
        .behaviour_mut()
        .kademlia
        .add_address(&n2_peer_id, n2_addr.clone());
    let _ = swarm.behaviour_mut().kademlia.bootstrap();
    swarm.dial(n2_addr)?;

    // Wait until connected to node-2
    info!("[bob]   connecting to node-2…");
    timeout(T_CONNECT, async {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id: p, .. } => {
                    info!("[bob]   connected to {p}");
                    break;
                }
                SwarmEvent::OutgoingConnectionError { error, .. } => {
                    error!("[bob]   connection error: {error}");
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("[bob]   listening on {address}");
                }
                _ => {}
            }
        }
    })
    .await
    .map_err(|_| anyhow::anyhow!("bob: timed out connecting to node-2"))?;

    // ── Receive messages ──────────────────────────────────────────────────────
    info!("[bob]   listening for group messages…");
    let mut replied = false;

    let _ = timeout(Duration::from_secs(50), async {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id: p, .. } => {
                    info!("[bob]   connected to {p}");
                }
                SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(
                    gossipsub::Event::Subscribed { peer_id, topic },
                )) => {
                    info!("[bob]   peer {peer_id} subscribed to {topic}");
                }
                SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message {
                        propagation_source,
                        message,
                        ..
                    },
                )) => {
                    let text = String::from_utf8_lossy(&message.data).to_string();
                    info!(
                        "[bob]   ← received on {} from {propagation_source}: \"{text}\"",
                        message.topic
                    );

                    {
                        let mut r = results.lock().unwrap();
                        if text == ALICE_MSG_1 {
                            info!("[bob]   ✓ got alice_msg_1");
                            r.bob_got_msg1 = true;
                        } else if text == ALICE_MSG_2 {
                            info!("[bob]   ✓ got alice_msg_2");
                            r.bob_got_msg2 = true;
                        }
                    }

                    // Reply once, as soon as the first message arrives
                    if !replied {
                        replied = true;
                        info!("[bob]   → replying: \"{BOB_REPLY}\"");
                        if let Err(e) = swarm
                            .behaviour_mut()
                            .gossipsub
                            .publish(topic.clone(), BOB_REPLY.as_bytes())
                        {
                            warn!("[bob]   publish reply failed: {:?}", e);
                        }
                    }

                    let r = results.lock().unwrap();
                    if r.bob_got_msg1 && r.bob_got_msg2 {
                        break;
                    }
                }
                SwarmEvent::Behaviour(NodeBehaviourEvent::Kademlia(
                    kad::Event::RoutingUpdated { peer, .. },
                )) => {
                    info!("[bob]   DHT routing updated: {peer}");
                }
                _ => {}
            }
        }

        // Stay alive a bit longer so Alice can receive the reply
        drain(&mut swarm, Duration::from_secs(10)).await;
    })
    .await;

    let r = results.lock().unwrap();
    if !r.bob_got_msg1 {
        warn!("[bob] did not receive alice_msg_1");
    }
    if !r.bob_got_msg2 {
        warn!("[bob] did not receive alice_msg_2");
    }
    Ok(())
}

// ── Shared helpers ─────────────────────────────────────────────────────────────

fn build_swarm(key: &identity::Keypair) -> Result<Swarm<NodeBehaviour>> {
    let peer_id = PeerId::from(key.public());
    Ok(Swarm::new(
        make_transport(key)?,
        make_behaviour(key)?,
        peer_id,
        libp2p::swarm::Config::with_tokio_executor()
            .with_idle_connection_timeout(Duration::from_secs(60)),
    ))
}

fn make_transport(key: &identity::Keypair) -> Result<Boxed<(PeerId, StreamMuxerBox)>> {
    Ok(tcp::tokio::Transport::new(tcp::Config::default())
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise::Config::new(key)?)
        .multiplex(yamux::Config::default())
        .boxed())
}

fn make_behaviour(key: &identity::Keypair) -> Result<NodeBehaviour> {
    let peer_id = PeerId::from(key.public());

    let kademlia = kad::Behaviour::with_config(
        peer_id,
        kad::store::MemoryStore::new(peer_id),
        kad::Config::default(),
    );

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1)) // fast mesh formation in tests
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .map_err(|e| anyhow::anyhow!("gossipsub config: {e}"))?;

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(key.clone()),
        gossipsub_config,
    )
    .map_err(|e| anyhow::anyhow!("gossipsub: {e}"))?;

    Ok(NodeBehaviour {
        kademlia,
        gossipsub,
        identify: identify::Behaviour::new(identify::Config::new(
            "/gigi/1.0.0".to_string(),
            key.public(),
        )),
        ping: ping::Behaviour::new(ping::Config::new()),
        relay: relay::Behaviour::new(peer_id, Default::default()),
    })
}

/// Drive the swarm for `duration`, discarding all events.
async fn drain(swarm: &mut Swarm<NodeBehaviour>, duration: Duration) {
    let _ = timeout(duration, async {
        loop {
            swarm.select_next_some().await;
        }
    })
    .await;
}

fn pass(ok: bool) -> &'static str {
    if ok {
        "PASS ✓"
    } else {
        "FAIL ✗"
    }
}
