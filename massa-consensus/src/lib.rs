// Copyright (c) 2021 MASSA LABS <info@massa.net>

#![doc = include_str!("../endorsements.md")]
#![feature(async_closure)]
#![feature(bool_to_option)]
#![feature(hash_drain_filter)]
#![feature(map_first_last)]
#![feature(int_roundings)]

#[macro_use]
extern crate massa_logging;

pub use block_graph::BootstrapableGraph;
pub use block_graph::{
    BlockGraphExport, DiscardReason, ExportActiveBlock, ExportBlockStatus, ExportCompiledBlock,
    LedgerDataExport, Status,
};
pub use consensus_controller::{
    start_consensus_controller, ConsensusChannels, ConsensusCommandSender, ConsensusEventReceiver,
    ConsensusManager,
};
pub use consensus_worker::{ConsensusCommand, ConsensusEvent};
pub use error::ConsensusError;
pub use ledger::LedgerSubset;
pub use pos::{ExportProofOfStake, RollCounts, RollUpdate, RollUpdates, ThreadCycleState};
pub use settings::{ConsensusConfig, ConsensusSettings};

mod block_graph;
mod consensus_controller;
mod consensus_worker;
pub mod error;
pub mod ledger;
mod pos;
pub mod settings;
#[cfg(test)]
mod tests;
