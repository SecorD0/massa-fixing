// Copyright (c) 2022 MASSA LABS <info@massa.net>

use std::net::SocketAddr;
use jsonrpc_core::serde::Deserialize;

/// API settings.
/// the api settings
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct APISettings {
    /// when looking for next draw we want to look at max draw_lookahead_period_count
    pub draw_lookahead_period_count: u64,
    /// bind for the private api
    pub bind_private: SocketAddr,
    /// bind for the public api
    pub bind_public: SocketAddr,
    /// max argument count
    pub max_arguments: u64,
}
