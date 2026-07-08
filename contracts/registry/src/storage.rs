//! Thin wrappers around `env.storage()` so the contract logic in `lib.rs`
//! reads as business logic rather than storage plumbing.

use crate::types::{
    AccountRecord, AssetRecord, DataKey, EntityStats, IssuerRecord, ReportedEntity, Reporter,
    ScamReport,
};
use soroban_sdk::{Address, BytesN, Env, String, Vec};

/// Ledger close time is ~5 seconds, so a day is ~17280 ledgers.
const DAY_IN_LEDGERS: u32 = 17280;
const INSTANCE_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS * 30;
const INSTANCE_BUMP_AMOUNT: u32 = DAY_IN_LEDGERS * 60;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS * 90;
const PERSISTENT_BUMP_AMOUNT: u32 = DAY_IN_LEDGERS * 180;

pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

fn extend_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(
        key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn init(env: &Env, governance: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::Governance, governance);
    env.storage().instance().set(&DataKey::ReportCount, &0u64);
}

pub fn governance(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Governance)
        .expect("contract not initialized")
}

pub fn reputation(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Reputation)
}

pub fn set_reputation(env: &Env, reputation: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::Reputation, reputation);
}

pub fn next_report_id(env: &Env) -> u64 {
    let id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::ReportCount)
        .unwrap_or(0)
        + 1;
    env.storage().instance().set(&DataKey::ReportCount, &id);
    id
}

pub fn report_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::ReportCount)
        .unwrap_or(0)
}

pub fn get_report(env: &Env, id: u64) -> Option<ScamReport> {
    let key = DataKey::Report(id);
    let report = env.storage().persistent().get(&key);
    if report.is_some() {
        extend_persistent_ttl(env, &key);
    }
    report
}

pub fn set_report(env: &Env, report: &ScamReport) {
    let key = DataKey::Report(report.id);
    env.storage().persistent().set(&key, report);
    extend_persistent_ttl(env, &key);
}

pub fn reporter_history(env: &Env, reporter: &Address) -> Vec<u64> {
    let key = DataKey::ReporterHistory(reporter.clone());
    match env.storage().persistent().get(&key) {
        Some(history) => {
            extend_persistent_ttl(env, &key);
            history
        }
        None => Vec::new(env),
    }
}

pub fn append_reporter_history(env: &Env, reporter: &Address, report_id: u64) {
    let mut history = reporter_history(env, reporter);
    history.push_back(report_id);
    let key = DataKey::ReporterHistory(reporter.clone());
    env.storage().persistent().set(&key, &history);
    extend_persistent_ttl(env, &key);
}

pub fn reporter_profile(env: &Env, reporter: &Address) -> Option<Reporter> {
    let key = DataKey::ReporterProfile(reporter.clone());
    let profile = env.storage().persistent().get(&key);
    if profile.is_some() {
        extend_persistent_ttl(env, &key);
    }
    profile
}

pub fn set_reporter_profile(env: &Env, profile: &Reporter) {
    let key = DataKey::ReporterProfile(profile.address.clone());
    env.storage().persistent().set(&key, profile);
    extend_persistent_ttl(env, &key);
}

pub fn account_record(env: &Env, address: &Address) -> Option<AccountRecord> {
    let key = DataKey::AccountRecord(address.clone());
    let rec = env.storage().persistent().get(&key);
    if rec.is_some() {
        extend_persistent_ttl(env, &key);
    }
    rec
}

fn set_account_record(env: &Env, record: &AccountRecord) {
    let key = DataKey::AccountRecord(record.address.clone());
    env.storage().persistent().set(&key, record);
    extend_persistent_ttl(env, &key);
}

pub fn issuer_record(env: &Env, issuer: &Address) -> Option<IssuerRecord> {
    let key = DataKey::IssuerRecord(issuer.clone());
    let rec = env.storage().persistent().get(&key);
    if rec.is_some() {
        extend_persistent_ttl(env, &key);
    }
    rec
}

fn set_issuer_record(env: &Env, record: &IssuerRecord) {
    let key = DataKey::IssuerRecord(record.issuer.clone());
    env.storage().persistent().set(&key, record);
    extend_persistent_ttl(env, &key);
}

pub fn asset_record(env: &Env, asset_code: &String, issuer: &Address) -> Option<AssetRecord> {
    let key = DataKey::AssetRecord(asset_code.clone(), issuer.clone());
    let rec = env.storage().persistent().get(&key);
    if rec.is_some() {
        extend_persistent_ttl(env, &key);
    }
    rec
}

fn set_asset_record(env: &Env, record: &AssetRecord) {
    let key = DataKey::AssetRecord(record.asset_code.clone(), record.issuer.clone());
    env.storage().persistent().set(&key, record);
    extend_persistent_ttl(env, &key);
}

pub fn domain_report_id(env: &Env, domain: &String) -> Option<u64> {
    let key = DataKey::DomainReportId(domain.clone());
    let id = env.storage().persistent().get(&key);
    if id.is_some() {
        extend_persistent_ttl(env, &key);
    }
    id
}

fn set_domain_report_id(env: &Env, domain: &String, id: u64) {
    let key = DataKey::DomainReportId(domain.clone());
    env.storage().persistent().set(&key, &id);
    extend_persistent_ttl(env, &key);
}

pub fn transaction_report_id(env: &Env, tx_hash: &BytesN<32>) -> Option<u64> {
    let key = DataKey::TransactionReportId(tx_hash.clone());
    let id = env.storage().persistent().get(&key);
    if id.is_some() {
        extend_persistent_ttl(env, &key);
    }
    id
}

fn set_transaction_report_id(env: &Env, tx_hash: &BytesN<32>, id: u64) {
    let key = DataKey::TransactionReportId(tx_hash.clone());
    env.storage().persistent().set(&key, &id);
    extend_persistent_ttl(env, &key);
}

/// Folds a freshly-submitted report into the aggregate record for its
/// entity, creating the record on first sight. Also used as the
/// duplicate-prevention check: callers should reject the submission before
/// this is called if a record/report-id already exists for the entity.
pub fn record_new_report(env: &Env, entity: &ReportedEntity, report: &ScamReport) {
    match entity {
        ReportedEntity::Account(address) => {
            set_account_record(
                env,
                &AccountRecord {
                    address: address.clone(),
                    stats: EntityStats::first(report),
                },
            );
        }
        ReportedEntity::AssetIssuer(issuer) => {
            set_issuer_record(
                env,
                &IssuerRecord {
                    issuer: issuer.clone(),
                    stats: EntityStats::first(report),
                },
            );
        }
        ReportedEntity::Asset(asset_code, issuer) => {
            set_asset_record(
                env,
                &AssetRecord {
                    asset_code: asset_code.clone(),
                    issuer: issuer.clone(),
                    stats: EntityStats::first(report),
                },
            );
        }
        ReportedEntity::Domain(domain) => {
            set_domain_report_id(env, domain, report.id);
        }
        ReportedEntity::Transaction(tx_hash) => {
            set_transaction_report_id(env, tx_hash, report.id);
        }
    }
}

/// Updates the status carried by an entity's aggregate record after a report
/// referencing it is validated/rejected/archived. Domains and transactions
/// have no aggregate record (see [`DataKey`]), so there is nothing to update
/// for them beyond the [`ScamReport`] itself.
pub fn sync_entity_status(env: &Env, entity: &ReportedEntity, report: &ScamReport) {
    match entity {
        ReportedEntity::Account(address) => {
            if let Some(mut rec) = account_record(env, address) {
                if rec.stats.latest_report_id == report.id {
                    rec.stats.status = report.status;
                    set_account_record(env, &rec);
                }
            }
        }
        ReportedEntity::AssetIssuer(issuer) => {
            if let Some(mut rec) = issuer_record(env, issuer) {
                if rec.stats.latest_report_id == report.id {
                    rec.stats.status = report.status;
                    set_issuer_record(env, &rec);
                }
            }
        }
        ReportedEntity::Asset(asset_code, issuer) => {
            if let Some(mut rec) = asset_record(env, asset_code, issuer) {
                if rec.stats.latest_report_id == report.id {
                    rec.stats.status = report.status;
                    set_asset_record(env, &rec);
                }
            }
        }
        ReportedEntity::Domain(_) | ReportedEntity::Transaction(_) => {}
    }
}

/// Looks up the report id already on file for `entity`, if any, whether via
/// its aggregate record (Account/Issuer/Asset) or its dedicated
/// duplicate-prevention index (Domain/Transaction).
pub fn existing_report_id(env: &Env, entity: &ReportedEntity) -> Option<u64> {
    match entity {
        ReportedEntity::Account(address) => {
            account_record(env, address).map(|r| r.stats.latest_report_id)
        }
        ReportedEntity::AssetIssuer(issuer) => {
            issuer_record(env, issuer).map(|r| r.stats.latest_report_id)
        }
        ReportedEntity::Asset(asset_code, issuer) => {
            asset_record(env, asset_code, issuer).map(|r| r.stats.latest_report_id)
        }
        ReportedEntity::Domain(domain) => domain_report_id(env, domain),
        ReportedEntity::Transaction(tx_hash) => transaction_report_id(env, tx_hash),
    }
}
