# ERC-8004 Contract Usage Guide

Step-by-step workflows for interacting with the Identity, Reputation, and Validation registries.

---

## Table of Contents

1. [Setup & Initialization](#setup--initialization)
2. [Agent Registration Flow](#agent-registration-flow)
3. [Agent Management Flow](#agent-management-flow)
4. [Reputation Flow](#reputation-flow)
5. [Validation Flow](#validation-flow)
6. [Query & Discovery Flow](#query--discovery-flow)
7. [Admin Operations](#admin-operations)

---

## Setup & Initialization

### Step 1: Deploy Contracts

```bash
# 1. Deploy Identity Registry
stellar contract deploy \
  --wasm target/wasm32v1-none/release/identity_registry.wasm \
  --source <DEPLOYER_SECRET> \
  --network testnet

# Save the contract ID: IDENTITY_CONTRACT_ID

# 2. Deploy Reputation Registry
stellar contract deploy \
  --wasm target/wasm32v1-none/release/reputation_registry.wasm \
  --source <DEPLOYER_SECRET> \
  --network testnet

# Save the contract ID: REPUTATION_CONTRACT_ID

# 3. Deploy Validation Registry
stellar contract deploy \
  --wasm target/wasm32v1-none/release/validation_registry.wasm \
  --source <DEPLOYER_SECRET> \
  --network testnet

# Save the contract ID: VALIDATION_CONTRACT_ID
```

### Step 2: Initialize Identity Registry

```bash
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <ADMIN_SECRET> \
  --network testnet \
  -- init \
  --admin <ADMIN_ADDRESS> \
  --name "AgentRegistry" \
  --symbol "AGENT"
```

**Verify:**
```bash
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- name

stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- admin
```

### Step 3: Initialize Reputation Registry

```bash
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --source <ADMIN_SECRET> \
  --network testnet \
  -- init \
  --admin <ADMIN_ADDRESS> \
  --identity_registry <IDENTITY_CONTRACT_ID>
```

**Verify:**
```bash
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --network testnet \
  -- identity_registry
```

### Step 4: Initialize Validation Registry

```bash
stellar contract invoke \
  --id <VALIDATION_CONTRACT_ID> \
  --source <ADMIN_SECRET> \
  --network testnet \
  -- init \
  --admin <ADMIN_ADDRESS> \
  --identity_registry <IDENTITY_CONTRACT_ID>
```

---

## Agent Registration Flow

### Complete Flow: Register a New Agent

#### Step 1: Prepare Registration JSON

Create `agent-registration.json`:

```json
{
  "type": "https://eips.ethereum.org/EIPS/eip-8004#registration-v1",
  "name": "MyAgent",
  "description": "AI agent for financial analysis",
  "image": "https://example.com/agent.png",
  "services": [
    { "name": "web", "endpoint": "https://agent.example.com/" },
    { "name": "MCP", "endpoint": "https://mcp.agent.example.com/", "version": "2025-06-18" }
  ],
  "active": true,
  "registrations": [
    {
      "agentRegistry": "stellar:testnet:<IDENTITY_CONTRACT_ID>",
      "agentId": 0
    }
  ],
  "supportedTrust": ["reputation", "validation"]
}
```

#### Step 2: Host Registration JSON

Upload to IPFS, HTTPS, or any accessible URL:
```
https://example.com/agent-registration.json
```

#### Step 3: Register Agent

```bash
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <OWNER_SECRET> \
  --network testnet \
  -- register \
  --owner <OWNER_ADDRESS> \
  --agent_uri "https://example.com/agent-registration.json"
```

**Returns:** `token_id` (e.g., `0`)

#### Step 4: Verify Registration

```bash
# Get token owner
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- owner_of \
  --token_id 0

# Get token URI
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- token_uri \
  --token_id 0

# Get agent wallet (initially same as owner)
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- get_agent_wallet \
  --token_id 0

# Check balance
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- balance_of \
  --owner <OWNER_ADDRESS>
```

---

## Agent Management Flow

### Update Agent URI

```bash
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <OWNER_SECRET> \
  --network testnet \
  -- set_agent_uri \
  --owner <OWNER_ADDRESS> \
  --token_id 0 \
  --new_uri "https://example.com/agent-registration-v2.json"
```

### Update Agent Wallet (Dual Authorization Required)

**Important:** Both current owner AND new wallet must sign.

```bash
# Transaction 1: Owner signs
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <OWNER_SECRET> \
  --network testnet \
  -- set_agent_wallet \
  --owner <OWNER_ADDRESS> \
  --token_id 0 \
  --new_wallet <NEW_WALLET_ADDRESS>

# Transaction 2: New wallet signs (same call)
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <NEW_WALLET_SECRET> \
  --network testnet \
  -- set_agent_wallet \
  --owner <OWNER_ADDRESS> \
  --token_id 0 \
  --new_wallet <NEW_WALLET_ADDRESS>
```

### Set Custom Metadata

```bash
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <OWNER_SECRET> \
  --network testnet \
  -- set_metadata \
  --owner <OWNER_ADDRESS> \
  --token_id 0 \
  --key <32_BYTE_KEY> \
  --value <32_BYTE_VALUE>
```

### Transfer Agent Identity

```bash
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <OWNER_SECRET> \
  --network testnet \
  -- transfer_from \
  --from <OWNER_ADDRESS> \
  --to <NEW_OWNER_ADDRESS> \
  --token_id 0
```

**Note:** Transfer automatically clears `agentWallet` metadata (ERC-8004 requirement).

### Approve Token Transfer

```bash
# Approve specific address for one token
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <OWNER_SECRET> \
  --network testnet \
  -- approve \
  --owner <OWNER_ADDRESS> \
  --approved <APPROVED_ADDRESS> \
  --token_id 0

# Approve operator for all tokens
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <OWNER_SECRET> \
  --network testnet \
  -- set_approval_for_all \
  --owner <OWNER_ADDRESS> \
  --operator <OPERATOR_ADDRESS> \
  --approved true
```

---

## Reputation Flow

### Give Feedback to an Agent

```bash
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --source <CLIENT_SECRET> \
  --network testnet \
  -- give_feedback \
  --client <CLIENT_ADDRESS> \
  --agent_id 0 \
  --value 100 \
  --decimals 2 \
  --tag1 "quality" \
  --tag2 "fast" \
  --endpoint_uri "https://agent.example.com/api" \
  --feedback_uri "https://example.com/feedback/1" \
  --feedback_hash <32_BYTE_HASH>
```

**Returns:** `feedback_index` (e.g., `0`)

**Self-feedback Prevention:**
- Automatically blocks if `client` is token owner
- Blocks if `client` is approved operator
- Blocks if `client` matches agent wallet

### Read Feedback

```bash
# Read specific feedback
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --network testnet \
  -- read_feedback \
  --agent_id 0 \
  --client <CLIENT_ADDRESS> \
  --idx 0

# Read all feedback from specific clients
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --network testnet \
  -- read_all_feedback \
  --agent_id 0 \
  --clients [<CLIENT1>,<CLIENT2>] \
  --tag1 "quality" \
  --tag2 null \
  --include_revoked false
```

### Get Reputation Summary

```bash
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --network testnet \
  -- get_summary \
  --agent_id 0 \
  --clients [<CLIENT1>,<CLIENT2>,<CLIENT3>] \
  --tag1 null \
  --tag2 null
```

**Returns:**
```json
{
  "count": 5,
  "total_value": 450,
  "decimals": 2,
  "positive_count": 4,
  "negative_count": 1
}
```

### Revoke Feedback

```bash
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --source <CLIENT_SECRET> \
  --network testnet \
  -- revoke_feedback \
  --client <CLIENT_ADDRESS> \
  --agent_id 0 \
  --feedback_idx 0
```

### Add Response to Feedback

```bash
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --source <RESPONDER_SECRET> \
  --network testnet \
  -- append_response \
  --responder <RESPONDER_ADDRESS> \
  --agent_id 0 \
  --client <CLIENT_ADDRESS> \
  --feedback_idx 0 \
  --response_uri "https://agent.example.com/response/1" \
  --response_hash <32_BYTE_HASH> \
  --tag "clarification"
```

### Get All Clients for Agent

```bash
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --network testnet \
  -- get_agent_clients \
  --agent_id 0
```

---

## Validation Flow

### Submit Validation Request

**Requires:** Caller must be token owner or approved operator.

```bash
stellar contract invoke \
  --id <VALIDATION_CONTRACT_ID> \
  --source <OWNER_SECRET> \
  --network testnet \
  -- validation_request \
  --caller <OWNER_ADDRESS> \
  --agent_id 0 \
  --validator <VALIDATOR_ADDRESS> \
  --request_uri "https://validator.example.com/request/1" \
  --request_hash <32_BYTE_HASH>
```

### Check Request Status

```bash
stellar contract invoke \
  --id <VALIDATION_CONTRACT_ID> \
  --network testnet \
  -- get_validation_status \
  --request_hash <32_BYTE_HASH>
```

**Returns:**
```json
{
  "request": {
    "agent_id": 0,
    "validator": "<VALIDATOR_ADDRESS>",
    "requester": "<OWNER_ADDRESS>",
    "request_uri": "...",
    "request_hash": "...",
    "timestamp_ledger": 12345,
    "responded": false
  },
  "has_response": false,
  "response_code": 0,
  "response_uri": "",
  "response_hash": "0000...",
  "response_tag": "",
  "response_timestamp": 0
}
```

### Submit Validation Response

**Requires:** Must be the designated validator.

```bash
stellar contract invoke \
  --id <VALIDATION_CONTRACT_ID> \
  --source <VALIDATOR_SECRET> \
  --network testnet \
  -- validation_response \
  --validator <VALIDATOR_ADDRESS> \
  --request_hash <32_BYTE_HASH> \
  --response_code 1 \
  --response_uri "https://validator.example.com/response/1" \
  --response_hash <32_BYTE_HASH> \
  --tag "identity"
```

**Response Codes:**
- `0` = PENDING
- `1` = VALID
- `2` = INVALID
- `3` = UNABLE_TO_VALIDATE
- `-1` = TIMEOUT
- `-2` = MALFORMED_REQUEST

### Get Validation Summary

```bash
stellar contract invoke \
  --id <VALIDATION_CONTRACT_ID> \
  --network testnet \
  -- get_summary \
  --agent_id 0 \
  --validators [<VALIDATOR1>,<VALIDATOR2>] \
  --tag null
```

**Returns:**
```json
{
  "total_requests": 3,
  "responded_count": 2,
  "valid_count": 1,
  "invalid_count": 0,
  "other_count": 1
}
```

### Get All Validations for Agent

```bash
stellar contract invoke \
  --id <VALIDATION_CONTRACT_ID> \
  --network testnet \
  -- get_agent_validations \
  --agent_id 0
```

### Get All Requests for Validator

```bash
stellar contract invoke \
  --id <VALIDATION_CONTRACT_ID> \
  --network testnet \
  -- get_validator_requests \
  --validator <VALIDATOR_ADDRESS>
```

---

## Query & Discovery Flow

### Discover All Agents

**Note:** Requires event indexing (RPC has 7-day retention).

1. Query events for `AgentRegistered`:
```bash
# Using Stellar SDK or RPC
curl -X POST https://horizon-testnet.stellar.org \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getEvents",
    "params": {
      "contractId": "<IDENTITY_CONTRACT_ID>",
      "filters": [{"type": "AgentRegistered"}]
    }
  }'
```

2. For each `token_id`, fetch:
   - `token_uri(token_id)` → Download registration JSON
   - `owner_of(token_id)` → Current owner
   - `get_agent_wallet(token_id)` → Agent wallet (if set)

### Get Agent Profile

```bash
# 1. Get basic info
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- owner_of \
  --token_id 0

stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- token_uri \
  --token_id 0

# 2. Get reputation summary
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --network testnet \
  -- get_agent_clients \
  --agent_id 0

# Then get summary with those clients
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --network testnet \
  -- get_summary \
  --agent_id 0 \
  --clients [<CLIENT1>,<CLIENT2>]

# 3. Get validation status
stellar contract invoke \
  --id <VALIDATION_CONTRACT_ID> \
  --network testnet \
  -- get_agent_validations \
  --agent_id 0
```

### Check Token Approvals

```bash
# Check specific token approval
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- get_approved \
  --token_id 0

# Check operator approval
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- is_approved_for_all \
  --owner <OWNER_ADDRESS> \
  --operator <OPERATOR_ADDRESS>
```

---

## Admin Operations

### Update Admin

```bash
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <CURRENT_ADMIN_SECRET> \
  --network testnet \
  -- set_admin \
  --new_admin <NEW_ADMIN_ADDRESS>
```

### Upgrade Contract

```bash
# 1. Build new WASM
stellar contract build --manifest-path contracts/identity-registry/Cargo.toml

# 2. Deploy new WASM (get new hash)
stellar contract deploy \
  --wasm target/wasm32v1-none/release/identity_registry.wasm \
  --source <DEPLOYER_SECRET> \
  --network testnet

# 3. Upgrade contract
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <ADMIN_SECRET> \
  --network testnet \
  -- upgrade \
  --new_wasm_hash <NEW_WASM_HASH>
```

### Extend TTL (Anyone Can Call)

```bash
# Extend contract instance TTL
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- extend_ttl

# Extend specific agent's TTL
stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --network testnet \
  -- extend_agent_ttl \
  --token_id 0
```

---

## Complete Example Workflow

### Scenario: Register Agent → Get Feedback → Validate

```bash
# 1. Register agent
TOKEN_ID=$(stellar contract invoke \
  --id <IDENTITY_CONTRACT_ID> \
  --source <OWNER_SECRET> \
  --network testnet \
  -- register \
  --owner <OWNER_ADDRESS> \
  --agent_uri "https://example.com/agent.json")

# 2. Client gives positive feedback
FEEDBACK_IDX=$(stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --source <CLIENT_SECRET> \
  --network testnet \
  -- give_feedback \
  --client <CLIENT_ADDRESS> \
  --agent_id $TOKEN_ID \
  --value 100 \
  --decimals 2 \
  --tag1 "quality" \
  --tag2 "fast" \
  --endpoint_uri "" \
  --feedback_uri "" \
  --feedback_hash <HASH>)

# 3. Get reputation summary
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --network testnet \
  -- get_summary \
  --agent_id $TOKEN_ID \
  --clients [<CLIENT_ADDRESS>]

# 4. Request validation
REQUEST_HASH=$(echo -n "validation_request_1" | sha256sum | cut -d' ' -f1)
stellar contract invoke \
  --id <VALIDATION_CONTRACT_ID> \
  --source <OWNER_SECRET> \
  --network testnet \
  -- validation_request \
  --caller <OWNER_ADDRESS> \
  --agent_id $TOKEN_ID \
  --validator <VALIDATOR_ADDRESS> \
  --request_uri "https://validator.example.com/request/1" \
  --request_hash $REQUEST_HASH

# 5. Validator responds
stellar contract invoke \
  --id <VALIDATION_CONTRACT_ID> \
  --source <VALIDATOR_SECRET> \
  --network testnet \
  -- validation_response \
  --validator <VALIDATOR_ADDRESS> \
  --request_hash $REQUEST_HASH \
  --response_code 1 \
  --response_uri "https://validator.example.com/response/1" \
  --response_hash <HASH> \
  --tag "identity"

# 6. Check final status
stellar contract invoke \
  --id <VALIDATION_CONTRACT_ID> \
  --network testnet \
  -- get_validation_status \
  --request_hash $REQUEST_HASH
```

---

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `NotAuthorized` | Wrong caller or missing approval | Verify caller is owner/operator |
| `SelfFeedbackNotAllowed` | Client is owner/operator/wallet | Use different client address |
| `TokenNotFound` | Invalid token_id | Verify token exists |
| `EmptyClientList` | Summary requires clients | Provide non-empty client list |
| `AlreadyResponded` | Validation already has response | Use different request_hash |
| `NotValidator` | Wrong validator address | Use designated validator |

### Best Practices

1. **Always verify token exists** before operations
2. **Check approvals** before cross-contract calls
3. **Use explicit client lists** for reputation summaries (anti-Sybil)
4. **Extend TTL regularly** for active agents
5. **Index events** for discovery (7-day RPC retention)
6. **Validate registration JSON** before setting URI

---

## Integration Examples

### JavaScript/TypeScript (Stellar SDK)

```typescript
import { Contract, Networks, TransactionBuilder } from '@stellar/stellar-sdk';

const identityContract = new Contract(IDENTITY_CONTRACT_ID);

// Register agent
const tx = new TransactionBuilder(account, {
  fee: '100',
  networkPassphrase: Networks.TESTNET,
})
.addOperation(identityContract.call('register', {
  owner: ownerAddress,
  agent_uri: 'https://example.com/agent.json',
}))
.setTimeout(30)
.build();

// Sign and submit
const signedTx = tx.sign(keypair);
const result = await server.submitTransaction(signedTx);
```

### Python (Stellar SDK)

```python
from stellar_sdk import Server, Keypair, TransactionBuilder, Network
from stellar_sdk.contract import Contract

server = Server("https://horizon-testnet.stellar.org")
identity_contract = Contract(IDENTITY_CONTRACT_ID)

# Register agent
tx = (
    TransactionBuilder(source_account, network_passphrase=Network.TESTNET_NETWORK_PASSPHASE)
    .append_contract_call(identity_contract.call('register', {
        'owner': owner_address,
        'agent_uri': 'https://example.com/agent.json',
    }))
    .build()
)

tx.sign(keypair)
response = server.submit_transaction(tx)
```

---

## Next Steps

1. **Build Event Indexer**: Implement persistent event storage (7-day RPC limit)
2. **Create Frontend**: Build UI for agent registration and discovery
3. **Add Rate Limiting**: Implement spam protection for feedback
4. **Domain Verification**: Add `.well-known/agent-registration.json` support
5. **Multi-chain Support**: Extend to other networks with same namespace pattern

