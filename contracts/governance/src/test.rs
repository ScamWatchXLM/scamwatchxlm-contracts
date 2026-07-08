#![cfg(test)]

use crate::{errors::Error, GovernanceContract, GovernanceContractClient, UPGRADE_TIMELOCK_SECS};
use scamwatchxlm_common::Role;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env,
};

fn setup<'a>() -> (Env, Address, GovernanceContractClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let contract_id = env.register(GovernanceContract, (owner.clone(),));
    let client = GovernanceContractClient::new(&env, &contract_id);
    (env, owner, client)
}

#[test]
fn constructor_sets_owner_as_implicit_admin() {
    let (_env, owner, client) = setup();
    assert_eq!(client.get_owner(), owner);
    assert!(client.is_admin(&owner));
    assert!(!client.is_paused());
}

#[test]
fn owner_can_add_and_remove_admin() {
    let (_env, owner, client) = setup();
    let admin = Address::generate(&_env);

    assert!(!client.is_admin(&admin));
    client.add_admin(&owner, &admin);
    assert!(client.is_admin(&admin));

    client.remove_admin(&owner, &admin);
    assert!(!client.is_admin(&admin));
}

#[test]
fn non_owner_cannot_add_admin() {
    let (env, _owner, client) = setup();
    let not_owner = Address::generate(&env);
    let target = Address::generate(&env);

    let result = client.try_add_admin(&not_owner, &target);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn duplicate_admin_is_rejected() {
    let (_env, owner, client) = setup();
    let admin = Address::generate(&_env);
    client.add_admin(&owner, &admin);

    let result = client.try_add_admin(&owner, &admin);
    assert_eq!(result, Err(Ok(Error::AlreadyAdmin)));
}

#[test]
fn admin_can_manage_validators() {
    let (env, owner, client) = setup();
    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    client.add_admin(&owner, &admin);

    assert!(!client.is_validator(&validator));
    client.add_validator(&admin, &validator);
    assert!(client.is_validator(&validator));

    client.remove_validator(&admin, &validator);
    assert!(!client.is_validator(&validator));
}

#[test]
fn plain_reporter_cannot_manage_validators() {
    let (env, _owner, client) = setup();
    let stranger = Address::generate(&env);
    let validator = Address::generate(&env);

    let result = client.try_add_validator(&stranger, &validator);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn pause_and_unpause_round_trip() {
    let (_env, owner, client) = setup();
    assert!(!client.is_paused());

    client.pause(&owner);
    assert!(client.is_paused());

    let result = client.try_pause(&owner);
    assert_eq!(result, Err(Ok(Error::AlreadyPaused)));

    client.unpause(&owner);
    assert!(!client.is_paused());
}

#[test]
fn upgrade_requires_timelock_and_owner_execution() {
    let (env, owner, client) = setup();
    let admin = Address::generate(&env);
    client.add_admin(&owner, &admin);
    let wasm_hash = BytesN::from_array(&env, &[7u8; 32]);

    client.propose_upgrade(&admin, &wasm_hash);
    let pending = client.get_pending_upgrade().unwrap();
    assert_eq!(pending.wasm_hash, wasm_hash);

    // Too early: timelock has not elapsed yet.
    let result = client.try_execute_upgrade(&owner);
    assert_eq!(result, Err(Ok(Error::TimelockNotElapsed)));

    // Admin (not owner) still cannot execute even after the timelock.
    let now = env.ledger().timestamp();
    env.ledger().set_timestamp(now + UPGRADE_TIMELOCK_SECS + 1);
    let result = client.try_execute_upgrade(&admin);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));

    // Owner can cancel instead of executing.
    client.cancel_upgrade(&owner);
    assert!(client.get_pending_upgrade().is_none());
}

#[test]
fn has_role_matches_the_specific_predicates() {
    let (env, owner, client) = setup();
    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let stranger = Address::generate(&env);
    client.add_admin(&owner, &admin);
    client.add_validator(&admin, &validator);

    assert!(client.has_role(&owner, &Role::Owner));
    assert!(!client.has_role(&admin, &Role::Owner));

    assert!(client.has_role(&owner, &Role::Admin));
    assert!(client.has_role(&admin, &Role::Admin));
    assert!(!client.has_role(&stranger, &Role::Admin));

    assert!(client.has_role(&validator, &Role::Validator));
    assert!(!client.has_role(&admin, &Role::Validator));
}

#[test]
fn admin_pagination_respects_offset_and_limit() {
    let (env, owner, client) = setup();
    for _ in 0..5 {
        let admin = Address::generate(&env);
        client.add_admin(&owner, &admin);
    }

    let page = client.list_admins(&0, &2);
    assert_eq!(page.len(), 2);

    let rest = client.list_admins(&2, &10);
    assert_eq!(rest.len(), 3);
}
