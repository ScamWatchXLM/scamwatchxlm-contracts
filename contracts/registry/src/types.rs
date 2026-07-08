use scamwatchxlm_common::{ReportStatus, RiskLevel};
use soroban_sdk::{contracttype, Address, BytesN, String};

/// A malicious entity that can be reported. Each variant carries whatever
/// data uniquely identifies that kind of entity, which doubles as the key
/// used for duplicate-prevention and lookups.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReportedEntity {
    /// A Stellar account (G... address) behaving maliciously.
    Account(Address),
    /// The issuing account of a scam asset.
    AssetIssuer(Address),
    /// An asset, identified by its code and issuing account.
    Asset(String, Address),
    /// A phishing domain, e.g. `"totally-legit-stellar.com"`.
    Domain(String),
    /// The hash of an on-chain transaction associated with a scam.
    Transaction(BytesN<32>),
}

/// A single report filed against a [`ReportedEntity`].
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScamReport {
    pub id: u64,
    pub entity: ReportedEntity,
    pub risk_level: RiskLevel,
    pub status: ReportStatus,
    pub reporter: Address,
    pub evidence_uri: String,
    pub notes: Option<String>,
    pub validator: Option<Address>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Per-reporter bookkeeping, independent of the reputation score kept by the
/// Reputation contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reporter {
    pub address: Address,
    pub reports_submitted: u32,
    pub reports_validated: u32,
    pub reports_rejected: u32,
    pub first_report_at: u64,
}

/// Aggregate statistics tracked for an entity across all reports filed
/// against it, regardless of individual report status.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityStats {
    pub report_count: u32,
    pub highest_risk: RiskLevel,
    pub status: ReportStatus,
    pub first_report_id: u64,
    pub latest_report_id: u64,
    pub first_reported_at: u64,
    pub last_reported_at: u64,
}

impl EntityStats {
    pub fn first(report: &ScamReport) -> Self {
        Self {
            report_count: 1,
            highest_risk: report.risk_level,
            status: report.status,
            first_report_id: report.id,
            latest_report_id: report.id,
            first_reported_at: report.created_at,
            last_reported_at: report.created_at,
        }
    }
}

/// Aggregate record for a reported Stellar account. Composition over
/// duplication: the shared counters live in [`EntityStats`], this just adds
/// the identifying key.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountRecord {
    pub address: Address,
    pub stats: EntityStats,
}

/// Aggregate record for a reported asset issuer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerRecord {
    pub issuer: Address,
    pub stats: EntityStats,
}

/// Aggregate record for a reported asset.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRecord {
    pub asset_code: String,
    pub issuer: Address,
    pub stats: EntityStats,
}

/// Storage keys used by the Registry contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Address of the Governance contract consulted for permissions.
    Governance,
    /// Address of the Reputation contract notified on report validation.
    /// Optional: the registry works standalone if this is never set.
    Reputation,
    /// Monotonically increasing counter used to assign report ids.
    ReportCount,
    /// `ScamReport` by id.
    Report(u64),
    /// `Vec<u64>` of report ids filed by a given reporter, for history
    /// retrieval and pagination.
    ReporterHistory(Address),
    /// `Reporter` profile by address.
    ReporterProfile(Address),
    /// `AccountRecord` by account address.
    AccountRecord(Address),
    /// `IssuerRecord` by issuer address.
    IssuerRecord(Address),
    /// `AssetRecord` by (asset code, issuer).
    AssetRecord(String, Address),
    /// Report id already filed against a domain, for duplicate prevention.
    /// Domains do not yet have a dedicated aggregate record type (community
    /// contribution opportunity: add a `DomainRecord` analogous to
    /// [`AccountRecord`]).
    DomainReportId(String),
    /// Report id already filed against a transaction hash, for duplicate
    /// prevention. Same future-work note as `DomainReportId` applies.
    TransactionReportId(BytesN<32>),
}
