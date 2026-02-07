//! # Identity Registry Contract (ERC-8004 on Soroban)
//!
//! SEP-50 NFT-based agent identity registry implementing ERC-8004 semantics.
//! Each agent identity is represented as an NFT with associated metadata.
//!
//! ## Features
//! - SEP-50 NFT interface (transfer, approve, operator approvals)
//! - ERC-8004 extras: register, setAgentURI, metadata map, agentWallet
//! - Clear agentWallet on transfer (forces re-verification)
//! - Discovery events for indexing

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror,
    Address, BytesN, Env, String,
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
    /// Not authorized to perform this action
    NotAuthorized = 3,
    /// Token does not exist
    TokenNotFound = 4,
    /// Not the token owner
    NotOwner = 5,
    /// Invalid approval - cannot approve self
    InvalidApproval = 6,
    /// Transfer to zero address not allowed
    InvalidRecipient = 7,
    /// Metadata key not found
    MetadataNotFound = 8,
    /// Reserved metadata key - cannot be modified directly
    ReservedMetadataKey = 9,
    /// Invalid URI provided
    InvalidUri = 10,
}

// ============================================================================
// Storage Keys
// ============================================================================

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Contract admin address
    Admin,
    /// Collection name
    Name,
    /// Collection symbol
    Symbol,
    /// Next token ID to mint
    NextTokenId,
    /// Token owner: TokenOwner(token_id) -> Address
    TokenOwner(u64),
    /// Token URI: TokenUri(token_id) -> String
    TokenUri(u64),
    /// Token approval: TokenApproval(token_id) -> Address
    TokenApproval(u64),
    /// Operator approval: OperatorApproval(owner, operator) -> bool
    OperatorApproval(Address, Address),
    /// Agent wallet metadata: AgentWallet(token_id) -> Address
    AgentWallet(u64),
    /// Generic metadata: Metadata(token_id, key) -> Bytes
    Metadata(u64, BytesN<32>),
    /// Total supply
    TotalSupply,
    /// Balance: Balance(owner) -> u64
    Balance(Address),
}

// ============================================================================
// Reserved Metadata Keys
// ============================================================================

/// Reserved key for agent wallet - SHA256("agentWallet") truncated
pub const AGENT_WALLET_KEY: [u8; 32] = [
    0x61, 0x67, 0x65, 0x6e, 0x74, 0x57, 0x61, 0x6c, 0x6c, 0x65, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// ============================================================================
// Contract
// ============================================================================

#[contract]
pub struct IdentityRegistryContract;

#[contractimpl]
impl IdentityRegistryContract {
    // ========================================================================
    // Initialization
    // ========================================================================

    /// Initialize the identity registry contract
    ///
    /// # Arguments
    /// * `admin` - Contract administrator address
    /// * `name` - Collection name
    /// * `symbol` - Collection symbol
    pub fn init(env: Env, admin: Address, name: String, symbol: String) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();

        storage::set_admin(&env, &admin);
        storage::set_name(&env, &name);
        storage::set_symbol(&env, &symbol);
        storage::set_next_token_id(&env, 0);
        storage::set_total_supply(&env, 0);

        // Extend instance TTL on init
        ttl::extend_instance_ttl(&env);

        Ok(())
    }

    // ========================================================================
    // Agent Registration (ERC-8004)
    // ========================================================================

    /// Register a new agent identity (mint NFT)
    ///
    /// # Arguments
    /// * `owner` - Address that will own the agent identity
    /// * `agent_uri` - URI pointing to the agent registration JSON
    ///
    /// # Returns
    /// * `u64` - The newly minted token ID
    pub fn register(env: Env, owner: Address, agent_uri: String) -> Result<u64, Error> {
        if !storage::has_admin(&env) {
            return Err(Error::NotInitialized);
        }

        owner.require_auth();

        let token_id = storage::get_next_token_id(&env);
        
        // Mint the token
        storage::set_token_owner(&env, token_id, &owner);
        storage::set_token_uri(&env, token_id, &agent_uri);
        storage::set_agent_wallet(&env, token_id, &owner);
        
        // Update counters
        storage::set_next_token_id(&env, token_id + 1);
        storage::increment_balance(&env, &owner);
        storage::increment_total_supply(&env);

        // Extend TTL for the new token data
        ttl::extend_persistent_ttl(&env);

        // Emit event
        events::emit_agent_registered(&env, token_id, &owner, &agent_uri);

        Ok(token_id)
    }

    /// Set the agent URI for an existing token
    ///
    /// # Arguments
    /// * `owner` - Current token owner (must authorize)
    /// * `token_id` - Token ID to update
    /// * `new_uri` - New URI for the agent registration JSON
    pub fn set_agent_uri(
        env: Env,
        owner: Address,
        token_id: u64,
        new_uri: String,
    ) -> Result<(), Error> {
        owner.require_auth();
        Self::require_owner_or_approved(&env, token_id, &owner)?;

        storage::set_token_uri(&env, token_id, &new_uri);
        ttl::extend_persistent_ttl(&env);

        events::emit_agent_uri_updated(&env, token_id, &new_uri);

        Ok(())
    }

    /// Set the agent wallet for a token (requires dual authorization)
    ///
    /// This is stronger than EIP-712: both current owner AND new wallet must sign.
    ///
    /// # Arguments
    /// * `owner` - Current token owner (must authorize)
    /// * `token_id` - Token ID to update
    /// * `new_wallet` - New wallet address (must also authorize)
    pub fn set_agent_wallet(
        env: Env,
        owner: Address,
        token_id: u64,
        new_wallet: Address,
    ) -> Result<(), Error> {
        // Require dual authorization - stronger than EIP-712
        owner.require_auth();
        new_wallet.require_auth();

        Self::require_owner_or_approved(&env, token_id, &owner)?;

        storage::set_agent_wallet(&env, token_id, &new_wallet);
        ttl::extend_persistent_ttl(&env);

        events::emit_agent_wallet_updated(&env, token_id, &new_wallet);

        Ok(())
    }

    // ========================================================================
    // Metadata (ERC-8004)
    // ========================================================================

    /// Get metadata value for a token
    ///
    /// # Arguments
    /// * `token_id` - Token ID
    /// * `key` - Metadata key (32 bytes)
    ///
    /// # Returns
    /// * `Bytes` - Metadata value
    pub fn get_metadata(env: Env, token_id: u64, key: BytesN<32>) -> Result<BytesN<32>, Error> {
        if !storage::token_exists(&env, token_id) {
            return Err(Error::TokenNotFound);
        }

        // Special handling for agentWallet key
        if key.to_array() == AGENT_WALLET_KEY {
            // Return the agent wallet as bytes
            if storage::get_agent_wallet(&env, token_id).is_some() {
                // Convert address to bytes - for now return empty as placeholder
                // In production, serialize the address properly
                return Ok(BytesN::from_array(&env, &[0u8; 32]));
            }
            return Err(Error::MetadataNotFound);
        }

        storage::get_metadata(&env, token_id, &key).ok_or(Error::MetadataNotFound)
    }

    /// Set metadata value for a token
    ///
    /// # Arguments
    /// * `owner` - Token owner (must authorize)
    /// * `token_id` - Token ID
    /// * `key` - Metadata key (32 bytes)
    /// * `value` - Metadata value (32 bytes)
    pub fn set_metadata(
        env: Env,
        owner: Address,
        token_id: u64,
        key: BytesN<32>,
        value: BytesN<32>,
    ) -> Result<(), Error> {
        owner.require_auth();
        Self::require_owner_or_approved(&env, token_id, &owner)?;

        // Prevent direct modification of reserved keys
        if key.to_array() == AGENT_WALLET_KEY {
            return Err(Error::ReservedMetadataKey);
        }

        storage::set_metadata(&env, token_id, &key, &value);
        ttl::extend_persistent_ttl(&env);

        Ok(())
    }

    // ========================================================================
    // SEP-50 NFT Interface
    // ========================================================================

    /// Get collection name
    pub fn name(env: Env) -> Result<String, Error> {
        storage::get_name(&env).ok_or(Error::NotInitialized)
    }

    /// Get collection symbol
    pub fn symbol(env: Env) -> Result<String, Error> {
        storage::get_symbol(&env).ok_or(Error::NotInitialized)
    }

    /// Get token URI
    pub fn token_uri(env: Env, token_id: u64) -> Result<String, Error> {
        storage::get_token_uri(&env, token_id).ok_or(Error::TokenNotFound)
    }

    /// Get total supply of tokens
    pub fn total_supply(env: Env) -> u64 {
        storage::get_total_supply(&env)
    }

    /// Get balance of an address
    pub fn balance_of(env: Env, owner: Address) -> u64 {
        storage::get_balance(&env, &owner)
    }

    /// Get owner of a token
    pub fn owner_of(env: Env, token_id: u64) -> Result<Address, Error> {
        storage::get_token_owner(&env, token_id).ok_or(Error::TokenNotFound)
    }

    /// Get the agent wallet for a token
    pub fn get_agent_wallet(env: Env, token_id: u64) -> Result<Address, Error> {
        storage::get_agent_wallet(&env, token_id).ok_or(Error::TokenNotFound)
    }

    /// Get approved address for a token
    pub fn get_approved(env: Env, token_id: u64) -> Option<Address> {
        storage::get_token_approval(&env, token_id)
    }

    /// Check if operator is approved for all tokens of owner
    pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
        storage::is_operator_approved(&env, &owner, &operator)
    }

    /// Approve an address to transfer a specific token
    pub fn approve(
        env: Env,
        owner: Address,
        approved: Address,
        token_id: u64,
    ) -> Result<(), Error> {
        owner.require_auth();

        let token_owner = storage::get_token_owner(&env, token_id).ok_or(Error::TokenNotFound)?;

        if token_owner != owner {
            return Err(Error::NotOwner);
        }

        if owner == approved {
            return Err(Error::InvalidApproval);
        }

        storage::set_token_approval(&env, token_id, &approved);
        ttl::extend_persistent_ttl(&env);

        events::emit_approval(&env, &owner, &approved, token_id);

        Ok(())
    }

    /// Set or revoke operator approval for all tokens
    pub fn set_approval_for_all(
        env: Env,
        owner: Address,
        operator: Address,
        approved: bool,
    ) -> Result<(), Error> {
        owner.require_auth();

        if owner == operator {
            return Err(Error::InvalidApproval);
        }

        storage::set_operator_approval(&env, &owner, &operator, approved);
        ttl::extend_persistent_ttl(&env);

        events::emit_approval_for_all(&env, &owner, &operator, approved);

        Ok(())
    }

    /// Transfer token from one address to another
    ///
    /// NOTE: Clears agentWallet on transfer per ERC-8004 guidance
    pub fn transfer_from(
        env: Env,
        from: Address,
        to: Address,
        token_id: u64,
    ) -> Result<(), Error> {
        from.require_auth();

        if to == from {
            return Err(Error::InvalidRecipient);
        }

        let token_owner = storage::get_token_owner(&env, token_id).ok_or(Error::TokenNotFound)?;

        // Check authorization
        let is_owner = from == token_owner;
        let is_approved = storage::get_token_approval(&env, token_id)
            .map(|a| a == from)
            .unwrap_or(false);
        let is_operator = storage::is_operator_approved(&env, &token_owner, &from);

        if !is_owner && !is_approved && !is_operator {
            return Err(Error::NotAuthorized);
        }

        // Update ownership
        storage::set_token_owner(&env, token_id, &to);
        storage::decrement_balance(&env, &token_owner);
        storage::increment_balance(&env, &to);

        // Clear approval
        storage::clear_token_approval(&env, token_id);

        // CRITICAL: Clear agentWallet on transfer (ERC-8004 requirement)
        storage::clear_agent_wallet(&env, token_id);

        ttl::extend_persistent_ttl(&env);

        events::emit_transfer(&env, &token_owner, &to, token_id);

        Ok(())
    }

    /// Transfer token (simplified - caller must be owner)
    pub fn transfer(env: Env, to: Address, token_id: u64) -> Result<(), Error> {
        let from = storage::get_token_owner(&env, token_id).ok_or(Error::TokenNotFound)?;
        Self::transfer_from(env, from, to, token_id)
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

    /// Set new admin (current admin only)
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();

        storage::set_admin(&env, &new_admin);
        ttl::extend_persistent_ttl(&env);

        Ok(())
    }

    /// Extend TTL for contract instance (anyone can call to keep contract alive)
    pub fn extend_ttl(env: Env) {
        ttl::extend_instance_ttl(&env);
        ttl::extend_persistent_ttl(&env);
    }

    /// Extend TTL for a specific agent's data
    pub fn extend_agent_ttl(env: Env, token_id: u64) -> Result<(), Error> {
        if !storage::token_exists(&env, token_id) {
            return Err(Error::TokenNotFound);
        }
        ttl::extend_persistent_ttl(&env);
        Ok(())
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    fn require_owner_or_approved(env: &Env, token_id: u64, caller: &Address) -> Result<(), Error> {
        let owner = storage::get_token_owner(env, token_id).ok_or(Error::TokenNotFound)?;

        if *caller == owner {
            return Ok(());
        }

        // Check token-level approval
        if let Some(approved) = storage::get_token_approval(env, token_id) {
            if *caller == approved {
                return Ok(());
            }
        }

        // Check operator approval
        if storage::is_operator_approved(env, &owner, caller) {
            return Ok(());
        }

        Err(Error::NotAuthorized)
    }
}

