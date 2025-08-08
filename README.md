# ohms-model — Model Repository Canister

Governance‑gated storage and serving of APQ artifacts (manifest + shards). Pending → Active → Deprecated lifecycle; immutable Active models; SHA‑256 integrity.

## Overview

The Model Repository is the central storage canister for OHMS quantized models. It enforces the Pending → Active → Deprecated lifecycle and provides immutable storage for Active models.

### Features

- Governance‑gated activation
- Immutable Active storage
- Chunk serving (≤ 2 MiB)
- SHA‑256 integrity checks
- Audit log

## Architecture

```
Upload → Pending → [Governance Vote] → Active → [Optional] Deprecated
```

## API

Upload: `submit_model`, `submit_quantized_model`, `add_authorized_uploader`

Lifecycle: `activate_model`, `deprecate_model`

Query: `get_manifest`, `get_model_meta`, `get_chunk`, `list_models(state?)`, `list_quantized_models`, `get_global_stats`, `get_audit_log`, `get_compression_stats`

## Deployment

### Prerequisites
- DFX SDK
- Rust toolchain
- Internet Computer network access

### Build & Deploy (local)
```bash
dfx build ohms_model
dfx deploy ohms_model --network local
```

### Initialize uploader
```bash
# Add initial authorized uploader
dfx canister call ohms_model add_authorized_uploader '("<principal>")'
```

### Mainnet quick publish
```bash
# Pack APQ artifact from CLI (ohms-adaptq)
apq pack --input /path/model.sapq --out /tmp/model_artifact.json

# Publish to ohms_model canister
apq publish \
  --canister <CANISTER_ID> \
  --model-id <model_id> \
  --source-model "hf:owner/repo:file" \
  --artifact /tmp/model_artifact.json \
  --network https://ic0.app --identity <dfx_identity_name>

# Activate
dfx canister call <CANISTER_ID> activate_model '("<model_id>")' --network ic
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
