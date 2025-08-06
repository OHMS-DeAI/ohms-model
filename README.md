# OHMS-Model

**Model Repository Canister**

Governance-gated storage and serving of quantized model shards and manifests for the OHMS platform.

## Overview

The Model Repository is the central storage canister for OHMS quantized models. It enforces the Pending → Active → Deprecated lifecycle and provides immutable storage for Active models.

### Key Features

- **Governance-gated activation**: Models require approval to become Active
- **Immutable Active storage**: No modifications allowed once activated
- **Chunk-based serving**: Efficient ≤2 MiB shard retrieval
- **Integrity verification**: SHA-256 hash validation for all chunks
- **Audit trail**: Complete log of all model operations

## Architecture

```
Upload → Pending → [Governance Vote] → Active → [Optional] Deprecated
```

## API Surface

### Upload Operations
- `submit_model()` - Upload model artifacts (Pending state)
- `add_authorized_uploader()` - Grant upload permissions

### Activation Operations  
- `activate_model()` - Move Pending → Active (governance-gated)
- `deprecate_model()` - Move Active → Deprecated

### Query Operations
- `get_manifest()` - Retrieve model manifest
- `get_chunk()` - Retrieve specific shard chunk
- `list_models()` - List models by state filter
- `get_audit_log()` - View operation history

## Deployment

### Prerequisites
- DFX SDK
- Rust toolchain
- Internet Computer network access

### Build & Deploy
```bash
dfx build ohms_model
dfx deploy ohms_model --network local
```

### Initialize
```bash
# Add initial authorized uploader
dfx canister call ohms_model add_authorized_uploader '("principal-id")'
```

## Development

### Local Testing
```bash
# Start local replica
dfx start --background

# Deploy canister
dfx deploy ohms_model

# Run tests
cargo test
```

## License

MIT - See LICENSE file
