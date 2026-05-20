# CLAUDE.md — big-o-python-to-rust agent instructions

## CRITICAL: Contract-first

**NEVER write code without a corresponding YAML contract.** Every `pub` item in this workspace must be bound to at least one contract in `contracts/binding.yaml`. `pmat comply check` enforces coverage; the build fails when an unbound public item exists.

## Three modes of proof

| Mode | Where | Tool |
|---|---|---|
| Empirical | `m2-empirical`, `m5-capstone` | `criterion` |
| Structural | `m3-structural` | hand-written + `proptest` |
| Formal | `lean/BigOFromZero/` | Lean 4 |

When a contract has `lean_theorem: <name>`, the corresponding Lean proof must be `status: proved`. When it's `null` (e.g. `complexity-preserved-across-transpile-v1`), empirical-only is acceptable.

## Coverage gate

`cargo llvm-cov --fail-under-lines 100`. There is no "good enough" — 100% line coverage on the workspace is required for CI to pass. `m*/src/main.rs` and `benches/` are excluded; everything else must hit 100%.

## Contract authoring rules

1. New `pub` function → new or extended contract YAML in `contracts/`.
2. Add binding in `contracts/binding.yaml` under the relevant contract.
3. Run `make validate score lint` until clean.
4. Then implement; tests in `m*/src/lib.rs` or `m*/tests/` reference the contract by name in a doc comment.
5. `pmat comply refresh-bindings` regenerates `.pmat/binding-index.json`.

## Module conventions

- Each `m<N>-<theme>` crate has `src/lib.rs` (public API, fully covered), optionally `src/main.rs` (demo binary, excluded from coverage), and `benches/` for criterion.
- Demo binaries print a `contract: <contract-name> holds — OK` line on stderr at exit when their bound contracts pass at runtime.
- Tests in `tests/` may reference contracts from any module in the workspace.

## Toolchain pins

- `pv` (aprender-contracts-cli) 0.32+
- `pmat` latest
- Rust 1.75+
- `criterion` 0.5 with HTML reports
- `proptest` 1.x
- `cargo-llvm-cov` for coverage

## When in doubt

The reference companion repo is `/home/noah/src/wasm-from-zero` (Cargo workspace shape, Makefile, contract YAML format, Lean wiring). Pattern-match against it before reinventing.
