//! Event emissions for Identity Registry

use soroban_sdk::{Address, Env, String, Symbol};

/// Emit when a new agent is registered
pub fn emit_agent_registered(env: &Env, token_id: u64, owner: &Address, agent_uri: &String) {
    let topics = (Symbol::new(env, "AgentRegistered"), token_id, owner.clone());
    env.events().publish(topics, agent_uri.clone());
}

/// Emit when agent URI is updated
pub fn emit_agent_uri_updated(env: &Env, token_id: u64, new_uri: &String) {
    let topics = (Symbol::new(env, "AgentURIUpdated"), token_id);
    env.events().publish(topics, new_uri.clone());
}

/// Emit when agent wallet is updated
pub fn emit_agent_wallet_updated(env: &Env, token_id: u64, new_wallet: &Address) {
    let topics = (Symbol::new(env, "AgentWalletUpdated"), token_id);
    env.events().publish(topics, new_wallet.clone());
}

/// Emit on token transfer (SEP-50)
pub fn emit_transfer(env: &Env, from: &Address, to: &Address, token_id: u64) {
    let topics = (
        Symbol::new(env, "Transfer"),
        from.clone(),
        to.clone(),
        token_id,
    );
    env.events().publish(topics, ());
}

/// Emit on token approval (SEP-50)
pub fn emit_approval(env: &Env, owner: &Address, approved: &Address, token_id: u64) {
    let topics = (
        Symbol::new(env, "Approval"),
        owner.clone(),
        approved.clone(),
        token_id,
    );
    env.events().publish(topics, ());
}

/// Emit on operator approval (SEP-50)
pub fn emit_approval_for_all(env: &Env, owner: &Address, operator: &Address, approved: bool) {
    let topics = (
        Symbol::new(env, "ApprovalForAll"),
        owner.clone(),
        operator.clone(),
    );
    env.events().publish(topics, approved);
}

