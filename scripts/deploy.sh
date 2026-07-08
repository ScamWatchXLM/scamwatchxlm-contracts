#!/usr/bin/env bash
# Example end-to-end deployment: builds all three contracts, deploys them to
# a network (testnet by default), and wires Registry <-> Reputation
# together. Intended as a starting point for your own deployment tooling,
# not as a one-command production deploy -- review it, and in particular
# decide who should hold the `owner`/admin keys before running against
# mainnet.
#
# Usage:
#   NETWORK=testnet SOURCE_IDENTITY=my-key scripts/deploy.sh
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

command -v stellar >/dev/null 2>&1 || {
  echo "stellar-cli not found. Run scripts/setup.sh first." >&2
  exit 1
}

NETWORK="${NETWORK:-testnet}"
SOURCE="${SOURCE_IDENTITY:-scamwatch-deployer}"

if ! stellar keys ls | grep -qx "$SOURCE"; then
  echo "==> Generating and funding identity '$SOURCE' on $NETWORK"
  stellar keys generate "$SOURCE" --network "$NETWORK" --fund
fi
OWNER="$(stellar keys public-key "$SOURCE")"
echo "==> Deploying as $SOURCE ($OWNER) on $NETWORK"

echo "==> Building contracts"
"$repo_root/scripts/build.sh"

echo "==> Deploying Governance"
GOVERNANCE_ID=$(stellar contract deploy \
  --wasm target/contracts/governance.wasm \
  --source-account "$SOURCE" \
  --network "$NETWORK" \
  --alias governance \
  -- --owner "$OWNER")

echo "==> Deploying Registry"
REGISTRY_ID=$(stellar contract deploy \
  --wasm target/contracts/registry.wasm \
  --source-account "$SOURCE" \
  --network "$NETWORK" \
  --alias registry \
  -- --governance "$GOVERNANCE_ID")

echo "==> Deploying Reputation"
REPUTATION_ID=$(stellar contract deploy \
  --wasm target/contracts/reputation.wasm \
  --source-account "$SOURCE" \
  --network "$NETWORK" \
  --alias reputation \
  -- --governance "$GOVERNANCE_ID")

echo "==> Wiring Registry <-> Reputation"
stellar contract invoke --id "$REPUTATION_ID" --source-account "$SOURCE" --network "$NETWORK" \
  -- set_registry_contract --caller "$OWNER" --registry "$REGISTRY_ID"
stellar contract invoke --id "$REGISTRY_ID" --source-account "$SOURCE" --network "$NETWORK" \
  -- set_reputation_contract --caller "$OWNER" --reputation "$REPUTATION_ID"

cat <<EOF

==> Deployed to $NETWORK:
    governance: $GOVERNANCE_ID
    registry:   $REGISTRY_ID
    reputation: $REPUTATION_ID
    owner:      $OWNER

Next steps: add validators with
  stellar contract invoke --id "$GOVERNANCE_ID" --source-account "$SOURCE" --network "$NETWORK" -- add_validator --caller "$OWNER" --validator <ADDRESS>
EOF
