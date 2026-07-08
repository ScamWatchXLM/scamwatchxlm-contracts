use soroban_sdk::{contracttype, Address, BytesN};

/// Keys under which the Governance contract keeps its instance storage.
///
/// Everything here lives in instance storage: it is small, read on nearly
/// every invocation (permission checks), and its TTL is extended as a single
/// unit whenever the contract is called.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// The single account that can manage admins and authorize upgrades.
    Owner,
    /// `Vec<Address>` of accounts with admin privileges (validator/pause
    /// management). Does not include the owner, who is implicitly an admin.
    Admins,
    /// `Vec<Validator>` of accounts allowed to validate registry reports.
    Validators,
    /// System-wide pause switch consulted by the Registry and Reputation
    /// contracts before any state-mutating call.
    Paused,
    /// The upgrade currently awaiting the owner's confirmation, if any.
    PendingUpgrade,
}

/// A validator recognized by governance, with basic bookkeeping about when
/// it was added.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Validator {
    pub address: Address,
    pub added_at: u64,
}

/// A Wasm upgrade proposed by an admin and awaiting the owner's execution.
///
/// Splitting proposal from execution, and requiring the owner (not just any
/// admin) to execute, keeps a single compromised/careless admin key from
/// being able to unilaterally replace contract code.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingUpgrade {
    pub wasm_hash: BytesN<32>,
    pub proposed_by: Address,
    pub proposed_at: u64,
}
