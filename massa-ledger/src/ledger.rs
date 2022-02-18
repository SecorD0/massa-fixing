// Copyright (c) 2021 MASSA LABS <info@massa.net>

use crate::ledger_changes::LedgerChanges;
use crate::ledger_entry::LedgerEntry;
use crate::types::{Applicable, SetUpdateOrDelete};
use crate::{LedgerConfig, LedgerError};
use massa_hash::hash::Hash;
use massa_models::{Address, Amount, Slot};
use std::collections::{BTreeMap, VecDeque};

/// temporary ledger bootstrap structure
pub struct FinalLedgerBootstrapState {
    /// ledger slot
    slot: Slot,
    /// sorted ledger
    sorted_ledger: BTreeMap<Address, LedgerEntry>,
}

/// represents a final ledger
pub struct FinalLedger {
    /// ledger config
    config: LedgerConfig,
    /// slot at which the final ledger is computed
    pub slot: Slot,
    /// sorted ledger tree
    /// TODO put it on the hard drive as it can reach 1TB
    sorted_ledger: BTreeMap<Address, LedgerEntry>,
    /// history of recent final ledger changes
    /// front = oldest, back = newest
    changes_history: VecDeque<(Slot, LedgerChanges)>,
}

impl Applicable<LedgerChanges> for FinalLedger {
    /// merges LedgerChanges to the final ledger
    fn apply(&mut self, changes: LedgerChanges) {
        // for all incoming changes
        for (addr, change) in changes.0 {
            match change {
                SetUpdateOrDelete::Set(new_entry) => {
                    // inserts/overwrites the entry with an incoming absolute value
                    self.sorted_ledger.insert(addr, new_entry);
                }
                SetUpdateOrDelete::Update(entry_update) => {
                    // applies updates to an entry
                    // if the entry does not exist, inserts a default one and applies the updates to it
                    self.sorted_ledger
                        .entry(addr)
                        .or_insert_with(|| Default::default())
                        .apply(entry_update);
                }
                SetUpdateOrDelete::Delete => {
                    // deletes an entry, if it exists
                    self.sorted_ledger.remove(&addr);
                }
            }
        }
    }
}

macro_rules! init_file_error {
    ($st:expr, $cfg:ident) => {
        |err| {
            LedgerError::FileError(format!(
                "error $st initial ledger file {}: {}",
                $cfg.initial_sce_ledger_path
                    .to_str()
                    .unwrap_or("(non-utf8 path)"),
                err
            ))
        }
    };
}
pub(crate) use init_file_error;

impl FinalLedger {
    /// init from file
    pub fn new(config: LedgerConfig) -> Result<Self, LedgerError> {
        // load file
        let sorted_ledger = serde_json::from_str::<BTreeMap<Address, Amount>>(
            &std::fs::read_to_string(&config.initial_sce_ledger_path)
                .map_err(init_file_error!("loading", config))?,
        )
        .map_err(init_file_error!("parsing", config))?
        .into_iter()
        .map(|(address, balance)| {
            (
                address,
                LedgerEntry {
                    parallel_balance: balance,
                    ..Default::default()
                },
            )
        })
        .collect();

        // generate final ledger
        Ok(FinalLedger {
            slot: Slot::new(0, config.thread_count.saturating_sub(1)),
            sorted_ledger,
            changes_history: Default::default(),
            config,
        })
    }

    /// load from bootstrap
    pub fn from_bootstrap_state(config: LedgerConfig, state: FinalLedgerBootstrapState) -> Self {
        FinalLedger {
            slot: state.slot,
            sorted_ledger: state.sorted_ledger,
            changes_history: Default::default(),
            config,
        }
    }

    /// get bootstrap state
    pub fn get_bootstrap_state(&self) -> FinalLedgerBootstrapState {
        FinalLedgerBootstrapState {
            slot: self.slot,
            sorted_ledger: self.sorted_ledger.clone(),
        }
    }

    /// gets a full cloned entry
    pub fn get_full_entry(&self, addr: &Address) -> Option<LedgerEntry> {
        self.sorted_ledger.get(addr).cloned()
    }

    /// settles a slot and saves the corresponding ledger changes to history
    pub fn settle_slot(&mut self, slot: Slot, changes: LedgerChanges) {
        // apply changes
        self.apply(changes.clone());

        // update the slot
        self.slot = slot;

        // update and prune changes history
        self.changes_history.push_back((slot, changes));
        while self.changes_history.len() > self.config.final_history_length {
            self.changes_history.pop_front();
        }
    }

    /// gets the parallel balance of an entry
    pub fn get_parallel_balance(&self, addr: &Address) -> Option<Amount> {
        self.sorted_ledger.get(addr).map(|v| v.parallel_balance)
    }

    /// gets a copy of the bytecode of an entry
    pub fn get_bytecode(&self, addr: &Address) -> Option<Vec<u8>> {
        self.sorted_ledger.get(addr).map(|v| v.bytecode.clone())
    }

    /// checks if an entry exists
    pub fn entry_exists(&self, addr: &Address) -> bool {
        self.sorted_ledger.contains_key(addr)
    }

    /// gets a copy of a data entry
    pub fn get_data_entry(&self, addr: &Address, key: &Hash) -> Option<Vec<u8>> {
        self.sorted_ledger
            .get(addr)
            .and_then(|v| v.datastore.get(key).cloned())
    }

    /// checks whether a data entry exists
    pub fn has_data_entry(&self, addr: &Address, key: &Hash) -> bool {
        self.sorted_ledger
            .get(addr)
            .map_or(false, |v| v.datastore.contains_key(key))
    }
}
