use soroban_sdk::{contracttype, Address};

/// Reputation and activity counters tracked for a single account. An
/// account can accrue both reporter and validator statistics if it plays
/// both roles.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReputationProfile {
    pub account: Address,
    /// Net reputation score. Starts at 0 and may go negative for accounts
    /// with a poor track record; there is intentionally no floor, so a bad
    /// track record remains visible rather than being clamped away.
    pub score: i64,
    pub reports_submitted: u32,
    pub reports_validated: u32,
    pub reports_rejected: u32,
    pub validations_performed: u32,
    pub updated_at: u64,
}

impl ReputationProfile {
    pub fn new(account: Address, now: u64) -> Self {
        Self {
            account,
            score: 0,
            reports_submitted: 0,
            reports_validated: 0,
            reports_rejected: 0,
            validations_performed: 0,
            updated_at: now,
        }
    }
}

/// Storage keys used by the Reputation contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Address of the Governance contract consulted for admin permissions.
    Governance,
    /// Address of the Registry contract authorized to call
    /// `record_report_outcome` / `record_validation`. Optional: manual
    /// `reward`/`penalize` by an admin work without it.
    Registry,
    /// `ReputationProfile` by account address.
    Profile(Address),
}
