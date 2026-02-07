# ERC-8004 Trustless Agents on Stellar Soroban

Implementation of the [ERC-8004 Trustless Agents](https://eips.ethereum.org/EIPS/eip-8004) standard on Stellar's Soroban smart contract platform.

## Overview

ERC-8004 defines a decentralized framework for AI agent identity, reputation, and validation. This implementation adapts the standard for Soroban with three core contracts:

| Contract | Description | Functions |
|----------|-------------|-----------|
| **Identity Registry** | SEP-50 NFT-based agent identity | 25 |
| **Reputation Registry** | Feedback and reputation tracking | 15 |
| **Validation Registry** | Validator request/response system | 14 |

## Project Structure

```
.
├── contracts/
│   ├── identity-registry/      # SEP-50 NFT + ERC-8004 Identity
│   │   └── src/
│   │       ├── lib.rs          # Main contract
│   │       ├── storage.rs      # Storage helpers
│   │       ├── events.rs       # Event emissions
│   │       ├── ttl.rs          # TTL management
│   │       └── test.rs         # Unit tests
│   │
│   ├── reputation-registry/    # Feedback & Reputation
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── storage.rs
│   │       ├── events.rs
│   │       ├── ttl.rs
│   │       └── test.rs
│   │
│   └── validation-registry/    # Validator System
│       └── src/
│           ├── lib.rs
│           ├── storage.rs
│           ├── events.rs
│           ├── ttl.rs
│           └── test.rs
│
├── schemas/
│   └── agent-registration.json # JSON schema for agent registration files
│
├── Cargo.toml                  # Workspace configuration
└── README.md
```

## Agent Identifier Format

Soroban agent identifiers follow the ERC-8004 pattern:

```
agentRegistry = stellar:{network}:{contract_id}
agentId = u64 token_id

Example: stellar:pubnet:CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC
```

---

## Identity Registry Contract

SEP-50 compliant NFT contract with ERC-8004 extensions for agent identity management.

### Functions

#### Initialization
| Function | Parameters | Description |
|----------|------------|-------------|
| `init` | `admin: Address, name: String, symbol: String` | Initialize the registry |

#### Agent Registration (ERC-8004)
| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `register` | `owner: Address, agent_uri: String` | `u64` | Mint new agent identity NFT |
| `set_agent_uri` | `owner: Address, token_id: u64, new_uri: String` | - | Update agent registration JSON URI |
| `set_agent_wallet` | `owner: Address, token_id: u64, new_wallet: Address` | - | Update agent wallet (requires dual auth) |
| `get_agent_wallet` | `token_id: u64` | `Address` | Get agent's associated wallet |

#### Metadata
| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `get_metadata` | `token_id: u64, key: BytesN<32>` | `BytesN<32>` | Get metadata value |
| `set_metadata` | `owner: Address, token_id: u64, key: BytesN<32>, value: BytesN<32>` | - | Set metadata value |

#### SEP-50 NFT Interface
| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `name` | - | `String` | Collection name |
| `symbol` | - | `String` | Collection symbol |
| `total_supply` | - | `u64` | Total minted tokens |
| `balance_of` | `owner: Address` | `u64` | Token balance of address |
| `owner_of` | `token_id: u64` | `Address` | Owner of token |
| `token_uri` | `token_id: u64` | `String` | Token metadata URI |
| `approve` | `owner: Address, approved: Address, token_id: u64` | - | Approve address for token |
| `get_approved` | `token_id: u64` | `Option<Address>` | Get approved address |
| `set_approval_for_all` | `owner: Address, operator: Address, approved: bool` | - | Set operator approval |
| `is_approved_for_all` | `owner: Address, operator: Address` | `bool` | Check operator approval |
| `transfer` | `to: Address, token_id: u64` | - | Transfer token |
| `transfer_from` | `from: Address, to: Address, token_id: u64` | - | Transfer token (clears agentWallet) |

#### Admin & TTL
| Function | Parameters | Description |
|----------|------------|-------------|
| `admin` | - | Get contract admin |
| `set_admin` | `new_admin: Address` | Update admin (admin only) |
| `upgrade` | `new_wasm_hash: BytesN<32>` | Upgrade contract (admin only) |
| `extend_ttl` | - | Extend contract TTL (anyone) |
| `extend_agent_ttl` | `token_id: u64` | Extend specific agent's TTL |

### Events

| Event | Data | Description |
|-------|------|-------------|
| `AgentRegistered` | `(token_id, owner, agent_uri)` | New agent registered |
| `AgentURIUpdated` | `(token_id, new_uri)` | Agent URI changed |
| `AgentWalletUpdated` | `(token_id, new_wallet)` | Agent wallet changed |
| `Transfer` | `(from, to, token_id)` | Token transferred |
| `Approval` | `(owner, approved, token_id)` | Token approval granted |
| `ApprovalForAll` | `(owner, operator, approved)` | Operator approval changed |

### Key Behaviors

- **Dual Authorization for Wallet Updates**: Both current owner AND new wallet must sign
- **Clear agentWallet on Transfer**: Automatically clears wallet association when token transfers (ERC-8004 requirement)
- **Automatic TTL Extension**: Every mutation extends storage TTL

---

## Reputation Registry Contract

Stores structured feedback for agents with anti-Sybil protections.

### Functions

#### Initialization
| Function | Parameters | Description |
|----------|------------|-------------|
| `init` | `admin: Address, identity_registry: Address` | Initialize with identity registry reference |

#### Feedback
| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `give_feedback` | `client: Address, agent_id: u64, value: i128, decimals: u32, tag1: Symbol, tag2: Symbol, endpoint_uri: String, feedback_uri: String, feedback_hash: BytesN<32>` | `u32` | Submit feedback for agent |
| `revoke_feedback` | `client: Address, agent_id: u64, feedback_idx: u32` | - | Revoke previously given feedback |
| `append_response` | `responder: Address, agent_id: u64, client: Address, feedback_idx: u32, response_uri: String, response_hash: BytesN<32>, tag: Symbol` | `u32` | Add response to feedback |

#### Read Functions
| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `read_feedback` | `agent_id: u64, client: Address, idx: u32` | `FeedbackRecord` | Get specific feedback |
| `read_all_feedback` | `agent_id: u64, clients: Vec<Address>, tag1?: Symbol, tag2?: Symbol, include_revoked: bool` | `Vec<FeedbackRecord>` | Get filtered feedback |
| `read_response` | `agent_id: u64, client: Address, feedback_idx: u32, response_idx: u32` | `ResponseRecord` | Get specific response |
| `get_summary` | `agent_id: u64, clients: Vec<Address>, tag1?: Symbol, tag2?: Symbol` | `FeedbackSummary` | Aggregated feedback summary |
| `get_agent_clients` | `agent_id: u64` | `Vec<Address>` | All clients who gave feedback |
| `get_response_count` | `agent_id: u64, client: Address, feedback_idx: u32` | `u32` | Count of responses |

#### Admin
| Function | Parameters | Description |
|----------|------------|-------------|
| `admin` | - | Get contract admin |
| `identity_registry` | - | Get linked identity registry |
| `upgrade` | `new_wasm_hash: BytesN<32>` | Upgrade contract |
| `extend_ttl` | - | Extend contract TTL |

### Data Types

```rust
FeedbackRecord {
    value: i128,              // Feedback value (positive or negative)
    decimals: u32,            // Decimal precision
    tag1: Symbol,             // Primary category
    tag2: Symbol,             // Secondary category
    endpoint_uri: String,     // Endpoint interacted with
    feedback_uri: String,     // Detailed feedback URI
    feedback_hash: BytesN<32>,// Content hash
    revoked: bool,            // Revocation status
    timestamp_ledger: u32,    // Ledger sequence
    client: Address,          // Feedback giver
}

FeedbackSummary {
    count: u32,               // Total feedback count
    total_value: i128,        // Normalized sum
    decimals: u32,            // Precision used
    positive_count: u32,      // Positive feedback count
    negative_count: u32,      // Negative feedback count
}
```

### Events

| Event | Data | Description |
|-------|------|-------------|
| `FeedbackGiven` | `(agent_id, client, idx, value, tag1, tag2)` | Feedback submitted |
| `FeedbackRevoked` | `(agent_id, client, idx)` | Feedback revoked |
| `ResponseAdded` | `(agent_id, client, feedback_idx, responder, tag)` | Response appended |

### Anti-Sybil Protections

- **Self-feedback Prevention**: Blocks feedback from token owner, approved operators, and agent wallet
- **Explicit Client List**: `get_summary` requires non-empty client list to prevent gaming
- **Cross-contract Verification**: Queries Identity Registry to verify relationships

---

## Validation Registry Contract

Request/response system for third-party agent validation.

### Functions

#### Initialization
| Function | Parameters | Description |
|----------|------------|-------------|
| `init` | `admin: Address, identity_registry: Address` | Initialize with identity registry reference |

#### Validation Flow
| Function | Parameters | Description |
|----------|------------|-------------|
| `validation_request` | `caller: Address, agent_id: u64, validator: Address, request_uri: String, request_hash: BytesN<32>` | Submit validation request (owner/operator only) |
| `validation_response` | `validator: Address, request_hash: BytesN<32>, response_code: i32, response_uri: String, response_hash: BytesN<32>, tag: Symbol` | Submit response (designated validator only) |

#### Read Functions
| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `get_validation_status` | `request_hash: BytesN<32>` | `ValidationStatus` | Get request + response status |
| `get_request` | `request_hash: BytesN<32>` | `ValidationRequest` | Get request details |
| `get_response` | `request_hash: BytesN<32>` | `ValidationResponse` | Get response details |
| `get_agent_validations` | `agent_id: u64` | `Vec<BytesN<32>>` | All request hashes for agent |
| `get_validator_requests` | `validator: Address` | `Vec<BytesN<32>>` | All requests for validator |
| `get_summary` | `agent_id: u64, validators: Vec<Address>, tag?: Symbol` | `ValidationSummary` | Aggregated validation summary |

#### Admin
| Function | Parameters | Description |
|----------|------------|-------------|
| `admin` | - | Get contract admin |
| `identity_registry` | - | Get linked identity registry |
| `upgrade` | `new_wasm_hash: BytesN<32>` | Upgrade contract |
| `extend_ttl` | - | Extend contract TTL |

### Response Codes

| Code | Name | Description |
|------|------|-------------|
| `0` | `PENDING` | Awaiting response |
| `1` | `VALID` | Agent validated successfully |
| `2` | `INVALID` | Agent failed validation |
| `3` | `UNABLE_TO_VALIDATE` | Inconclusive |
| `-1` | `TIMEOUT` | Validation timed out |
| `-2` | `MALFORMED_REQUEST` | Invalid request format |

### Data Types

```rust
ValidationRequest {
    agent_id: u64,
    validator: Address,
    requester: Address,
    request_uri: String,
    request_hash: BytesN<32>,
    timestamp_ledger: u32,
    responded: bool,
}

ValidationStatus {
    request: ValidationRequest,
    has_response: bool,
    response_code: i32,
    response_uri: String,
    response_hash: BytesN<32>,
    response_tag: Symbol,
    response_timestamp: u32,
}

ValidationSummary {
    total_requests: u32,
    responded_count: u32,
    valid_count: u32,
    invalid_count: u32,
    other_count: u32,
}
```

### Events

| Event | Data | Description |
|-------|------|-------------|
| `ValidationRequested` | `(agent_id, validator, requester, request_hash)` | Request submitted |
| `ValidationResponded` | `(agent_id, validator, request_hash, response_code, tag)` | Response submitted |

---

## Agent Registration JSON Schema

Agents point to a registration JSON file via their `token_uri`. See `schemas/agent-registration.json` for the full schema.

### Example

```json
{
  "type": "https://eips.ethereum.org/EIPS/eip-8004#registration-v1",
  "name": "MySorobanAgent",
  "description": "An AI agent on Stellar",
  "image": "https://example.com/agent.png",
  "services": [
    { "name": "web", "endpoint": "https://agent.example.com/" },
    { "name": "MCP", "endpoint": "https://mcp.agent.example.com/", "version": "2025-06-18" },
    { "name": "A2A", "endpoint": "https://agent.example.com/.well-known/agent-card.json", "version": "0.3.0" }
  ],
  "active": true,
  "registrations": [
    {
      "agentRegistry": "stellar:pubnet:CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
      "agentId": 42
    }
  ],
  "supportedTrust": ["reputation", "validation"]
}
```

---

## Building

```bash
# Build all contracts
stellar contract build

# Build specific contract
stellar contract build --manifest-path contracts/identity-registry/Cargo.toml

# Run tests
cargo test
```

### Output

WASM files are generated in `target/wasm32v1-none/release/`:
- `identity_registry.wasm`
- `reputation_registry.wasm`
- `validation_registry.wasm`

---

## Deployment

```bash
# Deploy Identity Registry
stellar contract deploy \
  --wasm target/wasm32v1-none/release/identity_registry.wasm \
  --source <DEPLOYER_SECRET> \
  --network testnet

# Initialize Identity Registry
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <ADMIN_SECRET> \
  --network testnet \
  -- init \
  --admin <ADMIN_ADDRESS> \
  --name "AgentRegistry" \
  --symbol "AGENT"

# Deploy Reputation Registry (requires Identity Registry address)
stellar contract deploy \
  --wasm target/wasm32v1-none/release/reputation_registry.wasm \
  --source <DEPLOYER_SECRET> \
  --network testnet

# Initialize Reputation Registry
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --source <ADMIN_SECRET> \
  --network testnet \
  -- init \
  --admin <ADMIN_ADDRESS> \
  --identity_registry <IDENTITY_CONTRACT_ID>
```

---

## Soroban-Specific Considerations

### TTL Management

Soroban storage has TTL (Time-To-Live). All contracts:
- Extend TTL on every mutation (threshold: 100,000 ledgers, extend to: 1,000,000 ledgers)
- Provide public `extend_ttl()` function for community stewardship
- Include `extend_agent_ttl(token_id)` for per-agent TTL extension

### Event Retention

Stellar RPC has **7-day event retention**. For production:
- Implement an event indexer polling `getEvents`
- Persist to PostgreSQL or similar
- Build query APIs for agent discovery

### Cross-Contract Calls

Reputation and Validation registries query Identity Registry for:
- Token ownership verification
- Operator approval status
- Agent wallet association

---

## License

MIT

## References

- [ERC-8004 Specification](https://eips.ethereum.org/EIPS/eip-8004)
- [ERC-8004 Reference Implementation](https://github.com/erc-8004/erc-8004-contracts)
- [SEP-50 Non-Fungible Tokens](https://github.com/orgs/stellar/discussions/1674)
- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
