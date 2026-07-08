use soroban_sdk::contracttype;

/// Severity of a reported scam entity, from least to most dangerous.
///
/// Stored as an explicit integer so ordering is stable across contract
/// upgrades and so it can be compared with plain `<`/`>` via [`RiskLevel::weight`].
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RiskLevel {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

impl RiskLevel {
    /// Numeric weight used to compare severities, e.g. when folding a new
    /// report into an entity's aggregate "highest risk seen" value.
    pub fn weight(&self) -> u32 {
        *self as u32
    }
}

/// Lifecycle state of a [`crate::types::Role`]-gated report in the Registry
/// contract.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ReportStatus {
    /// Submitted, awaiting validator review.
    Pending = 0,
    /// Confirmed as accurate by a validator.
    Validated = 1,
    /// Confirmed as inaccurate by a validator.
    Rejected = 2,
    /// Flagged for further review after being previously decided.
    Disputed = 3,
    /// Retired from active consideration by an admin (e.g. stale or spam).
    Archived = 4,
}

/// Roles recognized by the Governance contract's access control model.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Role {
    /// The single account that can manage admins and authorize upgrades.
    Owner = 0,
    /// Trusted operators that manage validators and can pause the system.
    Admin = 1,
    /// Trusted reviewers that can validate or reject submitted reports.
    Validator = 2,
}
