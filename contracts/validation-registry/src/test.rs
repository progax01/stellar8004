//! Tests for Validation Registry

#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Symbol};

fn setup_contract() -> (Env, Address, Address, ValidationRegistryContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ValidationRegistryContract, ());
    let client = ValidationRegistryContractClient::new(&env, &contract_id);

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
fn test_validation_request() {
    let (env, _admin, _identity_registry, client) = setup_contract();

    let requester = Address::generate(&env);
    let validator = Address::generate(&env);
    let agent_id: u64 = 1;
    let request_uri = String::from_str(&env, "https://example.com/validation-request");
    let request_hash = BytesN::from_array(&env, &[1u8; 32]);

    // Note: In a real scenario, requester would need to be owner/operator of agent_id
    // For testing, we mock all auths
    client.validation_request(
        &requester,
        &agent_id,
        &validator,
        &request_uri,
        &request_hash,
    );

    // Verify request was stored
    let request = client.get_request(&request_hash);
    assert_eq!(request.agent_id, agent_id);
    assert_eq!(request.validator, validator);
    assert_eq!(request.requester, requester);
    assert!(!request.responded);

    // Verify it's in agent's list
    let agent_requests = client.get_agent_validations(&agent_id);
    assert_eq!(agent_requests.len(), 1);

    // Verify it's in validator's list
    let validator_requests = client.get_validator_requests(&validator);
    assert_eq!(validator_requests.len(), 1);
}

#[test]
fn test_validation_response() {
    let (env, _admin, _identity_registry, client) = setup_contract();

    let requester = Address::generate(&env);
    let validator = Address::generate(&env);
    let agent_id: u64 = 1;
    let request_uri = String::from_str(&env, "https://example.com/validation-request");
    let request_hash = BytesN::from_array(&env, &[1u8; 32]);

    // Submit request
    client.validation_request(
        &requester,
        &agent_id,
        &validator,
        &request_uri,
        &request_hash,
    );

    // Submit response
    let response_uri = String::from_str(&env, "https://example.com/validation-response");
    let response_hash = BytesN::from_array(&env, &[2u8; 32]);
    let tag = Symbol::new(&env, "identity");

    client.validation_response(
        &validator,
        &request_hash,
        &response_codes::VALID,
        &response_uri,
        &response_hash,
        &tag,
    );

    // Verify status
    let status = client.get_validation_status(&request_hash);
    assert!(status.request.responded);
    assert!(status.has_response);
    assert_eq!(status.response_code, response_codes::VALID);
    assert_eq!(status.response_tag, tag);
}

#[test]
fn test_get_summary() {
    let (env, _admin, _identity_registry, client) = setup_contract();

    let requester = Address::generate(&env);
    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let agent_id: u64 = 1;
    let request_uri = String::from_str(&env, "https://example.com/validation-request");

    // Submit multiple requests
    let hash1 = BytesN::from_array(&env, &[1u8; 32]);
    let hash2 = BytesN::from_array(&env, &[2u8; 32]);

    client.validation_request(&requester, &agent_id, &validator1, &request_uri, &hash1);
    client.validation_request(&requester, &agent_id, &validator2, &request_uri, &hash2);

    // Submit responses
    let response_uri = String::from_str(&env, "https://example.com/response");
    let response_hash = BytesN::from_array(&env, &[3u8; 32]);
    let tag = Symbol::new(&env, "identity");

    client.validation_response(
        &validator1,
        &hash1,
        &response_codes::VALID,
        &response_uri,
        &response_hash,
        &tag,
    );

    client.validation_response(
        &validator2,
        &hash2,
        &response_codes::INVALID,
        &response_uri,
        &response_hash,
        &tag,
    );

    // Get summary
    let validators = soroban_sdk::vec![&env, validator1.clone(), validator2.clone()];
    let summary = client.get_summary(&agent_id, &validators, &None);

    assert_eq!(summary.total_requests, 2);
    assert_eq!(summary.responded_count, 2);
    assert_eq!(summary.valid_count, 1);
    assert_eq!(summary.invalid_count, 1);
    assert_eq!(summary.other_count, 0);
}

#[test]
fn test_cannot_respond_twice() {
    let (env, _admin, _identity_registry, client) = setup_contract();

    let requester = Address::generate(&env);
    let validator = Address::generate(&env);
    let agent_id: u64 = 1;
    let request_uri = String::from_str(&env, "https://example.com/validation-request");
    let request_hash = BytesN::from_array(&env, &[1u8; 32]);

    // Submit request
    client.validation_request(
        &requester,
        &agent_id,
        &validator,
        &request_uri,
        &request_hash,
    );

    // First response
    let response_uri = String::from_str(&env, "https://example.com/response");
    let response_hash = BytesN::from_array(&env, &[2u8; 32]);
    let tag = Symbol::new(&env, "identity");

    client.validation_response(
        &validator,
        &request_hash,
        &response_codes::VALID,
        &response_uri,
        &response_hash,
        &tag,
    );

    // Second response should fail
    let result = std::panic::catch_unwind(|| {
        client.validation_response(
            &validator,
            &request_hash,
            &response_codes::INVALID,
            &response_uri,
            &response_hash,
            &tag,
        );
    });

    assert!(result.is_err());
}

#[test]
fn test_wrong_validator_cannot_respond() {
    let (env, _admin, _identity_registry, client) = setup_contract();

    let requester = Address::generate(&env);
    let validator = Address::generate(&env);
    let wrong_validator = Address::generate(&env);
    let agent_id: u64 = 1;
    let request_uri = String::from_str(&env, "https://example.com/validation-request");
    let request_hash = BytesN::from_array(&env, &[1u8; 32]);

    // Submit request
    client.validation_request(
        &requester,
        &agent_id,
        &validator,
        &request_uri,
        &request_hash,
    );

    // Wrong validator response should fail
    let response_uri = String::from_str(&env, "https://example.com/response");
    let response_hash = BytesN::from_array(&env, &[2u8; 32]);
    let tag = Symbol::new(&env, "identity");

    let result = std::panic::catch_unwind(|| {
        client.validation_response(
            &wrong_validator,
            &request_hash,
            &response_codes::VALID,
            &response_uri,
            &response_hash,
            &tag,
        );
    });

    assert!(result.is_err());
}

#[test]
fn test_extend_ttl() {
    let (_env, _admin, _identity_registry, client) = setup_contract();
    client.extend_ttl();
}

