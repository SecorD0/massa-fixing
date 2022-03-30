use flurry::HashMap;
use massa_models::prehash::BuildMap;
use massa_models::{Block, BlockId};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug)]
pub struct StoredBlock {
    /// The block.
    pub block: Block,
    /// The serialized representation of the block.
    pub serialized: Vec<u8>,
    /// The serialized representation of the header, if any.
    /// Note: the header is written as part of propagation of headers.
    pub serialized_header: Option<Vec<u8>>,
}

/// A storage of block, shared by various components.
#[derive(Clone, Default)]
pub struct Storage {
    blocks: Arc<HashMap<BlockId, Arc<RwLock<StoredBlock>>, BuildMap<BlockId>>>,
}

impl Storage {
    /// Store a block, along with it's serialized representation.
    pub fn store_block(&self, block_id: BlockId, block: Block, serialized: Vec<u8>) {
        let guard = self.blocks.guard();
        if !self.blocks.contains_key(&block_id, &guard) {
            let stored_block = StoredBlock {
                block,
                serialized,
                serialized_header: None,
            };
            let to_store = Arc::new(RwLock::new(stored_block));
            self.blocks.insert(block_id, to_store, &guard);
        }
    }

    /// Get a (mutable) reference to the stored block.
    pub fn retrieve_block(&self, block_id: &BlockId) -> Option<Arc<RwLock<StoredBlock>>> {
        let guard = self.blocks.guard();
        self.blocks.get(block_id, &guard).map(Arc::clone)
    }

    /// Remove a list of blocks from storage.
    pub fn remove_blocks(&self, block_ids: &[BlockId]) {
        let guard = self.blocks.guard();
        for id in block_ids {
            self.blocks.remove(id, &guard);
        }
    }
}
