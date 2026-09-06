# Phase 12 — parity at scale, performance, hardening

**Read this after [`status-phase11.md`](status-phase11.md).** Phase 12 is the
last row of PLAN.md's plan: grow the parity corpus toward the full Java test
surface, benchmark against the JVM engine, fuzz the hostile-input surfaces,
break the bundle down by name, and close the panic/memory story on the wasm
boundary. No product feature ships in this phase; what ships is **evidence**.

The engine changed in exactly zero lines this phase. Everything below is
tests, fixtures, benches, tooling and documentation — which is why the corpus
could grow 32 % with the parity gate green on the first full replay.

---

## Gate numbers, all raw

| Gate | Result |
|---|---|
| `cargo test --release --workspace` | **3162 passed, 0 failed, 6 ignored** across 26 suites, exit 0. The delta reconciles exactly: 3155 (Phase 11) + 7 fuzz properties = 3162; nothing else moved |
| `cargo test -p frees-core --test parity` | `golden_corpus_parity` **ok** — all **701** fixtures match the Java oracle, 165 s (531 took 41 s; the growth is real documents, not padding). *Post-phase: 702 since 2026-08-06 — `heisler-transient` promoted by the `StringVariables` port.* |
| `cargo test -p frees-core --test fuzz_properties` | **7 properties pass** at CI case counts; a `PROPTEST_CASES=4096` soak (≈15 500 generated inputs) also passes |
| `cargo clippy` (native + wasm32, `--all-targets`) | exit 0 on both — including the new bench/fuzz targets, which are cfg-gated off wasm32 (proptest's `getrandom` refuses that target; found by running the gate, not by reading docs) |
| `cargo fmt --all --check` | exit 0 |
| `./node_modules/.bin/vitest run` | **40 files, 394 passed** (Phase 11: 39/388; +1 file/+6 is `engineClient.test.ts`) |
| `cargo bench -p frees-core --bench solve_bench` | five benches, table below |
| wasm bundle | **untouched at 2944 KiB** — the phase adds only dev-dependencies, and `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]` makes that a checked property, not an assumption |

New dependencies: **zero runtime**, two dev and native-only — `proptest` 1.x
(minus default features) and `criterion` 0.5 (minus default features).

---

## 1. The corpus: 531 → 701, and what the "1,237" really was

PLAN.md's Phase 12 row says "full 1,237-test corpus replay". Measured against
today's reference checkout, **that number is stale**: `backend/core` now has
**184 test files / 1,179 `@Test` methods** (some were consolidated upstream
since PLAN.md was written). More importantly, the right unit is not tests but
*documents*: only **~625** of those tests carry a `.frees` document that can
become a corpus fixture; the rest are unit tests of Java internals
(`DifferentiatorTest` alone is 65 tests of a class this port re-implements
behind the same fixtures).

The harvest (`tools/harvest-java-tests/harvest.py`, run against the live
oracle with the **macOS CoolProp dylib** — see below):

| Stage | Count |
|---|---|
| candidates extracted from 13 Java test classes | **212** |
| dropped at golden review (settings-bearing tests, `String.format` templates, `~ignored~N[` array sinks, one scalar-`~` leak) | 21 |
| kept and replayed | **191** |
| **passed parity at default tolerance, promoted** | **170** |
| failed, classified, staged in `corpus-pending/` | 21 |

The 21 failures divide cleanly — and the division is the finding:

* **6 are new witnesses of already-documented divergences**, not new bugs:
  five hit the `linalg::svd` column-sign convention (same mechanism as the
  recorded `estimator-gramian-balreal` hold; sign-only flips, invariants
  identical), one hits the pre-Newton property-probe divergence (same as
  `ev-battery-cooling-pid`).
* **12 are unported features**: `HAPropsSI` humid air (5), D1 untabulated
  fluids `Air`/`INCOMP::MEG` (4), and — the one genuinely new gap this
  harvest found — **`CALL eigenvalues`/`eigen` is not wired** (3 documents;
  ledger item 34). *(Since closed: the humid-air five and the fluid four by
  D9/rustprop on 2026-08-18, the eigen three by Wave A1 on 2026-08-21.)*
* **3 are oracle/comparison artifacts**: two D1 table-accuracy cases inside
  the documented band (worst 7.9e-8), promotable only with measured
  `tolerances.json` entries that were deliberately not added in bulk; one
  asymptotic-FP case where a column decays to machine zero and the engines
  sit one 4.5e-12 quantum apart.

Zero unexplained failures. The full triage, including the five-witness sign
table, is in `fixtures/README.md`'s Phase 12 re-check note.

**The macOS oracle.** This phase ran the Java oracle on a Mac for the first
time. Four traps, all now recorded in the README/status docs: `FREES_HOME`
defaults to a Linux path; the core jar was unbuilt; **the vendored
`libCoolProp.so` is a Linux ELF that JNA silently fails to load** — the fix
is `COOLPROP_LIBRARY=/usr/local/lib/libCoolProp.dylib`, verified against the
known ground truth (`Enthalpy(Water, 300 K, 101 325 Pa)` →
`112654.8996546125` vs the Linux oracle's `…4505` — a ~2e-12 CoolProp-build
difference, three orders below the parity tolerance); and SUNDIALS is not
installed, so `fixtures/dae-oracle.json` is **frozen on this machine**.

Also found and recorded (README + ledger item 35): an explicit `~` discard in
destructuring (`[whole, ~] = DivMod(…)`) leaks Java's JVM-global sink counter
into `display_names`, so such fixtures can never be frozen — a correction to
the README's previous "scalar sinks are safe" rule, which covers only
omitted-trailing sinks.

---

## 2. Property-based fuzzing (`tests/fuzz_properties.rs`)

The hand-written robustness suites (565 tests) encode chosen adversarial
cases; this file generates them. Seven properties over four surfaces:

* **Unstructured**: arbitrary unicode; arbitrary bytes lossily decoded.
* **Structure-aware documents**: a grammar strategy emitting parseable
  documents (unit annotations, FOR/GUESS blocks, function calls, matrices,
  builtin-colliding identifiers, `1e309`-class numbers) — these reach the
  blocker and Newton, where random bytes never arrive. Plus a determinism
  property: solving twice gives bitwise-identical results.
* **Unit annotations**: arbitrary printable text inside `[…]`.
* **MDF4**: byte-splicing and **aligned 64-bit link forging** inside the
  genuine asammdf recording, so the block-graph pre-flight engages with
  plausible structure instead of refusing at the header.

All seven pass at CI counts and at an 8× soak. That is the *expected* result
— it is the same contract the 565 hand-written tests enforce, now checked
over ~15 500 generated inputs per soak — and the honest reading is coverage
confirmation, not proof of absence: proptest is not coverage-guided. The
file's header requires any future minimized counterexample to be promoted
into the matching hand-written suite as a named regression.

---

## 3. Benchmarks: the port vs the JVM oracle

`benches/solve_bench.rs` (criterion, end-to-end through public `solve`) vs
the JVM driven through the golden dumper — same five documents, 50 copies
each to amortize startup, 2.87 s baseline subtracted, this machine:

| document | what it exercises | Rust (native) | JVM oracle | ratio |
|---|---|---|---|---|
| `scalar_two_block` | parse → block → Newton | **147 µs** | ~8.2 ms | ~56× |
| `rankine_cycle` | real-fluid property tables | **771 µs** | ~49 ms | ~63× |
| `component_mvem` | library expansion + mixed system | **776 µs** | ~8.8 ms | ~11× |
| `transient_dyn` | DYNAMIC → ODE integrator | **133 ms** | ~132 ms | **~1.0×** |
| `control_lqr` | state space, LQR, CAS helpers | **2.97 ms** | ~22 ms | ~7× |

Caveats, stated rather than buried: the JVM numbers include the dumper's
JSON serialization and only partial JIT warmup (50 iterations), so treat the
ratios as **user-visible-latency** comparisons, not microarchitectural
claims. The transient row is the honest anchor: ~1.0×, because the
integrator was ported line-for-line and both engines spend the time in the
same algorithm — which is exactly what D1/PLAN predicted (the port wins on
call overhead and property-table interpolation, not on numerics). The real
product win remains the term that is *absent* from the table: the network
round-trip every JVM solve pays in production and the wasm solve does not.

Not measured: the same benches inside the browser. Phase 9's in-browser CAS
timings (103 ms `Expand` in the shipped REPL vs 27 ms native) suggest a
2–4× wasm-vs-native factor; measuring it properly is future work.

### Measured 2026-08-23 (Wave G5): the same five documents in chromium

`web/bench/wasm-bench.spec.ts` (own Playwright config,
`playwright.bench.config.ts` — deliberately outside the CI-run `e2e/` dir,
for the same reason item 5 below keeps benches out of CI). The page imports
the wasm-pack output directly and times the boundary's `solve` export on the
main thread — no worker round-trip, comparable to the native bench's
end-to-end `solve`. Per document: 3 warmup calls, then single-call samples
until 2 s or 200 iterations. Same machine as the table above:

**CORRECTED the same day — the first run's numbers were contaminated.** The
original G5 table (1.20 / 9.50 / 6.00 / 223.8 / 16.3 ms, and two "findings"
drawn from it: a 1.7–12× wasm factor and a transient row that "inverts
against the JVM") was measured while a full `cargo test --release
--workspace` was running on the same machine. Re-measured idle, the
non-transient rows halve to a third and the "inversion" disappears — it was
machine load, not wasm. What follows is the corrected record; the
contaminated table is kept only in this paragraph so the correction is
checkable against the G5 commit message.

Corrected, idle machine, same day — Wave G5 corrected + the Wave G3
per-step-cache delta, both measured with the same spec:

| document | native (criterion) | wasm pre-G3 | wasm post-G3 | ×native |
|---|---|---|---|---|
| `scalar_two_block` | 70.8 µs | 0.80 ms (min 0.50) | 0.80 ms (min 0.40) | ~11× — see the boundary note |
| `rankine_cycle` | 1.41 ms | 3.10 ms (2.70) | 3.00 ms (2.60) | ×2.2 |
| `component_mvem` | 738 µs | 2.00 ms (1.80) | 2.00 ms (1.70) | ×2.7 |
| `transient_dyn` | 30.2 → **14.9 ms** (G3, ×2.0; 13.8 after G3b's Newton scratch reuse — a measured ~8 %, and G3b's commit records the negative result behind it) | 60.3 ms (50.7) | **23.9 ms** (22.5) | ×1.6 |
| `control_lqr` | 3.04 ms | 5.70 ms (5.10) | 7.4–7.6 ms (5.40) | ×1.9 (by min) |

The honest findings, corrected: the wasm-vs-native factor on real documents
is **~1.9–2.7×** — inside the 2–4× the REPL timings suggested all along.
`scalar_two_block`'s ~11× is not the engine: the wasm export parses and
serializes the REST-shaped JSON envelope per call while the native bench
calls `solve()` directly, and on a 147 µs document that fixed ~0.7 ms
boundary cost *is* the number. The transient never inverted: even pre-G3 the
browser's 60.3 ms beat the JVM's ~132 ms. `control_lqr`'s post-G3 median
wanders 5.7–7.6 ms across runs with the min stable at 5.1–5.4 — compare by
min. Native baselines are same-day criterion runs, not the table above:
those 2026-08-05 numbers predate D9 (rustprop made `rankine_cycle` ~1.35×
dearer — the F7 record) and A5's per-step cache (which took `transient_dyn`
133 → 30 ms before G3 took it to 14.9). Caveats that stand from the first
run: chromium quantizes `performance.now()` to 100 µs; single-call samples;
shipped-artifact numbers, not wasm's ceiling.

### Measured 2026-08-24 (Wave A2): where the per-step transient path's instructions actually go

Round 4 of the per-step work, and the first time the **remaining** recorded
lever — interned/indexed variable access — was measured rather than assumed.
Callgrind, `--cache-sim=no --branch-sim=no`, against the binary at `9924b0f`.
Instruction counts, not seconds: three other agents were compiling on this
machine and wall-clock was useless (the *same* binary ran
`docs_tutorials_05` in 8.99 s and 13.11 s twenty seconds apart).

**The string-keyed `Scope` really is the cost — on exactly two of four
workloads.** Every figure below is a share of that document's whole run:

| workload | total Ir | hashing + probing + key compare |
|---|---|---|
| `hot-transient` (2 160-step ode45 stand-in) | 532.8 M | **27.8 %** |
| `hot-stiff` (the hostile MAX_STEPS regime) | 4 256.4 M | **27.8 %** |
| `ev-battery-cooling-pid` (span cut to 0..0.4, 2 points) | 24 264.6 M | **12.5 %** |
| `pressure-cooker` (property-bound algebra) | 5 522.2 M | **0.2 %** |

Broken out on `hot-transient`: reads through `Env::get` are 72.1 M (13.5 %)
over 659 898 calls, **~109 Ir to fetch one `f64` by name**; writes through
`HashMap::<String, f64>::get_mut` are 60.1 M (11.3 %) over 598 968 calls,
~100 Ir each; the separable remainder inside `solve_block` and
`PreparedPinnedSolver::solve` is 14.4 M (2.7 %). Disassembling `Env::get`
accounts for its 71 Ir of *self* cost exactly: a six-register frame, an
out-of-line `hash_one` call for a four-byte key, an indirect `bcmp` through
the GOT, and **six register spills** LLVM inserts around that call because it
clobbers the probe loop's state.

The last two rows are the finding that bounds the lever. `pressure-cooker`
spends **69.3 %** in `rustprop_heos::alphar_all` and 7.8 % in `pow`;
`ev-battery-cooling-pid` spends 38.5 % in `alphar_all` and 8.7 % in libm.
Property-bound documents are not scope-bound, and no amount of interning
touches them. Worth recording alongside it: timing all 983 corpus documents
individually, **`ev-battery-cooling-pid` alone is 92.2 s of user time** and
the next-heaviest is 8.6 s — so the replay's own duration is one document,
and that document is property-bound.

**What was built** — `crates/frees-core/src/solver/slots.rs`, the scoped
version: a block's residuals and derivative entries compile **once**, in the
Wave-G3 block cache, to postfix programs over a dense slot vector, and
`solve_block` fills that vector from the `Scope` once per call instead of
per access. Arithmetic only (`Num`/`Var`/`Neg`/`Not`/`BinOp`/`Compare`/
`Logical`); anything else — every `Call` included — refuses to compile and
the block keeps the `Expr`/`Scope` path verbatim, so the two never mix. The
module header carries the byte-identity argument. Measured, cumulative:

| | `hot-transient` | `hot-stiff` |
|---|---|---|
| baseline (`9924b0f`) | 532 799 194 | 4 256 417 733 |
| compiled blocks | 486 467 370 (−8.70 %) | 3 886 812 306 (−8.68 %) |
| + fused `lhs − rhs`, caller-sized stack, pin-only literal refresh | **464 988 465 (−12.73 %)** | **3 708 586 185 (−12.87 %)** |

The second increment is worth −4.4 % on its own: the first cut's postfix
interpreter was *no cheaper* than the recursive tree walk it replaced
(`eval_program` alone was 13.7 % of the run), because it evaluated `lhs` and
`rhs` as two programs and pushed and popped through a growable `Vec`.

**`dyn_accessor_live`, the corpus's last cost hold**, run as back-to-back
base/new pairs on the same loaded machine (byte-identical output, same
`md5sum`): 237.13 → 201.00 s user (−15.2 %) and 268.05 → 216.47 s
(−19.2 %). Both base runs are well above I2's 183.5 s quiet-machine anchor,
which is the load showing; scaling the anchor by the measured ratios puts
the document at **~150–156 s**. That is a real cut and it is **not enough to
promote the hold** — the gate it has to fall under is the replay's own
~136 s, and it is still above it.

**Three negative results, all measured, none kept:**

1. **`#[inline]` on `Env::get`: −0.04 %.** Nothing. The probe is already the
   cheapest part of it.
2. **A `refill(buffer, n, value)` helper** replacing the five `clear();
   resize(n, v)` pairs across `newton.rs` and `solve_block` (identical
   postcondition, skipping the length/capacity bookkeeping a per-step solve
   pays three times per call): **−1.5 %.** Real and reproducible, but under
   the standing ~3 % bar, so it was reverted rather than carried. It is the
   cheapest thing left on the list if the bar ever moves.
3. **A measurement trap worth more than either.** `cargo build --release -p
   frees-cli` does **not** unify features, so `frees-core` builds without
   `rustprop-backend` and takes the (P,h) table path. `hot-transient` then
   measures **604.4 M Ir — 13.4 % higher — with no source change at all**.
   Always build the binary you are profiling with `--release --workspace`.

**Where the floor is now.** The compiled path still pays exactly two hashed
accesses per `solve_block` call — filling the slot vector, and writing the
answer back — measured at 51.4 M, **11.1 % of the new total**. That is this
design's floor, and the only way past it is for the prep's own `work_scope`
to become slot-native, which is a much larger change than this one and was
deliberately not attempted. The next-largest remaining items on
`hot-transient` are the allocator at 10.6 % (the per-call `Vec<&Equation>`
is one `malloc` per `solve_block` call, 211 322 of them) and `eval_program`
itself at 12.6 %.

---

## 4. The worker-death path, finally tested

`web/src/wasm/engineClient.test.ts` drives the **real** engineClient
singleton with a fake Worker — every prior suite mocked the module away. Six
tests pin the product's only recovery mechanism under `panic = "abort"`:
correlation by id (answered out of order), a worker `error` mid-flight
rejects **all** pending promises and terminates the corpse, **the next call
respawns**, `messageerror` fails over identically, a stray-id response is
ignored, an `{ok:false}` response rejects one call while keeping the worker,
and measurement bytes ride the transfer list (`transfer === [buffer]`), not
a structured clone.

On the "memory ceiling" row of PLAN's Phase 12 scope: the survey confirmed
`RETAINED_BYTES_BUDGET` (512 MiB, measurement) is the only byte-denominated
guard, and that the sweep/Monte-Carlo paths PLAN wanted guarded are **still
unreachable from the boundary** (Phase 8's gap). There is nothing to guard
until they are wired; writing a guard for dead code would be theater. The
structural bounds (18+ ceilings, catalogued in the survey and status docs)
plus worker respawn are the shipped memory story.

---

## 5. Bundle breakdown, by name

The tree finally has a **named, per-module breakdown** — something no
previous phase produced (`status-phase10.md` measured by differential build
because the shipped artifact strips its name section). Method: `twiggy` over
the **pre-bindgen** release artifact
(`target/wasm32-unknown-unknown/release/frees.wasm`, 4,458,455 B — the
one build that still carries names; the shipped 2944 KiB is this minus the
name section, wasm-bindgen processing and `wasm-opt -Oz`, so treat the
proportions as ranking, not shipping bytes):

| Component | bytes | % of named artifact |
|---|---|---|
| name section (stripped from the shipped artifact) | 1 012 611 | 22.7 % |
| `.rodata` — the two `(P,h)` property tables (~526 KB) + the embedded component library (~122 KB) + diagnostic strings | 890 222 | 20.0 % |
| `frees_core::cas` | 297 571 | 6.7 % |
| `frees_core::parser` | 291 925 | 6.5 % |
| `frees_core::control` | 231 626 | 5.2 % |
| `frees_core::eval` | 176 543 | 4.0 % |
| `core` (Rust language runtime) | 150 862 | 3.4 % |
| `frees` (the boundary itself) | 146 351 | 3.3 % |
| `frees_core::props` (code; tables are in `.rodata`) | 122 652 | 2.8 % |
| `alloc` | 115 840 | 2.6 % |
| `frees_core::components` (code; library text in `.rodata`) | 110 358 | 2.5 % |
| `frees_core::ode` | 99 816 | 2.2 % |
| `num_bigint` (CAS exact arithmetic) | 90 668 | 2.0 % |
| `frees_core::engine` | 90 213 | 2.0 % |
| `frees_core::ast` | 65 242 | 1.5 % |
| `mf4_rs` | 55 798 | 1.3 % |
| `frees_core::measurement` (code) | 54 368 | 1.2 % |
| `linalg` / `units` / `serde_json` / `libm` / `analysis` / `solver` | 48 590 / 48 298 / 44 214 / 42 385 / 41 754 / 31 990 | ~0.7 % each |
| everything else (petgraph, num_rational, dlmalloc, std, small modules) | ~57 000 + tail | — |

Reading it: **the two known debts are confirmed as the two biggest levers** —
`.rodata`'s property tables dominate shippable data exactly as the CI comment
claims, and no single code module exceeds ~7 %, so code-splitting buys little
until the CAS/control pair (~12 % together, REPL-only surfaces) is split with
the data. On the shipped artifact, `twiggy dominators` names the exports:
`solve` retains 215.8 KB, `repl_evaluate` 145.6 KB, `measurement_calc`
57.9 KB, `check` 23.6 KB, and the indirect-call `table[0]` 337.9 KB.

The budget gate is untouched: 2944 KiB raw against 3072 KiB, 95.8 %, and the
two recorded debts (fetch the property tables, split the engine) remain the
only real levers. This phase added nothing to the bundle and proved it by
gating the new dev-dependencies off the wasm target in the manifest.

---

## What Phase 12 did **not** deliver — ranked

1. **The corpus is 701, not "everything".** Remaining Java documents are the
   shapes the harvester cannot represent: tests passing extra `solve(...)`
   arguments (complex mode, `ProcDef` function tables — the biggest block),
   `String.format` templates, cross-file constants. Plus the two untouched
   growth sources: `../frEES/frontend/src/docs/*.md` snippets and the 295
   component library sources. A follow-up needs harvester features, not more
   sweat.
   *(Closed 2026-08-24, Wave I — with one correction. The harvester grew the
   features: `.formatted`/`String.format` evaluation, helper-method inlining,
   a cross-file constant registry, and `FunctionTableDef` extra-defs staged
   as `.tables.json` sidecars that flow dumper → fixture `function_tables`
   field → `tests/parity.rs` → core's `solve_with_tables` (the Wave-H
   channel). 77 promoted, corpus 905 → 982 — including all 10
   `CurveFunctionTest` request-table documents, the item's "biggest block",
   bit-exact at the corpus default. The correction: an inventory over all
   197 test classes showed the three named shapes were never the bulk of the
   remainder — cross-file constants resolve **zero** solve sites, complex
   mode is 9 sites and formatted templates ~52 documents, while ~100 classes
   with plain representable documents were simply never in the harvester's
   class list. The docs snippets and component sources were separately
   closed by Waves D1 and G4. `fixtures/README.md` growth item 3 has the
   full accounting, the 18 property-chain pendings, and the four new
   ledger-35 witnesses.)*
   *(And the sweep that correction called for is done — 2026-08-24, Wave J.
   Measured, not estimated: **138 of the 197 test classes hold a harvestable
   document; 23 were in `CLASSES` and 115 were never listed at all** (Wave I
   guessed ~100; 114 are now swept by name and one is a deliberate
   `SKIP_CLASSES` entry), across 540 solve-call sites — 504 resolving — and
   337 text blocks. The class list stopped being the sweep: `harvest.py` now
   walks the whole test tree and `CLASSES` only pins the pre-Wave-J prefixes,
   with the unrepresentable *sites* dropped by tag instead of staged. 285
   candidates staged, 284 goldened, **255 promoted — corpus 983 → 1238** — 19
   numeric holds staged into `corpus-pending/`, 11 classified in
   `fixtures/README.md` (five ≈0-superheat denominator collapses, five engine
   divergences incl. the new ledger item 38, one the oracle cannot golden at
   all). Three further documents were declined on fixture *size*: a new
   `MAX_DECLARED_POINTS` guard refuses a `DYNAMIC` grid denser than 2 000
   samples, because at `points = 60001` the golden is **139 MB** (the corpus's
   largest was 640 KB) and the corpus-wide sample-ceiling assertion in
   `dynamics_robustness` breaks. This item is now closed on every axis it
   named: what is left un-harvested is 37 tagged-unrepresentable sites, 36
   unresolvable arguments, 3 oversampled documents and four ledger-35 tilde
   documents. The one cost is time — the release replay is **353.3 s** where
   983 fixtures took ~145 s, and that figure was taken on a machine carrying
   a load average of 9–20 from a parallel agent, so it is an upper bound.)*
2. **`CALL eigenvalues`/`eigen` found unwired and left unwired.** This was a
   hardening phase; wiring a new CALL belongs with the Phase 8 backlog. The
   three documents wait in `corpus-pending/` as the acceptance test.
   *(Closed 2026-08-21, Wave A1: wired and all three promoted — ledger
   item 34.)*
3. **No browser-side benchmark.** Native-vs-JVM only; the wasm factor is
   inferred from Phase 9's REPL timings, not measured on these documents.
   *(Closed 2026-08-23, Wave G5: `web/bench/wasm-bench.spec.ts` measures the
   five documents in chromium — corrected the same day after the first run
   proved load-contaminated: the real factor is ~1.9–2.7× on real documents,
   a fixed ~0.7 ms JSON-boundary cost dominates the trivial one, and the
   transient never inverted against the JVM. §3 above has the corrected
   table, the contamination note, and the Wave G3 delta.)*
4. **The DAE surface is still un-fuzzed at API level**, and the SUNDIALS
   oracle cannot be regenerated on this machine (not installed) — its
   `ORACLE_*` constants are effectively frozen until someone installs
   SUNDIALS ≥ 6 with KLU.
   *(Closed 2026-08-23, Wave G6 — both halves. The "not installed" had
   rotted: Ubuntu 24.04's `libsundials-dev` 6.4.1 with `sunlinsolklu` was
   already present, and `tools/dae-probe/run.sh` reproduces the committed
   `fixtures/dae-oracle.json` **bit-identically** against live SUNDIALS
   IDA + KLU, so the `ORACLE_*` constants are re-measurable and freshly
   validated. The API-level fuzz is `tests/dae_robustness.rs`: proptest
   over the raw `IdaDaeSolver` contract — hostile driver sequences,
   closed-form grading of a linear DAE, consistent-IC repair of an
   inconsistent algebraic start, root location/direction, a poisoned
   mid-integration residual, sparse-vs-dense agreement above
   `SPARSE_THRESHOLD`, and typed misuse refusals. 7 properties, green at a
   4096-case soak and in debug.)*
5. **Benchmarks are not a CI gate.** They run locally (`cargo bench`); no
   regression threshold exists. Deliberate for now — criterion noise on
   shared CI runners produces flaky gates — but it means a performance
   regression is caught by a human re-running the table, or not at all.
   *(Re-assessed 2026-08-26, and half of the worry turns out to be already
   covered. The benches cannot **rot**: `cargo clippy --workspace
   --all-targets`, which CI runs, compiles `benches/solve_bench.rs` — verified
   by appending a call to a non-existent function to it, which produced 2
   clippy errors under `-D warnings`, then reverting. So a bench that stops
   building fails the build today. What is still absent is a **timing**
   threshold, and that stays declined on the original reasoning: criterion on a
   shared runner measures the runner. If a regression gate is ever wanted, the
   honest form is instruction counts under callgrind, which is what Waves
   Q3/R3/A2 used precisely because wall clock on this hardware was useless —
   the same baseline binary ran `dyn_accessor_live` in 168.23 s and 159.24 s.)*
6. **Fuzzing is not coverage-guided.** proptest generates blind; a
   `cargo-fuzz`/libFuzzer target with the same oracles would explore deeper.
   The repo still has no nightly toolchain requirement, and adding one for
   fuzzing was judged not worth it this phase.
   *(Re-assessed 2026-08-26. Still open, and still a decision rather than a
   chore: `rust-toolchain.toml` pins **stable**, and `cargo-fuzz`/libFuzzer
   needs nightly, so this cannot be picked up without changing what every
   contributor and every CI job installs. The cheap half-measure — a scheduled
   deeper `PROPTEST_CASES` soak — is not wired either; CI runs proptest at its
   defaults, with no `PROPTEST_CASES` anywhere in the workflow. Whoever takes
   it should decide the nightly question first; the fuzz targets themselves are
   fine.)*
7. **Carried forward from Phase 11**: no CI job runs the service worker
   offline; the remote-fallback adapter stays unwired; the precache-size
   opt-out was explicitly deprioritized (features may be clipped instead).
   *(Two of these three shipped and nobody annotated the item — corrected
   2026-08-26. **The offline service-worker gate exists**: Wave D4 made the
   Playwright offline PWA proof a CI step, and it runs in the `web` job as
   "Offline PWA proof (Playwright)". **The precache-size opt-out shipped** as
   the data-saver opt-out in the C/D web wave. What remains open is the
   **remote-fallback adapter**, and it is open by design, not by neglect:
   PLAN.md scope decision 2 makes the remote path an optional adapter and never
   a requirement, so wiring it is a product decision — it would send document
   text off the machine, which is the property this whole port exists to
   remove.)*

---

## Divergences opened by this pass

Ledger items **34** (`CALL eigenvalues`/`eigen` unwired — closed 2026-08-21,
Wave A1) and **35** (the
destructuring-`~` sink-counter leak, an oracle-side authoring hazard) in
[`status-phase1.md`](status-phase1.md#opened-by-phase-12-2026-08-05). The six
new failing witnesses of items already on the ledger are recorded against
those items in `fixtures/README.md`, not double-counted here.
