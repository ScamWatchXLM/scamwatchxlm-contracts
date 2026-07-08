use soroban_sdk::{contractevent, Address, Symbol};

/// Emitted whenever an account's reputation score changes, whether from an
/// automated Registry callback or a manual admin adjustment. `delta`'s sign
/// indicates increase vs. decrease, so one event type covers both.
#[contractevent]
pub struct ReputationAdjusted {
    #[topic]
    pub account: Address,
    pub delta: i64,
    pub new_score: i64,
    pub reason: Symbol,
}
