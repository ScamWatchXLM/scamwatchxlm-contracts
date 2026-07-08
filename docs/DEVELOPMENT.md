# Development Guide

## Prerequisites

- Rust, installed via [rustup](https://rustup.rs). The toolchain and the
  `wasm32v1-none` target are pinned in [`rust-toolchain.toml`](../rust-toolchain.toml)
  and installed automatically by rustup when you run any `cargo`/`rustc`
  command in this repo.
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli/stellar-cli)
  (`stellar`), for building the exact way the network expects and for
  deploying. Install both prerequisites with:

  ```bash
  scripts/setup.sh
  ```

## Repository layout

```
contracts/
  common/       Shared types + cross-contract client interfaces (not a deployed contract)
  governance/   Owner/admin/validator management, pause, upgrade authorization
  registry/     Scam reports and aggregate risk records
  reputation/   Reputation scores for reporters and validators
scripts/        setup.sh / build.sh / deploy.sh
docs/           This file, ARCHITECTURE.md, DEPLOYMENT.md
.github/        CI workflow
```

Each contract crate follows the same internal shape:

```
src/
  lib.rs        #[contract] struct + #[contractimpl] -- the public interface
  types.rs      #[contracttype] structs/enums + storage DataKey enum
  errors.rs     #[contracterror] enum
  events.rs     #[contractevent] structs
  storage.rs    env.storage() wrappers (not present in governance; small enough to inline)
  test.rs       #[cfg(test)] unit tests
```

## Building

```bash
scripts/build.sh
```

This runs `stellar contract build` per contract (falling back to
`cargo build --release --target wasm32v1-none` if `stellar-cli` isn't
installed) and copies the resulting `.wasm` files to `target/contracts/`.

Do not build with plain `cargo build --target wasm32-unknown-unknown` --
Soroban contracts target `wasm32v1-none` specifically; see the note at the
top of `rust-toolchain.toml` and the
[soroban-sdk README](https://github.com/stellar/rs-soroban-sdk#build-target)
for why.

## Testing

```bash
cargo test --workspace --features testutils
```

Each contract's tests deploy the real contracts it depends on (e.g.
Registry's tests deploy a real Governance contract and a real Reputation
contract, not hand-written mocks) using `soroban_sdk::Env::register` and
`soroban_sdk::testutils::Address::generate`. This is why `governance` and
`reputation` are `[dev-dependencies]` of `registry`'s `Cargo.toml`, and
`governance` is a dev-dependency of `reputation`'s: it lets tests exercise
real permission checks, real pause behavior, and the real invoker-contract
authorization on the Registry -> Reputation callback, instead of drifting
from what's actually deployed.

## Linting and formatting

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --features testutils -- -D warnings
```

Both run in CI (see `.github/workflows/ci.yml`) and are expected to be clean
before merging.

## Adding a new contract function

1. Add the function to the relevant `#[contractimpl]` block in `lib.rs`.
   Keep it thin: input validation and permission checks at the top, then
   delegate to `storage.rs` helpers for actual reads/writes.
2. If it changes state meaningfully, add a `#[contractevent]` in
   `events.rs` and publish it.
3. Add a new `#[contracterror]` variant only if no existing one fits --
   prefer reusing `NotAuthorized`/`InvalidInput`/etc. over adding
   near-duplicates.
4. Write a test in `test.rs` covering the success path, and at least one
   failure path (unauthorized caller, invalid input, or duplicate/
   conflicting state, whichever applies).
5. Run `cargo fmt`, `cargo clippy`, and `cargo test` before opening a PR.

## A note on `#[contracttype]` naming

Struct/enum names, field names, and enum variant names used in
`#[contracttype]`/`#[contractevent]` are encoded as `Symbol`s and capped at
**30 characters** by the current SDK (see the compile error macro emits if
you exceed it -- it's enforced, not just a style guideline). Ordinary
descriptive Rust names are almost never close to this limit; it's called out
here because older SDK docs mention a stricter (and outdated) 9-10 character
limit that no longer applies to `soroban-sdk` 27.
