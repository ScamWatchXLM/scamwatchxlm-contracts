use soroban_sdk::contracterror;

/// Errors returned by the Governance contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Caller is neither the owner nor an admin (or not the owner, where the
    /// owner specifically is required).
    NotAuthorized = 1,
    /// Target address is already an admin.
    AlreadyAdmin = 2,
    /// Target address is not an admin.
    NotAdmin = 3,
    /// Target address is already a validator.
    AlreadyValidator = 4,
    /// Target address is not a validator.
    NotValidator = 5,
    /// The system is already paused.
    AlreadyPaused = 6,
    /// The system is not currently paused.
    NotPaused = 7,
    /// There is no upgrade awaiting execution.
    NoPendingUpgrade = 8,
    /// An upgrade is already pending; cancel or execute it first.
    UpgradeAlreadyPending = 9,
    /// The upgrade timelock has not yet elapsed.
    TimelockNotElapsed = 10,
    /// The two addresses supplied must be different.
    SameAddress = 11,
    /// `offset`/`limit` pagination arguments are out of range.
    InvalidPagination = 12,
}
