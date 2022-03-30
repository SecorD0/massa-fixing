use dashmap::{mapref::entry::Entry, DashMap};
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
    blocks: Arc<DashMap<BlockId, Arc<RwLock<StoredBlock>>, BuildMap<BlockId>>>,
}

impl Storage {
    /// Store a block, along with it's serialized representation.
    pub fn store_block(&self, block_id: BlockId, block: Block, serialized: Vec<u8>) {
        match self.blocks.entry(block_id) {
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                let stored_block = StoredBlock {
                    block,
                    serialized,
                    serialized_header: None,
                };
                let to_store = Arc::new(RwLock::new(stored_block));
                entry.insert(to_store);
            }
        }
    }

    /// Get a (mutable) reference to the stored block.
    pub fn retrieve_block(&self, block_id: &BlockId) -> Option<Arc<RwLock<StoredBlock>>> {
        self.blocks.get(block_id).map(|v| Arc::clone(v.value()))
    }

    /// Remove a list of blocks from storage.
    pub fn remove_blocks(&self, block_ids: &[BlockId]) {
        for id in block_ids {
            self.blocks.remove(id);
        }
    }
}
