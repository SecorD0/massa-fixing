use std::net::{IpAddr, Ipv4Addr};

pub const BOOTSTRAP_RANDOMNES_SIZE_BYTES: usize = 32;
pub const BASE_BOOTSTRAP_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(169, 202, 0, 10));
pub const CHANNEL_SIZE: usize = 256;
pub const HASH_SIZE_BYTES: usize = 32;
pub const PRIVATE_KEY_SIZE_BYTES: usize = 32;
pub const PUBLIC_KEY_SIZE_BYTES: usize = 33;
pub const SIGNATURE_SIZE_BYTES: usize = 64;
pub const ADDRESS_SIZE_BYTES: usize = HASH_SIZE_BYTES;
pub const AMOUNT_DECIMAL_FACTOR: u64 = 1_000_000_000;
pub const BLOCK_ID_SIZE_BYTES: usize = HASH_SIZE_BYTES;
pub const ENDORSEMENT_ID_SIZE_BYTES: usize = HASH_SIZE_BYTES;
pub const OPERATION_ID_SIZE_BYTES: usize = HASH_SIZE_BYTES;
pub const SLOT_KEY_SIZE: usize = 9;
pub const NODE_SEND_CHANNEL_SIZE: usize = 1024;
pub const HANDSHAKE_RANDOMNES_SIZE_BYTES: usize = 32;
pub const BASE_NETWORK_CONTROLLER_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(169, 202, 0, 10));
