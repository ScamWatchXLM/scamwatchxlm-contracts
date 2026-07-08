//! Shared types and cross-contract interfaces for the ScamWatchXLM contract
//! suite (`registry`, `reputation`, `governance`).
//!
//! This crate intentionally contains no `#[contract]` of its own. It exists
//! so the three contracts can share data models and a typed client for
//! calling the Governance contract without depending on each other's crates.
#![no_std]

pub mod governance;
pub mod pagination;
pub mod reputation;
pub mod types;

pub use governance::GovernanceClient;
pub use reputation::ReputationClient;
pub use types::{ReportStatus, RiskLevel, Role};
