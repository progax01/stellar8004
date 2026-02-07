//! # Validation Registry Contract (ERC-8004 on Soroban)
//!
//! Supports request/response hooks for validators. Implements ERC-8004 validation semantics.
//! Note: Per ERC-8004 reference repo, this is "still under active update" so we implement
//! the minimal surface and keep it upgradeable.
//!
//! ## Features
//! - Validation request by owner/operator
//! - Validation response by designated validator
//! - Status tracking and summary aggregation
//! - Upgradeable design

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror,
    Address, BytesN, Env, IntoVal, String, Symbol, Val, Vec,
};

mod storage;
mod events;
mod ttl;

pub use storage::*;
pub use events::*;
pub use ttl::*;

#[cfg(test)]
mod test;

// ============================================================================
// Errors
// ============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Contract already initialized
    AlreadyInitialized = 1,
    /// Contract not initialized
    NotInitialized = 2,
    /// Not authorized
    NotAuthorized = 3,
    /// Request not found
    RequestNotFound = 4,
    /// Not the designated validator
    NotValidator = 5,
    /// Request already responded
    AlreadyResponded = 6,
    /// Invalid request hash
    InvalidRequestHash = 7,
    /// Agent not found
    AgentNotFound = 8,
    /// Empty validator list
    EmptyValidatorList = 9,
}

// ============================================================================
// Response Codes (standardized)
// ============================================================================

/// Standard validation response codes
pub mod response_codes {
    /// Validation is pending
    pub const PENDING: i32 = 0;
    /// Agent is valid
    pub const VALID: i32 = 1;
    /// Agent is invalid
    pub const INVALID: i32 = 2;
    /// Unable to validate (inconclusive)
    pub const UNABLE_TO_VALIDATE: i32 = 3;
    /// Validation timed out
    pub const TIMEOUT: i32 = -1;
    /// Malformed request
    pub const MALFORMED_REQUEST: i32 = -2;
}

// ============================================================================
// Data Types
// ============================================================================

/// Validation request record
#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidationRequest {
    /// Agent ID being validated
    pub agent_id: u64,
    /// Designated validator
    pub validator: Address,
    /// Requester (owner/operator)
    pub requester: Address,
    /// URI with request details
    pub request_uri: String,
    /// Hash of request content
    pub request_hash: BytesN<32>,
    /// Ledger sequence when request was made
    pub timestamp_ledger: u32,
    /// Whether response has been received
    pub responded: bool,
}

/// Validation response record
#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidationResponse {
    /// Response code (see response_codes module)
    pub response_code: i32,
    /// URI with response details
    pub response_uri: String,
    /// Hash of response content
    pub response_hash: BytesN<32>,
    /// Response category tag
    pub tag: Symbol,
    /// Ledger sequence when response was made
    pub timestamp_ledger: u32,
}

/// Combined request + response status
#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidationStatus {
    pub request: ValidationRequest,
    /// Whether a response exists
    pub has_response: bool,
    /// Response code (0 if no response)
    pub response_code: i32,
    /// Response URI (empty if no response)
    pub response_uri: String,
    /// Response hash (zeroed if no response)
    pub response_hash: BytesN<32>,
    /// Response tag (empty if no response)
    pub response_tag: Symbol,
    /// Response timestamp (0 if no response)
    pub response_timestamp: u32,
}

/// Summary of validations for an agent
#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidationSummary {
    /// Total validation requests
    pub total_requests: u32,
    /// Requests with responses
    pub responded_count: u32,
    /// Count of VALID responses
    pub valid_count: u32,
    /// Count of INVALID responses
    pub invalid_count: u32,
    /// Count of other responses
    pub other_count: u32,
}

// ============================================================================
// Storage Keys
// ============================================================================

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Contract admin
    Admin,
    /// Identity registry contract address
    IdentityRegistry,
    /// Validation request by hash: Request(request_hash) -> ValidationRequest
    Request(BytesN<32>),
    /// Validation response by hash: Response(request_hash) -> ValidationResponse
    Response(BytesN<32>),
    /// Agent's validation request hashes: AgentRequests(agent_id) -> Vec<BytesN<32>>
    AgentRequests(u64),
    /// Validator's request hashes: ValidatorRequests(validator) -> Vec<BytesN<32>>
    ValidatorRequests(Address),
}

// ============================================================================
// Contract
// ============================================================================

#[contract]
pub struct ValidationRegistryContract;

#[contractimpl]
impl ValidationRegistryContract {
    // ========================================================================
    // Initialization
    // ========================================================================

    /// Initialize the validation registry
    ///
    /// # Arguments
    /// * `admin` - Contract administrator
    /// * `identity_registry` - Address of the Identity Registry contract
    pub fn init(env: Env, admin: Address, identity_registry: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();

        storage::set_admin(&env, &admin);
        storage::set_identity_registry(&env, &identity_registry);

        ttl::extend_instance_ttl(&env);

        Ok(())
    }

    // ========================================================================
    // Validation Request
    // ========================================================================

    /// Submit a validation request (owner/operator only)
    ///
    /// # Arguments
    /// * `caller` - Requester (must be owner or approved operator)
    /// * `agent_id` - Token ID of the agent to validate
    /// * `validator` - Address of the designated validator
    /// * `request_uri` - URI pointing to validation request details
    /// * `request_hash` - Hash of the request content
    pub fn validation_request(
        env: Env,
        caller: Address,
        agent_id: u64,
        validator: Address,
        request_uri: String,
        request_hash: BytesN<32>,
    ) -> Result<(), Error> {
        caller.require_auth();

        // Verify caller is owner or approved operator
        Self::require_owner_or_operator(&env, agent_id, &caller)?;

        // Check if request hash already exists
        if storage::request_exists(&env, &request_hash) {
            return Err(Error::InvalidRequestHash);
        }

        let request = ValidationRequest {
            agent_id,
            validator: validator.clone(),
            requester: caller.clone(),
            request_uri,
            request_hash: request_hash.clone(),
            timestamp_ledger: env.ledger().sequence(),
            responded: false,
        };

        storage::set_request(&env, &request_hash, &request);
        storage::add_request_to_agent(&env, agent_id, &request_hash);
        storage::add_request_to_validator(&env, &validator, &request_hash);

        ttl::extend_persistent_ttl(&env);

        events::emit_validation_requested(
            &env,
            agent_id,
            &validator,
            &caller,
            &request_hash,
        );

        Ok(())
    }

    // ========================================================================
    // Validation Response
    // ========================================================================

    /// Submit a validation response (validator only)
    ///
    /// # Arguments
    /// * `validator` - Validator address (must match request's validator)
    /// * `request_hash` - Hash of the original request
    /// * `response_code` - Response code (see response_codes module)
    /// * `response_uri` - URI pointing to response details
    /// * `response_hash` - Hash of the response content
    /// * `tag` - Response category tag
    pub fn validation_response(
        env: Env,
        validator: Address,
        request_hash: BytesN<32>,
        response_code: i32,
        response_uri: String,
        response_hash: BytesN<32>,
        tag: Symbol,
    ) -> Result<(), Error> {
        validator.require_auth();

        // Get the request
        let mut request = storage::get_request(&env, &request_hash)
            .ok_or(Error::RequestNotFound)?;

        // Verify validator matches
        if request.validator != validator {
            return Err(Error::NotValidator);
        }

        // Check if already responded
        if request.responded {
            return Err(Error::AlreadyResponded);
        }

        // Mark request as responded
        request.responded = true;
        storage::set_request(&env, &request_hash, &request);

        // Store response
        let response = ValidationResponse {
            response_code,
            response_uri,
            response_hash,
            tag: tag.clone(),
            timestamp_ledger: env.ledger().sequence(),
        };

        storage::set_response(&env, &request_hash, &response);

        ttl::extend_persistent_ttl(&env);

        events::emit_validation_responded(
            &env,
            request.agent_id,
            &validator,
            &request_hash,
            response_code,
            &tag,
        );

        Ok(())
    }

    // ========================================================================
    // Read Functions
    // ========================================================================

    /// Get validation status for a request
    pub fn get_validation_status(
        env: Env,
        request_hash: BytesN<32>,
    ) -> Result<ValidationStatus, Error> {
        let request = storage::get_request(&env, &request_hash)
            .ok_or(Error::RequestNotFound)?;

        let response = storage::get_response(&env, &request_hash);

        match response {
            Some(r) => Ok(ValidationStatus {
                request,
                has_response: true,
                response_code: r.response_code,
                response_uri: r.response_uri,
                response_hash: r.response_hash,
                response_tag: r.tag,
                response_timestamp: r.timestamp_ledger,
            }),
            None => Ok(ValidationStatus {
                request,
                has_response: false,
                response_code: 0,
                response_uri: String::from_str(&env, ""),
                response_hash: BytesN::from_array(&env, &[0u8; 32]),
                response_tag: Symbol::new(&env, ""),
                response_timestamp: 0,
            }),
        }
    }

    /// Get all validation request hashes for an agent
    pub fn get_agent_validations(env: Env, agent_id: u64) -> Vec<BytesN<32>> {
        storage::get_agent_requests(&env, agent_id)
    }

    /// Get all validation request hashes for a validator
    pub fn get_validator_requests(env: Env, validator: Address) -> Vec<BytesN<32>> {
        storage::get_validator_requests(&env, &validator)
    }

    /// Get summary of validations for an agent from specified validators
    ///
    /// # Arguments
    /// * `agent_id` - Agent token ID
    /// * `validators` - List of validators to include (anti-Sybil: must be non-empty)
    /// * `tag` - Optional filter by response tag
    pub fn get_summary(
        env: Env,
        agent_id: u64,
        validators: Vec<Address>,
        tag: Option<Symbol>,
    ) -> Result<ValidationSummary, Error> {
        if validators.is_empty() {
            return Err(Error::EmptyValidatorList);
        }

        let request_hashes = storage::get_agent_requests(&env, agent_id);

        let mut total_requests: u32 = 0;
        let mut responded_count: u32 = 0;
        let mut valid_count: u32 = 0;
        let mut invalid_count: u32 = 0;
        let mut other_count: u32 = 0;

        for hash in request_hashes.iter() {
            if let Some(request) = storage::get_request(&env, &hash) {
                // Check if validator is in the list
                let mut validator_match = false;
                for v in validators.iter() {
                    if v == request.validator {
                        validator_match = true;
                        break;
                    }
                }

                if !validator_match {
                    continue;
                }

                total_requests += 1;

                if let Some(response) = storage::get_response(&env, &hash) {
                    // Filter by tag if specified
                    if let Some(ref t) = tag {
                        if response.tag != *t {
                            continue;
                        }
                    }

                    responded_count += 1;

                    match response.response_code {
                        response_codes::VALID => valid_count += 1,
                        response_codes::INVALID => invalid_count += 1,
                        _ => other_count += 1,
                    }
                }
            }
        }

        Ok(ValidationSummary {
            total_requests,
            responded_count,
            valid_count,
            invalid_count,
            other_count,
        })
    }

    /// Get a specific validation request
    pub fn get_request(env: Env, request_hash: BytesN<32>) -> Result<ValidationRequest, Error> {
        storage::get_request(&env, &request_hash).ok_or(Error::RequestNotFound)
    }

    /// Get a specific validation response
    pub fn get_response(
        env: Env,
        request_hash: BytesN<32>,
    ) -> Result<ValidationResponse, Error> {
        storage::get_response(&env, &request_hash).ok_or(Error::RequestNotFound)
    }

    // ========================================================================
    // Admin Functions
    // ========================================================================

    /// Upgrade the contract (admin only)
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    /// Get contract admin
    pub fn admin(env: Env) -> Result<Address, Error> {
        storage::get_admin(&env).ok_or(Error::NotInitialized)
    }

    /// Get identity registry address
    pub fn identity_registry(env: Env) -> Result<Address, Error> {
        storage::get_identity_registry(&env).ok_or(Error::NotInitialized)
    }

    /// Extend TTL (anyone can call)
    pub fn extend_ttl(env: Env) {
        ttl::extend_instance_ttl(&env);
        ttl::extend_persistent_ttl(&env);
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Verify caller is owner or approved operator for the agent
    /// 
    /// Note: Cross-contract calls will panic if the agent doesn't exist.
    /// This is acceptable as validation requests for non-existent agents are invalid.
    fn require_owner_or_operator(env: &Env, agent_id: u64, caller: &Address) -> Result<(), Error> {
        let identity_registry = storage::get_identity_registry(env)
            .ok_or(Error::NotInitialized)?;

        let token_id_val: Val = agent_id.into_val(env);

        // Check if caller is owner
        // owner_of returns Address (panics if token doesn't exist)
        let owner: Address = env.invoke_contract(
            &identity_registry,
            &Symbol::new(env, "owner_of"),
            Vec::<Val>::from_array(env, [token_id_val.clone()]),
        );

        if *caller == owner {
            return Ok(());
        }

        // Check if caller is approved operator
        let owner_val: Val = owner.into_val(env);
        let caller_val: Val = caller.clone().into_val(env);
        let is_operator: bool = env.invoke_contract(
            &identity_registry,
            &Symbol::new(env, "is_approved_for_all"),
            Vec::<Val>::from_array(env, [owner_val, caller_val]),
        );

        if is_operator {
            return Ok(());
        }

        // Check if caller is approved for this token
        let approved: Option<Address> = env.invoke_contract(
            &identity_registry,
            &Symbol::new(env, "get_approved"),
            Vec::<Val>::from_array(env, [token_id_val]),
        );

        if let Some(approved_addr) = approved {
            if *caller == approved_addr {
                return Ok(());
            }
        }

        Err(Error::NotAuthorized)
    }
}

