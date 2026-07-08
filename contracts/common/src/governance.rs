use soroban_sdk::{contractclient, Address, Env};

/// The subset of the Governance contract's public interface that the
/// Registry and Reputation contracts need in order to check permissions.
///
/// This is a plain trait (no `#[contract]`/`#[contractimpl]`) shared as a
/// crate so Registry and Reputation can call an already-deployed Governance
/// contract by address without depending on the `governance` crate itself.
/// The Governance contract implements these exact function signatures, and
/// `#[contractclient]` generates [`GovernanceClient`], a typed wrapper around
/// `env.invoke_contract` that calls them over the network/host boundary.
///
/// See `contracts/governance/src/lib.rs` for the concrete implementation.
#[contractclient(name = "GovernanceClient")]
pub trait GovernanceInterface {
    /// Returns `true` if `address` is the owner or an admin.
    fn is_admin(env: Env, address: Address) -> bool;

    /// Returns `true` if `address` is a registered validator.
    fn is_validator(env: Env, address: Address) -> bool;

    /// Returns `true` if the system-wide pause switch is engaged.
    fn is_paused(env: Env) -> bool;
}
