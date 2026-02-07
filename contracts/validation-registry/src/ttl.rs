//! TTL management for Validation Registry

use soroban_sdk::Env;

/// TTL threshold - extend if remaining TTL is below this
pub const TTL_THRESHOLD: u32 = 100_000;

/// TTL extension target - extend to this many ledgers (~5 months on mainnet)
pub const TTL_EXTEND_TO: u32 = 1_000_000;

/// Extend instance storage TTL
pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

/// Extend persistent storage TTL
pub fn extend_persistent_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

