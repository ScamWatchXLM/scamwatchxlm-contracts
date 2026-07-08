use soroban_sdk::contracterror;

/// Errors returned by the Registry contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Caller is not a recognized admin or validator for the action
    /// attempted.
    NotAuthorized = 1,
    /// The Governance contract reports the system is paused.
    ContractPaused = 2,
    /// A report already exists for this exact entity.
    DuplicateReport = 3,
    /// No report exists with the given id.
    ReportNotFound = 4,
    /// `evidence_uri` (or another string input) is empty or too long.
    InvalidInput = 5,
    /// The report is not in a state that allows the requested transition
    /// (e.g. validating an already-archived report).
    InvalidStatusTransition = 6,
}
