# Deployment Guide

## Overview

Deployment order matters because Registry and Reputation each take the
Governance contract's address as a constructor argument:

1. Deploy **Governance** with `--owner <your address>`.
2. Deploy **Registry** with `--governance <governance contract id>`.
3. Deploy **Reputation** with `--governance <governance contract id>`.
4. (Optional but recommended) Wire them together:
   - `reputation.set_registry_contract(owner, registry_id)`
   - `registry.set_reputation_contract(owner, reputation_id)`
5. Add validators via `governance.add_validator(owner, validator_address)`.

`scripts/deploy.sh` automates all five steps against a network of your
choice (testnet by default). Read it before running it -- it's a starting
point, not a black box, and you should decide who holds the `owner` key
before running it against any network that matters.

## Prerequisites

```bash
scripts/setup.sh   # installs stellar-cli + the wasm32v1-none target
```

You'll also need a funded account on whichever network you're deploying to.
On testnet, `stellar keys generate <name> --network testnet --fund` (which
`scripts/deploy.sh` does automatically) funds the account via Friendbot. On
mainnet, fund the account yourself before deploying.

## Testnet

```bash
NETWORK=testnet SOURCE_IDENTITY=my-deployer scripts/deploy.sh
```

This prints the three contract ids and the owner address. Save them --
you'll need the ids for any further `stellar contract invoke` calls, and the
owner key to manage admins/validators/upgrades.

## Mainnet

The same script works with `NETWORK=mainnet`, but treat mainnet deployment
as a distinct, deliberate act, not a variable substitution:

- **This code has not been audited.** Do not deploy to mainnet with
  real user funds or reputational stakes riding on it without an independent
  security review first (see [`SECURITY.md`](../SECURITY.md)).
- **Decide the owner key's custody before deploying.** The owner can add/
  remove admins and execute upgrades (after the timelock). A single EOA
  key is a single point of failure; consider a multisig account or a
  dedicated smart account contract as the owner address instead of a plain
  keypair.
- **Decide your validator set and threshold for trust.** This contract suite
  ships single-validator approval (any one validator or admin can validate a
  report); if you need N-of-M consensus before a report is marked
  `Validated`, that's a Registry change worth making (and a good first
  community contribution) before relying on it at scale.
- Review `contracts/governance/src/lib.rs`'s `UPGRADE_TIMELOCK_SECS` (3 days
  by default) and decide if that's the right window for your governance
  process.

```bash
NETWORK=mainnet SOURCE_IDENTITY=my-mainnet-deployer scripts/deploy.sh
```

## Manual invocation examples

Once deployed, interact with the contracts directly via `stellar contract
invoke`. A few examples (substitute your contract ids/network/identity):

```bash
# Add a validator (owner or admin only).
stellar contract invoke --id "$GOVERNANCE_ID" --source-account my-deployer --network testnet \
  -- add_validator --caller "$OWNER" --validator "$VALIDATOR_ADDRESS"

# File a report against a malicious account.
stellar contract invoke --id "$REGISTRY_ID" --source-account my-reporter --network testnet \
  -- report_account --reporter "$REPORTER_ADDRESS" --address "$SCAM_ADDRESS" \
     --risk_level High --evidence_uri "https://example.com/evidence"

# Validate it.
stellar contract invoke --id "$REGISTRY_ID" --source-account my-validator --network testnet \
  -- validate_report --validator "$VALIDATOR_ADDRESS" --report_id 1 --approve true

# Check an account's reputation.
stellar contract invoke --id "$REPUTATION_ID" --source-account my-deployer --network testnet \
  -- get_reputation --account "$REPORTER_ADDRESS"
```

## Upgrading a deployed contract

Each contract's Wasm can be replaced via Governance's timelocked upgrade
flow (this replaces *Governance's own* code; Registry/Reputation would need
their own analogous upgrade entry point wired to check
`governance.is_admin`/`is_owner` before calling
`env.deployer().update_current_contract_wasm(..)` -- left as a follow-up,
since the pattern is the same one Governance already demonstrates):

```bash
stellar contract build --package scamwatchxlm-governance
NEW_HASH=$(stellar contract upload --wasm target/wasm32v1-none/release/governance.wasm \
  --source-account my-deployer --network testnet)

stellar contract invoke --id "$GOVERNANCE_ID" --source-account my-deployer --network testnet \
  -- propose_upgrade --caller "$OWNER" --wasm_hash "$NEW_HASH"

# Wait UPGRADE_TIMELOCK_SECS (3 days by default), then:
stellar contract invoke --id "$GOVERNANCE_ID" --source-account my-deployer --network testnet \
  -- execute_upgrade --caller "$OWNER"
```
