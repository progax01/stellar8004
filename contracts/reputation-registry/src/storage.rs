//! Storage helpers for Reputation Registry

use soroban_sdk::{Address, Env, Vec};

use crate::{DataKey, FeedbackRecord, ResponseRecord};

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
// Feedback Storage
// ============================================================================

pub fn feedback_exists(env: &Env, agent_id: u64, client: &Address, idx: u32) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Feedback(agent_id, client.clone(), idx))
}

pub fn get_feedback(
    env: &Env,
    agent_id: u64,
    client: &Address,
    idx: u32,
) -> Option<FeedbackRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Feedback(agent_id, client.clone(), idx))
}

pub fn set_feedback(
    env: &Env,
    agent_id: u64,
    client: &Address,
    idx: u32,
    record: &FeedbackRecord,
) {
    env.storage()
        .persistent()
        .set(&DataKey::Feedback(agent_id, client.clone(), idx), record);
}

pub fn get_feedback_count(env: &Env, agent_id: u64, client: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::FeedbackCount(agent_id, client.clone()))
        .unwrap_or(0)
}

pub fn set_feedback_count(env: &Env, agent_id: u64, client: &Address, count: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::FeedbackCount(agent_id, client.clone()), &count);
}

// ============================================================================
// Response Storage
// ============================================================================

pub fn get_response(
    env: &Env,
    agent_id: u64,
    client: &Address,
    feedback_idx: u32,
    response_idx: u32,
) -> Option<ResponseRecord> {
    env.storage().persistent().get(&DataKey::Response(
        agent_id,
        client.clone(),
        feedback_idx,
        response_idx,
    ))
}

pub fn set_response(
    env: &Env,
    agent_id: u64,
    client: &Address,
    feedback_idx: u32,
    response_idx: u32,
    record: &ResponseRecord,
) {
    env.storage().persistent().set(
        &DataKey::Response(agent_id, client.clone(), feedback_idx, response_idx),
        record,
    );
}

pub fn get_response_count(env: &Env, agent_id: u64, client: &Address, feedback_idx: u32) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::ResponseCount(
            agent_id,
            client.clone(),
            feedback_idx,
        ))
        .unwrap_or(0)
}

pub fn set_response_count(
    env: &Env,
    agent_id: u64,
    client: &Address,
    feedback_idx: u32,
    count: u32,
) {
    env.storage().persistent().set(
        &DataKey::ResponseCount(agent_id, client.clone(), feedback_idx),
        &count,
    );
}

// ============================================================================
// Agent Clients Tracking
// ============================================================================

pub fn get_agent_clients(env: &Env, agent_id: u64) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::AgentClients(agent_id))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_client_to_agent(env: &Env, agent_id: u64, client: &Address) {
    let mut clients = get_agent_clients(env, agent_id);

    // Check if client already exists
    let mut exists = false;
    for c in clients.iter() {
        if c == *client {
            exists = true;
            break;
        }
    }

    if !exists {
        clients.push_back(client.clone());
        env.storage()
            .persistent()
            .set(&DataKey::AgentClients(agent_id), &clients);
    }
}

