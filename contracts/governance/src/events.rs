use soroban_sdk::{contractevent, Address, BytesN};

#[contractevent]
pub struct OwnerChanged {
    #[topic]
    pub previous_owner: Address,
    pub new_owner: Address,
}

#[contractevent]
pub struct AdminAdded {
    #[topic]
    pub admin: Address,
    pub added_by: Address,
}

#[contractevent]
pub struct AdminRemoved {
    #[topic]
    pub admin: Address,
    pub removed_by: Address,
}

#[contractevent]
pub struct ValidatorAdded {
    #[topic]
    pub validator: Address,
    pub added_by: Address,
}

#[contractevent]
pub struct ValidatorRemoved {
    #[topic]
    pub validator: Address,
    pub removed_by: Address,
}

#[contractevent]
pub struct SystemPaused {
    #[topic]
    pub by: Address,
}

#[contractevent]
pub struct SystemUnpaused {
    #[topic]
    pub by: Address,
}

#[contractevent]
pub struct UpgradeProposed {
    #[topic]
    pub wasm_hash: BytesN<32>,
    pub proposed_by: Address,
    pub proposed_at: u64,
}

#[contractevent]
pub struct UpgradeExecuted {
    #[topic]
    pub wasm_hash: BytesN<32>,
}

#[contractevent]
pub struct UpgradeCancelled {
    #[topic]
    pub wasm_hash: BytesN<32>,
    pub cancelled_by: Address,
}
