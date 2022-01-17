// Copyright (c) 2021 MASSA LABS <info@massa.net>

use core::usize;
use std::convert::TryInto;

use serde::{Deserialize, Serialize};

use crate::prehash::{BuildMap, Set};
use crate::settings::BLOCK_ID_SIZE_BYTES;
use crate::{
    array_from_slice, u8_from_slice, with_serialization_context, BlockId, DeserializeCompact,
    DeserializeVarInt, ModelsError, SerializeCompact, SerializeVarInt,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Clique {
    pub block_ids: Set<BlockId>,
    pub fitness: u64,
    pub is_blockclique: bool,
}

impl SerializeCompact for Clique {
    /// ## Example
    /// ```rust
    /// use massa_models::clique::Clique;
    /// # use massa_models::{SerializeCompact, DeserializeCompact, SerializationContext, BlockId};
    /// # use massa_hash::hash::Hash;
    /// # use std::str::FromStr;
    /// # massa_models::init_serialization_context(massa_models::SerializationContext {
    /// #     max_block_operations: 1024,
    /// #     parent_count: 2,
    /// #     max_peer_list_length: 128,
    /// #     max_message_size: 3 * 1024 * 1024,
    /// #     max_block_size: 3 * 1024 * 1024,
    /// #     max_bootstrap_blocks: 100,
    /// #     max_bootstrap_cliques: 100,
    /// #     max_bootstrap_deps: 100,
    /// #     max_bootstrap_children: 100,
    /// #     max_ask_blocks_per_message: 10,
    /// #     max_operations_per_message: 1024,
    /// #     max_endorsements_per_message: 1024,
    /// #     max_bootstrap_message_size: 100000000,
    /// #     max_bootstrap_pos_cycles: 10000,
    /// #     max_bootstrap_pos_entries: 10000,
    /// #     max_block_endorsements: 8,
    /// # });
    /// # pub fn get_dummy_block_id(s: &str) -> BlockId {
    /// #     BlockId(Hash::compute_from(s.as_bytes()))
    /// # }
    /// let clique = Clique {
    ///         block_ids: vec![get_dummy_block_id("parent1"), get_dummy_block_id("parent2")].into_iter().collect(),
    ///         fitness: 123,
    ///         is_blockclique: true,
    ///     };
    /// let bytes = clique.clone().to_bytes_compact().unwrap();
    /// let (res, _) = Clique::from_bytes_compact(&bytes).unwrap();
    /// assert_eq!(clique.block_ids, res.block_ids);
    /// assert_eq!(clique.is_blockclique, res.is_blockclique);
    /// assert_eq!(clique.fitness, res.fitness);
    /// ```
    ///
    /// Checks performed:
    /// - Number of blocks.
    fn to_bytes_compact(&self) -> Result<Vec<u8>, crate::ModelsError> {
        let mut res: Vec<u8> = Vec::new();

        // block_ids
        let block_ids_count: u32 = self.block_ids.len().try_into().map_err(|err| {
            ModelsError::SerializeError(format!("too many blocks in in clique: {}", err))
        })?;
        res.extend(&block_ids_count.to_varint_bytes());
        for b_id in self.block_ids.iter() {
            res.extend(&b_id.to_bytes());
        }

        // fitness
        res.extend(&self.fitness.to_varint_bytes());

        // is_blockclique
        res.push(if self.is_blockclique { 1u8 } else { 0u8 });

        Ok(res)
    }
}

/// Checks performed:
/// - Number of blocks.
/// - Validity of block ids.
/// - Validity of fitness.
/// - Validity of the `is_blockclique` flag.
impl DeserializeCompact for Clique {
    fn from_bytes_compact(buffer: &[u8]) -> Result<(Self, usize), crate::ModelsError> {
        let mut cursor = 0usize;
        let max_bootstrap_blocks =
            with_serialization_context(|context| context.max_bootstrap_blocks);

        let (block_count, delta) = u32::from_varint_bytes(&buffer[cursor..])?;
        if block_count > max_bootstrap_blocks {
            return Err(ModelsError::DeserializeError(
                "too many blocks in clique for deserialization".to_string(),
            ));
        }
        cursor += delta;
        let mut block_ids =
            Set::<BlockId>::with_capacity_and_hasher(block_count as usize, BuildMap::default());
        for _ in 0..block_count {
            let b_id = BlockId::from_bytes(&array_from_slice(&buffer[cursor..])?)?;
            cursor += BLOCK_ID_SIZE_BYTES;
            block_ids.insert(b_id);
        }

        // fitness
        let (fitness, delta) = u64::from_varint_bytes(&buffer[cursor..])?;
        cursor += delta;

        // is_blockclique
        let is_blockclique = match u8_from_slice(&buffer[cursor..])? {
            0u8 => false,
            1u8 => true,
            _ => {
                return Err(ModelsError::SerializeError(
                    "could not deserialize active_block.production_events.has_created".into(),
                ))
            }
        };
        cursor += 1;

        Ok((
            Clique {
                block_ids,
                fitness,
                is_blockclique,
            },
            cursor,
        ))
    }
}
