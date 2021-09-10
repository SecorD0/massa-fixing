// Copyright (c) 2021 MASSA LABS <info@massa.net>

use super::mock_network_controller::MockNetworkController;
use crate::common::NodeId;
use crate::network::NetworkCommand;
use crate::protocol::{
    start_protocol_controller, ProtocolCommandSender, ProtocolConfig, ProtocolEvent,
    ProtocolEventReceiver, ProtocolManager, ProtocolPoolEvent, ProtocolPoolEventReceiver,
};
use crypto::{
    hash::Hash,
    signature::{derive_public_key, generate_random_private_key, PrivateKey, PublicKey},
};
use futures::Future;
use models::{
    Address, Amount, Block, BlockHeader, BlockHeaderContent, BlockId, SerializeCompact, Slot,
};
use models::{Endorsement, EndorsementContent, Operation, OperationContent, OperationType};
use std::collections::HashMap;
use time::UTime;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub private_key: PrivateKey,
    pub id: NodeId,
}

pub fn create_node() -> NodeInfo {
    let private_key = generate_random_private_key();
    let id = NodeId(derive_public_key(&private_key));
    NodeInfo { private_key, id }
}

pub async fn create_and_connect_nodes(
    num: usize,
    network_controller: &mut MockNetworkController,
) -> Vec<NodeInfo> {
    let mut nodes = vec![];
    for _ in 0..num {
        // Create a node, and connect it with protocol.
        let info = create_node();
        network_controller.new_connection(info.id).await;
        nodes.push(info);
    }
    nodes
}

/// Creates a block for use in protocol,
/// without paying attention to consensus related things
/// like slot, parents, and merkle root.
pub fn create_block(private_key: &PrivateKey, public_key: &PublicKey) -> Block {
    let (_, header) = BlockHeader::new_signed(
        private_key,
        BlockHeaderContent {
            creator: public_key.clone(),
            slot: Slot::new(1, 0),
            parents: vec![
                BlockId(Hash::hash("Genesis 0".as_bytes())),
                BlockId(Hash::hash("Genesis 1".as_bytes())),
            ],
            operation_merkle_root: Hash::hash(&Vec::new()),
            endorsements: Vec::new(),
        },
    )
    .unwrap();

    Block {
        header,
        operations: Vec::new(),
    }
}

pub fn create_block_with_operations(
    private_key: &PrivateKey,
    public_key: &PublicKey,
    slot: Slot,
    operations: Vec<Operation>,
) -> Block {
    let operation_merkle_root = Hash::hash(
        &operations.iter().fold(Vec::new(), |acc, v| {
            let res = [acc, v.get_operation_id().unwrap().to_bytes().to_vec()].concat();
            res
        })[..],
    );
    let (_, header) = BlockHeader::new_signed(
        private_key,
        BlockHeaderContent {
            creator: public_key.clone(),
            slot,
            parents: vec![
                BlockId(Hash::hash("Genesis 0".as_bytes())),
                BlockId(Hash::hash("Genesis 1".as_bytes())),
            ],
            operation_merkle_root,
            endorsements: Vec::new(),
        },
    )
    .unwrap();

    Block { header, operations }
}

pub async fn send_and_propagate_block(
    network_controller: &mut MockNetworkController,
    block: Block,
    valid: bool,
    source_node_id: NodeId,
    protocol_event_receiver: &mut ProtocolEventReceiver,
) {
    let expected_hash = block.header.compute_block_id().unwrap();

    // Send block to protocol.
    network_controller.send_block(source_node_id, block).await;

    // Check protocol sends block to consensus.
    let hash = match wait_protocol_event(protocol_event_receiver, 1000.into(), |evt| match evt {
        evt @ ProtocolEvent::ReceivedBlock { .. } => Some(evt),
        _ => None,
    })
    .await
    {
        Some(ProtocolEvent::ReceivedBlock { block_id, .. }) => Some(block_id),
        None => None,
        _ => panic!("Unexpected or no protocol event."),
    };
    if valid {
        assert_eq!(
            expected_hash,
            hash.expect("block not propagated before timeout")
        );
    } else {
        assert!(hash.is_none(), "unexpected protocol event")
    }
}

/// Creates an operation for use in protocol tests,
/// without paying attention to consensus related things.
pub fn create_operation() -> Operation {
    let sender_priv = crypto::generate_random_private_key();
    let sender_pub = crypto::derive_public_key(&sender_priv);

    let recv_priv = crypto::generate_random_private_key();
    let recv_pub = crypto::derive_public_key(&recv_priv);

    let op = OperationType::Transaction {
        recipient_address: Address::from_public_key(&recv_pub).unwrap(),
        amount: Amount::default(),
    };
    let content = OperationContent {
        fee: Amount::default(),
        op,
        sender_public_key: sender_pub,
        expire_period: 0,
    };
    let hash = Hash::hash(&content.to_bytes_compact().unwrap());
    let signature = crypto::sign(&hash, &sender_priv).unwrap();

    Operation { content, signature }
}

/// Creates an endorsement for use in protocol tests,
/// without paying attention to consensus related things.
pub fn create_endorsement() -> Endorsement {
    let sender_priv = crypto::generate_random_private_key();
    let sender_public_key = crypto::derive_public_key(&sender_priv);

    let content = EndorsementContent {
        sender_public_key,
        slot: Slot::new(10, 1),
        index: 0,
        endorsed_block: BlockId(Hash::hash(&[])),
    };
    let hash = Hash::hash(&content.to_bytes_compact().unwrap());
    let signature = crypto::sign(&hash, &sender_priv).unwrap();
    Endorsement {
        content: content.clone(),
        signature,
    }
}

pub fn create_operation_with_expire_period(
    sender_priv: PrivateKey,
    sender_pub: PublicKey,
    expire_period: u64,
) -> Operation {
    let recv_priv = crypto::generate_random_private_key();
    let recv_pub = crypto::derive_public_key(&recv_priv);

    let op = OperationType::Transaction {
        recipient_address: Address::from_public_key(&recv_pub).unwrap(),
        amount: Amount::default(),
    };
    let content = OperationContent {
        fee: Amount::default(),
        op,
        sender_public_key: sender_pub,
        expire_period,
    };
    let hash = Hash::hash(&content.to_bytes_compact().unwrap());
    let signature = crypto::sign(&hash, &sender_priv).unwrap();

    Operation { content, signature }
}

// create a ProtocolConfig with typical values
pub fn create_protocol_config() -> ProtocolConfig {
    // Init the serialization context with a default,
    // can be overwritten with a more specific one in the test.
    models::init_serialization_context(models::SerializationContext {
        max_block_operations: 1024,
        parent_count: 2,
        max_peer_list_length: 128,
        max_message_size: 3 * 1024 * 1024,
        max_block_size: 3 * 1024 * 1024,
        max_bootstrap_blocks: 100,
        max_bootstrap_cliques: 100,
        max_bootstrap_deps: 100,
        max_bootstrap_children: 100,
        max_ask_blocks_per_message: 10,
        max_operations_per_message: 1024,
        max_endorsements_per_message: 1024,
        max_bootstrap_message_size: 100000000,
        max_bootstrap_pos_entries: 1000,
        max_bootstrap_pos_cycles: 5,
        max_block_endorsments: 8,
    });

    ProtocolConfig {
        ask_block_timeout: 500.into(),
        max_node_known_blocks_size: 100,
        max_node_wanted_blocks_size: 100,
        max_simultaneous_ask_blocks_per_node: 10,
        max_send_wait: UTime::from(100),
        max_known_ops_size: 1000,
        max_known_endorsements_size: 1000,
        max_known_operations_size: 1000,
    }
}

pub async fn wait_protocol_event<F>(
    protocol_event_receiver: &mut ProtocolEventReceiver,
    timeout: UTime,
    filter_map: F,
) -> Option<ProtocolEvent>
where
    F: Fn(ProtocolEvent) -> Option<ProtocolEvent>,
{
    let timer = sleep(timeout.into());
    tokio::pin!(timer);
    loop {
        tokio::select! {
            evt_opt = protocol_event_receiver.wait_event() => match evt_opt {
                Ok(orig_evt) => if let Some(res_evt) = filter_map(orig_evt) { return Some(res_evt); },
                _ => return None
            },
            _ = &mut timer => return None
        }
    }
}

pub async fn wait_protocol_pool_event<F>(
    protocol_event_receiver: &mut ProtocolPoolEventReceiver,
    timeout: UTime,
    filter_map: F,
) -> Option<ProtocolPoolEvent>
where
    F: Fn(ProtocolPoolEvent) -> Option<ProtocolPoolEvent>,
{
    let timer = sleep(timeout.into());
    tokio::pin!(timer);
    loop {
        tokio::select! {
            evt_opt = protocol_event_receiver.wait_event() => match evt_opt {
                Ok(orig_evt) => if let Some(res_evt) = filter_map(orig_evt) { return Some(res_evt); },
                _ => return None
            },
            _ = &mut timer => return None
        }
    }
}

pub async fn assert_hash_asked_to_node(
    hash_1: BlockId,
    node_id: NodeId,
    network_controller: &mut MockNetworkController,
) {
    let ask_for_block_cmd_filter = |cmd| match cmd {
        NetworkCommand::AskForBlocks { list } => Some(list),
        _ => None,
    };
    let list = network_controller
        .wait_command(1000.into(), ask_for_block_cmd_filter)
        .await
        .expect("Hash not asked for before timer.");

    assert!(list.get(&node_id).unwrap().contains(&hash_1));
}

pub async fn asked_list(
    network_controller: &mut MockNetworkController,
) -> HashMap<NodeId, Vec<BlockId>> {
    let ask_for_block_cmd_filter = |cmd| match cmd {
        NetworkCommand::AskForBlocks { list } => Some(list),
        _ => None,
    };
    network_controller
        .wait_command(1000.into(), ask_for_block_cmd_filter)
        .await
        .expect("Hash not asked for before timer.")
}

pub async fn assert_banned_node(node_id: NodeId, network_controller: &mut MockNetworkController) {
    let banned_node = network_controller
        .wait_command(1000.into(), |cmd| match cmd {
            NetworkCommand::Ban(node) => Some(node),
            _ => None,
        })
        .await
        .expect("Node not banned before timeout.");
    assert_eq!(banned_node, node_id);
}

pub async fn assert_banned_nodes(
    mut nodes: Vec<NodeId>,
    network_controller: &mut MockNetworkController,
) {
    let timer = sleep(UTime::from(5000).into());
    tokio::pin!(timer);
    loop {
        tokio::select! {
            msg = network_controller
                   .wait_command(1000.into(), |cmd| match cmd {
                       NetworkCommand::Ban(node) => Some(node),
                       _ => None,
                   })
             =>  {
                 let banned_node = msg.expect("Nodes not banned before timeout.");
                 nodes.drain_filter(|id| *id == banned_node);
                 if nodes.is_empty() {
                     break;
                 }
            },
            _ = &mut timer => panic!("Nodes not banned before timeout.")
        }
    }
}

pub async fn protocol_test<F, V>(cfg: ProtocolConfig, test: F)
where
    F: FnOnce(
        MockNetworkController,
        ProtocolEventReceiver,
        ProtocolCommandSender,
        ProtocolManager,
        ProtocolPoolEventReceiver,
    ) -> V,
    V: Future<
        Output = (
            MockNetworkController,
            ProtocolEventReceiver,
            ProtocolCommandSender,
            ProtocolManager,
            ProtocolPoolEventReceiver,
        ),
    >,
{
    let (network_controller, network_command_sender, network_event_receiver) =
        MockNetworkController::new();

    // start protocol controller
    let (
        protocol_command_sender,
        protocol_event_receiver,
        protocol_pool_event_receiver,
        protocol_manager,
    ) = start_protocol_controller(
        cfg.clone(),
        5u64,
        network_command_sender,
        network_event_receiver,
    )
    .await
    .expect("could not start protocol controller");

    let (
        _network_controller,
        protocol_event_receiver,
        _protocol_command_sender,
        protocol_manager,
        protocol_pool_event_receiver,
    ) = test(
        network_controller,
        protocol_event_receiver,
        protocol_command_sender,
        protocol_manager,
        protocol_pool_event_receiver,
    )
    .await;

    protocol_manager
        .stop(protocol_event_receiver, protocol_pool_event_receiver)
        .await
        .expect("Failed to shutdown protocol.");
}
