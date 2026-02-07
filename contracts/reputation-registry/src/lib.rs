//! # Reputation Registry Contract (ERC-8004 on Soroban)
//!
//! Stores structured feedback for agents. Implements ERC-8004 reputation semantics:
//! - Feedback with value, decimals, tags, URIs, and hashes
//! - Self-feedback prevention by querying Identity Registry
//! - Revocation support
//! - Response append capability
//! - Summary aggregation with explicit client list (anti-Sybil)

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
    /// Self-feedback not allowed
    SelfFeedbackNotAllowed = 4,
    /// Feedback not found
    FeedbackNotFound = 5,
    /// Already revoked
    AlreadyRevoked = 6,
    /// Response not found
    ResponseNotFound = 7,
    /// Agent not found in identity registry
    AgentNotFound = 8,
    /// Empty client list (anti-Sybil requirement)
    EmptyClientList = 9,
    /// Invalid decimals value
    InvalidDecimals = 10,
}

// ============================================================================
// Data Types
// ============================================================================

/// Feedback record stored on-chain
#[contracttype]
#[derive(Clone, Debug)]
pub struct FeedbackRecord {
    /// Feedback value (can be negative for negative feedback)
    pub value: i128,
    /// Decimal places for the value
    pub decimals: u32,
    /// Primary category tag
    pub tag1: Symbol,
    /// Secondary category tag
    pub tag2: Symbol,
    /// Optional endpoint URI that was interacted with
    pub endpoint_uri: String,
    /// Optional URI pointing to detailed feedback
    pub feedback_uri: String,
    /// Optional hash of the feedback content (SHA-256)
    pub feedback_hash: BytesN<32>,
    /// Whether this feedback has been revoked
    pub revoked: bool,
    /// Ledger sequence when feedback was given
    pub timestamp_ledger: u32,
    /// Address that gave the feedback
    pub client: Address,
}

/// Response to feedback (from agent or validators)
#[contracttype]
#[derive(Clone, Debug)]
pub struct ResponseRecord {
    /// Address that gave the response
    pub responder: Address,
    /// URI pointing to response content
    pub response_uri: String,
    /// Hash of response content (SHA-256)
    pub response_hash: BytesN<32>,
    /// Response tag/category
    pub tag: Symbol,
    /// Ledger sequence when response was added
    pub timestamp_ledger: u32,
}

/// Summary of feedback for an agent
#[contracttype]
#[derive(Clone, Debug)]
pub struct FeedbackSummary {
    /// Total count of non-revoked feedback
    pub count: u32,
    /// Sum of all feedback values (normalized to highest decimals)
    pub total_value: i128,
    /// Decimals used for total_value
    pub decimals: u32,
    /// Count of positive feedback (value > 0)
    pub positive_count: u32,
    /// Count of negative feedback (value < 0)
    pub negative_count: u32,
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
    /// Feedback: Feedback(agent_id, client, idx) -> FeedbackRecord
    Feedback(u64, Address, u32),
    /// Feedback count per client: FeedbackCount(agent_id, client) -> u32
    FeedbackCount(u64, Address),
    /// Response: Response(agent_id, client, feedback_idx, response_idx) -> ResponseRecord
    Response(u64, Address, u32, u32),
    /// Response count: ResponseCount(agent_id, client, feedback_idx) -> u32
    ResponseCount(u64, Address, u32),
    /// All clients who gave feedback to an agent: AgentClients(agent_id) -> Vec<Address>
    AgentClients(u64),
}

// ============================================================================
// Contract
// ============================================================================

#[contract]
pub struct ReputationRegistryContract;

// Cross-contract calls are done via env.invoke_contract() with dynamic dispatch
// This avoids compile-time dependency on identity-registry WASM

#[contractimpl]
impl ReputationRegistryContract {
    // ========================================================================
    // Initialization
    // ========================================================================

    /// Initialize the reputation registry
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
    // Feedback Functions
    // ========================================================================

    /// Give feedback for an agent
    ///
    /// # Arguments
    /// * `client` - Address giving feedback (must authorize)
    /// * `agent_id` - Token ID of the agent in the Identity Registry
    /// * `value` - Feedback value (positive or negative)
    /// * `decimals` - Decimal places for the value
    /// * `tag1` - Primary category tag
    /// * `tag2` - Secondary category tag
    /// * `endpoint_uri` - Optional endpoint that was interacted with
    /// * `feedback_uri` - Optional URI with detailed feedback
    /// * `feedback_hash` - Optional hash of feedback content
    ///
    /// # Returns
    /// * `u32` - Index of the feedback record
    pub fn give_feedback(
        env: Env,
        client: Address,
        agent_id: u64,
        value: i128,
        decimals: u32,
        tag1: Symbol,
        tag2: Symbol,
        endpoint_uri: String,
        feedback_uri: String,
        feedback_hash: BytesN<32>,
    ) -> Result<u32, Error> {
        client.require_auth();

        if decimals > 18 {
            return Err(Error::InvalidDecimals);
        }

        // Self-feedback prevention
        Self::check_self_feedback(&env, agent_id, &client)?;

        // Get next feedback index for this client
        let feedback_idx = storage::get_feedback_count(&env, agent_id, &client);

        let record = FeedbackRecord {
            value,
            decimals,
            tag1: tag1.clone(),
            tag2: tag2.clone(),
            endpoint_uri,
            feedback_uri,
            feedback_hash,
            revoked: false,
            timestamp_ledger: env.ledger().sequence(),
            client: client.clone(),
        };

        storage::set_feedback(&env, agent_id, &client, feedback_idx, &record);
        storage::set_feedback_count(&env, agent_id, &client, feedback_idx + 1);
        storage::add_client_to_agent(&env, agent_id, &client);

        ttl::extend_persistent_ttl(&env);

        events::emit_feedback_given(&env, agent_id, &client, feedback_idx, value, &tag1, &tag2);

        Ok(feedback_idx)
    }

    /// Revoke previously given feedback
    ///
    /// # Arguments
    /// * `client` - Original feedback giver (must authorize)
    /// * `agent_id` - Agent token ID
    /// * `feedback_idx` - Index of the feedback to revoke
    pub fn revoke_feedback(
        env: Env,
        client: Address,
        agent_id: u64,
        feedback_idx: u32,
    ) -> Result<(), Error> {
        client.require_auth();

        let mut record = storage::get_feedback(&env, agent_id, &client, feedback_idx)
            .ok_or(Error::FeedbackNotFound)?;

        if record.revoked {
            return Err(Error::AlreadyRevoked);
        }

        record.revoked = true;
        storage::set_feedback(&env, agent_id, &client, feedback_idx, &record);

        ttl::extend_persistent_ttl(&env);

        events::emit_feedback_revoked(&env, agent_id, &client, feedback_idx);

        Ok(())
    }

    /// Append a response to feedback
    ///
    /// # Arguments
    /// * `responder` - Address responding (must authorize)
    /// * `agent_id` - Agent token ID
    /// * `client` - Original feedback client
    /// * `feedback_idx` - Feedback index
    /// * `response_uri` - URI with response content
    /// * `response_hash` - Hash of response content
    /// * `tag` - Response category tag
    ///
    /// # Returns
    /// * `u32` - Index of the response
    pub fn append_response(
        env: Env,
        responder: Address,
        agent_id: u64,
        client: Address,
        feedback_idx: u32,
        response_uri: String,
        response_hash: BytesN<32>,
        tag: Symbol,
    ) -> Result<u32, Error> {
        responder.require_auth();

        // Verify feedback exists
        if !storage::feedback_exists(&env, agent_id, &client, feedback_idx) {
            return Err(Error::FeedbackNotFound);
        }

        let response_idx = storage::get_response_count(&env, agent_id, &client, feedback_idx);

        let record = ResponseRecord {
            responder: responder.clone(),
            response_uri,
            response_hash,
            tag: tag.clone(),
            timestamp_ledger: env.ledger().sequence(),
        };

        storage::set_response(&env, agent_id, &client, feedback_idx, response_idx, &record);
        storage::set_response_count(&env, agent_id, &client, feedback_idx, response_idx + 1);

        ttl::extend_persistent_ttl(&env);

        events::emit_response_added(&env, agent_id, &client, feedback_idx, &responder, &tag);

        Ok(response_idx)
    }

    // ========================================================================
    // Read Functions
    // ========================================================================

    /// Read a specific feedback record
    pub fn read_feedback(
        env: Env,
        agent_id: u64,
        client: Address,
        idx: u32,
    ) -> Result<FeedbackRecord, Error> {
        storage::get_feedback(&env, agent_id, &client, idx).ok_or(Error::FeedbackNotFound)
    }

    /// Read all feedback for an agent from specified clients
    ///
    /// # Arguments
    /// * `agent_id` - Agent token ID
    /// * `clients` - List of clients to include (anti-Sybil: must be non-empty)
    /// * `tag1` - Optional filter by primary tag
    /// * `tag2` - Optional filter by secondary tag
    /// * `include_revoked` - Whether to include revoked feedback
    pub fn read_all_feedback(
        env: Env,
        agent_id: u64,
        clients: Vec<Address>,
        tag1: Option<Symbol>,
        tag2: Option<Symbol>,
        include_revoked: bool,
    ) -> Result<Vec<FeedbackRecord>, Error> {
        if clients.is_empty() {
            return Err(Error::EmptyClientList);
        }

        let mut result = Vec::new(&env);

        for client in clients.iter() {
            let count = storage::get_feedback_count(&env, agent_id, &client);
            for idx in 0..count {
                if let Some(record) = storage::get_feedback(&env, agent_id, &client, idx) {
                    // Filter by revoked status
                    if !include_revoked && record.revoked {
                        continue;
                    }

                    // Filter by tag1
                    if let Some(ref t1) = tag1 {
                        if record.tag1 != *t1 {
                            continue;
                        }
                    }

                    // Filter by tag2
                    if let Some(ref t2) = tag2 {
                        if record.tag2 != *t2 {
                            continue;
                        }
                    }

                    result.push_back(record);
                }
            }
        }

        Ok(result)
    }

    /// Get summary of feedback for an agent
    ///
    /// # Arguments
    /// * `agent_id` - Agent token ID
    /// * `clients` - List of clients to include (anti-Sybil: must be non-empty)
    /// * `tag1` - Optional filter by primary tag
    /// * `tag2` - Optional filter by secondary tag
    pub fn get_summary(
        env: Env,
        agent_id: u64,
        clients: Vec<Address>,
        tag1: Option<Symbol>,
        tag2: Option<Symbol>,
    ) -> Result<FeedbackSummary, Error> {
        if clients.is_empty() {
            return Err(Error::EmptyClientList);
        }

        let mut count: u32 = 0;
        let mut total_value: i128 = 0;
        let mut max_decimals: u32 = 0;
        let mut positive_count: u32 = 0;
        let mut negative_count: u32 = 0;

        // First pass: find max decimals
        for client in clients.iter() {
            let feedback_count = storage::get_feedback_count(&env, agent_id, &client);
            for idx in 0..feedback_count {
                if let Some(record) = storage::get_feedback(&env, agent_id, &client, idx) {
                    if record.revoked {
                        continue;
                    }
                    if let Some(ref t1) = tag1 {
                        if record.tag1 != *t1 {
                            continue;
                        }
                    }
                    if let Some(ref t2) = tag2 {
                        if record.tag2 != *t2 {
                            continue;
                        }
                    }
                    if record.decimals > max_decimals {
                        max_decimals = record.decimals;
                    }
                }
            }
        }

        // Second pass: sum normalized values
        for client in clients.iter() {
            let feedback_count = storage::get_feedback_count(&env, agent_id, &client);
            for idx in 0..feedback_count {
                if let Some(record) = storage::get_feedback(&env, agent_id, &client, idx) {
                    if record.revoked {
                        continue;
                    }
                    if let Some(ref t1) = tag1 {
                        if record.tag1 != *t1 {
                            continue;
                        }
                    }
                    if let Some(ref t2) = tag2 {
                        if record.tag2 != *t2 {
                            continue;
                        }
                    }

                    // Normalize value to max decimals
                    let decimal_diff = max_decimals - record.decimals;
                    let multiplier = 10i128.pow(decimal_diff);
                    let normalized_value = record.value * multiplier;

                    total_value += normalized_value;
                    count += 1;

                    if record.value > 0 {
                        positive_count += 1;
                    } else if record.value < 0 {
                        negative_count += 1;
                    }
                }
            }
        }

        Ok(FeedbackSummary {
            count,
            total_value,
            decimals: max_decimals,
            positive_count,
            negative_count,
        })
    }

    /// Get all clients who have given feedback to an agent
    pub fn get_agent_clients(env: Env, agent_id: u64) -> Vec<Address> {
        storage::get_agent_clients(&env, agent_id)
    }

    /// Read a specific response
    pub fn read_response(
        env: Env,
        agent_id: u64,
        client: Address,
        feedback_idx: u32,
        response_idx: u32,
    ) -> Result<ResponseRecord, Error> {
        storage::get_response(&env, agent_id, &client, feedback_idx, response_idx)
            .ok_or(Error::ResponseNotFound)
    }

    /// Get response count for a feedback
    pub fn get_response_count(
        env: Env,
        agent_id: u64,
        client: Address,
        feedback_idx: u32,
    ) -> u32 {
        storage::get_response_count(&env, agent_id, &client, feedback_idx)
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

    /// Check if client is allowed to give feedback (self-feedback prevention)
    /// 
    /// Note: Cross-contract calls will panic if the agent doesn't exist.
    /// This is acceptable as feedback for non-existent agents is invalid anyway.
    fn check_self_feedback(env: &Env, agent_id: u64, client: &Address) -> Result<(), Error> {
        let identity_registry = storage::get_identity_registry(env)
            .ok_or(Error::NotInitialized)?;

        // Call identity registry to check ownership/approval
        // Using explicit type parameters for invoke_contract
        
        let token_id_val: Val = agent_id.into_val(env);

        // Check if client is owner
        // owner_of returns Address (panics if token doesn't exist)
        let owner: Address = env.invoke_contract(
            &identity_registry,
            &Symbol::new(env, "owner_of"),
            Vec::<Val>::from_array(env, [token_id_val.clone()]),
        );

        if *client == owner {
            return Err(Error::SelfFeedbackNotAllowed);
        }

        // Check if client is approved for this specific token
        let approved: Option<Address> = env.invoke_contract(
            &identity_registry,
            &Symbol::new(env, "get_approved"),
            Vec::<Val>::from_array(env, [token_id_val.clone()]),
        );

        if let Some(approved_addr) = approved {
            if *client == approved_addr {
                return Err(Error::SelfFeedbackNotAllowed);
            }
        }

        // Check if client is agent wallet
        let wallet: Address = env.invoke_contract(
            &identity_registry,
            &Symbol::new(env, "get_agent_wallet"),
            Vec::<Val>::from_array(env, [token_id_val]),
        );

        if *client == wallet {
            return Err(Error::SelfFeedbackNotAllowed);
        }

        Ok(())
    }
}

