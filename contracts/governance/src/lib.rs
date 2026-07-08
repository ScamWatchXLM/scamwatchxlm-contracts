//! Governance contract: owner/admin/validator management, the system-wide
//! pause switch, and timelocked upgrade authorization for the ScamWatchXLM
//! suite.
//!
//! The Registry and Reputation contracts hold this contract's address and
//! call back into it (via [`scamwatchxlm_common::GovernanceClient`]) to check
//! `is_admin`, `is_validator`, and `is_paused` before mutating their own
//! state. Keeping permissions here means role changes take effect for the
//! whole suite immediately, with no need to update each contract separately.
#![no_std]

mod errors;
mod events;
mod types;

#[cfg(test)]
mod test;

use errors::Error;
use events::{
    AdminAdded, AdminRemoved, OwnerChanged, SystemPaused, SystemUnpaused, UpgradeCancelled,
    UpgradeExecuted, UpgradeProposed, ValidatorAdded, ValidatorRemoved,
};
use scamwatchxlm_common::pagination::paginate;
use scamwatchxlm_common::Role;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};
use types::{DataKey, PendingUpgrade, Validator};

/// Ledger close time is ~5 seconds, so a day is ~17280 ledgers.
const DAY_IN_LEDGERS: u32 = 17280;
/// Extend the instance's TTL once it has less than 30 days left.
const INSTANCE_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS * 30;
/// ...and extend it out to 60 days when we do.
const INSTANCE_BUMP_AMOUNT: u32 = DAY_IN_LEDGERS * 60;
/// Minimum time an upgrade must sit proposed before the owner can execute it,
/// giving the community a window to notice and react to a malicious or
/// buggy proposal.
pub const UPGRADE_TIMELOCK_SECS: u64 = 3 * 24 * 60 * 60;

fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

fn get_owner(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Owner)
        .expect("contract not initialized")
}

fn get_admins(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::Admins)
        .unwrap_or_else(|| Vec::new(env))
}

fn get_validators(env: &Env) -> Vec<Validator> {
    env.storage()
        .instance()
        .get(&DataKey::Validators)
        .unwrap_or_else(|| Vec::new(env))
}

fn require_owner(env: &Env, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    if *caller != get_owner(env) {
        return Err(Error::NotAuthorized);
    }
    Ok(())
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    caller.require_auth();
    let owner = get_owner(env);
    if *caller == owner || get_admins(env).iter().any(|a| a == *caller) {
        return Ok(());
    }
    Err(Error::NotAuthorized)
}

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    /// Initializes the contract with `owner` as its sole owner and first
    /// (implicit) admin. Called exactly once, at deployment.
    pub fn __constructor(env: Env, owner: Address) {
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage()
            .instance()
            .set(&DataKey::Admins, &Vec::<Address>::new(&env));
        env.storage()
            .instance()
            .set(&DataKey::Validators, &Vec::<Validator>::new(&env));
        env.storage().instance().set(&DataKey::Paused, &false);
        extend_instance_ttl(&env);
    }

    /// Transfers ownership to `new_owner`. Only the current owner may call
    /// this. The previous owner immediately loses owner-only privileges but
    /// keeps admin privileges only if separately added via [`Self::add_admin`].
    pub fn transfer_ownership(env: Env, caller: Address, new_owner: Address) -> Result<(), Error> {
        require_owner(&env, &caller)?;
        if new_owner == caller {
            return Err(Error::SameAddress);
        }
        env.storage().instance().set(&DataKey::Owner, &new_owner);
        extend_instance_ttl(&env);
        OwnerChanged {
            previous_owner: caller,
            new_owner,
        }
        .publish(&env);
        Ok(())
    }

    /// Grants admin privileges to `admin`. Only the owner may call this.
    pub fn add_admin(env: Env, caller: Address, admin: Address) -> Result<(), Error> {
        require_owner(&env, &caller)?;
        let mut admins = get_admins(&env);
        if admin == get_owner(&env) || admins.iter().any(|a| a == admin) {
            return Err(Error::AlreadyAdmin);
        }
        admins.push_back(admin.clone());
        env.storage().instance().set(&DataKey::Admins, &admins);
        extend_instance_ttl(&env);
        AdminAdded {
            admin,
            added_by: caller,
        }
        .publish(&env);
        Ok(())
    }

    /// Revokes admin privileges from `admin`. Only the owner may call this.
    pub fn remove_admin(env: Env, caller: Address, admin: Address) -> Result<(), Error> {
        require_owner(&env, &caller)?;
        let admins = get_admins(&env);
        let mut new_admins = Vec::new(&env);
        let mut found = false;
        for a in admins.iter() {
            if a == admin {
                found = true;
            } else {
                new_admins.push_back(a);
            }
        }
        if !found {
            return Err(Error::NotAdmin);
        }
        env.storage().instance().set(&DataKey::Admins, &new_admins);
        extend_instance_ttl(&env);
        AdminRemoved {
            admin,
            removed_by: caller,
        }
        .publish(&env);
        Ok(())
    }

    /// Grants validator privileges to `validator`. Callable by the owner or
    /// any admin.
    pub fn add_validator(env: Env, caller: Address, validator: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        let mut validators = get_validators(&env);
        if validators.iter().any(|v| v.address == validator) {
            return Err(Error::AlreadyValidator);
        }
        validators.push_back(Validator {
            address: validator.clone(),
            added_at: env.ledger().timestamp(),
        });
        env.storage()
            .instance()
            .set(&DataKey::Validators, &validators);
        extend_instance_ttl(&env);
        ValidatorAdded {
            validator,
            added_by: caller,
        }
        .publish(&env);
        Ok(())
    }

    /// Revokes validator privileges from `validator`. Callable by the owner
    /// or any admin.
    pub fn remove_validator(env: Env, caller: Address, validator: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        let validators = get_validators(&env);
        let mut new_validators = Vec::new(&env);
        let mut found = false;
        for v in validators.iter() {
            if v.address == validator {
                found = true;
            } else {
                new_validators.push_back(v);
            }
        }
        if !found {
            return Err(Error::NotValidator);
        }
        env.storage()
            .instance()
            .set(&DataKey::Validators, &new_validators);
        extend_instance_ttl(&env);
        ValidatorRemoved {
            validator,
            removed_by: caller,
        }
        .publish(&env);
        Ok(())
    }

    /// Engages the system-wide pause switch. Callable by the owner or any
    /// admin. Registry and Reputation reject state-mutating calls while
    /// paused.
    pub fn pause(env: Env, caller: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        if env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(Error::AlreadyPaused);
        }
        env.storage().instance().set(&DataKey::Paused, &true);
        extend_instance_ttl(&env);
        SystemPaused { by: caller }.publish(&env);
        Ok(())
    }

    /// Releases the system-wide pause switch. Callable by the owner or any
    /// admin.
    pub fn unpause(env: Env, caller: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        if !env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(Error::NotPaused);
        }
        env.storage().instance().set(&DataKey::Paused, &false);
        extend_instance_ttl(&env);
        SystemUnpaused { by: caller }.publish(&env);
        Ok(())
    }

    /// Proposes replacing this contract's Wasm with `wasm_hash`. Callable by
    /// the owner or any admin. Execution is a separate, owner-only step that
    /// cannot happen until [`UPGRADE_TIMELOCK_SECS`] has elapsed, so the
    /// community has time to review the proposal.
    pub fn propose_upgrade(env: Env, caller: Address, wasm_hash: BytesN<32>) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        if env.storage().instance().has(&DataKey::PendingUpgrade) {
            return Err(Error::UpgradeAlreadyPending);
        }
        let proposed_at = env.ledger().timestamp();
        env.storage().instance().set(
            &DataKey::PendingUpgrade,
            &PendingUpgrade {
                wasm_hash: wasm_hash.clone(),
                proposed_by: caller.clone(),
                proposed_at,
            },
        );
        extend_instance_ttl(&env);
        UpgradeProposed {
            wasm_hash,
            proposed_by: caller,
            proposed_at,
        }
        .publish(&env);
        Ok(())
    }

    /// Cancels a pending upgrade proposal without executing it. Callable by
    /// the owner or any admin.
    pub fn cancel_upgrade(env: Env, caller: Address) -> Result<(), Error> {
        require_admin(&env, &caller)?;
        let pending: PendingUpgrade = env
            .storage()
            .instance()
            .get(&DataKey::PendingUpgrade)
            .ok_or(Error::NoPendingUpgrade)?;
        env.storage().instance().remove(&DataKey::PendingUpgrade);
        extend_instance_ttl(&env);
        UpgradeCancelled {
            wasm_hash: pending.wasm_hash,
            cancelled_by: caller,
        }
        .publish(&env);
        Ok(())
    }

    /// Executes a pending upgrade proposal, replacing this contract's Wasm.
    /// Only the owner may call this, and only after the timelock has
    /// elapsed.
    pub fn execute_upgrade(env: Env, caller: Address) -> Result<(), Error> {
        require_owner(&env, &caller)?;
        let pending: PendingUpgrade = env
            .storage()
            .instance()
            .get(&DataKey::PendingUpgrade)
            .ok_or(Error::NoPendingUpgrade)?;
        if env.ledger().timestamp() < pending.proposed_at + UPGRADE_TIMELOCK_SECS {
            return Err(Error::TimelockNotElapsed);
        }
        env.storage().instance().remove(&DataKey::PendingUpgrade);
        env.deployer()
            .update_current_contract_wasm(pending.wasm_hash.clone());
        UpgradeExecuted {
            wasm_hash: pending.wasm_hash,
        }
        .publish(&env);
        Ok(())
    }

    /// Returns the current owner.
    pub fn get_owner(env: Env) -> Address {
        get_owner(&env)
    }

    /// Returns `true` if `address` is the owner or an admin.
    ///
    /// Part of [`scamwatchxlm_common::governance::GovernanceInterface`]:
    /// called cross-contract by the Registry and Reputation contracts.
    pub fn is_admin(env: Env, address: Address) -> bool {
        address == get_owner(&env) || get_admins(&env).iter().any(|a| a == address)
    }

    /// Returns `true` if `address` is a registered validator.
    ///
    /// Part of [`scamwatchxlm_common::governance::GovernanceInterface`].
    pub fn is_validator(env: Env, address: Address) -> bool {
        get_validators(&env).iter().any(|v| v.address == address)
    }

    /// Returns `true` if `address` currently holds `role`. A role-generic
    /// convenience over `is_admin`/`is_validator`/`get_owner`, for callers
    /// (e.g. a frontend or indexer) that want to check by [`Role`] rather
    /// than call a specific predicate.
    pub fn has_role(env: Env, address: Address, role: Role) -> bool {
        match role {
            Role::Owner => address == get_owner(&env),
            Role::Admin => {
                address == get_owner(&env) || get_admins(&env).iter().any(|a| a == address)
            }
            Role::Validator => get_validators(&env).iter().any(|v| v.address == address),
        }
    }

    /// Returns `true` if the system-wide pause switch is engaged.
    ///
    /// Part of [`scamwatchxlm_common::governance::GovernanceInterface`].
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    /// Lists admins, paginated. `limit` is clamped to
    /// [`scamwatchxlm_common::pagination::MAX_PAGE_SIZE`].
    pub fn list_admins(env: Env, offset: u32, limit: u32) -> Vec<Address> {
        paginate(&env, &get_admins(&env), offset, limit)
    }

    /// Lists validators, paginated. `limit` is clamped to
    /// [`scamwatchxlm_common::pagination::MAX_PAGE_SIZE`].
    pub fn list_validators(env: Env, offset: u32, limit: u32) -> Vec<Validator> {
        paginate(&env, &get_validators(&env), offset, limit)
    }

    /// Returns the upgrade currently awaiting execution, if any.
    pub fn get_pending_upgrade(env: Env) -> Option<PendingUpgrade> {
        env.storage().instance().get(&DataKey::PendingUpgrade)
    }
}
