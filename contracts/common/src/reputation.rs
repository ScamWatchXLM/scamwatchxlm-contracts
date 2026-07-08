use soroban_sdk::{contractclient, Address, Env};

/// The subset of the Reputation contract's public interface that the
/// Registry contract needs in order to report validation outcomes.
///
/// Mirrors [`crate::governance::GovernanceInterface`]: a plain trait shared
/// as a crate so Registry can call an already-deployed Reputation contract
/// by address without depending on the `reputation` crate itself. See
/// `contracts/reputation/src/lib.rs` for the concrete implementation and the
/// `registry: Address` + `require_auth` check that authorizes these calls
/// (Soroban's invoker-contract auto-authorization: it succeeds only when the
/// Registry contract itself is the direct caller).
#[contractclient(name = "ReputationClient")]
pub trait ReputationInterface {
    /// Records the outcome of validating a report filed by `reporter`,
    /// adjusting their reputation score up (approved) or down (rejected).
    fn record_report_outcome(env: Env, registry: Address, reporter: Address, approved: bool);

    /// Records that `validator` performed a validation, giving a small
    /// participation reward.
    fn record_validation(env: Env, registry: Address, validator: Address);
}
