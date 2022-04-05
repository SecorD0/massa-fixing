// Copyright (c) 2022 MASSA LABS <info@massa.net>
//! Log utilities

#![warn(missing_docs)]

use tracing::{enabled, Level};

#[macro_export]
/// tracing with some context
macro_rules! massa_trace {
    if enabled!(Level::TRACE) {
        ($evt:expr, $params:tt) => {
            tracing::trace!("massa:{}:{}", $evt, serde_json::json!($params));
        };
    }
}
