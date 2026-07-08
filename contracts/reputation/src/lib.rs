//! Reputation contract: tracks a reputation score for reporters and
//! validators across the ScamWatchXLM suite.
//!
//! Two ways a score changes:
//! - Automatically, via [`RegistryContract::record_report_outcome`]/
//!   [`RegistryContract::record_validation`]-style callbacks from the
//!   Registry contract (see `contracts/registry`), authorized using
//!   Soroban's invoker-contract auto-authorization: `registry.require_auth()`
//!   succeeds without a signature only when the Registry contract is
//!   literally the direct caller.
//! - Manually, via admin-only [`ReputationContract::reward`]/
//!   [`ReputationContract::penalize`], for cases judgment calls fall outside
//!   what the Registry's automated flow covers.
#![no_std]

mod errors;
mod events;
mod storage;
#[cfg(test)]
mod test;
mod types;

use errors::Error;
use events::ReputationAdjusted;
use scamwatchxlm_common::GovernanceClient;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};
use types::ReputationProfile;

/// Reward applied to a reporter's score when their report is validated.
const REPORT_VALIDATED_REWARD: i64 = 10;
/// Penalty applied to a reporter's score when their report is rejected as
/// false.
const REPORT_REJECTED_PENALTY: i64 = 15;
/// Small reward applied to a validator's score for performing a review.
const VALIDATION_PARTICIPATION_REWARD: i64 = 2;

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    if GovernanceClient::new(env, &storage::governance(env)).is_admin(caller) {
        return Ok(());
    }
    Err(Error::NotAuthorized)
}

/// Only the configured Registry contract may call this. Relies on Soroban's
/// invoker-contract authorization: `registry.require_auth()` is satisfied
/// without a signature precisely when the Registry contract is the direct
/// caller of this invocation, so a third party cannot forge it by merely
/// passing the Registry's address as an argument.
fn require_registry(env: &Env, registry: &Address) -> Result<(), Error> {
    registry.require_auth();
    let configured = storage::registry(env).ok_or(Error::RegistryNotSet)?;
    if *registry != configured {
        return Err(Error::NotAuthorized);
    }
    Ok(())
}

fn adjust_score(env: &Env, account: &Address, delta: i64, reason: Symbol) -> i64 {
    let now = env.ledger().timestamp();
    let mut profile = storage::profile(env, account)
        .unwrap_or_else(|| ReputationProfile::new(account.clone(), now));
    profile.score += delta;
    profile.updated_at = now;
    let new_score = profile.score;
    storage::set_profile(env, &profile);
    storage::extend_instance_ttl(env);
    ReputationAdjusted {
        account: account.clone(),
        delta,
        new_score,
        reason,
    }
    .publish(env);
    new_score
}

#[contract]
pub struct ReputationContract;

#[contractimpl]
impl ReputationContract {
    /// Initializes the contract, binding it to a Governance contract used
    /// for admin permission checks on the manual `reward`/`penalize` path.
    pub fn __constructor(env: Env, governance: Address) {
        storage::init(&env, &governance);
        storage::extend_instance_ttl(&env);
    }

    /// Sets (or replaces) the Registry contract authorized to call
    /// `record_report_outcome`/`record_validation`. Admin-only.
    pub fn set_registry_contract(
        env: Env,
        caller: Address,
        registry: Address,
    ) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        storage::set_registry(&env, &registry);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Adjusts `reporter`'s score after their report was validated
    /// (`approved = true`, score up) or rejected as false (`approved =
    /// false`, score down). Callable only by the configured Registry
    /// contract.
    ///
    /// Part of [`scamwatchxlm_common::reputation::ReputationInterface`].
    pub fn record_report_outcome(env: Env, registry: Address, reporter: Address, approved: bool) {
        require_registry(&env, &registry).unwrap();
        let now = env.ledger().timestamp();
        let mut profile = storage::profile(&env, &reporter)
            .unwrap_or_else(|| ReputationProfile::new(reporter.clone(), now));
        if approved {
            profile.reports_validated += 1;
        } else {
            profile.reports_rejected += 1;
        }
        storage::set_profile(&env, &profile);

        let (delta, reason) = if approved {
            (REPORT_VALIDATED_REWARD, symbol_short!("approved"))
        } else {
            (-REPORT_REJECTED_PENALTY, symbol_short!("rejected"))
        };
        adjust_score(&env, &reporter, delta, reason);
    }

    /// Rewards `validator` with a small participation bonus for performing a
    /// review. Callable only by the configured Registry contract.
    ///
    /// Part of [`scamwatchxlm_common::reputation::ReputationInterface`].
    pub fn record_validation(env: Env, registry: Address, validator: Address) {
        require_registry(&env, &registry).unwrap();
        let now = env.ledger().timestamp();
        let mut profile = storage::profile(&env, &validator)
            .unwrap_or_else(|| ReputationProfile::new(validator.clone(), now));
        profile.validations_performed += 1;
        storage::set_profile(&env, &profile);

        adjust_score(
            &env,
            &validator,
            VALIDATION_PARTICIPATION_REWARD,
            symbol_short!("valid8n"),
        );
    }

    /// Manually increases `account`'s score. Admin-only: for cases outside
    /// the Registry's automated flow (e.g. off-chain investigation).
    pub fn reward(
        env: Env,
        caller: Address,
        account: Address,
        amount: u32,
        reason: Symbol,
    ) -> Result<i64, Error> {
        require_admin(&env, &caller)?;
        if amount == 0 {
            return Err(Error::InvalidAmount);
        }
        Ok(adjust_score(&env, &account, amount as i64, reason))
    }

    /// Manually decreases `account`'s score. Admin-only.
    pub fn penalize(
        env: Env,
        caller: Address,
        account: Address,
        amount: u32,
        reason: Symbol,
    ) -> Result<i64, Error> {
        require_admin(&env, &caller)?;
        if amount == 0 {
            return Err(Error::InvalidAmount);
        }
        Ok(adjust_score(&env, &account, -(amount as i64), reason))
    }

    /// Returns `account`'s full reputation profile, defaulting to a fresh,
    /// zero-score profile if it has never been touched.
    pub fn get_reputation(env: Env, account: Address) -> ReputationProfile {
        storage::profile(&env, &account)
            .unwrap_or_else(|| ReputationProfile::new(account, env.ledger().timestamp()))
    }

    /// Returns `account`'s current score, defaulting to `0`.
    pub fn get_score(env: Env, account: Address) -> i64 {
        storage::profile(&env, &account)
            .map(|p| p.score)
            .unwrap_or(0)
    }
}
