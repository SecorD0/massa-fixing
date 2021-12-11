use std::sync::{Arc, Mutex};

use massa_models::{Address, BlockId, Slot};
use tracing::debug;

use crate::interface_impl::INTERFACE;
use crate::sce_ledger::{SCELedger, SCELedgerChanges};
use crate::types::{ExecutionContext, ExecutionStep, OperationSC, StepHistory};
use crate::ExecutionConfig;

lazy_static::lazy_static! {
    pub(crate) static ref CONTEXT: Arc<Mutex::<ExecutionContext>> = {
        let ledger = SCELedger::default(); // TODO Bootstrap
        Arc::new(Mutex::new(ExecutionContext::new(ledger)))
    };
}

pub struct VM {
    _cfg: ExecutionConfig,
    step_history: StepHistory,
}

impl VM {
    pub fn new(_cfg: ExecutionConfig) -> VM {
        VM {
            _cfg,
            step_history: Default::default(),
        }
    }

    /// runs an SCE-final execution step
    /// # Parameters
    ///   * step: execution step to run
    pub(crate) fn run_final_step(&mut self, step: &ExecutionStep) {
        if let Some(cached) = self.is_already_done(step) {
            // execution was already done, apply cached ledger changes to final ledger
            let context = CONTEXT.lock().unwrap();
            let mut final_ledger_guard = context.ledger_step.final_ledger.lock().unwrap();
            (*final_ledger_guard).apply_changes(&cached);
            return;
        }
        // nothing found in cache, or cache mismatch: reset history, run step and make it final
        // this should almost never happen, so the heavy step.clone() is OK
        self.step_history.clear();
        self.run_active_step(step);

        if let Some(cached) = self.is_already_done(step) {
            // execution was already done, apply cached ledger changes to final ledger
            // It should always happen
            let context = CONTEXT.lock().unwrap();
            let mut final_ledger_guard = context.ledger_step.final_ledger.lock().unwrap();
            (*final_ledger_guard).apply_changes(&cached);
        }
    }

    fn is_already_done(&mut self, step: &ExecutionStep) -> Option<SCELedgerChanges> {
        // check if step already in history front
        if let Some((slot, opt_block, ledger_changes)) = self.step_history.pop_front() {
            if slot == step.slot {
                match (&opt_block, &step.block) {
                    (None, None) => Some(ledger_changes), // matching miss
                    (Some(b_id_hist), Some((b_id_step, _b_step))) => {
                        if b_id_hist == b_id_step {
                            Some(ledger_changes) // matching block
                        } else {
                            None // block mismatch
                        }
                    }
                    (None, Some(_)) => None, // miss/block mismatch
                    (Some(_), None) => None, // block/miss mismatch
                }
            } else {
                None // slot mismatch
            }
        } else {
            None
        }
    }

    fn clear_and_update_context(&self) {
        let mut context = CONTEXT.lock().unwrap();
        context.ledger_step.caused_changes.clear();
        context.ledger_step.cumulative_history_changes =
            SCELedgerChanges::from(self.step_history.clone());
    }

    /// Prepare (update) the shared context before the new operation
    /// TODO: do not ignore the results
    /// TODO consider dispatching with edorsers/endorsed as well
    fn prepare_context(
        &self,
        operation: &OperationSC,
        block_creator_addr: Address,
        block_id: BlockId,
        slot: Slot,
    ) -> SCELedgerChanges {
        let mut context = CONTEXT.lock().unwrap();
        // credit the sender with "coins"
        let _result =
            context
                .ledger_step
                .set_balance_delta(operation.sender, operation.coins, true);
        // credit the block creator with max_gas*gas_price
        let _result = context.ledger_step.set_balance_delta(
            block_creator_addr,
            operation
                .gas_price
                .checked_mul_u64(operation.max_gas)
                .unwrap(),
            true,
        );
        // Save the Initial ledger changes before execution
        // It contains a copy of the initial coin credits that will be popped back if bytecode execution fails in order to cancel its effects

        // fill context for execution
        context.gas_price = operation.gas_price;
        context.max_gas = operation.max_gas;
        context.coins = operation.coins;
        context.slot = slot;
        context.opt_block_id = Some(block_id);
        context.opt_block_creator_addr = Some(block_creator_addr);
        context.call_stack = vec![operation.sender].into();
        context.ledger_step.caused_changes.clone()
    }

    /// runs an SCE-active execution step
    ///
    /// 1. Get step history (cache of final ledger changes by slot and block_id history)
    /// 2. clear caused changes
    /// 3. accumulated step history
    /// 4. Execute each block of each operation
    ///
    /// # Parameters
    ///   * step: execution step to run
    pub(crate) fn run_active_step(&mut self, step: &ExecutionStep) {
        // accumulate active ledger changes history
        self.clear_and_update_context();

        // run implicit and async calls
        // TODO

        // run explicit calls within the block (if the slot is not a miss)
        // note that total block gas is not checked, because currently Protocol makes the block invalid if it overflows gas
        let opt_block_id: Option<BlockId>;
        if let Some((block_id, block)) = &step.block {
            opt_block_id = Some(*block_id);

            // get block creator addr
            let block_creator_addr =
                Address::from_public_key(&block.header.content.creator).unwrap();
            // run all operations
            for (op_idx, operation) in block.operations.clone().into_iter().enumerate() {
                let operation_sc = OperationSC::try_from(operation.content);
                if operation_sc.is_err() {
                    // only fail if the operation cannot parse the sender address
                    continue;
                }
                let operation = &operation_sc.unwrap();
                let ledger_changes_backup =
                    self.prepare_context(operation, block_creator_addr, *block_id, step.slot);

                let run_result =
                    assembly_simulator::run(&operation._module, operation.max_gas, &INTERFACE);
                if let Err(err) = run_result {
                    debug!(
                        "failed running bytecode in operation index {} in block {}: {}",
                        op_idx, block_id, err
                    );
                    // cancel the effects of execution only, pop back init_changes
                    let mut context = CONTEXT.lock().unwrap();
                    context.ledger_step.caused_changes = ledger_changes_backup;
                }
            }
        } else {
            // There is no block for this step, miss
            opt_block_id = None;
        }

        let context = CONTEXT.lock().unwrap();
        // push step into history
        self.step_history.push_back((
            step.slot,
            opt_block_id,
            context.ledger_step.caused_changes.clone(),
        ))
    }

    pub fn reset_to_final(&mut self) {
        self.step_history.clear();
    }
}
