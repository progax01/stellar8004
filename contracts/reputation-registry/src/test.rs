//! Tests for Reputation Registry

#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Symbol};

fn setup_contract() -> (Env, Address, Address, ReputationRegistryContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ReputationRegistryContract, ());
    let client = ReputationRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let identity_registry = Address::generate(&env);

    client.init(&admin, &identity_registry);

    (env, admin, identity_registry, client)
}

#[test]
fn test_init() {
    let (_env, admin, identity_registry, client) = setup_contract();

    assert_eq!(client.admin(), admin);
    assert_eq!(client.identity_registry(), identity_registry);
}

#[test]
fn test_give_feedback() {
    let (env, _admin, _identity_registry, client) = setup_contract();

    let feedback_client = Address::generate(&env);
    let agent_id: u64 = 1;
    let value: i128 = 100;
    let decimals: u32 = 2;
    let tag1 = Symbol::new(&env, "quality");
    let tag2 = Symbol::new(&env, "fast");
    let endpoint_uri = String::from_str(&env, "https://agent.example/api");
    let feedback_uri = String::from_str(&env, "https://example.com/feedback/1");
    let feedback_hash = BytesN::from_array(&env, &[0u8; 32]);

    let idx = client.give_feedback(
        &feedback_client,
        &agent_id,
        &value,
        &decimals,
        &tag1,
        &tag2,
        &endpoint_uri,
        &feedback_uri,
        &feedback_hash,
    );

    assert_eq!(idx, 0);

    let record = client.read_feedback(&agent_id, &feedback_client, &idx);
    assert_eq!(record.value, value);
    assert_eq!(record.decimals, decimals);
    assert_eq!(record.tag1, tag1);
    assert_eq!(record.tag2, tag2);
    assert!(!record.revoked);
}

#[test]
fn test_multiple_feedback() {
    let (env, _admin, _identity_registry, client) = setup_contract();

    let client1 = Address::generate(&env);
    let client2 = Address::generate(&env);
    let agent_id: u64 = 1;
    let tag1 = Symbol::new(&env, "quality");
    let tag2 = Symbol::new(&env, "fast");
    let empty_uri = String::from_str(&env, "");
    let empty_hash = BytesN::from_array(&env, &[0u8; 32]);

    // First client gives feedback
    let idx1 = client.give_feedback(
        &client1, &agent_id, &100, &2, &tag1, &tag2, &empty_uri, &empty_uri, &empty_hash,
    );

    // Second client gives feedback
    let idx2 = client.give_feedback(
        &client2, &agent_id, &-50, &2, &tag1, &tag2, &empty_uri, &empty_uri, &empty_hash,
    );

    // First client gives second feedback
    let idx3 = client.give_feedback(
        &client1, &agent_id, &75, &2, &tag1, &tag2, &empty_uri, &empty_uri, &empty_hash,
    );

    assert_eq!(idx1, 0);
    assert_eq!(idx2, 0);
    assert_eq!(idx3, 1);

    // Check agent clients
    let clients = client.get_agent_clients(&agent_id);
    assert_eq!(clients.len(), 2);
}

#[test]
fn test_revoke_feedback() {
    let (env, _admin, _identity_registry, client) = setup_contract();

    let feedback_client = Address::generate(&env);
    let agent_id: u64 = 1;
    let tag1 = Symbol::new(&env, "quality");
    let tag2 = Symbol::new(&env, "fast");
    let empty_uri = String::from_str(&env, "");
    let empty_hash = BytesN::from_array(&env, &[0u8; 32]);

    let idx = client.give_feedback(
        &feedback_client, &agent_id, &100, &2, &tag1, &tag2, &empty_uri, &empty_uri, &empty_hash,
    );

    // Revoke
    client.revoke_feedback(&feedback_client, &agent_id, &idx);

    let record = client.read_feedback(&agent_id, &feedback_client, &idx);
    assert!(record.revoked);
}

#[test]
fn test_append_response() {
    let (env, _admin, _identity_registry, client) = setup_contract();

    let feedback_client = Address::generate(&env);
    let responder = Address::generate(&env);
    let agent_id: u64 = 1;
    let tag1 = Symbol::new(&env, "quality");
    let tag2 = Symbol::new(&env, "fast");
    let empty_uri = String::from_str(&env, "");
    let empty_hash = BytesN::from_array(&env, &[0u8; 32]);

    let feedback_idx = client.give_feedback(
        &feedback_client, &agent_id, &100, &2, &tag1, &tag2, &empty_uri, &empty_uri, &empty_hash,
    );

    let response_uri = String::from_str(&env, "https://agent.example/response/1");
    let response_hash = BytesN::from_array(&env, &[1u8; 32]);
    let response_tag = Symbol::new(&env, "clarification");

    let response_idx = client.append_response(
        &responder,
        &agent_id,
        &feedback_client,
        &feedback_idx,
        &response_uri,
        &response_hash,
        &response_tag,
    );

    assert_eq!(response_idx, 0);

    let response = client.read_response(&agent_id, &feedback_client, &feedback_idx, &response_idx);
    assert_eq!(response.responder, responder);
    assert_eq!(response.response_uri, response_uri);
    assert_eq!(response.tag, response_tag);
}

#[test]
fn test_get_summary() {
    let (env, _admin, _identity_registry, client) = setup_contract();

    let client1 = Address::generate(&env);
    let client2 = Address::generate(&env);
    let agent_id: u64 = 1;
    let tag1 = Symbol::new(&env, "quality");
    let tag2 = Symbol::new(&env, "fast");
    let empty_uri = String::from_str(&env, "");
    let empty_hash = BytesN::from_array(&env, &[0u8; 32]);

    // Give positive feedback
    client.give_feedback(
        &client1, &agent_id, &100, &2, &tag1, &tag2, &empty_uri, &empty_uri, &empty_hash,
    );

    // Give negative feedback
    client.give_feedback(
        &client2, &agent_id, &-50, &2, &tag1, &tag2, &empty_uri, &empty_uri, &empty_hash,
    );

    // Get summary
    let clients_list = soroban_sdk::vec![&env, client1.clone(), client2.clone()];
    let summary = client.get_summary(&agent_id, &clients_list, &None, &None);

    assert_eq!(summary.count, 2);
    assert_eq!(summary.total_value, 50); // 100 + (-50)
    assert_eq!(summary.decimals, 2);
    assert_eq!(summary.positive_count, 1);
    assert_eq!(summary.negative_count, 1);
}

#[test]
fn test_read_all_feedback_with_filters() {
    let (env, _admin, _identity_registry, client) = setup_contract();

    let client1 = Address::generate(&env);
    let agent_id: u64 = 1;
    let tag_quality = Symbol::new(&env, "quality");
    let tag_speed = Symbol::new(&env, "speed");
    let tag_default = Symbol::new(&env, "default");
    let empty_uri = String::from_str(&env, "");
    let empty_hash = BytesN::from_array(&env, &[0u8; 32]);

    // Give feedback with different tags
    client.give_feedback(
        &client1, &agent_id, &100, &2, &tag_quality, &tag_default, &empty_uri, &empty_uri, &empty_hash,
    );
    client.give_feedback(
        &client1, &agent_id, &80, &2, &tag_speed, &tag_default, &empty_uri, &empty_uri, &empty_hash,
    );

    let clients_list = soroban_sdk::vec![&env, client1.clone()];

    // Read all
    let all = client.read_all_feedback(&agent_id, &clients_list, &None, &None, &false);
    assert_eq!(all.len(), 2);

    // Filter by tag1
    let quality_only = client.read_all_feedback(&agent_id, &clients_list, &Some(tag_quality), &None, &false);
    assert_eq!(quality_only.len(), 1);
    assert_eq!(quality_only.get(0).unwrap().value, 100);
}

#[test]
fn test_extend_ttl() {
    let (_env, _admin, _identity_registry, client) = setup_contract();
    client.extend_ttl();
}

