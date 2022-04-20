use std::{collections::VecDeque, fmt::Display};

use crate::{output_event::SCOutputEvent, Slot};
use serde::{Deserialize, Serialize};

/// The result of the read-only execution.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ReadOnlyResult {
    /// An error occurred during execution.
    Error(String),
    /// The result of a successful execution.
    /// TODO: specify result.
    Ok,
}

/// The response to a request for a read-only execution.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExecuteReadOnlyResponse {
    /// The slot at which the read-only execution occurred.
    pub executed_at: Slot,
    /// The result of the read-only execution.
    pub result: ReadOnlyResult,
    /// The output events generated by the read-only execution.
    // NOTE: update doc
    pub output_events: VecDeque<SCOutputEvent>,
}

impl Display for ExecuteReadOnlyResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Executed at slot: {}", self.executed_at)?;
        writeln!(
            f,
            "Result: {}",
            match &self.result {
                ReadOnlyResult::Error(e) =>
                    format!("an error occurred during the execution: {}", e),
                ReadOnlyResult::Ok => "ok".to_string(),
            }
        )?;
        if !self.output_events.is_empty() {
            writeln!(f, "Generated events:",)?;
            for event in self.output_events.iter() {
                writeln!(f, "{}", event)?; // id already displayed in event
            }
        }
        Ok(())
    }
}
