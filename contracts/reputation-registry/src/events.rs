//! Event emissions for Reputation Registry

use soroban_sdk::{Address, Env, Symbol};

/// Emit when feedback is given
pub fn emit_feedback_given(
    env: &Env,
    agent_id: u64,
    client: &Address,
    feedback_idx: u32,
    value: i128,
    tag1: &Symbol,
    tag2: &Symbol,
) {
    let topics = (
        Symbol::new(env, "FeedbackGiven"),
        agent_id,
        client.clone(),
        feedback_idx,
    );
    env.events().publish(topics, (value, tag1.clone(), tag2.clone()));
}

/// Emit when feedback is revoked
pub fn emit_feedback_revoked(env: &Env, agent_id: u64, client: &Address, feedback_idx: u32) {
    let topics = (
        Symbol::new(env, "FeedbackRevoked"),
        agent_id,
        client.clone(),
        feedback_idx,
    );
    env.events().publish(topics, ());
}

/// Emit when a response is added to feedback
pub fn emit_response_added(
    env: &Env,
    agent_id: u64,
    client: &Address,
    feedback_idx: u32,
    responder: &Address,
    tag: &Symbol,
) {
    let topics = (
        Symbol::new(env, "ResponseAdded"),
        agent_id,
        client.clone(),
        feedback_idx,
    );
    env.events().publish(topics, (responder.clone(), tag.clone()));
}

