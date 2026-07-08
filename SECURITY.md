# Security Policy

## Status

These contracts have **not** undergone an independent security audit. Treat
them as an early-stage foundation suitable for testnet experimentation and
review, not as production-ready code for handling real funds or
reputation-critical decisions until an audit has been completed.

## Supported versions

This project is pre-1.0. Security fixes are made against the `main` branch
only; there are no maintained release branches yet.

## Reporting a vulnerability

**Please do not open a public GitHub issue for a security vulnerability.**

Instead, use GitHub's private vulnerability reporting for this repository
(the "Report a vulnerability" button under the repo's **Security** tab,
backed by a private security advisory). This lets maintainers triage and fix
the issue before it's publicly disclosed.

When reporting, please include:

- The contract(s) and function(s) affected
- A description of the impact (funds at risk, permission bypass, denial of
  service, incorrect state, etc.)
- Steps to reproduce, ideally as a failing test against this repo
- Your suggested severity, if you have one

## Scope

In scope:

- `contracts/governance`, `contracts/registry`, `contracts/reputation`,
  `contracts/common`
- The deployment tooling in `scripts/`

Out of scope:

- The Soroban SDK / Stellar network itself (report to
  [stellar/rs-soroban-sdk](https://github.com/stellar/rs-soroban-sdk) or
  [stellar/stellar-core](https://github.com/stellar/stellar-core))
- Any off-chain indexer, frontend, or API that consumes these contracts'
  events -- those live in other repositories

## Disclosure process

1. You report privately as above.
2. A maintainer confirms receipt and begins triage.
3. A fix is developed and tested privately.
4. Once a fix is deployed (or a coordinated disclosure date is agreed), a
   public GitHub Security Advisory is published crediting the reporter
   (unless you prefer to stay anonymous).
