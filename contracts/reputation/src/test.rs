#![cfg(test)]

use crate::{errors::Error, ReputationContract, ReputationContractClient};
use governance::{GovernanceContract, GovernanceContractClient};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

struct Harness<'a> {
    env: Env,
    owner: Address,
    // Kept so `setup` reflects the real deployment shape even though most
    // tests only need `owner`'s address, not governance calls directly.
    #[allow(dead_code)]
    governance: GovernanceContractClient<'a>,
    reputation: ReputationContractClient<'a>,
}

fn setup<'a>() -> Harness<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let governance_id = env.register(GovernanceContract, (owner.clone(),));
    let governance = GovernanceContractClient::new(&env, &governance_id);

    let reputation_id = env.register(ReputationContract, (governance_id,));
    let reputation = ReputationContractClient::new(&env, &reputation_id);

    Harness {
        env,
        owner,
        governance,
        reputation,
    }
}

#[test]
fn new_account_has_zero_score() {
    let h = setup();
    let someone = Address::generate(&h.env);
    assert_eq!(h.reputation.get_score(&someone), 0);
}

#[test]
fn admin_can_reward_and_penalize() {
    let h = setup();
    let account = Address::generate(&h.env);

    let score = h
        .reputation
        .reward(&h.owner, &account, &20, &symbol_short!("bonus"));
    assert_eq!(score, 20);

    let score = h
        .reputation
        .penalize(&h.owner, &account, &5, &symbol_short!("minor"));
    assert_eq!(score, 15);
}

#[test]
fn non_admin_cannot_reward() {
    let h = setup();
    let stranger = Address::generate(&h.env);
    let account = Address::generate(&h.env);

    let result = h
        .reputation
        .try_reward(&stranger, &account, &10, &symbol_short!("bonus"));
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn zero_amount_reward_is_rejected() {
    let h = setup();
    let account = Address::generate(&h.env);

    let result = h
        .reputation
        .try_reward(&h.owner, &account, &0, &symbol_short!("bonus"));
    assert_eq!(result, Err(Ok(Error::InvalidAmount)));
}

#[test]
fn only_configured_registry_can_record_outcomes() {
    let h = setup();
    let fake_registry = Address::generate(&h.env);
    let reporter = Address::generate(&h.env);

    // No registry configured yet.
    let result = h
        .reputation
        .try_record_report_outcome(&fake_registry, &reporter, &true);
    assert!(result.is_err());

    h.reputation.set_registry_contract(&h.owner, &fake_registry);
    h.reputation
        .record_report_outcome(&fake_registry, &reporter, &true);
    assert_eq!(h.reputation.get_score(&reporter), 10);

    h.reputation
        .record_report_outcome(&fake_registry, &reporter, &false);
    assert_eq!(h.reputation.get_score(&reporter), -5);
}

#[test]
fn validation_participation_is_rewarded() {
    let h = setup();
    let registry = Address::generate(&h.env);
    let validator = Address::generate(&h.env);
    h.reputation.set_registry_contract(&h.owner, &registry);

    h.reputation.record_validation(&registry, &validator);
    let profile = h.reputation.get_reputation(&validator);
    assert_eq!(profile.validations_performed, 1);
    assert_eq!(profile.score, 2);
}
