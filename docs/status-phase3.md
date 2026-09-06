# Status — Phase 3 complete: the browser vertical slice, proven offline

**Date:** 2026-07-30 · Everything below was run and verified on this date:
644 Rust tests green, `clippy -D warnings` clean, `cargo fmt` clean,
`npm run build` green, and a real browser drove Editor → Check → Solve →
Solution against `web/dist/` served by a dumb static file server — with
**zero `/api/` requests**.

## What the vertical slice does

`web/` is the frEES React frontend (vendored snapshot, see
`web/WASM-PORT.md`) with the compute loop collapsed into the tab:

```
editor text ──> api.ts (same exported signatures as the REST client)
                  └─> engineClient.ts  (lazy singleton, id-correlated RPC)
                        └─> engine.worker.ts    (module Web Worker)
                              └─> frees_bg.wasm  (Rust engine)
```

- `solve()` and `check()` in `web/src/api.ts` keep their former signatures
  and wire shapes (`SolveResponse` / `CheckResponse`) but call
  `wasmSolve`/`wasmCheck` instead of `fetch`. Syntax errors, causality
  diagnoses and solver failures all arrive as data in the same envelope the
  Java backend produced — the UI needed no rewrite.
- The engine runs **off the UI thread** in a module worker
  (`web/src/wasm/engine.worker.ts`); wasm instantiation starts at worker
  spawn so it overlaps the first request. A dead worker rejects everything
  in flight and is respawned on the next call (`engineClient.ts`).
- `web/src/wasm/pkg/` is generated output (gitignored): rebuild with
  `wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg`.
- The default document (`web/src/defaultExample.ts`) is a browser-solvable
  walkthrough: unit-annotated inputs, a sequential chain, and the canonical
  nonlinear pair `x^2 + y^3 = 77`, `x / y = 1.23456`.

## What is stubbed (per function, from `web/src/api.ts`)

Everything below is explicitly stubbed so **nothing in `api.ts` touches the
network**. Two flavors, chosen per call site: a *neutral resolved value*
where the UI consumes data at boot, otherwise a rejection / failed-DTO with
the message `"not yet available in the browser engine"`.

| Function | Former endpoint | Stub behavior |
|---|---|---|
| `replEvaluate` | `POST /api/repl/evaluate` | resolves a failed `ReplResponse` (terminal prints the error) |
| `replClear` | `POST /api/repl/clear` | no-op (fire-and-forget call site) |
| `optimize` | `POST /api/optimize` | resolves a failed `OptimizeResponse` (modal shows error inline) |
| `optimizeMulti` | `POST /api/optimize/multi` | resolves a failed `ParetoResponse` |
| `curveFit` | `POST /api/curve-fit` | resolves a failed `CurveFitResponse` |
| `parameterFit` | `POST /api/measurements/parameter-fit` | resolves a failed `ParameterFitResult` |
| `getFluids` | `GET /api/fluids` | resolves `[]` (the old "backend has no CoolProp" state) |
| `getReference` | `GET /api/reference` | resolves empty `{units, constants}` (Help renders empty tables) |
| `getPropertyDiagram` | `GET /api/plot/propplot` | rejects (PlotCard error state) |
| `getPsychrometricChart` | `GET /api/plot/psychart` | rejects (PlotCard error state) |
| `exportVector` | `POST` FOP transcoder | rejects (client-side SVG/PNG export still works) |
| `solveTable` | `POST /api/solve/table` | rejects (App writes the message into every row) |
| `runMonteCarlo` | `POST /api/solve/montecarlo` | rejects (modal error state) |
| `pidTune` | `POST /api/control/pidtune` | rejects (modal error state) |
| `extractPlant` | `POST /api/control/plant` | rejects (App degrades to "enter the plant manually") |

Kept exported but unwired: `runCompute` / `pollJob` (the 202+poll/SSE
machinery, with its tests) for a future hybrid remote path.
Untouched: `web/src/analyzer/measurementApi.ts` (Data Analyzer is Phase 10;
its fetch failures surface as user-visible upload errors).

## Bundle sizes (measured 2026-07-30)

| Artifact | Raw | Gzipped |
|---|---|---|
| `frees_bg.wasm` | 544,637 B (532 KiB) | 228,969 B (224 KiB) |
| Boot path (the 22 assets a cold load actually fetches, wasm included) | 4.17 MB | 1.15 MB |
| `web/dist/` total (342 files) | 29.1 MB | ~8.1 MB (js+css+wasm+html) |

The wasm grew from 397 KiB (Phase 1–2 note) to 532 KiB with the boundary +
unit checker work — still ~26 % of the 2 MiB CI budget. The dist total is
dominated by lazily-imported chunks that never load at boot (proven by the
network log below): `plotly.min` 4.8 MB, the spreadsheet/CSV chunk 5.7 MB,
mermaid diagram chunks, and KaTeX fonts. `engine.worker.js` itself is 4 KiB.

## The offline proof (ran, not imagined)

Procedure, executed locally on 2026-07-30:

1. `wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg`
2. `cd web && npm run build` (exit 0)
3. `python3 -m http.server` serving **`web/dist/` only** — a dumb static
   server with no `/api/` routes, no proxy, nothing to answer a fetch.
4. A real Chromium via the Playwright MCP drove the app:
   - Boot: title "frees — Equation Solver", default document in the editor,
     welcome dialog shown and dismissed.
   - **Check** → status pill "Check OK".
   - **Solve** → status pill "Solved", stats line
     `12 eqns · 11 blocks · 22 iters · max residual 6.82e-13`, and the
     Variable Explorer table showing **`x = 4.6940124`**, **`y = 3.8021744`**
     (oracle: `4.694012391660914` / `3.802174371161316`) plus the
     unit-carrying chain values (`m/s` rows etc.).
5. Full network log (static resources included): **24 requests, all
   `GET http://127.0.0.1:<port>/...` assets; zero `/api/` requests; zero
   requests of any kind after boot** — Check and Solve produced no network
   traffic at all. The engine loads were `assets/engine.worker-*.js` and
   `assets/frees_bg-*.wasm`, both static files.
6. Console: two benign 404s only — `/build-info.js` (written at container
   start by the nginx entrypoint, absent from a bare `dist/`) and
   `/favicon.ico`. No app or engine errors.

Conclusion: the solve happened in the Web Worker's wasm instance. The
server's only job was handing out static files, i.e. the app is
offline-capable once cached.

## CI

`.github/workflows/ci.yml` gained a `web` job alongside the untouched
`native`/`wasm`/`parity` jobs: checkout → Rust stable + `wasm32-unknown-unknown`
+ `rust-cache` + wasm-pack v0.13.1 (mirroring the `wasm` job) → build the
pkg into `web/src/wasm/pkg` → `setup-node@v4` (**node 22 via
`node-version-file: web/.nvmrc`**, npm cache keyed on `web/package-lock.json`)
→ `npm ci` → **`npx vitest run` (33 files / 324 tests)** → `npm run build` →
assert a `.wasm` asset landed in `dist/`. `node_modules` itself is deliberately
not cached: `npm ci` deletes it by design, so the supported `~/.npm` cache is
what actually helps.

Node 22 is required for the test step, not the build: jsdom@30 pulls undici@8,
whose `CacheStorage` calls `webidl.util.markAsUncloneable` (absent before
Node 22), so on Node 20 every test file dies at environment setup
(`web/WASM-PORT.md` has the full story).

**Caveat:** `web/package-lock.json` exists (and `npm ci` used it locally),
but the whole `web/` tree is currently **untracked** in git — this session
leaves changes unstaged per instructions. The `web` CI job can only pass
once `web/` (lockfile included) is committed. `web/src/wasm/pkg/` stays
gitignored; CI regenerates it.

## Gap-closure pass (2026-07-30)

The five "honest gaps" reported at the end of the Phase-3 workflow were fixed
and re-verified (749 Rust tests, 324 web tests, fresh Playwright offline proof):

1. **Partial failure diagnostics** — `engine::solve` now returns
   `Result<Solution, SolveFailure>` mirroring the Java
   `SolverException.FailureState` + `partialResult`: structured
   `failed_block_index` (the fragile `"Block N…"` message-parsing in the
   boundary is gone) and `PartialDiagnostics` with all blocks, every equation's
   residual at the stalled iterate (NaN where unevaluable), and partial stats.
   The wasm failure envelope now ships the Java 422 shape — blocks, finite
   residuals, populated stats, `failedBlockIndex` — so SolveDiagnostics can
   render a failed solve.
2. **`displayUnitSystem`** — `SolverApiSupport.toDisplay`/`convertToDisplayUnit`
   ported into the boundary: explicit Variable-Information units win in every
   system (140 kPa displays as `140 kPa`), else the system's preferred display
   unit (ENG_SI → kPa/kJ/kW, ENGLISH → psi/Btu/hp/…), else the recorded SI
   unit. Note the verified Java subtlety: a bare `P = 140 [kPa]` document in
   the SI system displays `140000 Pa` — the literal's declared spelling is
   collapsed to the SI name at parse time (`siDisplayName`), and only
   variableInfo units resurrect `kPa`.
3. **`#` constants fold at parse time** (`AstBuilder.visitVarAtom` parity):
   `pi#`/`g#`/… become literals carrying their raw SI unit string, so the unit
   checker grounds through them — the boot document's `mdot`/`KE_flux` now
   display kg/s and W instead of `-` (verified in the browser).
4. **Web tests run** — Node 22 (`web/.nvmrc`, CI `node-version-file`); all 33
   files / 324 tests pass, now gated in CI. Root cause documented in
   `web/WASM-PORT.md`.
5. **Leftovers** — `EXAMPLES[0]` rebound to the preserved EV component model
   (honest "not yet ported" note in its description) instead of silently
   showing the boot doc under the EV title; the About dialog now shows the
   engine version from the worker handshake (`frees-core x.y.z (wasm)`).

## Next milestones

1. **Corpus growth** — harvest `web/src/examples.ts` documents into
   `fixtures/corpus/`, regenerate golden fixtures from the Java oracle, and
   fix what diverges (the missing solve-retry ladder will surface here
   first; see the ranked divergence list in `status-phase1.md`).
2. **Phase 4** — `Differentiator` (unblocks the symbolic-Jacobian path,
   the top-ranked divergence), arrays & matrix intrinsics,
   `ComplexExpansion`, procedural bodies.
3. **Unstub in dependency order** — REPL + `getReference` need only
   engine-side exports of existing state/tables; `solveTable`/Monte Carlo
   are loops over the existing solver; optimize/curve-fit need `argmin`-class
   machinery; fluids/property plots wait on the CoolProp-to-wasm decision
   (`docs/dependency-map.md`).
