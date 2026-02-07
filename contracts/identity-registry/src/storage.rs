//! Storage helpers for Identity Registry

use soroban_sdk::{Address, BytesN, Env, String};

use crate::DataKey;

// ============================================================================
// Admin & Collection Info
// ============================================================================

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_name(env: &Env) -> Option<String> {
    env.storage().instance().get(&DataKey::Name)
}

pub fn set_name(env: &Env, name: &String) {
    env.storage().instance().set(&DataKey::Name, name);
}

pub fn get_symbol(env: &Env) -> Option<String> {
    env.storage().instance().get(&DataKey::Symbol)
}

pub fn set_symbol(env: &Env, symbol: &String) {
    env.storage().instance().set(&DataKey::Symbol, symbol);
}

// ============================================================================
// Token ID Counter
// ============================================================================

pub fn get_next_token_id(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::NextTokenId)
        .unwrap_or(0)
}

pub fn set_next_token_id(env: &Env, id: u64) {
    env.storage().instance().set(&DataKey::NextTokenId, &id);
}

// ============================================================================
// Total Supply & Balance
// ============================================================================

pub fn get_total_supply(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

pub fn set_total_supply(env: &Env, supply: u64) {
    env.storage().instance().set(&DataKey::TotalSupply, &supply);
}

pub fn increment_total_supply(env: &Env) {
    let supply = get_total_supply(env);
    set_total_supply(env, supply + 1);
}

pub fn get_balance(env: &Env, owner: &Address) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(owner.clone()))
        .unwrap_or(0)
}

pub fn increment_balance(env: &Env, owner: &Address) {
    let balance = get_balance(env, owner);
    env.storage()
        .persistent()
        .set(&DataKey::Balance(owner.clone()), &(balance + 1));
}

pub fn decrement_balance(env: &Env, owner: &Address) {
    let balance = get_balance(env, owner);
    if balance > 0 {
        env.storage()
            .persistent()
            .set(&DataKey::Balance(owner.clone()), &(balance - 1));
    }
}

// ============================================================================
// Token Owner
// ============================================================================

pub fn token_exists(env: &Env, token_id: u64) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::TokenOwner(token_id))
}

pub fn get_token_owner(env: &Env, token_id: u64) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::TokenOwner(token_id))
}

pub fn set_token_owner(env: &Env, token_id: u64, owner: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::TokenOwner(token_id), owner);
}

// ============================================================================
// Token URI
// ============================================================================

pub fn get_token_uri(env: &Env, token_id: u64) -> Option<String> {
    env.storage()
        .persistent()
        .get(&DataKey::TokenUri(token_id))
}

pub fn set_token_uri(env: &Env, token_id: u64, uri: &String) {
    env.storage()
        .persistent()
        .set(&DataKey::TokenUri(token_id), uri);
}

// ============================================================================
// Token Approval
// ============================================================================

pub fn get_token_approval(env: &Env, token_id: u64) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::TokenApproval(token_id))
}

pub fn set_token_approval(env: &Env, token_id: u64, approved: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::TokenApproval(token_id), approved);
}

pub fn clear_token_approval(env: &Env, token_id: u64) {
    env.storage()
        .persistent()
        .remove(&DataKey::TokenApproval(token_id));
}

// ============================================================================
// Operator Approval
// ============================================================================

pub fn is_operator_approved(env: &Env, owner: &Address, operator: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::OperatorApproval(owner.clone(), operator.clone()))
        .unwrap_or(false)
}

pub fn set_operator_approval(env: &Env, owner: &Address, operator: &Address, approved: bool) {
    let key = DataKey::OperatorApproval(owner.clone(), operator.clone());
    if approved {
        env.storage().persistent().set(&key, &true);
    } else {
        env.storage().persistent().remove(&key);
    }
}

// ============================================================================
// Agent Wallet (ERC-8004)
// ============================================================================

pub fn get_agent_wallet(env: &Env, token_id: u64) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::AgentWallet(token_id))
}

pub fn set_agent_wallet(env: &Env, token_id: u64, wallet: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::AgentWallet(token_id), wallet);
}

pub fn clear_agent_wallet(env: &Env, token_id: u64) {
    env.storage()
        .persistent()
        .remove(&DataKey::AgentWallet(token_id));
}

// ============================================================================
// Generic Metadata
// ============================================================================

pub fn get_metadata(env: &Env, token_id: u64, key: &BytesN<32>) -> Option<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::Metadata(token_id, key.clone()))
}

pub fn set_metadata(env: &Env, token_id: u64, key: &BytesN<32>, value: &BytesN<32>) {
    env.storage()
        .persistent()
        .set(&DataKey::Metadata(token_id, key.clone()), value);
}

