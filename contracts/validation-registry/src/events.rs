//! Event emissions for Validation Registry

use soroban_sdk::{Address, BytesN, Env, Symbol};

/// Emit when a validation request is submitted
pub fn emit_validation_requested(
    env: &Env,
    agent_id: u64,
    validator: &Address,
    requester: &Address,
    request_hash: &BytesN<32>,
) {
    let topics = (
        Symbol::new(env, "ValidationRequested"),
        agent_id,
        validator.clone(),
    );
    env.events().publish(topics, (requester.clone(), request_hash.clone()));
}

/// Emit when a validation response is submitted
pub fn emit_validation_responded(
    env: &Env,
    agent_id: u64,
    validator: &Address,
    request_hash: &BytesN<32>,
    response_code: i32,
    tag: &Symbol,
) {
    let topics = (
        Symbol::new(env, "ValidationResponded"),
        agent_id,
        validator.clone(),
    );
    env.events().publish(topics, (request_hash.clone(), response_code, tag.clone()));
}

