use std::{collections::HashMap, net::IpAddr};
use massa_models::{
    composite::PubkeySig, node::NodeId, stats::NetworkStats, Block, BlockHeader, BlockId,
    Endorsement, Operation,
};
use tokio::sync::oneshot;
use crate::{BootstrapPeers, Peers};

/// Commands that the worker can execute
#[derive(Debug)]
pub enum NetworkCommand {
    /// Ask for a block from a node.
    AskForBlocks {
        list: HashMap<NodeId, Vec<BlockId>>,
    },
    /// Send that block to node.
    SendBlock {
        node: NodeId,
        block: Block,
    },
    /// Send a header to a node.
    SendBlockHeader {
        node: NodeId,
        header: BlockHeader,
    },
    // (PeerInfo, Vec <(NodeId, bool)>) peer info + list of associated Id nodes in connexion out (true)
    GetPeers(oneshot::Sender<Peers>),
    GetBootstrapPeers(oneshot::Sender<BootstrapPeers>),
    Ban(NodeId),
    BanIp(Vec<IpAddr>),
    Unban(Vec<IpAddr>),
    BlockNotFound {
        node: NodeId,
        block_id: BlockId,
    },
    /// Require to the network to send a list of operation
    SendOperations {
        node: NodeId,
        operations: HashMap<OperationId, Option<Operation>>,
    },
    SendEndorsements {
        node: NodeId,
        endorsements: Vec<Endorsement>,
    },
    NodeSignMessage {
        msg: Vec<u8>,
        response_tx: oneshot::Sender<PubkeySig>,
    },
    GetStats {
        response_tx: oneshot::Sender<NetworkStats>,
    },
}

#[derive(Debug)]
pub enum NetworkEvent {
    NewConnection(NodeId),
    ConnectionClosed(NodeId),
    /// A block was received
    ReceivedBlock {
        node: NodeId,
        block: Block,
    },
    /// A block header was received
    ReceivedBlockHeader {
        source_node_id: NodeId,
        header: BlockHeader,
    },
    /// Someone ask for block with given header hash.
    AskedForBlocks {
        node: NodeId,
        list: Vec<BlockId>,
    },
    /// That node does not have this block
    BlockNotFound {
        node: NodeId,
        block_id: BlockId,
    },
    ReceivedOperations {
        node: NodeId,
        operations: Vec<Operation>,
    },
    ReceivedEndorsements {
        node: NodeId,
        endorsements: Vec<Endorsement>,
    },
}

#[derive(Debug)]
pub enum NetworkManagementCommand {}
