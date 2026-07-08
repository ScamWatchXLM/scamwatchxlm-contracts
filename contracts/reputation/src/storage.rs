use crate::types::{DataKey, ReputationProfile};
use soroban_sdk::{Address, Env};

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

pub fn init(env: &Env, governance: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::Governance, governance);
}

pub fn governance(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Governance)
        .expect("contract not initialized")
}

pub fn registry(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Registry)
}

pub fn set_registry(env: &Env, registry: &Address) {
    env.storage().instance().set(&DataKey::Registry, registry);
}

pub fn profile(env: &Env, account: &Address) -> Option<ReputationProfile> {
    let key = DataKey::Profile(account.clone());
    let profile = env.storage().persistent().get(&key);
    if profile.is_some() {
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }
    profile
}

pub fn set_profile(env: &Env, profile: &ReputationProfile) {
    let key = DataKey::Profile(profile.account.clone());
    env.storage().persistent().set(&key, profile);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}
