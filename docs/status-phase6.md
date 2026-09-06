# Status — Phase 6 complete (the component / connect layer)

**Date:** 2026-07-31 · Read after [`docs/status-phase5.md`](status-phase5.md),
which this supersedes as the current state.

Phase 6 ports `backend/core`'s acausal component system — `ComponentExpander`
(1,656 LOC), `ComponentLibrary`, `ast/{ComponentDef,ComponentInst,ConnectDecl}`,
`api/ComponentMetadata` and `api/CyclePathResolver`, 2,667 LOC of Java — and
ships the **295-component standard library as data**: the same 13 `.frees` files
the reference repo keeps in `resources/components/`, vendored verbatim,
`include_str!`d, and parsed by the ordinary front end. Not one component is
hand-translated.

```
2055 Rust tests passed, 0 failed, 2 ignored (21 suites)
361/361 golden fixtures match the Java oracle
   (17 of them at a declared, guarded table tolerance — 12 new this phase)
clippy -D warnings clean (host and wasm32-unknown-unknown)   cargo fmt clean
wasm 2184.5 KiB raw / 1086.2 KiB gzipped   (budget 3072 KiB raw — 71.1% used)
web 341 tests / 36 files green, vite build green
Browser proof: a 7-component thermofluid network and a 5-component connect(...)
   network solved in-tab against the Java golden, the component datasheet
   rendered from the AST, ZERO /api/ requests
```

Every number above was measured in this pass, raw, not carried forward. See
[Gate evidence](#gate-evidence).

---

## What Phase 6 delivers, by area

`crates/frees-core/src/components/` is **11,382 lines across 8 modules with 238
inline tests**, plus three integration suites (`component_library.rs` 6,
`component_families.rs` 4, `component_robustness.rs` 58).

| Area | Module | LOC / tests | What works |
|---|---|---|---|
| **The expander** | `expander.rs` | 5,127 / 80 | `ComponentExpander` line for line: definition resolution with built-in shadowing, positional **and** free-port (`connect`-wired) instantiation, parameter binding and substitution, hierarchical-subsystem flattening with namespacing, body rewriting onto flat scalar names, `rewriteStatements` so a dotted `P1.out.h` inside a `FOR` body is rewritten at the AST level, the union-find spanning-tree argument that keeps a closed loop square, the C-R-C index-2 rejection, and fluid-identity propagation across connected sets. |
| **The domain rules** | `domains.rs` | 1,761 / 54 | The physics core, and **pure**: `nodeDomain`'s exact priority order (through-variables first, then across-variable fallbacks, then the fluid default), `acrossMembers`, `checkSingleDomain`, `checkFluidConnectorType`, `kirchhoffBalance`, `massConservation`, `portDirection`, `portFluid`, and the `CANONICAL_UNITS` member-unit table that feeds `componentMemberUnits`. Domain separation is a **hard parse error**, as the parent engine specifies. |
| **The library** | `library.rs` + `library-data/` | 870 / 24 + **122,260 bytes of `.frees` text** | 13 domain files embedded and parsed by [`parse_document`], one file at a time so every definition remembers which file it came from. `source()` reproduces the Java's `"\n\n"`-joined concatenation byte for byte, and `the_library_parses_concatenated_exactly_as_java_feeds_it` parses that too. 295 components, asserted. |
| **Variants** | `variant.rs` | 473 / 15 | `model$` selection, `VARIANT … REQUIRE …` scoping (a parameter required only by an *unselected* variant stays optional), `REQUIRE` names promoted to parameters, `stringToken`'s two accepted spellings and its one rejection. |
| **The AST** | `def.rs` | 453 / 6 | `ComponentDef` / `Param` / `Variant` / `ComponentInst` / `ConnectDecl`, deliberately syntax-only — every rule the Java enforces at expansion time is absent here, so the port cannot refuse a document the Java accepts. |
| **The datasheet** | `metadata.rs` | 911 / 22 | `ComponentMetadata`: per-instance type and the *inputs* that exist only in the AST — each parameter with both its bound expression and that expression's solved value, so a shared `UA_rad` is visibly shared. This is the payload the Variable Explorer's COMPONENTS section renders. |
| **Cycle paths** | `cyclepath.rs` | 1,745 / 37 | `CyclePathResolver`: fill-missing-properties by state index, and the closed cycle-path interpolation (isobaric / isentropic / isothermal / isenthalpic / isochoric / linear) the property plots overlay — extended to index states off **component stream members** (`s1.P`, `s2.h`) so a component-built Rankine cycle draws on the same dome a hand-written `T1/P1/h1` document does. |

### Where it runs, and why that position is load-bearing

Expansion is wired into `engine.rs` as **pipeline stage 1b**, established by
reading `EquationParser.parseResult:265-345` and cited in a comment there: after
parse, **before** CALL flattening, before matrix and complex expansion, and well
before blocking. Two consequences are structural rather than incidental:

* the component equations are the **seed** of the equation list (the Java writes
  `new BoundedEquationList(componentEquations)` and then flattens the statements
  *into* it), so they precede every equation the document itself wrote — and the
  residual list, block ordering and `block_equations` all inherit that order;
* `rewriteStatements` runs on statements *before* they are flattened, so a
  dotted reference inside a `FOR` body or a `CALL` argument is rewritten once, at
  the AST level, and every later pass sees only flat names.

`check` runs the same expansion for the same reason `solve` does — without it a
component document has zero equations and reports as trivially unsolvable. The
Phase-5 stopgap `reject_unexpanded_components` is deleted.

### Storage without `DYNAMIC`

`hasStorage()` — any component body with `der(member) = …` — routes into the
document's `DYNAMIC` block in the Java. `DYNAMIC` is Phase 8 and is not parsed
here, so `dynamicSystems` is always empty and **the Java's other branch applies
verbatim**: the §8.2 steady/transient duality, where each `der(X) = rhs` becomes
the equilibrium constraint `rhs = 0` and the state is an ordinary unknown
(`engine.rs::steady_storage_equations`). One warning diagnostic says so, naming
the number of rewritten equations. Pinned by
`fixtures/corpus/adv_storage_steady_branch.frees`.

---

## Robustness: what the component fuzz found

`crates/frees-core/tests/component_robustness.rs` — **58 tests**, roughly 1,300
hostile documents — is Phase 6's half of `robustness.rs` and
`props_robustness.rs`. Phase 6 is the first layer in this port that is
*structurally recursive*, so the standing rule ("`parse_document`, `check` and
`solve` may return `Ok` or `Err` and must do nothing else") gains two clauses:

> an `Ok` must be **structurally right**, not merely produced — so the wide and
> long cases assert the physics (all 50 endpoints read the broadcast value, the
> 200-stage halving chain fed `2^200` lands on exactly 1, a `2^50` tower is
> `2^50`), not `is_ok()`;
>
> an `Err` must be **the right Err** — every rejection is matched against a
> substring of its message, because `assert!(result.is_err())` is also satisfied
> by a stack overflow caught in another thread.

The sweep covers self-instantiation (direct, mutual, and a three-cycle),
hierarchies 50 and 1,600 deep, `connect` nodes of 0, 1, 2, 24 and 50 endpoints,
a length-1 self-loop and a closed two-node loop, duplicate instance and
definition names, every port arity from 0 to 6 against a three-port built-in
(leaf **and** hierarchical), unknown component types at three casings, unknown
ports in a `connect` and in a body, unknown and unvalued parameters, `REQUIRE`
naming an undeclared parameter, `model$` naming no variant / given a number /
given `''` / given the wrong case, `VARIANT` blocks with no selector, 200-element
chains wired both ways, four kinds of domain-separation violation, undefined
locals at top level and one subsystem deep, self- and mutually-referential
parameter defaults, a 500-port component, 4,000-character names, non-ASCII
names, and — as bulk sweeps — **every one of the 295 built-ins instantiated with
nothing supplied (590 documents) and with a nonsense `model$` / `fluid$` (590
more)**.

**It found one real defect, now fixed and regression-tested, and one
performance cliff, now measured against the oracle and pinned.**

### 1. A deep hierarchy aborted the process (fixed)

`flatten_instance` recurses once per subsystem level. The self-instantiation
guard catches *cycles*, but it cannot catch a **finite tower**, because every
level is a different name. A 600-level document therefore died with
`fatal runtime error: stack overflow` — a `SIGABRT`, not an `Err`, that
`catch_unwind` cannot convert and that would take the whole wasm module down in
a browser tab. Measured before the fix: 400 levels survived, 600 did not, on a
2 MiB debug test-thread stack; the browser's stack is smaller.

Fixed by `MAX_HIERARCHY_DEPTH = 64` in `expander.rs`, with the same reasoning
and the same number as the parser's existing `MAX_BLOCK_DEPTH`. The shipped
library's deepest subsystem is **depth 1** (15 hierarchical components, none of
which nests another subsystem), so the ceiling is over sixty times what the
library needs. **This has no Java counterpart** —
`ComponentExpander.flattenInstance` recurses unguarded and dies with
`StackOverflowError`, which a JVM turns into a catchable `Error` on a thread it
can abandon; a wasm module cannot. Recorded as divergence 13 below. Pinned from
both sides by `the_hierarchy_ceiling_holds_exactly_where_it_says` (64 levels
solve, 66 are refused by name).

### 2. Parameter substitution is exponential in hierarchy depth (measured, not fixed)

`ComponentExpander` substitutes a parameter's *expression* into every place the
body names it. A subsystem that passes `k = k + k` to a child that uses `k`
twice doubles the expression tree at every level: at depth `n` the expanded
equation has `Θ(2^n)` nodes. The Rust `Expr` is an owned tree, so the
substitution deep-clones and the cost is paid in full.

Checked against the real Java engine (`tools/golden-dumper/run.sh`, this pass) —
this is measurement, not conjecture:

| depth | Java oracle | this port |
|---|---|---|
| 12 / 16 / 20 | solves, `y = 2^n` | solves, same values — 17 ms / 246 ms / 4.2 s |
| 24 | solves, `y = 16777216` | solves, same value — **65 s** |
| 28 | solves, `y = 268435456` | not attempted (projected ~17 min) |
| 32 | **`OutOfMemoryError: Java heap space`**, killing the process | not attempted |

So the port is *more* robust at the top end (Java dies where this returns an
answer eventually) and roughly an order of magnitude slower in the middle,
because Java's immutable AST nodes are shared by reference and become a DAG
where the Rust tree is materialised. **Neither engine is usable past ~depth 24**,
and nothing in the shipped library or the corpus comes close. Pinned at depth 16
by `parameter_substitution_stays_within_its_measured_exponential`, which asserts
the exact answer and a budget an order of magnitude below the next term.

### What the sweep did *not* find

Everything else was already correct. 1,600-level hierarchies, 2,000-endpoint
`connect` nodes and 2,000-component chains all return correct answers in
bounded time; no document in the corpus produced a non-finite value; every
refusal named the component or the instance rather than a mangled scalar
(explicitly asserted — `a_body_referencing_an_undefined_local_is_refused_and_names_it_readably`
fails if `c$undefined_thing` appears in the message).

---

## Component coverage — measured, not assumed

A library that parses but is never instantiated is not evidence of anything, so
`tests/component_families.rs` measures three different things and pins each as a
**floor that must be re-measured, never raised by hand** (the totals are read
from `components::library`, so growing the library cannot silently pass a
smaller fraction):

| Question | Answer |
|---|---|
| How many built-ins does a corpus document that **solves and matches the Java oracle** instantiate? | **127 / 295 (43 %)**, across 63 solving documents |
| How many does *any* corpus document instantiate, including the error fixtures? | **130 / 295**, across 86 documents |
| How many expand from a bare, generic single-instance probe? | **268 / 295**; the other 27 need a `TABLE` argument or name a variant the probe cannot invent, and are listed by name in the test output. **0 fail.** |
| Does every domain family have a document that solves end to end? | **Yes — 12 / 12** (`ac`, `electrical`, `fluid`, `heat`, `hydraulic`, `liquid`, `mechanical`, `moistair`, `pneumatic`, `powertrain`, `signal`, `twophase`; `control` ships one component and is exercised from the heat document, checked by *file* rather than by document name) |

**One number in the test output differs from the table above, on purpose.**
`corpus_component_coverage_is_measured_not_assumed` prints *"corpus instantiates
120/295 (41 %)"* because its floor is scoped to `fixtures/corpus/components_*`
— the family and wave documents it owns — and it counts error fixtures too. The
127 above is the broader question this report cares about: **every** corpus
document, `adv_*` included, restricted to the ones that actually solve. Neither
is wrong; read the 120 as the gate's floor and the 127 as the coverage claim.

Per library file, reached by a solving document:

```
fluid     11/31    liquid     9/21    twophase  12/47    ac         2/7
heat       9/17    electrical 15/31   mechanical 17/27   powertrain 13/19
control    1/1     moistair   5/19    pneumatic  4/18    hydraulic 12/23
signal    17/34
```

**43 % is the honest number and it is deliberately far from 100 %.** Full
instantiation coverage from hand-written *physical* documents is not a one-pass
goal: a well-posed boundary condition per component is a modelling problem, not
a test. What the port owns — that every definition's ports resolve, its
parameters bind, its variant selection runs and its body rewrites onto flat
scalar equations — is the 268/295 row, and that one is complete but for the 27
the probe cannot supply an argument for.

---

## Gate evidence

Every gate below was re-run raw in this pass. **Note for the next session:** the
`rtk` output filter rewrites `cargo`/`npx` invocations and *condenses* their
output — `cargo test` comes back as a one-line summary with no per-suite
results, clippy/fmt warnings are swallowed entirely, and even `ls | wc -l`
under-counted a directory during this pass (314 where `find` said 361). Invoke
the binary by absolute path (`"$HOME/.cargo/bin/cargo"`, `/usr/bin/grep`,
`./node_modules/.bin/vitest`) and redirect to a file. `rtk`'s `find` also
rejects `-exec`/`-not` and its `grep` chokes on `{` and on `$`.

| Gate | Command | Result |
|---|---|---|
| Tests | `cargo test --release --workspace` | **2055 passed, 0 failed, 2 ignored** (21 suites) |
| Parity | `cargo test --release -p frees-core --test parity` | **361/361 fixtures match the Java oracle**, 17 at a declared tolerance |
| Component fuzz | `cargo test --release -p frees-core --test component_robustness` | **58 passed**, 0 failed |
| Component fuzz (debug) | same, no `--release` | **58 passed** — run deliberately, because the stack-overflow defect above only reproduced in a debug build |
| Property fuzz | `cargo test --release -p frees-core --test props_robustness` | **20 passed**, 0 failed |
| Clippy (host) | `cargo clippy --workspace --all-targets -- -D warnings` | exit **0**, no output |
| Clippy (wasm32) | same, `--target wasm32-unknown-unknown` | exit **0**, no output |
| Format | `cargo fmt --all --check` | exit **0**, no output |
| wasm bundle | `wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg` | **2,236,887 B = 2184.5 KiB raw**, **1,112,260 B = 1086.2 KiB gzipped** |
| Web tests | `cd web && nvm use 22 && ./node_modules/.bin/vitest run` | **341 passed / 36 files**, 0 failed |
| Web build | `npm run build` | exit **0** (only the pre-existing rollup `/*#__PURE__*/` and chunk-size warnings from vendored deps) |

The two ignored tests are
`robustness.rs::the_slowest_quadrature_inputs_still_terminate`
(`#[ignore = "bounded but slow (~6 min)"]`, carried from Phase 4) and one
doctest in `components/cyclepath.rs`.

### Bundle size against the newly raised budget

| | Phase 5 | Phase 6 | Δ |
|---|---|---|---|
| raw | 1866.6 KiB | **2184.5 KiB** | +317.9 KiB |
| gzipped | 973.6 KiB | **1086.2 KiB** | +112.6 KiB |
| budget | 2048 KiB | **3072 KiB** | +1024 KiB |
| used | 91.1 % | **71.1 %** | — |
| headroom | 181.4 KiB | **887.5 KiB** | — |

The budget was raised from 2048 to 3072 in `.github/workflows/ci.yml` **as
accepted debt, and the debt is not one kilobyte closer to being paid.** The
comment in that file names two ways to pay it, in order of value:

1. move the 528 KB of linked property tables onto the fetch seam that already
   exists (`props/tables.rs::install_from_bytes`) — blocked on the solver being
   synchronous;
2. split the engine into lazily-loaded chunks so a document that names neither
   properties nor components pays for neither.

**Neither was attempted this phase.** What Phase 6 *did* do is make the case for
2 sharper: the component layer is 317.9 KiB that a scalar document never
touches, and `expand_component_layer` already early-returns before parsing a
byte of it (`doc.components.is_empty()`), so the chunk boundary is already drawn
in the engine — it is only the linker that does not know. The honest reading of
this table is that the raise bought room and the room got spent; the next phase
inherits 887.5 KiB and the same two exits.

### Browser proof

`web/dist` served by a static server **with an SPA fallback**
(`scratchpad/spa_server.py`; plain `python3 -m http.server` 404s on `/help` and
mis-types `.wasm`). Driven with the Playwright MCP tools. The CodeMirror handle
in this build is `.cm-content`'s **`cmTile.view`**, not `cmView`.

**Document 1 — a 7-component thermofluid loop wired by shared stream names**
(`fixtures/corpus/components_family_fluid.frees`: `Source → FlowSensor →
Splitter → 2 × Valve → Mixer → Sink`, on Water). F2 → **Solved**,
`28 eqns · 28 blocks · 27 iters · max residual 2.51e-13`. The Java oracle
reports **28 blocks** for the same document.

| Variable | Browser | Java oracle | rel |
|---|---|---|---|
| `mdot_meas` | 2.5 | 2.5 | 0 |
| `p_out` | 299857.95 | 299857.95454545453 | display precision |
| `h_out` | 196421.64 | 196421.36460792166 | **1.40e-06** |
| `f3.mdot` / `f4.mdot` / `f7.mdot` | 1.5 / 1 / 2.5 | identical | 0 |
| `f5.p` / `f6.p` | 299857.95 / 299936.87 | 299857.95454545453 / 299936.8686868687 | display precision |

The one non-zero error is the declared table tolerance: `h` is the single
`Enthalpy(Water, P, T)` inverse bisection inside the library's `Source` body,
carried unchanged along the whole network, and `fixtures/tolerances.json`
records it at `measured: 1.3898e-6` under a `1e-5` gate. Every quantity the
network computes *exactly* is exact.

**Document 2 — a 5-component pneumatic mixer wired with `connect(...)`**, so the
union-find and the junction rules are exercised rather than shared naming. F2 →
**Solved**, `39 eqns · 39 blocks · 59 iters · max residual 2.17e-19`. Verified
against the Java oracle for the same document (39 blocks):

| Variable | Browser | Java oracle |
|---|---|---|
| `m_out` | 0.03 | 0.03 |
| `h_mix` | 313333.33 | 313333.3333333334 |
| `y_mix` | 0.04 | 0.04 |

Flow-weighted mixing of both the enthalpy rider and the species rider, exact.

**The component datasheet renders.** The Variable Explorer's COMPONENTS section
lists all seven instances of document 1 with their type, and expands to:

* streams `f1`…`f7` — **RESULTS**, each with `h` / `mdot` / `p`;
* `FS` (FlowSensor) — the named output `mdot_meas`;
* `SK` (Sink) — `h` / `mdot` / `p`;
* `SRC` (Source), `VA`, `VB` (Valve) — **PARAMETERS**, each row showing both the
  *binding* and its resolved *value* (`fluid$ → water`, `cv 0.004 → 0.004`,
  `model$ → fixed`), which is exactly what `metadata.rs` exists to supply.

**`/api/` requests: zero**, across the workspace, both solves, the Schematic tab
and the Help page. The full unfiltered list is the static bundle plus exactly
two engine artefacts — `assets/engine.worker-*.js` and
`assets/frees_bg-*.wasm`. The only non-200 in the whole session was
`/build-info.js` (injected by nginx in the Docker deploy, absent from a bare
`dist`) — the same one Phase 5 recorded.

**The Help page's component chapter renders offline**: *Connections & Junctions*,
*Reading the Schematic*, *Domains & Fluid Families*, *The Component Library*
(the thirteen-library table), *Fidelity Variants (model$)*, *Writing Your Own
Component*, *Steady ↔ Transient Networks*, *Cycle Plots & Diagnostics*, *The
Component Wizard*, *Troubleshooting Networks*.

**One thing did not work, and it is recorded as non-delivery 1**: the Schematic
tab draws the component *nodes* but never their *wiring*, and prints
"connections not shown — the document has errors; fix them and Check" on a
document that both checks clean and solves.

Screenshots: `scratchpad/phase6-fluid-network.png`,
`scratchpad/phase6-connect-network.png`.

---

## Fixtures

```
fixtures/corpus + fixtures/golden   361 promoted (was 268)
fixtures/corpus-pending             31 staged (was 29)
```

### Promoted this phase (93)

| Group | Count | What they pin |
|---|---|---|
| `components_family_*` | 12 | one solving network per domain family — the "representative document per family" gate |
| `components_wave*` | 23 | the physical-model waves: capillary tube, flash tank, ejector + oil separator, reversing valve, short-tube choke, liquid TMS, aux electrical, motor/inverter/DC-DC, kinematic pairs, torque converter + differential, hydraulic motor and valves, radiation + Peltier, automatic transmission, freewheel, gearbox + MVEM, hybrid ECMS, Pacejka tire, hydro turbine, PV MPPT, wind rotor, counterbalance, species chain |
| `components_signal_*`, `components_bsweep_*`, `components_enablers_*` | 8 | the causal signal library, `model$` fidelity sweeps, and the table/lookup enablers |
| `components_definition_*`, `components_user_defined_type` | 3 | user-authored `COMPONENT`s, including one defined and never instantiated |
| `adv_junction_*` | 9 | the junction rules per domain: four-way electrical, fluid mixer/splitter enthalpy, the `h`-is-an-equality rule, three-way heat, rotational and translational, and loop closure |
| `adv_domain_*` | 8 | domain separation as a hard error — fluid↔mechanical, gas↔oil, heat↔electrical, liquid↔twophase, signal↔heat, steam↔moistair, plus the two untagged-fluid cases |
| `adv_diag_*` | 7 | the diagnostic contract: duplicate instance, port arity, unknown component / parameter / port, two streams on one connect, and **no mangled scalar in the message** |
| `adv_variant_*` | 6 | default `model$`, selection, unknown model, missing and satisfied `REQUIRE`, `REQUIRE` naming an unknown parameter |
| `adv_naming_*` | 6 | case-folded members, locals across two instances, port-member chains, stream-vs-connect binding, top-level shadowing, an unconnected port |
| `adv_species_*`, `adv_moistair_*`, `adv_storage_*`, `adv_hierarchical_*`, `adv_cyclepath_*`, `adv_connect_*`, `adv_userdef_*`, `adv_signal_domain_fanout` | 11 | species riders, humid-air weighted mixing, the steady-storage branch, subsystem flattening, cycle-path ordering, `connect` keyword casing, named outputs, signal fan-out |

Twelve of the promoted component fixtures need a declared numeric tolerance —
`fixtures/tolerances.json` now names **17** fixtures, up from 5, each with its
measured error and the mechanism that produces it. The mechanism is always the
same one Phase 5 opened (divergence 9): a library body calls a real-fluid
property function, the tabulated backend answers it at `1e-7…2e-4` instead of
full CoolProp accuracy, and the error rides through the network. Both guards
still hold — a fixture named there but absent fails, and a fixture named there
that passes at the default `1e-9` fails, so a dead tolerance cannot sit
pretending it is needed. `display_names`, `block_count` and the error
classification stay exact for all 361.

### What Phase 6 closed in the pending list

Phase 5 listed **three** documents blocked on `COMPONENT` instantiation. All
three now get past the component layer; each has moved to a different blocker,
which is progress and not a promotion:

| Document | Was blocked on | Now blocked on |
|---|---|---|
| `ev-battery-cooling-pid` | COMPONENT + DYNAMIC | **DYNAMIC only** ~~(Phase 8)~~ → **Phase 7** *(label corrected 2026-07-31)*. **Phase 7 update:** `DYNAMIC` is no longer the blocker; it now fails in the property backend (`HAPropsSI` / uncovered states), so it has moved to the real-fluid row |
| `pressure-cooker` | COMPONENT + DYNAMIC | **DYNAMIC only** ~~(Phase 8)~~ → **Phase 7** *(label corrected 2026-07-31)*. **Phase 7 update:** `DYNAMIC` parses and reaches the engine; it now asks for `method = ida`, and the implicit-DAE path is assembled but not routed |
| `ev-thermal-management` | COMPONENT + DYNAMIC + `EG50` | **`INCOMP::MEG[0.50]` has no property table** (Phase 5, divergence 9) — it now reaches block 3 of 89 equations before failing |

### Pending: 31, replayed document-by-document

Every staged document was replayed through the current Rust engine this pass and
its failure classified. **1 solves, 3 both-refuse-with-different-classification,
27 the Rust engine refuses.**

| # | Blocked on | Documents |
|---|---|---|
| 6 | **Phase-9 control-systems CALLs** (`lqr`, `lqe`, `c2d`, `routh`, `residue`, `tf2ss`) | `controller-design-lqr-pid`, `estimator-gramian-balreal`, `digital-control-c2d`, `routh-stability`, `inverse-laplace-residue`, `multi-output-destructuring` |
| 5 | **`PLOT` blocks** ~~(Phase 7)~~ → the real blocker is **control-systems `CALL`s (Phase 9)** *(label corrected 2026-07-31)* | `control-analysis-report`, `cruise-control`, `nichols-chart`, `root-locus-analysis`, `step-impulse-response` |
| 7 | **`DYNAMIC` (ODE/DAE) blocks** ~~(Phase 8)~~ → **Phase 7** *(label corrected 2026-07-31)*. **Closed 2026-07-31 (Phase 7)**: the first five were promoted; `ev-battery-cooling-pid` and `pressure-cooker` are no longer blocked on `DYNAMIC` and have moved to the property-backend and `method = ida` rows respectively | `damped-oscillator-ode`, `engine-cycle-wiebe`, `newton-cooling-transient`, `sounding-rocket-trajectory`, `transient-heat-rod`, **`ev-battery-cooling-pid`**, **`pressure-cooker`** |
| 4 | **Real-fluid coverage the tables do not have** — humid air (`HAPropsSI`), transport properties, `Z`, and the `INCOMP::MEG[0.50]` glycol. All four are Phase-5 divergence 9 | **`adv_moistair_W_passthrough`**, **`adv_moistair_dryair_three_way`** (both new this phase), `hx-correlations-fluid`, `thermo-compliance`, and `ev-thermal-management` |
| 3 | **`PARAMETRIC` blocks** — error fixtures where the classifications still disagree: Java raises `SolverException`, Rust raises `ParseException`. Both refuse; the gate compares classification. **Closed 2026-07-31 (Phase 7)** — `PARAMETRIC` now has a home on `Document`, both engines classify alike, and all three were promoted | `damped-oscillator`, `driving-cycle-energy`, `projectile-trajectory` |
| 1 | **`STATE TABLE` block type** | `state-tables-multifluid` |
| 1 | **`SYMBOLIC` / CAS** — Symja replacement undecided (Phase 9) | `partial-fractions` |
| 1 | **String variables** — `geom$ = 'wall'` not ported | `heisler-transient` |
| 1 | **`MODULE` inside `FOR`** — pipeline-ordering deviation carried from Phase 4 | `module_inside_for_loop` |
| 1 | **Ill-posed by construction — held deliberately.** Structurally square but rank-deficient, so the solution set is a *line*. It passes today (Rust within 6.6e-2 of the Java point) but promoting it would freeze an arbitrary point of a continuum into the gate | `solver_singular_linear_cycle` |

(The counts sum to more than 31 because `ev-thermal-management` and
`ev-battery-cooling-pid` appear under two blockers each; 31 distinct documents.)

> **Phase-7/8 annotation (2026-07-31).** This table's two phase labels were
> wrong and are struck through above: per [`PLAN.md`](../PLAN.md) §5, **Phase 7
> is Dynamics** and **Phase 8 is Analysis & design**; **Phase 9** is CAS &
> control systems. The `DYNAMIC` row was labelled Phase 8 and belongs to Phase 7
> (which has since closed it), and the `PLOT` row was labelled Phase 7 but its
> five documents are control-systems documents needing Phase 9 — `PLOT` is not
> the operative blocker, the `rlocus` / `nichols` / `step` / `pole` / `tf`
> `CALL`s are.
>
> The list is now **23** — see
> [`docs/status-phase78.md`](status-phase78.md#fixtures-promoted-and-still-pending)
> for the current classification. Phase 7 promoted 29 fixtures (361 → 390),
> closing the `DYNAMIC` and `PARAMETRIC` rows outright. **Phases 7–8's
> robustness pass promoted none** — it added regression tests, not fixtures.

**Two documents were staged, not promoted, by this phase**:
`adv_moistair_W_passthrough` and `adv_moistair_dryair_three_way`. Both are
humid-air component networks written to pin the `W` rider's pass-through and
flow-weighted-at-a-mixer rules; both die in the property backend
(`humid-air property 'H' at (T, P, W) is not available`), not in the component
layer. The `W`-rider *expansion* rules are covered by
`adv_moistair_mixingbox_weighted` and `adv_moistair_dryair_basis_three_way`,
which are promoted and which avoid `HAPropsSI`.

---

## What Phase 6 did **not** deliver

Ranked by how likely each is to bite the next session.

1. **The Schematic never draws a connection.** The auto-rendered component
   network shows every instance under an **UNWIRED** heading and prints
   "connections not shown — the document has errors; fix them and Check" on a
   document that checks clean and solves. Root cause, exactly:
   `web/src/api.ts:433` hardcodes `connections: []` in the `check` response with
   the comment *"Not produced by the wasm boundary yet"*, and the wasm boundary
   indeed emits neither `connections` in `check` nor `component_connections` in
   `solve` — the field exists on `Solution` and is computed by the expander
   (`Solution::component_connections`, `expander::Connection`, with its
   `OrderedMap` insertion order chosen specifically so the schematic payload is
   stable), and stops at the boundary. `CheckReport` has no field for it at all.
   Closing this is three edits — a field on `CheckReport`, serialisation in
   `frees`, and deleting the hardcoded `[]` — plus an oracle check of
   `SolveDtos.connectionsOf`'s field order, domain spelling and null connector.
   **It is the single most visible gap in the phase**, because the Schematic is
   the component layer's flagship UI and it is inert.

2. **The budget was raised by 1 MiB and 318 KiB of it is already spent, with the
   debt untouched.** See [the size table](#bundle-size-against-the-newly-raised-budget).
   The engine already knows the component layer is optional
   (`expand_component_layer` early-returns before touching the 122 KB of library
   text), so the chunk boundary exists in the code and not in the linker. Nobody
   should raise `WASM_BUDGET_KB` again before paying one of the two debts the
   CI file names.

3. **Parameter substitution is exponential in hierarchy depth** — measured
   above, checked against the oracle, pinned at depth 16, and **not fixed**. A
   fix means structural sharing in `Expr` (an `Rc` tree or a hash-consed arena),
   which is a change to a contract file, or memoising `substitute_params` on
   `(expression identity, binding set)`. Nothing in the library or the corpus
   goes past depth 1, so this is a latent cliff rather than a live problem — but
   it is a cliff a user can walk off with fifteen lines of legal frees.

4. **Component coverage by a *solving* document is 43 %** (127/295). The other
   168 built-ins are proven only to expand, not to produce physics anyone has
   checked against the oracle. 47 of the 295 live in `twophase.frees` alone, of
   which 12 are exercised. Raising this is corpus work, not engine work, and the
   floors in `component_families.rs` exist so it cannot quietly fall.

5. **27 built-ins cannot be reached even by the bare-instantiation probe**,
   because they take a `TABLE`/`FUNCTION` argument or name a variant with no
   default: `compressor`, `fanmap`, `pumpmap`, `compressormap`, `heatexchanger`,
   `twozonehx`, `regenerator`, `propeller`, `liquidpumpmap`, `hydroturbine`,
   `twophasecompressor`, `threezonehx`, `batterycellmap`, `batterypack`,
   `motormap`, `mpptblock`, `cam`, `meanvalueengine`, `torqueconverter`,
   `drivecyclesource`, `automatictransmission`, `gradeprofile`,
   `hybridpowersplit`, `windrotor`, `sigtable`, `sigmap`, `sigmap2`. Several
   *are* covered by hand-written fixtures (`components_bsweep_mvem_wotmap`,
   `components_wave7_*`); the gap is that the generic probe cannot invent a
   table, so the 268 number understates them.

6. **`DYNAMIC` is still Phase 8, and every storage component silently becomes a
   steady one.** The rewrite is the Java's own branch and it emits a warning, but
   a user who writes `Capacitor`, `Inertia`, `ThermalMass` or any of the
   capacitive volumes gets an *operating point*, not a trajectory — and the
   warning is the only thing that says so. Seven pending documents wait on it,
   two of which Phase 6 moved into that queue.

7. **The component library's real-fluid ceiling is Phase 5's ceiling.** Any
   library body calling `Viscosity`, `Conductivity`, `Cp`, `Z`, `HAPropsSI` or a
   fluid other than Water/R134a fails at solve time with a property error, no
   matter how correct the expansion was. That is why twelve promoted fixtures
   need a declared tolerance and why four pending documents are blocked. The
   component layer made this *more* visible, not worse: `Pipe`, `GasPipe`'s
   thermal cousins and every correlation-driven component reach the backend.

8. **`check` and `solve` expand the component layer twice** on a document that
   is checked and then solved — once each, with no caching. For the corpus this
   is microseconds; for a 200-instance network in an editor that checks on every
   keystroke it is the obvious next profiling target. Not measured in a browser.

9. **The 47 `adv_*` fixtures and the 46 `components_*` fixtures overlap the
   robustness suite** in what they cover (duplicate instances, unknown ports,
   domain separation, unknown `model$`). That is deliberate — one gate compares
   *classification against the Java*, the other asserts *the contract holds and
   the message is right* — but it does mean a change to a diagnostic string
   fails in two places, and a reader may mistake the redundancy for coverage
   breadth.

10. **Everything Phases 4 and 5 did not deliver is still not delivered** — the
    43 refused CALL intrinsics, `MODULE` inside `FOR`, the SVD sign convention,
    the empty isobars/isentropes/isotherms on property diagrams, the
    psychrometric chart, `v_crit`/`MolarMass` of a tabulated fluid, the
    two-phase-plateau inverse, R134a's superheat ceiling, and the unmeasured
    speed half of the D1 spike. See
    [`docs/status-phase5.md`](status-phase5.md#what-phase-5-did-not-deliver) and
    [`docs/status-phase4.md`](status-phase4.md#what-phase-4-did-not-deliver);
    none of them moved.

---

## Next

1. **Close non-delivery 1** — it is three small edits and it turns the
   Schematic from decoration into the feature it is documented as.
2. **Phase 7 — `PLOT` blocks.** Five pending fixtures, and the cycle-path
   machinery (`cyclepath.rs`) that Phase 6 already ported is waiting for a
   consumer.
3. **Phase 8 — `DYNAMIC`.** Seven pending fixtures, and the other half of every
   storage component in the library.
4. **Pay one of the two bundle debts before writing Phase 7 code**, while there
   is still headroom to do it calmly.
5. **Grow the component corpus**, in twophase and moistair first — they are the
   two largest libraries with the thinnest solving coverage.
