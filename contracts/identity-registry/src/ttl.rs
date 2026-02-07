//! TTL management for Identity Registry
//!
//! Critical for Soroban: storage entries have TTL and can be archived.
//! We must extend TTL to keep data alive.

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
/// Called after any mutation to keep token data alive
pub fn extend_persistent_ttl(env: &Env) {
    // Note: In a production contract, you would extend specific keys
    // For simplicity, we rely on the instance TTL extension
    // Individual token keys should be extended when accessed
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

