# Mainnet Deployment Guide

Deploying `accensa-contracts` to the Stellar Mainnet is similar to the testnet process, but requires careful handling of real funds, precise configuration of the USDC SAC token, and an understanding of transaction and storage rent fees.

## Step-by-Step Mainnet Deployment

### 1. Fund a Deployer Account
You need a Stellar account with sufficient XLM to cover base reserves, transaction fees, and storage rent.
- Create an account on the Stellar Mainnet.
- Fund it with at least 50-100 XLM to comfortably cover contract storage rent and execution fees.

### 2. Configure Your Environment
Set up your Stellar CLI identity and network for Mainnet:
```bash
stellar network add mainnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Public Global Stellar Network ; September 2015"

stellar keys generate deployer --network mainnet
# (Make sure to fund the generated public key)
```

### 3. Deploy and Initialize
Unlike testnet, where the `deploy.sh` script defaults to the native XLM token, Mainnet deployments should use the official USDC Stellar Asset Contract (SAC).
- **USDC SAC Address (Mainnet)**: `CEQ...` (replace with the actual Mainnet USDC SAC ID).

Run the deployment script with the Mainnet parameters:
```bash
NETWORK=mainnet TOKEN=<mainnet-usdc-sac-id> ./deploy.sh
```
This will compile the contracts, deploy them, initialize them with your deployer identity as the admin, and output the contract IDs to `deployments/mainnet.env`.

## Fee and Rent Analysis

### Transaction Fees (Testnet Proxies)
Soroban transaction fees are generally highly predictable. Based on testnet benchmarks, here are the measured fee projections for core operations:

| Operation | Estimated Fee (XLM) | Notes |
|---|---|---|
| `anchor_batch` | ~0.02 - 0.05 XLM | Scales slightly with the number of persistent storage reads/writes. |
| `refund` | ~0.015 - 0.03 XLM | Involves cross-contract calls to the USDC SAC. |
| `verify_receipt` | 0 XLM | Read-only simulation. |

### Rent Cost Projection
Stellar's state archiving mechanism requires paying "rent" to keep data in `Persistent` storage. 

A single `BatchRecord` contains:
- `root`: 32 bytes
- `count`: 4 bytes
- `period_start`: 8 bytes
- `period_end`: 8 bytes
- Overhead: ~50 bytes

**Total size per batch**: ~100 bytes.

**Scenario**: 500-payment batches, 1 year retention.
- If you process 10,000 payments a day in batches of 500, that is **20 batches per day**.
- **Yearly volume**: 7,300 batches.
- **Storage required**: 7,300 * 100 bytes = ~730 KB.
- **Rent cost**: Persistent storage on Stellar costs roughly **0.5 XLM per KB per year**.
- **Total projected rent**: ~365 XLM per year for archiving the anchors of 3.65 million payments.

This amortizes to a negligible fraction of a cent per payment, making the on-chain verifiable receipt architecture highly economical.
