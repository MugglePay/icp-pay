# ICP Payment

ICP payment system that allows users to make payments using ICP/ICRC tokens and autoswap to ckUSDC token. The system is built on the Internet Computer and uses the _**Rust**_ programming language.

## Features

- **Payment**: Users can make payments using ICP/ICRC tokens.
- **Autoswap**: The system automatically swaps ICP/ICRC tokens to ckUSDC tokens.

## Deployment

#### Deploying the ICP Payment Backend

1. Build the project:

```bash
cargo build --release --target wasm32-unknown-unknown --package icp_payment_backend
```

2. Extract the Candid interface:

```bash
cargo install candid-extractor

candid-extractor target/wasm32-unknown-unknown/release/icp_payment_backend.wasm > src/icp_payment_backend/icp_payment_backend.did
```

3. Scripts:

- `deploy_ledger.sh`: Deploys the ledger canister.
- `deploy_canister.sh`: Deploys the ICP payment backend canister.

4. Deploy the ICP payment backend:

```bash
# Internet Computer deployment
dfx deploy icp_payment_backend --network ic
```
