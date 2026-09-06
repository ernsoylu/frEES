# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository status

**All 13 planned phases (0–12) are implemented — the plan is finished, and
post-plan work continues as decisions D5–D10 plus lettered Waves.** This
paragraph describes Phases 0–7; the paragraphs below cover 8–12. The Phase-3
wasm boundary is wired, Phase 4
(differentiator, matrix, complex, procedural, tables, integrals, kernels, latex,
solver retry ladder) is complete, Phase 5 ports `props/` in full and gives the
browser a **working real-fluid property backend**, Phase 6 ports the acausal
**component / `connect` layer** together with the 295-component standard library
as embedded `.frees` data, and Phase 7 wires the **transient path**: `DYNAMIC`
and `LINEARIZE` parse and reach the engine, ODE Tables publish on the wasm
boundary, and the parity replay compares them. A Rust workspace ports the frees
engine to WebAssembly. Solve, check, transient integration, real-fluid
properties, property diagrams, component networks, the component datasheet, the
language reference, the CAS REPL, the control-systems `CALL`s and the Data
Analyzer's CSV path all run in-browser with **zero `/api/` traffic** (the
`.mf4` reader shipped with Phase 10 and was later removed by D6 — see below).
Do not trust a count copied into this paragraph; the current ones are in the
block below. *(That sentence used to point at
[`docs/status-phase12.md`](docs/status-phase12.md) for "current gate numbers".
It is the right document for how Phase 12 was **argued** — the corpus triage,
the oracle traps, the fuzzing contract — but its numbers froze on 2026-08-05 at
corpus 701 and a 2944 KiB bundle, and roughly forty wave commits have landed
since. Waves A–S are recorded in their **commit messages**; there is no wave
status document, which is why this block exists.)*

**Gate numbers, re-measured 2026-08-27 at Wave T5** — re-run, not copied:

| Gate | Result |
|---|---|
| parity replay, release, unsharded | **1308 / 1308** match the JDK oracle — 65 fixtures at a declared tolerance, 11 variables on the absolute channel |
| the four shards CI runs | 11.97 / 17.36 / 46.35 / 52.49 s (release, measured 2026-08-26 and **not** re-run); `ev-battery-cooling-pid` is 45.86 s of the longest |
| `cargo fmt --all --check` | clean |
| `cargo clippy` (native + wasm32, `--all-targets -D warnings`) | clean |
| `vitest run` (Node 22) | **33 files, 424 tests**, all pass |
| wasm bundle | **3087.8 KiB raw / 1264.8 KiB gzipped** — 75.4 % of the 4096 KiB budget, 1008.2 KiB headroom |
| pending corpus | **2** documents (`fixtures/README.md`) |

> **No replay timing this round, on purpose.** The 126.98 s S1 measured is the
> quiet-box figure and still the one to quote; this round's replay shared the
> machine with a wasm build and came in at 175.53 s, which measures the
> contention and not the engine. Wave R3's rule applies to the gate as much as
> to a lever: wall clock on this hardware is only meaningful on an idle box.

> **The browser module is a separate artifact and this table does not grade
> it.** Every row above is the *native* build; nothing in CI solves a corpus
> document through the wasm, and 107 of them answer differently there. See the
> Wave T5 block below.

> **Properties: Phase 5's "working real-fluid property backend" was D1's
> precomputed `(P,h)` tables. That is no longer what the engine uses.** Since
> [D9](docs/decisions/0009-rustprop-backend.md) (2026-08-17) real-fluid and
> humid-air properties are answered by **rustprop**, a pure-Rust port of
> CoolProp 8.0.0 linked as a Cargo dependency, and the tables are a native-only
> fallback that no longer ships in the browser bundle. Read the D9 paragraph
> below and *The property backend* under **Workspace layout** before writing
> anything that touches `props/`.

**MDF4 is removed (decision [D6](docs/decisions/0006-remove-mdf4.md), after
Phase 12).** The `.mf4` reader, `mf4-rs` (and its `meval` → `nom 1.2.4`
future-incompat debt — the build's only such warning, now zero), the
boundary's opened-file registry, and the analyzer's remote-source path are
gone; **the Data Analyzer is CSV-only** and `measurement_calc` survives,
stateless, with inline inputs. *(Superseded 2026-08-24: D11 removed the Data
Analyzer and `measurement_calc` with it — nothing of the measurement surface
survives. See the Phase 10 paragraph below.)* The Phase 10 paragraph below is therefore
**historical**: its measurement surface shipped, was hardened, and was then
deliberately removed — read D6 for why, and ledger item 37. The binary
fixture it says must be committed no longer exists.

**The dead-end UI is clipped (decision
[D5](docs/decisions/0005-feature-clip.md), after Phase 12).** The Min/Max,
Curve Fit, PID Tuner, Monte Carlo and Parameter Estimation modals and the
PDF/EPS exports are removed — they only ever surfaced `NOT_IN_BROWSER_ENGINE`
stubs. The stubs, the pid helpers and Phase 8's `analysis/` module remain as
the wiring seam (ledger item 36); the Tables workbook stays (since Wave B1,
2026-08-22, its GUI Solve is **wired** — the wasm `solve_table` export drives
`analysis::parametric::run_sweep` behind the transcribed controller caps, and
the old note below is history: its GUI Solve was
also stubbed but the workbook is core UI — a wire-next candidate, not a clip).

**Phase 12 is implemented — the plan's last phase.** A hardening pass that
changed **zero engine lines**: the parity corpus grew **531 → 701** (a
212-candidate harvest of the Java test classes via
`tools/harvest-java-tests/harvest.py`, run against the live oracle **on
macOS** — `COOLPROP_LIBRARY` must point at the `/usr/local/lib` dylib, never
the vendored Linux `.so`, or fluid goldens silently record failures), with all
21 non-promoted fixtures classified in `fixtures/README.md` (6 witnesses of
already-ledgered divergences, 12 unported features incl. the newly-found
unwired `CALL eigenvalues`/`eigen` — ledger item 34, closed 2026-08-21 by
Wave A1 — and 3 oracle artifacts).
Property-based fuzzing landed (`tests/fuzz_properties.rs`, proptest, 7
properties over parser/units/solve/MDF4 — dev-and-native-only, cfg-gated off
wasm32 because proptest's getrandom refuses that target), the first benchmark
suite (`benches/solve_bench.rs` + a JVM-oracle comparison: **~1× on the
transient**, integrator-bound as predicted, up to ~60× on scalar/property
documents; the browser column was measured 2026-08-23, Wave G5, and
corrected the same day — `web/bench/wasm-bench.spec.ts`, a real factor of
~1.9–2.7× over native with a fixed ~0.7 ms JSON-boundary cost dominating the
trivial document; the first run's "transient inverts against the JVM" was
machine-load contamination and is retracted — `docs/status-phase12.md` §3
has the corrected table, the contamination note and the Wave G3 per-step
cache delta), the first **named** bundle breakdown (twiggy over the pre-bindgen
artifact; the property tables and CAS/control confirmed as the two real
levers), and the worker-death/respawn path finally has tests
(`web/src/wasm/engineClient.test.ts`). Read
[`docs/status-phase12.md`](docs/status-phase12.md)'s "did not deliver" list
before extending: the harvester's representable-document boundary and the
frozen SUNDIALS oracle are the sharpest edges.

**Phase 11 is implemented and proven.** The browser-native product layer:
the app is an **installable PWA** whose Workbox service worker precaches the
entire built app (334 entries / ~30 MB incl. the wasm engine — offline reload,
offline solve and the offline project library are proven by a Playwright run
against a dumb static server in `docs/status-phase11.md`), an **IndexedDB
project library** (`web/src/projectStore.ts`, decision
[D4](docs/decisions/0004-project-storage.md)) gives Save-to-browser /
Browser-Projects with name-keyed file semantics, and the workspace autosave is
**dual-written** — localStorage stays the synchronous boot cache, IndexedDB is
the durable mirror that survives quota and is *offered* (never forced) as a
restore when it is strictly newer at boot. The web deploy is **static-only**:
the nginx `/api` proxy blocks, rate limiter and real-ip machinery are deleted,
and the Dockerfile fails fast when the generated wasm pkg is missing (it also
now actually copies `public/`, which the old image never did). **No Rust
changed** — the wasm bundle is untouched at 2944 KiB and all 531 fixtures
still match. Zero new runtime dependencies; `vite-plugin-pwa` and
`fake-indexeddb` are dev-only. Share links and `.frees` file save/open
predate the phase (vendored) and are documented as such, honestly, in the
status doc — read its "did not deliver" list before building on this layer;
item 1 is that the optional remote-fallback adapter stays unwired by choice.

**Phase 10 is *removed*, and this paragraph is history (decision
[D11](docs/decisions/0011-remove-analyzer.md), 2026-08-24).** D6 took the
`.mf4` reader; D11 takes the rest. `crates/frees-core/src/measurement/` (3,251
lines), `crates/frees/src/measurement.rs` (1,184 lines), the
`measurement_calc` export and the `measurement_parity` /
`measurement_robustness` suites are gone with the Data Analyzer that was their
only consumer — **the engine has no measurement surface at all**, and measured
data now enters a document as a CSV-imported *function table* (Wave H),
callable in equations. Read D11 and ledger item 38 before reviving any of it.
What follows is what Phase 10 delivered, left as written on 2026-08-01:

**Phase 10 was implemented and wired.** `crates/frees-core/src/measurement/`
(MDF4 reading over `mf4-rs`, sampled series, envelope decimation, raster
construction, calculated signals) and `crates/frees/src/measurement.rs`
replace `/api/measurements/*`, and `web/src/analyzer/measurementApi.ts` now
calls the worker instead of `fetch`. **A `.mf4` opened in frees never leaves the
machine** — that is the point of the phase, and it is why the Java's
`Mf4Parser` → `FallbackMeasurementParser` → Python `mdf-sidecar` ladder collapses
to one rung. The cost of collapsing it is real and is the first entry in
[`docs/status-phase10.md`](docs/status-phase10.md)'s gaps list: **compressed
(`##DZ`) recordings — deflate, ZSTD and LZ4 — are refused, as are VLSD string
storage and multi-group files.** An OEM recording will probably not open.
One new dependency, `mf4-rs` 3.6 (MIT), which brings the repo's only
future-incompat warning (`meval` → `nom 1.2.4`); see that document for the exit.
The bundle is **2944 KiB against the 3072 KiB budget — green, with 128 KiB of
headroom**, and `fixtures/measurement/a_small_uncompressed.mf4` must be committed
with the `.rs` files or the core test build fails.

**Phase 9 is implemented and wired.** The Symja-replacement CAS
(`crates/frees-core/src/cas/`: exact rational algebra over ℚ, a Zassenhaus
factoriser, partial fractions, a Laplace transform table, `CasEngine` and
`CasIdentity`) and the control-systems suite
(`crates/frees-core/src/control/`: transfer functions, state space, LQR/place/
LQE, PID tuning, time responses, and the 41-name `CALL` flattener) both reach a
document, the REPL and the browser. Four new dependencies —
`num-bigint`/`num-integer`/`num-rational`/`num-traits`, all MIT OR Apache-2.0.
**531/531 fixtures** now match the oracle. One thing to know before building on
it: **`Integrate` is a closed pattern table, not an integrator**
(`docs/status-phase9.md` lists the exact boundary). *Correction:*
`docs/status-phase9.md` records the bundle as 3336 KiB and "the one red gate" —
that number is **pre-commit and stale**. The `opt-level = "s"` lever it describes
as "measured, not taken" was taken in the same commit, and re-measured at HEAD it
is worth 535 KiB. See `docs/status-phase10.md`'s bundle section.

**Phase 8 is *mostly* not implemented.** `crates/frees-core/src/analysis/` exists
and is unit-tested, but only **one** of its modules is reachable: `uncertainty`
is now wired into `engine::solve_with` at the Java positions, so
`UncertaintyOf(X) = expr` is lifted out of the equation stream, the propagation
and its second solve pass run, and `VariableDto.uncertainty` +
`uncertaintyBreakdown` reach the wasm boundary. The optimizer, NSGA-II, curve
fitter, Monte Carlo, parameter fit, all-roots solver and parametric sweep driver
are still unreachable from a document and absent from the boundary. A robustness
pass over Phases 7–8 hardened both surfaces and is written up in
[`docs/status-phase78.md`](docs/status-phase78.md); read its gaps list before
planning further work.

> **The web test gate needs Node 22.** Under `node v20.x`, `npx vitest run`
> fails all 38 files in `jsdom`→`undici` (`webidl.util.markAsUncloneable is not
> a function`) before running a test. `web/.nvmrc` asks for 22; use it.

**The property coverage gap is narrowed (decision
[D7](docs/decisions/0007-auxiliary-property-grids.md), 2026-08-06).** The
long-running "real-fluid coverage the tables do not have" divergence (ledger
item 9) cost the catalog its flagship Systems example: `ev-thermal-management`
failed at `Block 3 (89 equations): no property table for fluid
'INCOMP::MEG[0.50]'`, and behind that lay **three more** missing capabilities —
`R1234yf`, air transport for `htc_extair`, and saturation-line transport for
`htc_evap`/`htc_cond`/`dp_2phase`, which was absent for *every* fluid including
the two already tabulated. All four are closed by one new artifact kind
(`FRAUX1`) plus an ordinary `table-gen` run. **The example solves** — 169 blocks,
231 Newton iterations, COP 3.75, with its open-circuit energy balance closing to
9e-13 W — and grades against the JDK oracle at `8.951e-4` worst variable,
`1.0e-6` median over 229 variables. Corpus 702 → **704**. It cost the bundle
budget a 273.1 KiB breach, which is **since closed** — see the size pass below.

**The accuracy path has LANDED — as rustprop, not `coolprop.wasm` (decision
[D9](docs/decisions/0009-rustprop-backend.md), 2026-08-17).** The wasm bundle
ships **rustprop**, the pure-Rust CoolProp 8.0.0 port, as its only in-bundle
property backend, and the linked `.phtab`/`.fraux` artifacts left with the new
`linked-tables` Cargo feature. Bit-exact CoolProp *and* smaller on the wire:
**2721.9 KiB raw / 1118.2 KiB gzipped** re-measured 2026-08-19 on `wave4-f9`
(88.6 % of the 3072 KiB budget, 350.1 KiB headroom), against 3042.3 / 1597.7
before the switch. *(Superseded 2026-08-22: Wave B wired the whole Phase-8
analysis layer for +340 KiB, taking the module to **3064.9 KiB**, and the
budget was raised **3072 → 4096 KiB** — owner-authorized, sanctioned by the
CI header's own rule since D9 paid debt (1); the header's dated entry has
the full lever-by-lever justification. Current headroom ≈ 1031 KiB.)*
*(Re-measured 2026-08-26 at HEAD — the first measurement since Wave G2, across
twelve Rust-touching commits (D10's `functionTables` port, D11's removal of the
measurement stack, P3, Q4, R2, R3, S1): **3085.2 KiB raw / 1259.0 KiB
gzipped**, 75.3 % of the 4096 KiB budget, **1010.8 KiB headroom**. The net of
all twelve is **−7.4 KiB raw**. Debt (2), the lazy chunk, is still untouched
and still unneeded.)* Thirteen of the 23 entries in `fixtures/tolerances.json` are
retired: the rustprop configuration is graded by
`fixtures/tolerances-rustprop.json` and its **65 relative entries plus 10
absolute ones covering 11 variables** (ten scalar since Wave-3 F5, Wave G1's
two transient entries, Wave G4's eleven component-harvest entries — all
2026-08-23 — Wave A1's fourteen two-phase-cycle entries and Wave C1's CO2
entry, 2026-08-24, then Wave P1's absolute channel, Wave P2's eighteen
class-sweep entries and Wave Q1's last two, 2026-08-24/25, **less Wave R2's
retirement of `components_g4_radiator`**, whose worst variable went 1.2042e-6 →
1.2947e-14 when the `props_si` cache made a repeated property call return the
identical double — 2026-08-25), one file per
backend, chosen by the same `rustprop-backend` cfg that chooses the backend.
Five mechanisms are catalogued and every entry names one: `oracle-ph-table`,
`upstream-ps-flash-residual`, `ida-adaptive-path`, `ode45-adaptive-path` and
Wave Q1's `smooth-clamp-regulariser` (a latent duty that is structurally zero,
where the golden is the model's own `1e-12` smooth-min regulariser recovered
by an eleven-digit cancellation — the first `absolute` entry graded in watts).
`HAPropsSI` answers. `Air` left the property-diagram picker with the
`air.fraux` grid it was the only backing for — *and came back at
Wave-2/Wave-3, once rustprop's own pseudo-pure `HSU_P`/`(D,P)` flashes landed:
`Air` is on `served_fluids` and in the picker, and the D6 amendment to D9
records why.* **`CO2` joined both on 2026-08-24 (Wave C1), by owner request**
— its data had been linked since Wave G2, so the picker entry cost a measured
+142 bytes; the Wave C1 amendment to D9 has what was verified through the
transcritical region and the one curve that is still absent on purpose.
**Still do not write a humid-air backend, an `air.phtab`, or
further `FRAUX1` grids** — rustprop supersedes all three. Read D9 before
touching `props/tables.rs`, `props/rustprop_backend.rs` or either tolerance
file.

**D8 is implemented and closed (decision
[D8](docs/decisions/0008-coolprop-wasm.md), decided 2026-08-07, closed
2026-08-19).** It asked for CoolProp-grade accuracy as the property path and
imagined an Emscripten `coolprop.wasm`; rustprop reached usable first and is
what the repo picked up, so the decision held and the implementation did not.
D8 records the measurement that forced the choice: an ideal-gas humid-air model
is 2.7e-3 off CoolProp on the `(T,R)` path (the enhancement factor reaches
1.0041), and transcribing ASHRAE RP-1485 does not fix it, because CoolProp's
`HAPropsSI` *is* RP-1485 evaluated against IAPWS-95 and Lemmon — whose
ancillary saturation equation alone is already 7.1e-5 off.

Both of its predictions held, and one of its risks inverted:

> **Twelve of twelve (Wave-3 F6 + F8, 2026-08-18).** D8 predicted a real
> CoolProp would clear twelve of the then-26 pending fixtures — humid air 7,
> `Air` `(P,h)` state 3, `(P,T)` transport 1, `Z` 1. All twelve promoted, every
> one at the corpus default `1e-9` with **no tolerance entry**: worst 1.31e-11
> across F6's nine, and worst graded 2.50e-14 across F8's three.
> **Corpus 707 → 719, pending 26 → 14, and none of the fourteen is a property
> hold.** `fixtures/README.md`'s two "Re-check 2026-08-18" sections have the
> per-document numbers.
>
> **The bundle risk inverted.** D8's central open risk was where the bytes for
> CoolProp's fluid data would come from — it expected the module to grow
> against 29.7 KiB of headroom. rustprop's data is per-fluid Cargo features, so
> the module *shrank* by 320.4 KiB raw / 479.5 KiB gzipped and the headroom is
> now 350.1 KiB.
>
> One rustprop-side gap is recorded and unclosed: a genuine low-quality `(P,h)`
> refusal window inside Air's dome at 79–101 K. No fixture goes near it (the
> pneumatic three are 167 K above Air's critical point); the measurement is in
> `fixtures/README.md`'s "Re-check 2026-08-18, Wave-3 F8".

**The size pass is done (2026-08-06), and it changed no engine behaviour.** It
took the wasm to **3031.0 KiB raw / 1589.9 KiB gzipped**, back under the 3072
KiB budget with 41 KiB spare, and the built web app from 20.25 MB to **14.7 MB
of `dist`**. *(Those wasm figures are that day's; D9 superseded them a fortnight
later — see the D9 paragraph above for the current bundle. The `dist` work and
the four findings below still stand.)* All 704 fixtures matched, `cargo clippy`
(native + wasm32) and `cargo fmt` were clean, and vitest was 40 files / 384
tests green. Four findings, each measured:

* **D7's "compression is not a lever" was wrong** — correctly measured, wrongly
  concluded. Plain deflate reaches only 0.89 on the `f32` property grids, but
  transposing each artifact into byte planes first (the HDF5 shuffle filter)
  takes 1014 KB → 685 KB, because the near-constant exponent bytes of thousands
  of consecutive samples then sit together. `crates/frees-core/build.rs` packs,
  `props/tables.rs` unpacks once at install time; it is a pure byte permutation
  plus a lossless codec, so the decoders still see the generator's own bytes and
  the artifacts under `src/props/data/` and `fixtures/` are untouched. One new
  dependency (`miniz_oxide`, MIT, inflate only, 11 KiB of code section).
* **Univer's ~80 hyphenation dictionaries were still being emitted** — 4.4 MB of
  `dist`. They had been excluded from the precache, which stopped the download
  but not the build output. They are stubbed at resolve time now. This also
  fixed a latent offline bug: Univer's `DocumentSkeleton` constructor eagerly
  loads the `en-gb` dictionary, so every spreadsheet render fetched a 102 KB
  chunk the service worker had been told to ignore. All ~80 imports redirect to
  one shared stub chunk, which is why `dist` is **95 files where it was 171**.
  The plugin needs `enforce: 'pre'` — without it Rollup resolves the real
  dictionary first and the 4.4 MB come back *with a green build*; the precache
  `globIgnores` list is kept as the backstop that catches exactly that.
* **The App chunk statically imported 1068 KB of reference documentation on
  every cold start**, to reach one 9 KB default-example string that Rollup's
  auto-placement had put in the `docs-data` chunk. Split. Opening the Examples
  modal likewise pulled the whole reference catalog; it now pulls 51 KB.
* **The nginx image served the entire bundle uncompressed** — no `gzip`
  directive at all, 19.3 MB on the wire where 6.2 MB would do. Only the Docker
  path was affected (Vercel compresses at its edge), which is why it went
  unnoticed. `gzip_static` plus precompression in the build stage.

One more, latency rather than bytes: KaTeX's stylesheet was imported at the
entry, and because `manualChunks` routes anything matching `/katex/` into the
`katex` chunk, that made its 254 KB of JS a static import of the entry — and
through the chunk graph, of every chunk in the app. KaTeX renders only inside
`<Latex>`, which only the lazy Help page mounts (the `$$…$$` blocks in
`symbolic_cas.md`, `language_fundamentals.md` and `tutorials.md`). The import
now lives in `Latex.tsx`.

**Two engine facts from Waves R and S (2026-08-25/26) that a new reader will
otherwise get wrong.** Both are recorded in full in their commit messages;
what follows is the part you must know before editing near them.

* **`props_si` answers are cached, at capacity one** (`props/propfun.rs`,
  `mod cache`, Wave R2). This is a *missing port*, not an optimisation: the
  Java's `props/CoolProp.java` has held `PROPS_CACHE`/`HA_CACHE` all along, so
  the goldens were produced by an engine that had it. It is worth **1.78×** on
  `ev-battery-cooling-pid` (78 → 44 s) and −21 % on the whole replay, and it
  retired a tolerance entry: `components_g4_radiator` went 1.2042e-6 →
  1.2947e-14 because an eleven-digit cancellation of two nearly-equal humidity
  ratios becomes exact once the repeated call returns the *identical* double.
  **The capacity is one on purpose.** `rustprop_warm` seeds each `(P,h)`/`(P,s)`
  flash from the previous answer, so an answer is a function of its arguments
  *and* of the seed the previous call left. At capacity 1 a hit means the
  previous call asked the same question, so the seed still is that call's
  converged state; at capacity 2 it is not, and bit-identity with the live call
  falls 99.72 % → 65 %. Wave S1 built the seed-identity tag that would lift the
  cap, measured it, and **declined** it: the tag keeps every seeded hit
  bit-identical but is worth only 1.3–1.6 points of hit rate (R2's estimate of
  13 was counting the dishonest hits), and every tagged configuration that held
  capacity 1's fidelity ran 5.1–6.5 % slower. S1 also corrected R2's accuracy
  note on the way: against a live
  CoolProp 8.0.0, neither the warm nor the cold path is uniformly nearer, and
  the dominant term is rustprop's own `Hmass(T,P)` rather than either flash.
* **A prepared block runs on a dense `Vec<f64>`, not on the name-keyed
  `Scope`** (`engine.rs`, `ode/dynamic.rs`, Wave R3), and the `Scope` is
  materialised only for a caller that actually reads it — `DynamicSolver`'s
  `rhs` reads one name out of the map, and the map is genuinely needed only at
  the output samples. The three `AlgebraicSolve` methods that carry this
  (`watch`/`watched`/`materialise`) are **defaulted to off**, so every other
  implementor is untouched. Worth −17.8 % instructions on both transient
  stand-ins, byte-identical everywhere. The failure mode this invites is
  recorded in the commit and is worth reading before touching it: a late guard
  silently disabled half the change, every output stayed byte-identical, the
  corpus stayed green, and **only the instruction counter could see it**.

**Three browser facts from Wave T5 (2026-08-27), all of which a reader who
trusts the native gate will get wrong.**

* **The browser and the parity gate are not running the same arithmetic.**
  `frees-core` routes every transcendental through the `libm` crate so the two
  targets agree bit-for-bit — but **rustprop does not**, and D9 made rustprop
  the browser's only property backend. It calls `std`'s `f64::exp`/`ln`/`powf`
  (158 / 151 / 459 sites, zero `libm::`), which is glibc natively and Rust's
  bundled `libm` on wasm32; they differ by one ulp on ~11 % of `exp` arguments.
  Replaying all 1308 fixtures through the shipped module under Node: **107 of
  the 1095 documents both builds solve return different numbers in the
  browser** (worst 5.4e-6 on a real quantity). A `libm` `LD_PRELOAD` shim into
  the native build closes 91 of the 107 and reproduces the wasm trajectories
  bit-for-bit, which is what pins it to rustprop. **No CI job solves a corpus
  document through the wasm module** (`native`, `wasm` = build + size only,
  `web` = build + vitest, `parity` = four *native* shards), which is why this
  went unseen; a Node replay of the whole corpus costs ~222 s. The full
  measurement is in `docs/dependency-map.md` under the Transcendentals row.
* **The transient wall-clock ceiling is 3600 s, not the Java's 60.** A
  deliberate divergence, owner-authorized: `SolverApiSupport`'s cap is
  per-solve on a *shared* worker, and in-browser the worker is the user's own
  tab. `MAX_ELAPSED_SECONDS_CAP` was a ceiling as well as a default, so
  `ev-battery-cooling-pid` — ~80 s of wasm, 56 s native — could not be solved
  in the browser at **any** Stop-Criteria setting. It can now.
* **A struck budget used to report someone else's error.** The block owning a
  `FinalValue(...)` wraps the transient, so a deadline strike failed that
  block, `solve_block_with_fallback` ran the retry ladder, and the merge
  rescue's `?` replaced the honest "exceeded its N-second wall-clock budget"
  with whatever the re-run failed on — in practice "Newton iteration stalled …
  unable to bracket the (p,X) solution", which sends the reader hunting for
  guess values that were never the problem. The ladder now refuses to run on a
  struck deadline (`engine.rs`, and it is ~2.4 s cheaper too).
  `crates/frees-core/tests/progress_and_deadline.rs` is the regression.

**Solve progress is a host-installed channel, exactly like the deadline**
(`crates/frees-core/src/progress.rs`, Wave T5). Core owns no UI, so the wasm
boundary installs a sink per request and clears it in a guard; nothing is
installed natively, so the replay cannot see it. Two things carry the design:
the **span** (`enter` subdivides the *current* slice, so the block loop and a
transient inside a block compose without knowing about each other) and the
**claim** — `engine::run_blocks` serves every pinned subsystem too, so without
one it reports `0, ¼, ½, ¾` once per integration step and the bar flickers
instead of advancing. The integrator takes a `Claim` over the whole transient
and reports `t/tf`; while it is held, `enter`/`report` are silent. That is a
measured failure, not a hypothetical — it is what the monotonicity test caught
on the first version.

**The named solver levers are spent, and round seven inherits a short list.**
A2, Q3 and R3 took the hashed-access family from 27.8 % of the transient
profile to under 2.8 %, and S1 closed the property-cache question by declining
it. What R3 measured and did **not** take: `Vec<f64>::resize` at 5.5 %
(A2's `refill` helper measured −1.5 % and was reverted under the bar — it is
cheaper now that the hashing is gone), `unpack_bounds_into` 3.7 % + `validate`
3.3 % per Newton call, and grisu at 3.1 %, which is Q3's lazy-pin source-text
item and still needs an `Equation` contract change. `solve_block`'s own 32.9 %
and `eval_program`'s 17.0 % are what the profile is now.

**Measure it the way those waves did, not with a stopwatch.** Wall clock on
this hardware is useless at this scale — the same baseline binary ran
`dyn_accessor_live` in 168.23 s and 159.24 s — so every keep/revert decision in
Q3, R3 and A2 rests on callgrind instruction counts, with byte-identity of the
solved output checked separately. R3's near miss is the reason both halves are
needed: identity checks cannot see a performance regression, and the counter
cannot see a wrong answer.

| Document | Contents |
|---|---|
| [`docs/status-wave3-f7.md`](docs/status-wave3-f7.md) | **The measured behaviour of the engine under the rustprop backend.** Wave-3 F7's robustness + performance sweep: the parity replay at 43–72 s against its ~180 s anchor (and why the backend is *not* the reason), the per-call budgets with their real margins (9,216-call hostile sweep worst 377 ms of 2 s; `all_survive` worst 155 ms of 20 s; the plateau at 500 µs), a 64–128x fuzz soak, the benches with `rankine_cycle`'s honest ~1.35x property cost, the audit that found **`nominal_enthalpy` seeding never runs** (and why that is faithful to the Java), the four ways `block_count` exactness was checked, and the one zero-headroom timing assertion the sweep broke. **Read its "How to read the numbers" section before quoting any second from it** |
| [`docs/decisions/0007-auxiliary-property-grids.md`](docs/decisions/0007-auxiliary-property-grids.md) | **D7 — read before touching the property backend or the bundle.** The `FRAUX1` grids: what the three surfaces are, why saturated transport is 1-D (and therefore cheap), why the incompressible grid is *exact* in pressure, why its concentration axis lands on exact nodes, the measured error per grid, the one operating-point amplification that sets `ev-thermal-management`'s tolerance, and the budget breach with its four measured options |
| [`docs/status-phase12.md`](docs/status-phase12.md) | **Read first.** Phase 12 — parity at scale, performance, hardening: the corpus at **701** with the 21-fixture triage, the macOS-oracle traps (CoolProp dylib, frozen SUNDIALS), the proptest fuzzing contract, the measured Rust-vs-JVM table with its honest ~1× transient anchor, the named twiggy bundle breakdown, the worker-respawn tests — and a ranked list of what Phase 12 did **not** deliver, starting with the harvester's representable-document boundary |
| [`docs/status-phase11.md`](docs/status-phase11.md) | Phase 11 — the browser-native product layer: the installable-PWA + full-precache offline story with its browser proof (offline reload, offline solve, offline project library, zero network), the IndexedDB project library and dual-written autosave (decision D4), the static-only deploy, what was **already there** (share links, `.frees` save/open — stated plainly rather than claimed), and a ranked list of what Phase 11 did **not** deliver, starting with the still-unwired remote-fallback adapter |
| [`docs/status-phase10.md`](docs/status-phase10.md) | Phase 10 — measured data in the tab: what shipped per area, the **fifteen** defects three adversarial sweeps found — listed individually so the count is checkable (seven end the session — five allocation aborts and two unbounded walks — two are silent wrong answers, one wedges the worker on an ordinary formula, and five are numeric-parity divergences against a live JDK oracle) — the browser proof against genuine asammdf bytes with **zero `/api/` requests**, the raw gate numbers, the measured `mf4-rs` bundle delta and the `nom 1.2.4` debt with its exit — and a ranked list of what Phase 10 did **not** deliver, starting with the `mdf-sidecar`'s three compressed formats, which now have no answer at all |
| [`docs/status-phase9.md`](docs/status-phase9.md) | Phase 9 — the from-scratch CAS and the control-systems suite, wired end to end: what shipped per area, the `O(n⁴)` CAS defect the adversarial sweep found and fixed (a 200-symbol `Expand` went from **256 s to 0.41 s**, and to **103 ms in the browser**), the twelve other attacks that found the bounds already in place, the browser proof, the raw gate numbers, the 31 promoted fixtures — and a ranked list of what Phase 9 did **not** deliver, starting with the exact boundary of `Integrate`. **Its bundle section is stale**: the 3336 KiB breach it reports was fixed by the `opt-level = "s"` change in its own commit; see `docs/status-phase10.md` |
| [`docs/status-phase78.md`](docs/status-phase78.md) | The Phase 7–8 robustness pass: the four defects an adversarial sweep of the transient and analysis surfaces found (two of them process *aborts*, one a silent wrong answer), the guards that close them, the browser proof, the raw gate numbers, and a ranked list of what these phases did **not** deliver — starting with "Phase 8's `analysis/` is not wired to anything" |
| [`docs/status-phase7.md`](docs/status-phase7.md) | What Phase 7 delivers per area, the raw gate numbers, which 29 fixtures were promoted, and its pending table — **annotated with what Phase 9 closed** — plus the one honest performance finding (`dyn_accessor_live`) |
| [`docs/status-phase6.md`](docs/status-phase6.md) | What Phase 6 delivers per area, the component-coverage numbers, what the component fuzz found, its gate numbers and fixture counts |
| [`docs/status-phase5.md`](docs/status-phase5.md) | Phase 5 per area, the measured table-vs-CoolProp error, and its pending-fixture table (annotated with what Phase 6 closed) |
| [`docs/status-phase4.md`](docs/status-phase4.md) | Phase 4 per area; its pending-fixture table is annotated with what Phase 5 closed |
| [`docs/status-phase1.md`](docs/status-phase1.md) | The maintained divergence ledger (items closed are struck through with a date; Phase 5 opened items 9–12, Phase 6 items 13–14, Phase 7 items 15–18, the Phase 7–8 robustness pass items 19–23, the Phase 9 robustness pass items 24–25, Phase 10 items 26–30), plus the Phase 0–3 inventory |
| [`PLAN.md`](PLAN.md) | The phased plan: architecture, decisions, parity strategy, 13 phases, risks |
| [`docs/dependency-map.md`](docs/dependency-map.md) | Every Java/native dependency → Rust replacement |
| [`docs/feature-inventory.md`](docs/feature-inventory.md) | All 134 `backend/core` files mapped to features and phases |
| [`docs/decisions/`](docs/decisions/) | D1 (precomputed `(P,h)` property tables), D2 (wasm32-unknown-unknown + wasm-pack), D3 (worker pool, no COOP/COEP), D4 (project storage), D5 (feature clip), D6 (remove MDF4), D7 (`FRAUX1` auxiliary grids + the bundle-budget breach — superseded for the browser by D9), D8 (CoolProp-grade accuracy becomes the property path — **implemented and closed**, by rustprop rather than by the `coolprop.wasm` it imagined), **D9 (rustprop is the wasm build's only property backend and the tables leave the bundle — read before writing any new property backend)**, D10 (the spreadsheet and Univer are removed — Wave H, 2026-08-23: the Tables workbook is a native glide grid, GUI function tables reach the engine on every request via the completed `functionTables` port with the document-wins collision rule, sweep/fit/CSV compose into callable functions, and the dist shrank 15.2 → 9.7 MB — read it before reintroducing any spreadsheet dependency or promising a GUI table can override a document `TABLE` block) |
| [`fixtures/README.md`](fixtures/README.md) | The parity harness: corpus (1308 — **Wave J** swept the 114 Java test classes that were never in the harvester's class list; +255; Wave A1 promoted 14 property holds, Wave C1 added CO2 and Wave P1 eight exact-zero documents, four of them re-harvested from Wave J's unstaged set; **Wave Q** +25, the Java sites whose *request* the fixture format could not carry) and golden fixtures, the `.tables.json` request-table sidecars (Wave I) and the `.request.json` solver-request sidecars (**Wave Q** — non-default stop criteria, complex mode, and `VariableSpec` guesses/bounds; an absent sidecar is the engine defaults, so it changed no existing fixture), the pending set (**2** — the `dyn_accessor_live` cost hold and Wave Q's `ode45`-shooting rocket, whose mechanism Wave R1 measured and whose promotion needs the *document* changed, not the gate; `ev-tms-api-model` left the set at Wave Q1), the decayed-signal measure (Wave G1) for ODE row cells and the **absolute channel** (Wave P1) for a quantity whose true value is a structurally exact zero, how to run the gate and why the single-package form refuses, tolerance policy (`fixtures/tolerances-rustprop.json` grades what ships — 65 relative + 10 absolute entries; `fixtures/tolerances.json` describes the table configuration and nothing replays it today), oracle-established ground truths |

## Build and test

```bash
export PATH="$HOME/.cargo/bin:$PATH"   # toolchain is rustup-installed; distro rustc is stale
cargo test --release --workspace       # all tests incl. the parity replay
                                       # (--release: the replay solves 1308 documents,
                                       #  127 s of the run measured 2026-08-26 on a
                                       #  quiet machine. Wave Q's 406 s predates R2's
                                       #  props_si cache and R3's slot-native prep;
                                       #  the same replay was 167.9 s before R2.)
cargo test --workspace --test parity   # golden-corpus parity only — what the four
                                       # sharded CI jobs run, in DEBUG. The `native`
                                       # job deliberately skips it (ci.yml) — the
                                       # shards already replay the whole corpus.
cargo test -p frees-core --features rustprop-backend --test parity   # same, single package
                                       # NOT `cargo test -p frees-core --test parity`:
                                       # the single-package form does not unify features,
                                       # and the corpus is unservable without rustprop.
                                       # It refuses with this command in the message.
cargo test -p frees-core --test fuzz_properties        # property-based fuzzing
                                       # (PROPTEST_CASES=4096 for a longer soak)
cargo bench -p frees-core --bench solve_bench          # the Phase 12 benchmarks
cargo test -p frees-core --test props_robustness       # the property-surface fuzz
cargo test -p frees-core --test component_robustness   # the component-surface fuzz
cargo test -p frees-core --test cas_control_robustness # the CAS + control fuzz
cargo test -p frees-core --test dae_robustness         # the DAE API fuzz (Wave G6)
cargo test -p frees-core --test dynamics_robustness    # the transient + analysis fuzz
                                       # (run this one in DEBUG too — the stack-overflow
                                       #  defect it found only reproduced unoptimised)
cargo clippy --workspace --all-targets -- -D warnings   # CI gate
cargo clippy --workspace --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo fmt --all --check                                 # CI gate
wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg
tools/golden-dumper/run.sh             # regenerate golden fixtures from the Java oracle
tools/table-gen/run.sh                 # regenerate fixtures/proptables from native CoolProp
tools/aux-gen/run.sh                   # regenerate fixtures/auxtables (FRAUX1 grids, D7)
tools/aux-gen/run.sh --sweep           # its error-vs-resolution ladder; writes nothing
```

> **The oracle tools find `../frees` themselves.** `tools/frees-home.sh`
> resolves the reference repo as a sibling of this one (both the `frees` and
> `frEES` spellings), so `classpath.sh` and every `run.sh` work without
> `FREES_HOME`. They used to hard-code `/home/eren/dev/frEES`, which outlived
> the directory it named and made all three tools report "reference repo not
> found" on a checkout where the reference was sitting right next door.
> Building the oracle jar once is still required:
> `(cd ../frees/backend && ./gradlew :core:jar)`.

> **The oracle has CoolProp.** `tools/golden-dumper/run.sh` exports
> `COOLPROP_LIBRARY` itself (the 12.4 MB `libCoolProp.so` vendored in
> `../frEES/backend/core/native/`), so every real-fluid property call can be
> checked against ground truth instead of guessed. Verified:
> `h = Enthalpy(Water, T=300 [K], P=101325 [Pa])` → `112654.89965464505`.

> **`rtk` condenses these commands' output.** `cargo test` comes back as a
> one-line summary with no per-suite results, and clippy/fmt warnings are
> swallowed entirely. To see real output, call the binary by absolute path
> (`"$HOME/.cargo/bin/cargo"`, `./node_modules/.bin/vitest`) and redirect to a
> file.

> **One agent per working tree.** Two agents editing this checkout at once has
> already broken a gate twice (recorded in `docs/status-phase78.md`, gap 8):
> one left the tree uncompilable for ~15 minutes, and fmt churn landed in
> files neither owned. The convention: an agent that edits runs in its own
> `git worktree` (`git worktree add ../frees-<task> <branch>`), merges
> back only with the gates green, and removes the worktree after; read-only
> agents may share the main tree freely. If a task cannot use a worktree, do
> not run a second editing agent beside it.

## Workspace layout

- `crates/frees-core` — the engine (target-agnostic; **must never depend on wasm-bindgen**)
- `crates/frees` — thin wasm-bindgen boundary (JSON-string in/out)
- `crates/frees-cli` — headless solve/check for the parity harness
- `tools/golden-dumper` — Java program run against the frEES core jar to emit `fixtures/golden/`
- `tools/table-gen` — Java program run against native CoolProp to emit `fixtures/proptables/*.phtab`
- `tools/aux-gen` — sibling of the above, emitting `fixtures/auxtables/*.fraux` (D7): the incompressible glycols, air transport, and saturation-line transport
- `tools/shared/GenSupport.java` — the byte sink, SHA-256, JSON writer and CoolProp-version binding both generators need; compiled into each tool's own `build/` by its `run.sh` (there is no jar). A divergence here would mean two artifacts disagreeing about their own checksums, so it lives in one file
- `tools/frees-home.sh` — resolves the reference repo as a sibling; the one place that path is decided
- `fixtures/` — parity corpus + golden results; grow it per `fixtures/README.md`

### The property backend

**Real-fluid properties are answered by rustprop** — the pure-Rust CoolProp
8.0.0 port, consumed as a git dependency pinned to the **`v0.1.0` tag** of
`github.com/ernsoylu/RustProp` (no sibling checkout is needed since commit
`cb1d7be`; it used to be a `path` dependency) and gated by `frees-core`'s
`rustprop-backend` feature. It is a
plain Cargo dependency with no external deps of its own, so `frees-core` stays
clear of wasm-bindgen; per-fluid data is opt-in at the feature level and this
build enables five (`water`, `r134a`, `r1234yf`, `air`, and — since Wave G2,
2026-08-23, at a measured +25.5 KiB on the wire — `carbondioxide`) plus `heos`,
`humid-air` and `incompressible`.

**Linking a fluid's data and listing it in `served_fluids` are separate
decisions**, and the difference is load-bearing: `served_fluids` gates the
property-diagram picker only (`props_si` never consults it), so a fluid can
answer `MolarMass` and still not be offered a chart. All five linked fluids are
now on the list — `CO2` joined on 2026-08-24 (Wave C1), by owner request, for a
measured +142 bytes, since G2 had already paid for the data. The glycol
families are served but have no dome to draw, so the published picker is
`["Air", "CO2", "R1234yf", "R134a", "Water"]`.

`props::tables::install_builtin_once` — which
every public entry point calls — installs `RustpropBackend`, and
`frees` requires it, so the browser and the native gate get the same
engine. **Decision [D9](docs/decisions/0009-rustprop-backend.md) is the
authority; read it before touching `props/tables.rs`,
`props/rustprop_backend.rs`, `props/rustprop_warm.rs` or either tolerance
file.**

Two things follow that a new reader will otherwise get wrong:

* **`cargo test --workspace` grades rustprop; `cargo test -p frees-core` does
  not.** Resolver-v2 unifies `frees-core`'s features over the members being
  built, and `frees` requires `rustprop-backend`, so the workspace form
  turns it on. The single-package form does not, and the parity corpus
  contains twelve documents no `(P,h)` table can serve, so
  `tests/parity.rs` **refuses** that configuration and prints the command to
  use. Everything else in `frees-core` still runs there.
* **The adapter is not a pass-through.** `RustpropBackend` refuses non-finite
  inputs before rustprop sees them and non-finite answers before the engine
  does. Fidelity to CoolProp is rustprop's contract; surviving `NaN` out of a
  Newton solve is this crate's. D9's Finding 2 has the panic that proved it.

**The D1 `(P,h)` tables are still here, and are no longer what the browser
downloads.** `crates/frees-core/src/props/data/*.phtab` are copies of
`fixtures/proptables/*.phtab`, and `data/*.fraux` of
`fixtures/auxtables/*.fraux`, all packed by `build.rs`, `include_bytes!`d by
`props/tables.rs` behind the `linked-tables` feature (on by default, off in
`frees` alone) and installed on the first `solve`/`check` **only when
`rustprop-backend` is off**. Regenerating them means copying them across as
well as into `fixtures/`. What survives in the browser is the decoders and the
`install_from_bytes` fetch seam — the offline path a host can fetch into.
**Do not add a fluid to them**: D8's moratorium stands and rustprop supersedes
an `air.phtab` and any fourth `FRAUX1` grid.

There are **two** artifact kinds, and the difference is load-bearing:

* `FRPHTAB1` (`.phtab`, `tools/table-gen`) — a phase-split `(P,h)` table for a
  fluid **with a saturation dome**, storing `T`, `Dmass`, `Smass`. Water, R134a,
  R1234yf.
* `FRAUX1` (`.fraux`, `tools/aux-gen`, decision
  [D7](docs/decisions/0007-auxiliary-property-grids.md)) — a rectangular grid of
  named outputs over two axes, for the three surfaces that geometry cannot
  carry: the **incompressible glycols** (`INCOMP::MEG`/`MPG`, which have no dome
  at all), **air transport** at `(P,T)`, and **transport on the saturation
  line** (`viscosity`/`conductivity`/`Cpmass` at `Q=0`/`Q=1` only — which is the
  only place `htc_evap`/`htc_cond`/`dp_2phase` ever ask, and the reason that
  costs ~14 KB per fluid instead of ~256 KB).

Neither kind is embedded verbatim. `crates/frees-core/build.rs` transposes each
artifact into `f32` byte planes and deflates it; `props/tables.rs` inflates it
once, at install time, and hands the decoders the generator's own bytes. That
takes the nine files from 1014 KB on disk to **~678 KB of data section**, which
is what put the module back under budget after D7's 273.1 KiB breach. The
shuffle is the load-bearing half: plain deflate only reaches 0.89 on these grids
(which is what D7 measured before concluding, wrongly, that compression was not
a lever), and byte planes take it to 0.68. **Regenerating a table needs no extra
step** — copy the new `.phtab`/`.fraux` into `src/props/data/` as before and
`build.rs` re-packs it; the checked-in artifacts stay exactly as the generators
wrote them.

*(D7's "read this before adding a fourth fluid, the headroom is 41 KiB" no
longer applies as written: D9 took those bytes out of the browser bundle
entirely, so a fourth `.phtab` would cost the native build and nothing on the
wire. It is also the wrong move — see the moratorium above.)*

The **component library is also linked in**, the same way and for the same
reason: `crates/frees-core/src/components/library-data/*.frees` is 122 KB of DSL
text embedded with `include_str!` and parsed by the ordinary front end. It is
**data — never hand-translate a component.** `engine.rs::expand_component_layer`
early-returns before touching any of it when a document declares no components.

Contract files (`ast.rs`, `token.rs`, `diag.rs`, `parser/mod.rs`, `units/quantity.rs`)
define fixed interfaces; change them deliberately, not incidentally. Unsupported
DSL blocks must fail with an explicit error — never silently skip
(`parser/toplevel.rs::unsupported_construct` is the seam; as of Phase 7 its list
is **empty** — every block form the grammar admits now has a home on
`Document`). Diagnostics are source-mapped and quote the user's text
(parent-engine rule).

**A solved `DYNAMIC` block puts nothing in `Solution::values`** — the trajectory
is a first-class `OdeTableResult` on `Solution::ode_tables`. Any check that
compares only `variables` therefore passes *vacuously* on a transient document;
`tests/parity.rs` compares `ode_tables` for exactly that reason, and
`fixtures/README.md` records the four perturbations that were used to prove the
comparison can fail.

**The replay also asserts that no fixture solves to a non-finite value** (Wave
T3), and that is *not* implied by matching the golden: `close`/`rel_diff` treat
NaN against NaN as agreement and infinities as exact hits — deliberately, so a
golden holding the oracle's own non-finite answer stays gradable — which would
otherwise let a fixture match while this engine produced garbage. It moved here
from `props_robustness`, where asserting it cost a second whole-corpus solve.
*A solution must be finite* in `fixtures/README.md` has the measurements and the
one deliberate behaviour change (each fixture is now checked under the settings
it ships with, not under bare defaults).

## What frees is, and what "browser-native" means here

**frees** (in `../frEES`, a separate git repository) is a web-based declarative equation-solving and acausal system-modeling environment: Java/Spring Boot backend + React 19/TypeScript frontend, deployed as Docker containers with RabbitMQ task dispatch and a Redis job store.

Today a solve is a network round-trip: editor text → `POST /api/solve` → RabbitMQ → compute worker → Redis → frontend polls `GET /api/jobs/{jobId}`. The goal of this project is to collapse that entire loop into the browser tab: the parser, unit checker, blocker, and solvers run as WASM, and the "server" disappears.

**`../frEES` is a read-only reference.** Read it freely to understand the engine; do not modify it from this repo.

## The reference implementation — read these first

| Path | Why it matters |
|---|---|
| `../frEES/CLAUDE.md` | Authoritative engine semantics: solver principles, the component/connect layer, the four domains, the deployment foot-guns. Read this in full before designing anything. |
| `../frEES/README.md` | Full system design + Agile plan. |
| `../frEES/docs/roadmap.md` | What's shipped vs deferred. Note: the WASM port is listed under **"Decided against (not deferred — closed)"** for frEES itself. |
| `../frEES/docs/critique-evaluation-2026-07.md` | The rationale for that decision — and the reason a port is *feasible*: "plausible long-term precisely because `core` is Spring-free by design". |
| `../frEES/backend/core/src/main/antlr/Frees.g4` | The 632-line grammar that defines the frees DSL. The port's source of truth for syntax. |

Because `../frEES` closed this port on its own roadmap, this repo is an independent exploration. Nothing here should assume changes will be accepted upstream.

## Port map: what carries over, what evaporates

The backend is a Gradle multi-module build, and the split was designed for exactly this kind of reuse:

- **`backend/core` (~134 files, ~38k LOC Java) is the port target.** Pure computation, zero Spring/AMQP/Redis: AST + parser, unit checker, Newton solver + Tarjan blocking, component expansion, ODE/DAE integrators, CAS, fluid/gas property models. Packages: `ast/`, `parser/`, `core/` (+`core/ode`, `core/dae`), `units/`, `props/`, `cas/`, `api/`, `measurement/`.
- **`backend/web` (~28 files, ~6.8k LOC) mostly evaporates.** RabbitMQ dispatch, Redis job store, `RequestGuardFilter` rate limiting, CORS, nginx proxying, client-IP trust — all of it exists because compute is remote. In-browser, none of it has a job. What survives is its **contract**: the REST endpoints define the surface the WASM module must expose as function calls.
- **The frontend is reusable as-is.** React 19 + TypeScript + Mantine (dark theme), CodeMirror editor, Plotly, Excalidraw whiteboard. The swap point is narrow: only **two** files issue HTTP calls — `frontend/src/api.ts` and `frontend/src/analyzer/measurementApi.ts`. That is the seam where a WASM binding replaces `fetch`.

### REST surface a WASM engine must cover

`/api/solve`, `/api/solve/table`, `/api/solve/montecarlo`, `/api/check`, `/api/repl/evaluate`, `/api/repl/clear`, `/api/optimize`, `/api/optimize/multi`, `/api/curve-fit`, `/api/plot/propplot`, `/api/plot/psychart`, `/api/control/plant`, `/api/control/pidtune`, `/api/reference`, `/api/fluids`, `/api/measurements/*`, `/api/jobs/{id}` (+ `/stream` SSE), `/api/health`.

The async job shape (`202 Accepted` + `jobId` + poll) exists only because of the broker. In-browser it becomes a Web Worker message round-trip — **keep the async shape at the `api.ts` boundary** so the UI doesn't need rewriting.

## Dependency substitution — the actual hard problem

Every `core` dependency needs a Rust/WASM answer. Ranked by risk:

| Java dependency | Used for | Port consideration |
|---|---|---|
| **Symja** (`matheclipse-core`) | CAS: Factor/Apart/Laplace/InverseLaplace/Diff/Integrate, symbolic `ss↔tf` | **Highest risk.** Large pure-Java CAS with no equivalent Rust crate. Options: keep CAS server-side (hybrid), reimplement the narrow subset actually used (`cas/` is 11 files), or evaluate `symbolica`/`egg`. Decide this early — it may define the project's scope. |
| **CoolProp** (via JNA) | Real-fluid + humid-air properties | **Lowest risk, highest value.** The binding surface is only four C functions: `PropsSI`, `Props1SI`, `HAPropsSI`, `get_global_param_string` (`props/CoolProp.java`, 215 lines). CoolProp is C++ and has an established Emscripten build — compile to WASM and keep the same four-call façade, including the existing LRU caches. **Both halves are settled:** D8/D9 replaced the Emscripten plan with rustprop, and **Wave R2 (2026-08-25) ported the LRU cache** — at **capacity 1**, not the Java's 20 000, because `rustprop_warm` seeds each flash from the previous answer, so an answer is a function of its arguments *and* of the seed. Capacity 1 keeps 99.72 % of hits bit-identical to the live call; capacity 2 keeps 65 %. Wave S1 built the seed-identity tag that would lift the cap and **declined** it: the tag does keep every seeded hit bit-identical, but it is worth only 1.3–1.6 points of hit rate — not the 13 R2 estimated, because those were the dishonest hits — and every tagged configuration that held capacity 1's fidelity ran **5.1–6.5 % slower**. Do not widen it without re-reading `props/propfun.rs::cache`. |
| **SUNDIALS IDA + KLU** (via JNA) | Transient DAE solve, sparse steady | C library; Emscripten-compilable, or replace with a Rust integrator. Note the parent repo's SUNDIALS-v6-vs-v7 ABI trap (`SUNContext`, MPI linkage) — a WASM build sidesteps the distro-version problem entirely. Bindings are small: `SundialsIda.java` (207) + `SparseSteadyKlu.java` (151). |
| **ANTLR 4** | `Frees.g4` → parser/visitor | Rewrite as a Rust parser (`pest`, `chumsky`, `lalrpop`, or hand-written recursive descent). The grammar file is the spec; `parser/AstBuilder.java` shows the intended AST shape. |
| **Apache Commons Math** | Newton–Raphson, Jacobians, SVD, Brent, eigen-decomposition, LQR Riccati, LM curve fitting | `nalgebra` / `faer` for linear algebra, `argmin` for optimization. Watch numerical parity — the solver's step-halving behavior is load-bearing. |
| **JGraphT** | Tarjan SCC blocking | `petgraph` has Tarjan SCC directly. Straightforward. |
| **mdf4j** | ASAM MDF4 measurement files | Optional for a first cut; the parent already isolates it behind a `MeasurementParser` interface and has an out-of-process `mdf-sidecar`. |
| **Jackson** | JSON payloads | `serde` / `serde_json`. The DTOs in `api/SolveDtos.java` define the wire format the frontend already parses. |
| **FOP transcoder** | SVG rendering | Browser renders SVG natively; likely deletable. |

## Engine invariants to preserve (from `../frEES/CLAUDE.md`)

A port that breaks these isn't frees:

- It is an **equation solver**, not a sequential language — equations are order-independent.
- Variable names are **case-insensitive**.
- **All calculations run in SI**; unit-annotated inputs convert at parse time, computed variables get dimensionally derived SI units, and **unit warnings never block solving**.
- Types come from naming convention: `$` → string, `#` → constant, `[]` → array, `_r`/`_i` → complex components.
- Solve = Tarjan SCC blocking → per-block Newton with step-halving.
- The **component layer is a parser/expander, not a second solver** — `COMPONENT`/`connect` expands to scalar equations that flow through the same Newton/Tarjan path. ~295 components ship; connector-domain separation is a hard parse error by design, not a warning.
- Diagnostics are source-mapped to component names, never to mangled scalars.

## Browser-specific design constraints (new, no upstream precedent)

- **Solving must not block the UI thread** — the compute engine belongs in a Web Worker; `api.ts` becomes a `postMessage` shim wearing the current async interface.
- **Threads need cross-origin isolation.** `wasm-bindgen-rayon`/SharedArrayBuffer require COOP/COEP headers. Decide early whether to require them, because it constrains hosting and embedding.
- **Payload size is a product constraint.** CoolProp's fluid data plus the component library is not small; plan for lazy-loaded WASM chunks and measure. The parent has no bundle-size budget to inherit — set one.
- **Memory is bounded** (wasm32 = 4 GB ceiling, realistically far less). The Java engine assumed a JVM with `maxHeapSize = 2g` for tests.
- **`f64` determinism differs** from the JVM. Where numerical parity matters, port the parent's tests as the oracle (`../frEES/backend/core/src/test/`).

## Working with the reference repo

```bash
# Run the reference stack to compare behavior (Docker; never start host processes for it)
cd ../frEES && ./frees.sh start        # frontend :5173, backend API :8080/api
cd ../frEES && ./frees.sh logs         # follow
cd ../frEES && ./frees.sh stop

# Reference engine tests — the behavioral oracle for the port
cd ../frEES/backend && ./gradlew :core:test
cd ../frEES/backend && ./gradlew :core:test --tests "com.frees.backend.core.EquationSystemSolverTest"
```

Module-qualify single-test runs (`:core:` / `:web:`); an unmatched module fails the whole build.

`../frEES` has CodeGraph initialized (`.codegraph/`). Per the global instructions, explore it via an Explore agent using `codegraph_explore` rather than calling that tool in the main session.

## Toolchain state (as of this file's creation)

Nothing WASM-related is installed yet. `rustc 1.75.0` exists at `/usr/bin/rustc` (a distro/source-tarball build), but there is **no `cargo`, no `rustup`, and no `wasm-pack`**, and no `wasm32-unknown-unknown` target. `node v20.20.2` / `npm` are available.

Setting up `rustup` (which also supplies a current `cargo` and the wasm target) is step zero of any implementation work. Do not write build instructions that assume a toolchain that isn't installed — install it, verify it, then document what actually ran.
