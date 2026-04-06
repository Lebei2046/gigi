// Copyright 2024 Gigi Team.
//
// Gigi DNS - Auto-discovery protocol for local networks with nicknames

pub mod behaviour;
pub mod interface;
pub mod protocol;
pub mod types;

pub use behaviour::{GigiDnsBehaviour, GigiDnsCommand};
pub use protocol::GigiDnsProtocol;
pub use types::{
    GigiDnsConfig, GigiDnsEvent, GigiDnsRecord, GigiPeerInfo, IPV4_MDNS_MULTICAST_ADDRESS,
    IPV6_MDNS_MULTICAST_ADDRESS,
};

/// Gigi DNS service name
pub const SERVICE_NAME: &[u8] = b"_gigi-dns._udp.local";
pub const SERVICE_NAME_FQDN: &str = "_gigi-dns._udp.local.";
