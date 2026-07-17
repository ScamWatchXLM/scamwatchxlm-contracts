//! Registry contract: the on-chain ledger of reported malicious accounts,
//! asset issuers, assets, phishing domains, and scam transactions.
//!
//! Permissions (who may validate/archive reports, whether the system is
//! paused) are delegated to the Governance contract, called cross-contract
//! via [`GovernanceClient`]. If a Reputation contract address is configured
//! (see [`RegistryContract::set_reputation_contract`]), validated/rejected
//! reports also update the reporter's reputation score there.
#![no_std]

mod errors;
mod events;
mod storage;
#[cfg(test)]
mod test;
mod types;

use errors::Error;
use events::{ReportArchived, ReportSubmitted, ReportValidated};
use scamwatchxlm_common::pagination::{clamp_limit, paginate};
use scamwatchxlm_common::{GovernanceClient, ReportStatus, ReputationClient, RiskLevel};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};
use types::{AccountRecord, AssetRecord, IssuerRecord, ReportedEntity, Reporter, ScamReport};

/// Evidence URIs longer than this are rejected as input validation: reports
/// carry a *link* to evidence (IPFS, HTTPS, etc.), not the evidence itself.
const MAX_EVIDENCE_URI_LEN: u32 = 200;

fn require_not_paused(env: &Env, governance: &Address) -> Result<(), Error> {
    if GovernanceClient::new(env, governance).is_paused() {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

fn require_reviewer(env: &Env, governance: &Address, caller: &Address) -> Result<(), Error> {
    let client = GovernanceClient::new(env, governance);
    if client.is_validator(caller) || client.is_admin(caller) {
        return Ok(());
    }
    Err(Error::NotAuthorized)
}

fn require_admin(env: &Env, governance: &Address, caller: &Address) -> Result<(), Error> {
    if GovernanceClient::new(env, governance).is_admin(caller) {
        return Ok(());
    }
    Err(Error::NotAuthorized)
}

fn validate_evidence_uri(evidence_uri: &String) -> Result<(), Error> {
    if evidence_uri.is_empty() || evidence_uri.len() > MAX_EVIDENCE_URI_LEN {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

/// Shared implementation behind every `report_*` entry point: validates
/// input, checks for a duplicate, persists the report, and updates the
/// entity's aggregate record plus the reporter's profile.
fn submit_report(
    env: &Env,
    reporter: Address,
    entity: ReportedEntity,
    risk_level: RiskLevel,
    evidence_uri: String,
) -> Result<u64, Error> {
    reporter.require_auth();
    let governance = storage::governance(env);
    require_not_paused(env, &governance)?;
    validate_evidence_uri(&evidence_uri)?;

    if storage::has_active_report(env, &entity) {
        return Err(Error::DuplicateReport);
    }

    let id = storage::next_report_id(env);
    let now = env.ledger().timestamp();
    let report = ScamReport {
        id,
        entity: entity.clone(),
        risk_level,
        status: ReportStatus::Pending,
        reporter: reporter.clone(),
        evidence_uri,
        notes: None,
        validator: None,
        created_at: now,
        updated_at: now,
    };
    storage::set_report(env, &report);
    storage::record_new_report(env, &entity, &report);
    storage::append_reporter_history(env, &reporter, id);

    let mut profile = storage::reporter_profile(env, &reporter).unwrap_or(Reporter {
        address: reporter.clone(),
        reports_submitted: 0,
        reports_validated: 0,
        reports_rejected: 0,
        first_report_at: now,
    });
    profile.reports_submitted += 1;
    storage::set_reporter_profile(env, &profile);
    storage::extend_instance_ttl(env);

    ReportSubmitted {
        report_id: id,
        reporter,
        risk_level,
    }
    .publish(env);

    Ok(id)
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// Initializes the contract, binding it to a Governance contract used
    /// for all permission checks.
    pub fn __constructor(env: Env, governance: Address) {
        storage::init(&env, &governance);
        storage::extend_instance_ttl(&env);
    }

    /// Sets (or replaces) the Reputation contract notified when reports are
    /// validated or rejected. Admin-only. Optional: the registry operates
    /// standalone (no reputation effects) until this is set.
    pub fn set_reputation_contract(
        env: Env,
        caller: Address,
        reputation: Address,
    ) -> Result<(), Error> {
        caller.require_auth();
        let governance = storage::governance(&env);
        require_admin(&env, &governance, &caller)?;
        storage::set_reputation(&env, &reputation);
        storage::extend_instance_ttl(&env);
        Ok(())
    }

    /// Reports a Stellar account as malicious.
    pub fn report_account(
        env: Env,
        reporter: Address,
        address: Address,
        risk_level: RiskLevel,
        evidence_uri: String,
    ) -> Result<u64, Error> {
        submit_report(
            &env,
            reporter,
            ReportedEntity::Account(address),
            risk_level,
            evidence_uri,
        )
    }

    /// Reports an asset issuer as malicious.
    pub fn report_asset_issuer(
        env: Env,
        reporter: Address,
        issuer: Address,
        risk_level: RiskLevel,
        evidence_uri: String,
    ) -> Result<u64, Error> {
        submit_report(
            &env,
            reporter,
            ReportedEntity::AssetIssuer(issuer),
            risk_level,
            evidence_uri,
        )
    }

    /// Reports an asset (identified by code + issuer) as a scam.
    pub fn report_asset(
        env: Env,
        reporter: Address,
        asset_code: String,
        issuer: Address,
        risk_level: RiskLevel,
        evidence_uri: String,
    ) -> Result<u64, Error> {
        submit_report(
            &env,
            reporter,
            ReportedEntity::Asset(asset_code, issuer),
            risk_level,
            evidence_uri,
        )
    }

    /// Reports a phishing domain.
    pub fn report_domain(
        env: Env,
        reporter: Address,
        domain: String,
        risk_level: RiskLevel,
        evidence_uri: String,
    ) -> Result<u64, Error> {
        submit_report(
            &env,
            reporter,
            ReportedEntity::Domain(domain),
            risk_level,
            evidence_uri,
        )
    }

    /// Reports a transaction hash associated with a scam.
    pub fn report_transaction(
        env: Env,
        reporter: Address,
        tx_hash: BytesN<32>,
        risk_level: RiskLevel,
        evidence_uri: String,
    ) -> Result<u64, Error> {
        submit_report(
            &env,
            reporter,
            ReportedEntity::Transaction(tx_hash),
            risk_level,
            evidence_uri,
        )
    }

    /// Validates or rejects a pending report. Callable by a Governance
    /// validator or admin. If a Reputation contract is configured, the
    /// reporter's score is adjusted accordingly.
    pub fn validate_report(
        env: Env,
        validator: Address,
        report_id: u64,
        approve: bool,
        notes: Option<String>,
    ) -> Result<(), Error> {
        validator.require_auth();
        let governance = storage::governance(&env);
        require_not_paused(&env, &governance)?;
        require_reviewer(&env, &governance, &validator)?;

        let mut report = storage::get_report(&env, report_id).ok_or(Error::ReportNotFound)?;
        if report.status != ReportStatus::Pending {
            return Err(Error::InvalidStatusTransition);
        }

        report.status = if approve {
            ReportStatus::Validated
        } else {
            ReportStatus::Rejected
        };
        report.validator = Some(validator.clone());
        report.notes = notes;
        report.updated_at = env.ledger().timestamp();
        storage::set_report(&env, &report);
        storage::sync_entity_status(&env, &report.entity, &report);

        let mut profile = storage::reporter_profile(&env, &report.reporter).unwrap_or(Reporter {
            address: report.reporter.clone(),
            reports_submitted: 0,
            reports_validated: 0,
            reports_rejected: 0,
            first_report_at: report.created_at,
        });
        if approve {
            profile.reports_validated += 1;
        } else {
            profile.reports_rejected += 1;
        }
        storage::set_reporter_profile(&env, &profile);

        if let Some(reputation) = storage::reputation(&env) {
            ReputationClient::new(&env, &reputation).record_report_outcome(
                &env.current_contract_address(),
                &report.reporter,
                &approve,
            );
            ReputationClient::new(&env, &reputation)
                .record_validation(&env.current_contract_address(), &validator);
        }

        storage::extend_instance_ttl(&env);
        ReportValidated {
            report_id,
            validator,
            approved: approve,
        }
        .publish(&env);
        Ok(())
    }

    /// Retires a report from active consideration (e.g. stale or spam)
    /// without asserting it was true or false. Admin-only.
    pub fn archive_report(env: Env, caller: Address, report_id: u64) -> Result<(), Error> {
        caller.require_auth();
        let governance = storage::governance(&env);
        require_not_paused(&env, &governance)?;
        require_admin(&env, &governance, &caller)?;

        let mut report = storage::get_report(&env, report_id).ok_or(Error::ReportNotFound)?;
        report.status = ReportStatus::Archived;
        report.updated_at = env.ledger().timestamp();
        storage::set_report(&env, &report);
        storage::sync_entity_status(&env, &report.entity, &report);

        storage::extend_instance_ttl(&env);
        ReportArchived {
            report_id,
            archived_by: caller,
        }
        .publish(&env);
        Ok(())
    }

    /// Returns a single report by id.
    pub fn get_report(env: Env, report_id: u64) -> Result<ScamReport, Error> {
        storage::get_report(&env, report_id).ok_or(Error::ReportNotFound)
    }

    /// Returns the total number of reports ever submitted (the highest
    /// assigned report id).
    pub fn report_count(env: Env) -> u64 {
        storage::report_count(&env)
    }

    /// Lists reports in ascending id order, paginated.
    pub fn list_reports(env: Env, offset: u32, limit: u32) -> Vec<ScamReport> {
        let total = storage::report_count(&env);
        let limit = clamp_limit(limit) as u64;
        let mut out = Vec::new(&env);
        let mut id = offset as u64 + 1;
        let end = id + limit;
        while id < end && id <= total {
            if let Some(report) = storage::get_report(&env, id) {
                out.push_back(report);
            }
            id += 1;
        }
        out
    }

    /// Lists the ids of reports filed by `reporter`, most recent last,
    /// paginated.
    pub fn get_reports_by_reporter(
        env: Env,
        reporter: Address,
        offset: u32,
        limit: u32,
    ) -> Vec<ScamReport> {
        let history = storage::reporter_history(&env, &reporter);
        let page_ids = paginate(&env, &history, offset, limit);
        let mut out = Vec::new(&env);
        for id in page_ids.iter() {
            if let Some(report) = storage::get_report(&env, id) {
                out.push_back(report);
            }
        }
        out
    }

    /// Returns the reporter's activity profile, defaulting to a zeroed
    /// profile if they have never filed a report.
    pub fn get_reporter_profile(env: Env, reporter: Address) -> Reporter {
        storage::reporter_profile(&env, &reporter).unwrap_or(Reporter {
            address: reporter,
            reports_submitted: 0,
            reports_validated: 0,
            reports_rejected: 0,
            first_report_at: 0,
        })
    }

    /// Returns the aggregate record for a reported account, if any.
    pub fn get_account_record(env: Env, address: Address) -> Option<AccountRecord> {
        storage::account_record(&env, &address)
    }

    /// Returns the aggregate record for a reported asset issuer, if any.
    pub fn get_issuer_record(env: Env, issuer: Address) -> Option<IssuerRecord> {
        storage::issuer_record(&env, &issuer)
    }

    /// Returns the aggregate record for a reported asset, if any.
    pub fn get_asset_record(env: Env, asset_code: String, issuer: Address) -> Option<AssetRecord> {
        storage::asset_record(&env, &asset_code, &issuer)
    }

    /// Returns `true` if `address` has at least one report on file.
    pub fn is_account_flagged(env: Env, address: Address) -> bool {
        storage::account_record(&env, &address).is_some()
    }

    /// Returns `true` if `issuer` has at least one report on file.
    pub fn is_issuer_flagged(env: Env, issuer: Address) -> bool {
        storage::issuer_record(&env, &issuer).is_some()
    }

    /// Returns `true` if the asset has at least one report on file.
    pub fn is_asset_flagged(env: Env, asset_code: String, issuer: Address) -> bool {
        storage::asset_record(&env, &asset_code, &issuer).is_some()
    }

    /// Returns `true` if `domain` has at least one report on file.
    pub fn is_domain_flagged(env: Env, domain: String) -> bool {
        storage::domain_report_id(&env, &domain).is_some()
    }

    /// Returns `true` if `tx_hash` has at least one report on file.
    pub fn is_transaction_flagged(env: Env, tx_hash: BytesN<32>) -> bool {
        storage::transaction_report_id(&env, &tx_hash).is_some()
    }
}
