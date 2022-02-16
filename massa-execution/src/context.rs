use crate::sce_ledger::SCELedgerChanges;
use crate::speculative_ledger::SpeculativeLedger;
use crate::types::ExecutionStackElement;
use crate::{event_store::EventStore, ExecutionError};
use massa_ledger::{FinalLedger, LedgerChanges};
use massa_models::{Address, Amount, BlockId, OperationId, Slot};
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

pub(crate) struct ExecutionContext {
    // speculative ledger
    speculative_ledger: SpeculativeLedger,

    /// max gas for this execution
    pub max_gas: u64,

    /// gas price of the execution
    pub gas_price: Amount,

    /// slot at which the execution happens
    pub slot: Slot,

    /// counter of newly created addresses so far during this execution
    pub created_addr_index: u64,

    /// counter of newly created events so far during this execution
    pub created_event_index: u64,

    /// block ID, if one is present at this slot
    pub opt_block_id: Option<BlockId>,

    /// block creator addr, if there is a block at this slot
    pub opt_block_creator_addr: Option<Address>,

    /// address call stack, most recent is at the back
    pub stack: Vec<ExecutionStackElement>,

    /// True if it's a read-only context
    pub read_only: bool,

    /// geerated events during this execution, with multiple indexes
    pub events: EventStore,

    /// Unsafe RNG state
    pub unsafe_rng: Xoshiro256PlusPlus,

    /// origin operation id
    pub origin_operation_id: Option<OperationId>,
}

impl ExecutionContext {
    pub(crate) fn new(final_ledger: Arc<RwLock<FinalLedger>>) -> Self {
        ExecutionContext {
            speculative_ledger: SpeculativeLedger::new(final_ledger, Default::default()),
            max_gas: Default::default(),
            gas_price: Default::default(),
            slot: Slot::new(0, 0),
            created_addr_index: Default::default(),
            created_event_index: Default::default(),
            opt_block_id: Default::default(),
            opt_block_creator_addr: Default::default(),
            stack: Default::default(),
            read_only: Default::default(),
            events: Default::default(),
            unsafe_rng: Xoshiro256PlusPlus::from_seed([0u8; 32]),
            origin_operation_id: Default::default(),
        }
    }

    /// gets the address at the top of the stack
    pub fn get_current_address(&self) -> Result<Address, ExecutionError> {
        match self.stack.last() {
            Some(addr) => Ok(addr.address),
            _ => {
                return Err(ExecutionError::RuntimeError(
                    "failed to read current address: call stack empty".into(),
                ))
            }
        }
    }

    /// gets the current list of owned addresses (top of the stack)
    /// ordering is conserved for determinism
    pub fn get_current_owned_addresses(&self) -> Result<Vec<Address>, ExecutionError> {
        match self.stack.last() {
            Some(v) => Ok(v.owned_addresses.clone()),
            None => {
                return Err(ExecutionError::RuntimeError(
                    "failed to read current owned addresses list: call stack empty".into(),
                ))
            }
        }
    }

    /// gets the current call coins
    pub fn get_current_call_coins(&self) -> Result<Amount, ExecutionError> {
        match self.stack.last() {
            Some(v) => Ok(v.coins),
            None => {
                return Err(ExecutionError::RuntimeError(
                    "failed to read current call coins: call stack empty".into(),
                ))
            }
        }
    }

    /// gets the call stack (addresses)
    pub fn get_call_stack(&self) -> Vec<Address> {
        self.stack.iter().map(|v| v.address).collect()
    }

    /// check whether the context grants write access on a given address
    pub fn has_write_rights_on(&self, addr: &Address) -> bool {
        self.stack
            .last()
            .map_or(false, |v| v.owned_addresses.contains(&addr))
    }

    /// creates a new smart contract address with initial bytecode, within the current execution context
    pub fn create_new_sc_address(&mut self, bytecode: Vec<u8>) -> Result<Address, ExecutionError> {
        // TODO: security problem:
        //  prefix addresses to know if they are SCs or normal, otherwise people can already create new accounts by sending coins to the right hash
        //  they won't have ownership over it but this can still be a pain

        // generate address
        let (slot, created_addr_index) = (self.slot, self.created_addr_index);
        let mut data: Vec<u8> = slot.to_bytes_key().to_vec();
        data.append(&mut self.created_addr_index.to_be_bytes().to_vec());
        if self.read_only {
            data.push(0u8);
        } else {
            data.push(1u8);
        }
        let address = Address(massa_hash::hash::Hash::compute_from(&data));

        // create address in the speculative ledger
        self.speculative_ledger
            .create_new_sc_address(address, bytecode)?;

        // add to owned addresses
        match self.stack.last_mut() {
            Some(v) => {
                v.owned_addresses.push(address);
            }
            None => {
                return Err(ExecutionError::RuntimeError(
                    "owned addresses not found in context stack".into(),
                ))
            }
        };

        // increment the address creation counter at this slot
        self.created_addr_index += 1;

        Ok(address)
    }

    /// gets the bytecode of an address if it exists
    pub fn get_bytecode(&self, address: &Address) -> Option<Vec<u8>> {
        self.speculative_ledger.get_bytecode(address)
    }

    /// gets the bytecode of an address if it exists
    pub fn get_parallel_balance(&self, address: &Address) -> Option<Amount> {
        self.speculative_ledger.get_parallel_balance(address)
    }

    /// Transfers parallel coins from one address to another.
    /// No changes are retained in case of failure.
    /// Spending is only allowed from existing addresses we have write acess on
    ///
    /// # parameters
    /// * from_addr: optional spending address (use None for pure coin creation)
    /// * to_addr: optional crediting address (use None for pure coin destruction)
    /// * amount: amount of coins to transfer
    pub fn transfer_parallel_coins(
        &mut self,
        from_addr: Option<Address>,
        to_addr: Option<Address>,
        amount: Amount,
    ) -> Result<(), ExecutionError> {
        // check access rights
        if let Some(from_addr) = &from_addr {
            if !self.has_write_rights_on(from_addr) {
                return Err(ExecutionError::RuntimeError(format!(
                    "spending from address {} is not allowed in this context",
                    from_addr
                )));
            }
        }
        // do the transfer
        self.speculative_ledger
            .transfer_parallel_coins(from_addr, to_addr, amount)
    }
}
