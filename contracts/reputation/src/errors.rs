use soroban_sdk::contracterror;

/// Errors returned by the Reputation contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Caller is not the registered Registry contract, nor an admin, for an
    /// action that requires one of those.
    NotAuthorized = 1,
    /// No Registry contract has been configured yet.
    RegistryNotSet = 2,
    /// A manual `reward`/`penalize` amount of `0` was supplied.
    InvalidAmount = 3,
}
