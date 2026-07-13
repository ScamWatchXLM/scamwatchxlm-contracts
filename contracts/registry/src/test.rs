#![cfg(test)]

use crate::{errors::Error, RegistryContract, RegistryContractClient};
use governance::{GovernanceContract, GovernanceContractClient};
use reputation::{ReputationContract, ReputationContractClient};
use scamwatchxlm_common::RiskLevel;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

struct Harness<'a> {
    env: Env,
    owner: Address,
    governance: GovernanceContractClient<'a>,
    registry: RegistryContractClient<'a>,
}

fn setup<'a>() -> Harness<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let governance_id = env.register(GovernanceContract, (owner.clone(),));
    let governance = GovernanceContractClient::new(&env, &governance_id);

    let registry_id = env.register(RegistryContract, (governance_id.clone(),));
    let registry = RegistryContractClient::new(&env, &registry_id);

    Harness {
        env,
        owner,
        governance,
        registry,
    }
}

fn evidence(env: &Env) -> String {
    String::from_str(env, "https://evidence.example/report/1")
}

#[test]
fn submit_and_fetch_account_report() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);

    let id = h
        .registry
        .report_account(&reporter, &scammer, &RiskLevel::High, &evidence(&h.env));
    assert_eq!(id, 1);

    let report = h.registry.get_report(&id);
    assert_eq!(report.reporter, reporter);
    assert_eq!(report.risk_level, RiskLevel::High);
    assert!(h.registry.is_account_flagged(&scammer));

    let record = h.registry.get_account_record(&scammer).unwrap();
    assert_eq!(record.stats.report_count, 1);
}

#[test]
fn duplicate_report_for_same_entity_is_rejected() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);

    h.registry
        .report_account(&reporter, &scammer, &RiskLevel::Medium, &evidence(&h.env));

    let result =
        h.registry
            .try_report_account(&reporter, &scammer, &RiskLevel::Medium, &evidence(&h.env));
    assert_eq!(result, Err(Ok(Error::DuplicateReport)));
}

#[test]
fn rejected_report_allows_entity_to_be_reported_again() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);
    let validator = Address::generate(&h.env);
    h.governance.add_validator(&h.owner, &validator);

    let first_id =
        h.registry
            .report_account(&reporter, &scammer, &RiskLevel::Low, &evidence(&h.env));
    h.registry
        .validate_report(&validator, &first_id, &false, &None);

    // The entity is not stuck unreportable just because the first report
    // (wrongly, or no longer applicably) was rejected.
    let second_id =
        h.registry
            .report_account(&reporter, &scammer, &RiskLevel::Critical, &evidence(&h.env));
    assert_ne!(first_id, second_id);

    let record = h.registry.get_account_record(&scammer).unwrap();
    assert_eq!(record.stats.report_count, 2);
    assert_eq!(record.stats.latest_report_id, second_id);
    assert_eq!(record.stats.highest_risk, RiskLevel::Critical);
    assert_eq!(
        record.stats.status,
        scamwatchxlm_common::ReportStatus::Pending
    );

    // A third report is blocked again while the second is still pending.
    let result =
        h.registry
            .try_report_account(&reporter, &scammer, &RiskLevel::Critical, &evidence(&h.env));
    assert_eq!(result, Err(Ok(Error::DuplicateReport)));
}

#[test]
fn archived_report_allows_entity_to_be_reported_again() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);
    let admin = Address::generate(&h.env);
    h.governance.add_admin(&h.owner, &admin);

    let first_id =
        h.registry
            .report_account(&reporter, &scammer, &RiskLevel::Low, &evidence(&h.env));
    h.registry.archive_report(&admin, &first_id);

    let second_id =
        h.registry
            .report_account(&reporter, &scammer, &RiskLevel::High, &evidence(&h.env));
    assert_ne!(first_id, second_id);
    assert!(h.registry.is_account_flagged(&scammer));
}

#[test]
fn validated_report_still_blocks_resubmission() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);
    let validator = Address::generate(&h.env);
    h.governance.add_validator(&h.owner, &validator);

    let id =
        h.registry
            .report_account(&reporter, &scammer, &RiskLevel::Critical, &evidence(&h.env));
    h.registry.validate_report(&validator, &id, &true, &None);

    let result =
        h.registry
            .try_report_account(&reporter, &scammer, &RiskLevel::Critical, &evidence(&h.env));
    assert_eq!(result, Err(Ok(Error::DuplicateReport)));
}

#[test]
fn empty_evidence_uri_is_rejected() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);

    let result = h.registry.try_report_account(
        &reporter,
        &scammer,
        &RiskLevel::Low,
        &String::from_str(&h.env, ""),
    );
    assert_eq!(result, Err(Ok(Error::InvalidInput)));
}

#[test]
fn only_validator_or_admin_can_validate_reports() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);
    let stranger = Address::generate(&h.env);

    let id =
        h.registry
            .report_account(&reporter, &scammer, &RiskLevel::Critical, &evidence(&h.env));

    let result = h.registry.try_validate_report(&stranger, &id, &true, &None);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn validator_can_approve_a_report() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);
    let validator = Address::generate(&h.env);
    h.governance.add_validator(&h.owner, &validator);

    let id =
        h.registry
            .report_account(&reporter, &scammer, &RiskLevel::Critical, &evidence(&h.env));
    h.registry.validate_report(&validator, &id, &true, &None);

    let report = h.registry.get_report(&id);
    assert_eq!(report.status, scamwatchxlm_common::ReportStatus::Validated);
    assert_eq!(report.validator, Some(validator));

    let profile = h.registry.get_reporter_profile(&reporter);
    assert_eq!(profile.reports_validated, 1);
}

#[test]
fn validating_twice_is_rejected() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);
    let validator = Address::generate(&h.env);
    h.governance.add_validator(&h.owner, &validator);

    let id =
        h.registry
            .report_account(&reporter, &scammer, &RiskLevel::Critical, &evidence(&h.env));
    h.registry.validate_report(&validator, &id, &true, &None);

    let result = h
        .registry
        .try_validate_report(&validator, &id, &true, &None);
    assert_eq!(result, Err(Ok(Error::InvalidStatusTransition)));
}

#[test]
fn pause_blocks_new_reports_but_not_reads() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);
    h.governance.pause(&h.owner);

    let result =
        h.registry
            .try_report_account(&reporter, &scammer, &RiskLevel::Low, &evidence(&h.env));
    assert_eq!(result, Err(Ok(Error::ContractPaused)));

    // Reads still work while paused.
    assert!(!h.registry.is_account_flagged(&scammer));
}

#[test]
fn admin_can_archive_a_report() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);
    let admin = Address::generate(&h.env);
    h.governance.add_admin(&h.owner, &admin);

    let id = h
        .registry
        .report_account(&reporter, &scammer, &RiskLevel::Low, &evidence(&h.env));
    h.registry.archive_report(&admin, &id);

    let report = h.registry.get_report(&id);
    assert_eq!(report.status, scamwatchxlm_common::ReportStatus::Archived);
}

#[test]
fn pagination_over_reporter_history() {
    let h = setup();
    let reporter = Address::generate(&h.env);

    for _ in 0..5 {
        let scammer = Address::generate(&h.env);
        h.registry
            .report_account(&reporter, &scammer, &RiskLevel::Low, &evidence(&h.env));
    }

    let page = h.registry.get_reports_by_reporter(&reporter, &0, &2);
    assert_eq!(page.len(), 2);

    let rest = h.registry.get_reports_by_reporter(&reporter, &2, &10);
    assert_eq!(rest.len(), 3);
}

#[test]
fn different_entity_kinds_do_not_collide() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let issuer = Address::generate(&h.env);
    let tx_hash = BytesN::from_array(&h.env, &[9u8; 32]);

    h.registry
        .report_asset_issuer(&reporter, &issuer, &RiskLevel::High, &evidence(&h.env));
    h.registry
        .report_transaction(&reporter, &tx_hash, &RiskLevel::High, &evidence(&h.env));
    h.registry.report_domain(
        &reporter,
        &String::from_str(&h.env, "phishing.example"),
        &RiskLevel::High,
        &evidence(&h.env),
    );

    assert!(h.registry.is_issuer_flagged(&issuer));
    assert!(h.registry.is_transaction_flagged(&tx_hash));
    assert!(h
        .registry
        .is_domain_flagged(&String::from_str(&h.env, "phishing.example")));
    assert_eq!(h.registry.report_count(), 3);
}

#[test]
fn validated_report_updates_reputation_when_configured() {
    let h = setup();
    let reporter = Address::generate(&h.env);
    let scammer = Address::generate(&h.env);
    let validator = Address::generate(&h.env);
    h.governance.add_validator(&h.owner, &validator);

    let reputation_id = h
        .env
        .register(ReputationContract, (h.governance.address.clone(),));
    let reputation = ReputationContractClient::new(&h.env, &reputation_id);
    reputation.set_registry_contract(&h.owner, &h.registry.address);
    h.registry.set_reputation_contract(&h.owner, &reputation_id);

    let id =
        h.registry
            .report_account(&reporter, &scammer, &RiskLevel::Critical, &evidence(&h.env));
    h.registry.validate_report(&validator, &id, &true, &None);

    assert_eq!(reputation.get_score(&reporter), 10);
    assert_eq!(reputation.get_score(&validator), 2);
}
