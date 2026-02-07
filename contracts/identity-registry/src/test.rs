//! Tests for Identity Registry

#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup_contract() -> (Env, Address, IdentityRegistryContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let name = String::from_str(&env, "AgentRegistry");
    let symbol = String::from_str(&env, "AGENT");

    client.init(&admin, &name, &symbol);

    (env, admin, client)
}

#[test]
fn test_init() {
    let (env, admin, client) = setup_contract();

    assert_eq!(client.name(), String::from_str(&env, "AgentRegistry"));
    assert_eq!(client.symbol(), String::from_str(&env, "AGENT"));
    assert_eq!(client.admin(), admin);
    assert_eq!(client.total_supply(), 0);
}

#[test]
fn test_register_agent() {
    let (env, _admin, client) = setup_contract();

    let owner = Address::generate(&env);
    let agent_uri = String::from_str(&env, "https://example.com/agent.json");

    let token_id = client.register(&owner, &agent_uri);

    assert_eq!(token_id, 0);
    assert_eq!(client.owner_of(&token_id), owner);
    assert_eq!(client.token_uri(&token_id), agent_uri);
    assert_eq!(client.get_agent_wallet(&token_id), owner);
    assert_eq!(client.balance_of(&owner), 1);
    assert_eq!(client.total_supply(), 1);
}

#[test]
fn test_register_multiple_agents() {
    let (env, _admin, client) = setup_contract();

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let uri1 = String::from_str(&env, "https://example.com/agent1.json");
    let uri2 = String::from_str(&env, "https://example.com/agent2.json");

    let token_id1 = client.register(&owner1, &uri1);
    let token_id2 = client.register(&owner2, &uri2);

    assert_eq!(token_id1, 0);
    assert_eq!(token_id2, 1);
    assert_eq!(client.total_supply(), 2);
    assert_eq!(client.balance_of(&owner1), 1);
    assert_eq!(client.balance_of(&owner2), 1);
}

#[test]
fn test_set_agent_uri() {
    let (env, _admin, client) = setup_contract();

    let owner = Address::generate(&env);
    let agent_uri = String::from_str(&env, "https://example.com/agent.json");
    let new_uri = String::from_str(&env, "https://example.com/agent_v2.json");

    let token_id = client.register(&owner, &agent_uri);
    client.set_agent_uri(&owner, &token_id, &new_uri);

    assert_eq!(client.token_uri(&token_id), new_uri);
}

#[test]
fn test_set_agent_wallet() {
    let (env, _admin, client) = setup_contract();

    let owner = Address::generate(&env);
    let new_wallet = Address::generate(&env);
    let agent_uri = String::from_str(&env, "https://example.com/agent.json");

    let token_id = client.register(&owner, &agent_uri);

    // Dual auth: both owner and new_wallet must sign
    client.set_agent_wallet(&owner, &token_id, &new_wallet);

    assert_eq!(client.get_agent_wallet(&token_id), new_wallet);
}

#[test]
fn test_transfer_clears_agent_wallet() {
    let (env, _admin, client) = setup_contract();

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let agent_uri = String::from_str(&env, "https://example.com/agent.json");

    let token_id = client.register(&owner, &agent_uri);

    // Verify agent wallet is set
    assert_eq!(client.get_agent_wallet(&token_id), owner);

    // Transfer token
    client.transfer_from(&owner, &new_owner, &token_id);

    // Verify ownership changed
    assert_eq!(client.owner_of(&token_id), new_owner);

    // Verify agent wallet is cleared (ERC-8004 requirement)
    // This should panic as wallet is cleared
    let result = std::panic::catch_unwind(|| {
        client.get_agent_wallet(&token_id)
    });
    assert!(result.is_err());
}

#[test]
fn test_approve_and_transfer() {
    let (env, _admin, client) = setup_contract();

    let owner = Address::generate(&env);
    let approved = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let agent_uri = String::from_str(&env, "https://example.com/agent.json");

    let token_id = client.register(&owner, &agent_uri);

    // Approve
    client.approve(&owner, &approved, &token_id);
    assert_eq!(client.get_approved(&token_id), Some(approved.clone()));

    // Transfer by approved address
    client.transfer_from(&approved, &new_owner, &token_id);
    assert_eq!(client.owner_of(&token_id), new_owner);

    // Approval should be cleared
    assert_eq!(client.get_approved(&token_id), None);
}

#[test]
fn test_operator_approval() {
    let (env, _admin, client) = setup_contract();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let agent_uri = String::from_str(&env, "https://example.com/agent.json");

    let token_id = client.register(&owner, &agent_uri);

    // Set operator approval
    client.set_approval_for_all(&owner, &operator, &true);
    assert!(client.is_approved_for_all(&owner, &operator));

    // Operator can transfer
    client.transfer_from(&operator, &new_owner, &token_id);
    assert_eq!(client.owner_of(&token_id), new_owner);

    // Revoke operator approval
    client.set_approval_for_all(&owner, &operator, &false);
    assert!(!client.is_approved_for_all(&owner, &operator));
}

#[test]
fn test_set_metadata() {
    let (env, _admin, client) = setup_contract();

    let owner = Address::generate(&env);
    let agent_uri = String::from_str(&env, "https://example.com/agent.json");

    let token_id = client.register(&owner, &agent_uri);

    let key = BytesN::from_array(&env, &[1u8; 32]);
    let value = BytesN::from_array(&env, &[2u8; 32]);

    client.set_metadata(&owner, &token_id, &key, &value);

    let retrieved = client.get_metadata(&token_id, &key);
    assert_eq!(retrieved, value);
}

#[test]
fn test_extend_ttl() {
    let (_env, _admin, client) = setup_contract();

    // Should not panic
    client.extend_ttl();
}

