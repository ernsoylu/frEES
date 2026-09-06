# Dependency map — Java/native → Rust/wasm

Every dependency of `../frEES`, the exact version in use, its license, and what replaces it in the browser build. Versions read from `backend/*/build.gradle`, `frontend/package.json`, and `THIRD-PARTY-NOTICES.md` on 2026-07-29.

Legend: **✅ direct swap** · **🔨 port frees' own code** · **⚠️ needs a spike** · **❌ deleted**

---

## 1. `backend/core` — the port target

| Java dependency | Version | License | Used for | Rust replacement | |
|---|---|---|---|---|---|
| `org.antlr:antlr4` + `antlr4-runtime` | 4.13.2 | BSD-3 | `Frees.g4` → lexer/parser/visitor | Hand-written lexer + recursive-descent parser (~55 rules). `chumsky` / `pest` / `lalrpop` are viable but a hand-rolled parser gives better diagnostics, which frees depends on. | 🔨 |
| `org.jgrapht:jgrapht-core` | 1.5.3 | EPL-2.0 / **LGPL-2.1** | Tarjan SCC in `Blocker` | `petgraph` — `algo::tarjan_scc`. MIT/Apache-2.0, which also **removes the LGPL static-linking problem**. | ✅ |
| `org.apache.commons:commons-math3` | 3.6.1 | Apache-2.0 | Jacobians, Newton–Raphson, SVD, Brent, eigen/Schur (LQR Riccati), Levenberg–Marquardt | `nalgebra` (LU, QR, **SVD**, **Schur**, symmetric eigen) and/or `faer` for larger dense work; `levenberg-marquardt` (nalgebra-based) or `argmin` for LM/Brent. | ✅ |
| `net.java.dev.jna:jna` | 5.19.1 | Apache-2.0 / LGPL-2.1 | Loading `libCoolProp.so`, SUNDIALS | Gone, and nothing replaced it: CoolProp is a **pure-Rust dependency** (see Native libraries below) and there is no FFI in the property path at all. | ❌ |
| `org.matheclipse:matheclipse-core` (Symja) | 3.2.0 | **LGPL-3.0** | 13 CAS ops via a string bridge | **Own CAS** over exact rationals (`num-rational` + `num-bigint`). See §4. `symbolica` rejected — source-available, commercial license, incompatible with MIT. | 🔨⚠️ |
| `com.fasterxml.jackson.core:jackson-databind` | 2.22.1 | Apache-2.0 | JSON DTOs (`api/SolveDtos.java`) | `serde` + `serde_json`, `serde-wasm-bindgen` at the boundary. | ✅ |
| `de.richardliebscher.mdf4j:mdf4j` | 0.2.0 | Apache-2.0 | ASAM MDF4 reading | Candidates: `mf4-rs`, `mdf4-rs`, `asammdf`, `mdfr`. **Must read from a byte slice** — several use `memmap2`, which does not exist in wasm. Spike required. | ⚠️ |
| `org.apache.xmlgraphics:fop-transcoder` | 2.11 | Apache-2.0 | SVG transcoding for export | Deleted — the browser renders and exports SVG natively (`plots/exportPlot.ts` already exists). | ❌ |
| `org.slf4j:slf4j-api` | 2.0.18 | MIT | Logging | `tracing` (native) + `tracing-wasm` / `console_log` (browser). | ✅ |
| `org.junit.jupiter:junit-jupiter` | 6.1.2 | EPL-2.0 | 1,237 tests | Rust `#[test]` + the golden-fixture replay harness (`PLAN.md` §4). | 🔨 |

### Native libraries

| Component | Version / origin | License | Replacement | |
|---|---|---|---|---|
| **CoolProp** | vendored `backend/core/native/libCoolProp.so`; upstream now **8.0.0** | MIT | **Resolved 2026-08-17 — [rustprop](https://github.com/ernsoylu/RustProp), a pure-Rust port of CoolProp 8.0.0, linked as an ordinary Cargo dependency** (decision [D9](decisions/0009-rustprop-backend.md)). No Emscripten module, no JS boundary, no lazy chunk. The four-function binding surface (`PropsSI`, `Props1SI`, `HAPropsSI`, `get_global_param_string`) survives as the `RealFluid` trait in `props/propfun.rs`. D1-C's precomputed tables were the interim path and are now a native-only fallback, out of the browser bundle. | ✅ |
| **SUNDIALS** (IDA + KLU) | v6, Ubuntu 24.04 `libsundials-dev` | BSD-3 | Either Emscripten-build IDA, or `diffsol` (pure-Rust BDF/SDIRK with mass matrices for semi-explicit DAEs; works over `nalgebra`/`faer`). A wasm build **eliminates the v6/v7 MPI ABI trap** documented in `../frEES/CLAUDE.md` — there is no distro to drift. | ⚠️ |

MIT and BSD-3 both permit static linking into the wasm binary with attribution. Preserve a `THIRD-PARTY-NOTICES.md` in this repo.

---

## 2. `backend/web` — deleted, contract retained

| Dependency | Version | Fate |
|---|---|---|
| `org.springframework.boot:spring-boot-starter-web` | 4.1.0 | ❌ No HTTP server in the browser |
| `spring-boot-starter-amqp` (RabbitMQ) | 4.1.0 | ❌ No broker — solve is a direct call |
| `spring-boot-starter-data-redis` | 4.1.0 | ❌ No job store — no jobs |
| `spring-boot-starter-opentelemetry` | 4.1.0 | ❌ Replaced by browser performance marks if needed |
| `org.springdoc:springdoc-openapi-starter-webmvc-api` | 3.0.3 | ❌ The TypeScript RPC types are the contract |
| `netty` (pinned override) | 4.2.16.Final | ❌ Transitive only |
| `testcontainers-*`, `spring-boot-webmvc-test` | — | ❌ Integration tests target a broker that no longer exists |

The **REST contract survives** as the worker RPC surface — 22 methods, listed in `feature-inventory.md` §8.

---

## 3. New Rust/wasm dependencies

| Concern | Crate | Notes |
|---|---|---|
| Wasm boundary | `wasm-bindgen`, `js-sys`, `web-sys`, `serde-wasm-bindgen` | Only in `crates/frees`, never in `frees-core` |
| Build | `wasm-pack` + `vite-plugin-wasm` (or `trunk`) | Feeds the existing Vite 6 build |
| Panics | `console_error_panic_hook` | Turns wasm traps into readable console errors |
| Graph | `petgraph` | Tarjan SCC |
| Linear algebra | `nalgebra`, optionally `faer` | SVD, Schur, LU, QR |
| Optimization | `argmin`, `levenberg-marquardt` | Brent, LM, line search |
| ODE/DAE | port frees' own; `diffsol` for the DAE path | See `PLAN.md` §5 Phase 7 note |
| Exact arithmetic | `num-rational`, `num-bigint`, `num-complex` | CAS + the existing complex `_r`/`_i` support |
| Transcendentals | `libm` | Used uniformly **in `frees-core`** so native and wasm builds agree bit-for-bit with each other — see the caveat below, which has held since D9 |
| RNG | `rand` (+ `getrandom` `js` feature) | Monte Carlo, NSGA-II |
| Parallelism | worker pool (no crate) or `wasm-bindgen-rayon` | Deferred by decision **D3** |
| Serialization | `serde`, `serde_json` | DTO parity with Jackson |
| Logging | `tracing`, `tracing-wasm` | |
| MDF4 | one of `mf4-rs` / `mdf4-rs` / `asammdf` / `mdfr` | Spike — slice-based reading required |

> **The bit-for-bit claim above is `frees-core`'s alone, and rustprop is
> outside it (measured 2026-08-27).** `frees-core` really does route every
> transcendental through the `libm` crate — `eval.rs` alone has ~40 `libm::`
> call sites and no `f64::exp()` — so its own arithmetic is identical on both
> targets. **rustprop is not built that way**: it calls `std`'s methods (158
> `.exp()`, 151 `.ln()`, 459 `.powf()`, zero `libm::`), which resolve to
> **glibc natively and Rust's bundled `libm` on `wasm32-unknown-unknown`**.
> Those two disagree by one ulp on about 11 % of `exp` arguments and 4.5 % of
> `ln` arguments, sampled over 600 random values each.
>
> Since [D9](decisions/0009-rustprop-backend.md) made rustprop the browser's
> only property backend, that is a real native-vs-wasm divergence in every
> real-fluid document. Measured by replaying all 1308 corpus documents through
> the shipped `web/src/wasm/pkg` module under Node and diffing against
> `frees-cli`: **107 of the 1095 documents both builds solve return different
> numbers**, worst 5.4e-6 on a real quantity (two subcooling variables flip
> around a structural zero, which is what the absolute channel exists for).
> `LD_PRELOAD`ing a ~20-line shim that routes `exp`/`log`/`pow`/trig to the
> `libm` crate closes **91 of the 107** and reproduces the wasm trajectories
> bit-for-bit, which is what pins the mechanism to rustprop rather than to
> codegen, `wasm-opt`, or feature flags.
>
> It is invisible to CI because **no job solves a corpus document through the
> wasm module** — `ci.yml` has `native`, `wasm` (build + size gate only),
> `web` (build + vitest) and `parity` (four *native* shards). A full wasm
> replay under Node takes ~222 s, so a gate is affordable; closing the
> divergence itself means pointing rustprop at `libm`, which moves the native
> baseline too and therefore needs a full replay and a look at
> `fixtures/tolerances-rustprop.json`. Neither is done.

---

## 4. Symja replacement, in detail

The entire Symja dependency is `ExprEvaluator` — string in, string out — reachable from four files (`CasEngine`, `ExprToSymja`, `CasIdentity`, `StateSpace`). The complete operation set:

| Symja op | Reached from | Rust plan |
|---|---|---|
| `Factor` | `CasEngine.factor`, REPL | Polynomial factorization over ℚ (square-free → Zassenhaus/Hensel) |
| `Expand` | `CasEngine.expand`, REPL | Distribute + normalize |
| `Simplify` | `CasEngine.simplify`, REPL | Rational normalization + like-term collection |
| `Together` | REPL | Common denominator |
| `Cancel` | REPL | GCD-reduce numerator/denominator |
| `Numerator` / `Denominator` | REPL | Structural |
| `Apart` | `CasEngine.apart`, REPL | Partial fractions — the Laplace residue workflow |
| `Collect` | REPL | Group by powers of a variable |
| `D` | REPL | **Route to the ported `ast/Differentiator` (536 LOC) — already native frees code** |
| `LaplaceTransform` | `CasEngine.laplace` | Transform table + linearity; the textbook surface |
| `InverseLaplaceTransform` | `CasEngine.inverseLaplace` | `Apart` + inverse table |
| `Integrate` | REPL | **Hardest.** Pattern-matched table for v1; record the gap. Optional remote fallback. |

Everything else people assume is "the CAS" — transfer-function algebra, symbolic `ss↔tf`, LQR Riccati, PID tuning, time responses, polynomial helpers — is **already frees' own Java** (`cas/`, ~3.3k LOC excluding the Symja bridge) and ports directly with no CAS dependency at all.

---

## 5. Frontend — unchanged

Every frontend dependency is permissive and browser-native; none needs replacing. Retained at current versions: React 19.2, Mantine 9.5 (+ hooks/notifications/spotlight), CodeMirror 6 (`@uiw/react-codemirror` 4.25), Plotly.js 3.7, uPlot 1.6, KaTeX 0.18, Excalidraw 0.18, Univer 0.25, dockview 7.0, glide-data-grid 6.0, papaparse 5.5, lz-string 1.5, marked 18, rxjs 7.8, Vite 6.4, TypeScript 6.0, Vitest 4.1.

**Only two files change:** `src/api.ts` (18 functions) and `src/analyzer/measurementApi.ts` (4 functions) — `fetch` becomes a worker RPC call with identical promise signatures.

New frontend additions: the generated `wasm-pack` package, a worker entry point, and (Phase 11) a service worker + PWA manifest.

---

## 6. Deleted infrastructure

Docker (both Dockerfiles, three compose files), `frees.sh`, `install.sh`, nginx config + entrypoint scripts, GHCR image publishing, the Railway deployment, RabbitMQ, Redis, OpenTelemetry + Jaeger, the Python `mdf-sidecar` (FastAPI + asammdf, LGPL-3.0), and the `RequestGuardFilter` rate-limiting/client-IP-trust machinery.

The four deployment foot-guns catalogued in `../frEES/CLAUDE.md` — pinned SUNDIALS base image, nginx upstream re-resolution, CORS defaults, and `X-Forwarded-For` trust — **all cease to exist**. They are properties of a server topology this build does not have.
