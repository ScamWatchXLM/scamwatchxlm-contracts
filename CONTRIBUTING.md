# Contributing to ScamWatchXLM Contracts

Thanks for considering a contribution. This project is intentionally left
~60-70% complete -- there's a lot of good, well-scoped work available. See
the README's "Status" section and the "future work" notes scattered through
the code (`grep -rn "community\|future work" contracts/`) for starting
points, or open an issue to propose your own.

## Before you start

For anything beyond a small fix, open an issue first describing what you
want to change and why. This avoids duplicated work and lets maintainers
flag design concerns (e.g. storage layout changes, new cross-contract calls)
before you've written the code.

## Development setup

```bash
scripts/setup.sh   # Rust wasm target + stellar-cli
cargo test --workspace --features testutils
```

See [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) for the full loop, including
the repo's conventions for how each contract crate is organized.

## Pull request checklist

Before opening a PR, make sure:

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets --features testutils -- -D warnings` passes
- [ ] `cargo test --workspace --features testutils` passes
- [ ] New public functions have a doc comment explaining *why*, not just
      what (the function signature already says what)
- [ ] New state-changing functions emit a `#[contractevent]`
- [ ] New behavior has a test covering both the success path and at least
      one failure path
- [ ] No new `#[contracterror]` variant duplicates an existing one in the
      same contract

CI runs all of the above (plus `cargo audit`) on every PR.

## Code style

- No comments explaining *what* code does when the code already says it
  (good names > comments). Doc comments on public items should explain
  non-obvious *why* (an invariant, a constraint from the Soroban host, a
  trade-off).
- Prefer extending an existing `#[contracterror]`/`#[contractevent]` variant
  set over adding a near-duplicate.
- Keep `lib.rs` as orchestration (auth checks, calling into `storage.rs`,
  emitting events); push storage read/write details into `storage.rs`.
- Enums over stringly-typed data wherever the set of values is fixed (see
  `RiskLevel`, `ReportStatus`, `Role`).

## Reporting a security issue

Please don't open a public issue for a security vulnerability -- see
[`SECURITY.md`](SECURITY.md).

## License

By contributing, you agree your contributions will be licensed under this
project's [MIT License](LICENSE).
