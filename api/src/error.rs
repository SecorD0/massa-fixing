use displaydoc::Display;
use thiserror::Error;

use consensus::ConsensusError;
use crypto::CryptoError;
use models::ModelsError;
use network::NetworkError;
use pool::PoolError;
use storage::StorageError;
use time::TimeError;

#[non_exhaustive]
#[derive(Display, Error, Debug)]
pub enum PublicApiError {
    /// send  channel error: {0}
    SendChannelError(String),
    /// receive  channel error: {0}
    ReceiveChannelError(String),
    /// crypto error : {0}
    CryptoError(#[from] CryptoError),
    /// consensus error : {0}
    ConsensusError(#[from] ConsensusError),
    /// network error : {0}
    NetworkError(#[from] NetworkError),
    /// models error : {0}
    ModelsError(#[from] ModelsError),
    /// time error : {0}
    TimeError(#[from] TimeError),
    /// pool error : {0}
    PoolError(#[from] PoolError),
    /// storage error : {0}
    StorageError(#[from] StorageError),
    /// not found
    NotFound,
    /// inconsistency: {0}
    InconsistencyError(String),
    /// missing command sender {0}
    MissingCommandSender(String),
    /// missing config {0}
    MissingConfig(String),
}

impl From<PublicApiError> for jsonrpc_core::Error {
    fn from(err: PublicApiError) -> Self {
        jsonrpc_core::Error {
            code: jsonrpc_core::ErrorCode::ServerError(500),
            message: err.to_string(),
            data: None,
        }
    }
}

#[non_exhaustive]
#[derive(Display, Error, Debug)]
pub enum PrivateApiError {
    /// send  channel error: {0}
    SendChannelError(String),
    /// receive  channel error: {0}
    ReceiveChannelError(String),
    /// crypto error : {0}
    CryptoError(#[from] CryptoError),
    /// consensus error : {0}
    ConsensusError(#[from] ConsensusError),
    /// network error : {0}
    NetworkError(#[from] NetworkError),
    /// models error : {0}
    ModelsError(#[from] ModelsError),
    /// time error : {0}
    TimeError(#[from] TimeError),
    /// not found
    NotFound,
    /// inconsistency: {0}
    InconsistencyError(String),
    /// missing command sender {0}
    MissingCommandSender(String),
    /// missing config {0}
    MissingConfig(String),
}

impl From<PrivateApiError> for jsonrpc_core::Error {
    fn from(err: PrivateApiError) -> Self {
        jsonrpc_core::Error {
            code: jsonrpc_core::ErrorCode::ServerError(500),
            message: err.to_string(),
            data: None,
        }
    }
}
