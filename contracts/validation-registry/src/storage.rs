//! Storage helpers for Validation Registry

use soroban_sdk::{Address, BytesN, Env, Vec};

use crate::{DataKey, ValidationRequest, ValidationResponse};

// ============================================================================
// Admin & Config
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

pub fn get_identity_registry(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::IdentityRegistry)
}

pub fn set_identity_registry(env: &Env, registry: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::IdentityRegistry, registry);
}

// ============================================================================
// Request Storage
// ============================================================================

pub fn request_exists(env: &Env, request_hash: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Request(request_hash.clone()))
}

pub fn get_request(env: &Env, request_hash: &BytesN<32>) -> Option<ValidationRequest> {
    env.storage()
        .persistent()
        .get(&DataKey::Request(request_hash.clone()))
}

pub fn set_request(env: &Env, request_hash: &BytesN<32>, request: &ValidationRequest) {
    env.storage()
        .persistent()
        .set(&DataKey::Request(request_hash.clone()), request);
}

// ============================================================================
// Response Storage
// ============================================================================

pub fn get_response(env: &Env, request_hash: &BytesN<32>) -> Option<ValidationResponse> {
    env.storage()
        .persistent()
        .get(&DataKey::Response(request_hash.clone()))
}

pub fn set_response(env: &Env, request_hash: &BytesN<32>, response: &ValidationResponse) {
    env.storage()
        .persistent()
        .set(&DataKey::Response(request_hash.clone()), response);
}

// ============================================================================
// Agent Requests Tracking
// ============================================================================

pub fn get_agent_requests(env: &Env, agent_id: u64) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::AgentRequests(agent_id))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_request_to_agent(env: &Env, agent_id: u64, request_hash: &BytesN<32>) {
    let mut requests = get_agent_requests(env, agent_id);
    requests.push_back(request_hash.clone());
    env.storage()
        .persistent()
        .set(&DataKey::AgentRequests(agent_id), &requests);
}

// ============================================================================
// Validator Requests Tracking
// ============================================================================

pub fn get_validator_requests(env: &Env, validator: &Address) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::ValidatorRequests(validator.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_request_to_validator(env: &Env, validator: &Address, request_hash: &BytesN<32>) {
    let mut requests = get_validator_requests(env, validator);
    requests.push_back(request_hash.clone());
    env.storage()
        .persistent()
        .set(&DataKey::ValidatorRequests(validator.clone()), &requests);
}

