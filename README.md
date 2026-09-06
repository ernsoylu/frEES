# frees

A Rust/WebAssembly port of the **frees** engine — a declarative equation solver
and acausal system-modeling environment — that runs **entirely in the browser
tab**. No backend, no API calls, no job queue.

Upstream, a solve is a network round-trip: editor text → `POST /api/solve` →
RabbitMQ → compute worker → Redis → the frontend polls for a job id. This repo
collapses that loop into WebAssembly. The parser, unit checker, blocker, Newton
solver, ODE/DAE integrators, CAS, component expander and real-fluid property
backend are all compiled to `wasm32-unknown-unknown` and run in a Web Worker.

```frees
m_dot = 2.5 [kg/s]
h_in = Enthalpy(Water, T = 300 [K], P = 101325 [Pa])
h_out = Enthalpy(Water, T = 360 [K], P = 101325 [Pa])
Q_dot = m_dot * (h_out - h_in)
```

Equations are order-independent, names are case-insensitive, everything is
solved in SI, and unit annotations convert at parse time.

---

## Status

All 13 planned phases are implemented; work since then continues as numbered
decisions (D1–D11) and lettered waves. The engine is feature-complete against
the reference implementation for everything the port set out to carry.

| | |
|---|---|
| Parity corpus | **1308 documents**, all matching the Java oracle |
| Component library | **295 components** across 13 physical domains |
| wasm bundle | ~3085 KiB raw / ~1259 KiB gzipped (budget 4096 KiB, gated in CI) |
| Property backend | rustprop — a pure-Rust port of CoolProp 8.0.0 |
| Frontend tests | 34 files, 429 tests |
| CI | ~3.5 min end to end |

Numbers move; the ones CI enforces are the bundle budget and the corpus. For a
re-measured snapshot see the gate-numbers table at the top of
[`CLAUDE.md`](CLAUDE.md).

## Quick start

The toolchain is `rustup`-managed and pinned by
[`rust-toolchain.toml`](rust-toolchain.toml) (stable, plus the
`wasm32-unknown-unknown` target, `rustfmt` and `clippy`).

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# Solve a document headlessly
printf 'x = 2\ny = x^2\n' | cargo run -qp frees-cli -- solve
cargo run -qp frees-cli -- check path/to/document.frees

# The whole test suite, including the 1308-document parity replay
cargo test --release --workspace
```

For the browser app (**Node 22 is required** — under Node 20 the whole vitest
suite dies in `jsdom`→`undici` before running a test; `web/.nvmrc` pins it):

```bash
wasm-pack build crates/frees --release --target web \
  --out-dir ../../web/src/wasm/pkg
cd web && npm ci && npm run dev
```

## Layout

| Path | What it is |
|---|---|
| `crates/frees-core` | The engine. Target-agnostic, and **must never depend on wasm-bindgen** |
| `crates/frees` | The wasm-bindgen boundary — JSON string in, JSON string out |
| `crates/frees-cli` | Headless `solve`/`check`, used by the parity harness |
| `fixtures/` | The parity corpus, its goldens, and the tolerance files |
| `tools/` | Oracle generators that run against the reference Java + native CoolProp |
| `web/` | The React frontend, with the engine wired in place of `fetch` |
| `docs/` | Status documents, the divergence ledger, and the decision records |

## How correctness is established

This is the part worth understanding before changing anything, because it is
what the repo is actually built around.

The reference implementation (a separate, **read-only** Java repository sitting
beside this one) is the oracle. `tools/golden-dumper` runs documents through it
and records the answers as golden fixtures. `cargo test --workspace --test
parity` replays all 1308 of them through this engine and compares variables,
display names, block counts, ODE trajectories and error classifications.

Three properties keep that gate honest:

- **The default tolerance is `1e-9`**, and every fixture that needs more must
  say so in `fixtures/tolerances-rustprop.json` with a *measured* error and a
  named mechanism. Currently 65 fixtures carry a relative entry and 10 carry an
  absolute one.
- **Dead tolerances fail the build.** An entry whose fixture now passes at the
  default fails, as does a catalogued mechanism no entry cites. A relaxation
  cannot quietly outlive its cause.
- **The replay is sharded four ways in CI** and each shard prints a census whose
  `replayed` counts must sum to the corpus, so a partition that silently
  under-replays cannot report green.

[`fixtures/README.md`](fixtures/README.md) is the authority on all of it —
promotion rules, the pending set, the decayed-signal measure for ODE rows, and
the absolute channel for quantities whose true value is exactly zero.

## Properties

Real-fluid and humid-air properties come from
[rustprop](https://github.com/ernsoylu/RustProp), a pure-Rust port of CoolProp
8.0.0 consumed as a pinned git dependency. **CoolProp itself is not linked into
anything here** — there is no C FFI in the workspace and no native library in
the bundle. The name survives in three legitimate places: rustprop is a port
*of* it, fluid and parameter names follow its conventions (`INCOMP::MEG[0.50]`,
`Dmass`), and it remains the oracle every accuracy claim is graded against.

Decision [D9](docs/decisions/0009-rustprop-backend.md) is the authority; read it
before writing anything that touches `props/`.

## Contributing

Four gates, all of which CI runs:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo test --workspace                    # the corpus replay is sharded separately in CI
```

Two conventions that are easy to violate by accident:

- **Fixtures are frozen.** A `.frees` file in `fixtures/corpus/` is an oracle
  input whose golden was produced from exactly those bytes. Don't edit one —
  even a comment — without re-dumping its golden.
- **Unsupported constructs must fail loudly**, never be silently skipped, and
  diagnostics quote the user's own source text.

## Where to read next

| | |
|---|---|
| [`CLAUDE.md`](CLAUDE.md) | The working brief: current gate numbers, every decision in force, and the traps. Read this first |
| [`PLAN.md`](PLAN.md) | The original 13-phase plan, the dependency substitutions, and the risk register |
| [`fixtures/README.md`](fixtures/README.md) | The parity harness in full |
| [`docs/decisions/`](docs/decisions/) | D1–D11 — why the property backend, the threading model, and the removed features are what they are |
| [`docs/status-phase1.md`](docs/status-phase1.md) | The maintained divergence ledger: every known difference from the Java, open or closed |

## License

MIT — see [`LICENSE`](LICENSE). The MIT constraint is load-bearing: it is why
the CAS was written from scratch rather than binding Symja (LGPL-3.0), and why
Symbolica was ruled out.
