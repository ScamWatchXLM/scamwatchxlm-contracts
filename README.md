# ScamWatchXLM Contracts

[![CI](https://github.com/scamwatchxlm/scamwatchxlm-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/scamwatchxlm/scamwatchxlm-contracts/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Smart contracts powering the on-chain components of **ScamWatchXLM**, an
open-source scam detection platform for the Stellar ecosystem. Anyone can
report malicious accounts, asset issuers, assets, phishing domains, and scam
transactions; a permissioned set of validators confirms or rejects reports;
and a reputation score keeps the incentives honest.

Built with [Soroban](https://developers.stellar.org/docs/build/smart-contracts/overview),
Stellar's smart contract platform.

## Contracts

| Contract | Crate | Responsibility |
| --- | --- | --- |
| **Registry** | [`contracts/registry`](contracts/registry) | Files and stores reports against accounts, issuers, assets, domains, and transactions; duplicate prevention; aggregate risk records; pagination. |
| **Reputation** | [`contracts/reputation`](contracts/reputation) | Tracks a reputation score for reporters and validators; rewards validated reports, penalizes false ones. |
| **Governance** | [`contracts/governance`](contracts/governance) | Owner/admin/validator management, role-based permissions, system-wide pause switch, timelocked upgrade authorization. |
| `common` | [`contracts/common`](contracts/common) | Shared types (`RiskLevel`, `ReportStatus`, `Role`) and typed cross-contract clients. Not itself deployed. |

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for how the three contracts
call into each other and why.

## Quick start

```bash
# One-time: Rust wasm target, rustfmt/clippy, and the Stellar CLI.
scripts/setup.sh

# Build all three contracts to release Wasm (target/contracts/*.wasm).
scripts/build.sh

# Run the full test suite.
cargo test --workspace --features testutils

# Deploy to testnet and wire the contracts together.
scripts/deploy.sh
```

See [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) for the local dev loop and
[`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) for deploying to testnet/mainnet.

## Status

This is an early-stage, open-source foundation -- functional and tested, but
deliberately not feature-complete. It covers the core report/validate/pause
flow end-to-end; things like a `DomainRecord`/`TransactionRecord` aggregate
(mirroring `AccountRecord`), dispute resolution, multi-signature validation
thresholds, and a reputation leaderboard are intentionally left for the
community. Contributions welcome -- see [`CONTRIBUTING.md`](CONTRIBUTING.md).

**Not yet audited. Do not use in production with real funds without an
independent security review.**

## Documentation

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) -- how the contracts fit together
- [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) -- local build/test loop
- [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) -- deploying to testnet/mainnet
- [`CONTRIBUTING.md`](CONTRIBUTING.md) -- how to contribute
- [`SECURITY.md`](SECURITY.md) -- reporting vulnerabilities

## License

[MIT](LICENSE)
