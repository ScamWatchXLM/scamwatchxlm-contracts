use scamwatchxlm_common::RiskLevel;
use soroban_sdk::{contractevent, Address};

#[contractevent]
pub struct ReportSubmitted {
    #[topic]
    pub report_id: u64,
    #[topic]
    pub reporter: Address,
    pub risk_level: RiskLevel,
}

#[contractevent]
pub struct ReportValidated {
    #[topic]
    pub report_id: u64,
    #[topic]
    pub validator: Address,
    pub approved: bool,
}

#[contractevent]
pub struct ReportArchived {
    #[topic]
    pub report_id: u64,
    pub archived_by: Address,
}
