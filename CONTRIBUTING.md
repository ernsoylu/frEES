# Contributing to frees

Thank you for your interest in contributing to `frees`! This guide outlines the development workflow, coding standards, and quality gates expected for all contributions.

---

## 1. Architectural Principles

Before submitting code, keep in mind the core architectural invariants of the project:

1. **Zero External Backend**: All compilation, equation solving, and visualization execute client-side via WebAssembly in the browser or natively via the CLI. Never introduce runtime network dependencies or remote API calls.
2. **Target-Agnostic Core**: `crates/frees-core` contains pure mathematics, algorithms, and models. It must **never** import `wasm-bindgen`, `web-sys`, or browser APIs.
3. **Root-Cause Engineering**: Fix bugs at their algorithmic origin in the compiler or solver, rather than adding conditional patches or bypasses in downstream callers.
4. **WASM Budget Gate**: The raw compiled WebAssembly engine binary (`frees.wasm`) must strictly remain under **4,096 KiB** (currently ~3,283 KiB).
5. **Deterministic Numerics**: The 1,308 golden regression test fixtures in `fixtures/corpus/` represent frozen reference behavior. Discrepancies must be investigated numerically; never loosen global tolerances without an evidence-backed reason in `fixtures/tolerances-rustprop.json`.

---

## 2. Prerequisites & Local Environment

- **Rust**: Stable toolchain (managed via `rustup` per `rust-toolchain.toml`) with the `wasm32-unknown-unknown` target.
- **Node.js**: Node 22 (pinned in `web/.nvmrc` and `web/package.json`).
- **wasm-pack**: Required for building the WASM engine bridge into `web/src/wasm/pkg`.

```bash
# Install wasm target and wasm-pack
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

---

## 3. Development Workflow

### Building the Project

```bash
# 1. Compile the WebAssembly engine
wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg

# 2. Install web dependencies & compile docs
cd web
npm ci
npm run compile-docs

# 3. Start local development server
npm run dev
```

### Running Verification Gates

Every pull request is validated against strict automated gates in CI:

```bash
# 1. Rust formatting & strict linting
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --target wasm32-unknown-unknown --all-targets -- -D warnings

# 2. Rust core tests
cargo test --workspace -- --skip golden_corpus_parity

# 3. Golden regression replay (1,308 fixtures)
cargo test --release --test parity

# 4. Frontend unit tests (Vitest under Node 22)
cd web && npm test

# 5. Frontend lint and production bundle build
npm run lint
npm run build
```

---

Parser/evaluator changes also run the [cargo-fuzz smoke targets](fuzz/README.md).
The [R15 pilot protocol](R15_PILOT.md) records the human validation required by Phase 2.5.

## 4. Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification to facilitate automated changelogs and semantic versioning via Release-Please:

- `feat(scope): ...` — A new feature
- `fix(scope): ...` — A bug fix
- `docs(scope): ...` — Documentation updates
- `perf(scope): ...` — Performance optimizations
- `refactor(scope): ...` — Code refactoring without behavior change
- `test(scope): ...` — Adding or fixing test suites
- `ci: ...` — CI/CD workflow updates

Example: `fix(solver): preserve user casing in display names during prepared solve`

---

## 5. Submitting a Pull Request

1. Fork the repository and create your branch from `main`.
2. Ensure all verification commands above pass locally.
3. Keep pull requests focused on a single change or fix.
4. If modifying solver behavior or property models, verify that `cargo test --release --test parity` succeeds without tolerance regressions.
