# Status — Phase 5 complete (properties)

> **Superseded as the current state by
> [`docs/status-phase6.md`](status-phase6.md) (2026-07-31).** Kept for its
> property-backend detail and its measured table-error tables, which Phase 6 did
> not change. The pending-fixture table below is annotated with what Phase 6
> closed and what it moved.

**Date:** 2026-07-30 · Read after [`docs/status-phase4.md`](status-phase4.md),
which this supersedes as the current state.

Phase 5 ports `backend/core/props/` — 28 Java files, 5,246 LOC — plus
`core/HeislerCharts.java`, and answers the question Phase 4 left open: **what
serves real-fluid properties in a browser tab when CoolProp is a 12 MB C++
library.** The answer is decision D1's precomputed `(P,h)` tables, and this phase
is where that decision stopped being a document and became a running engine.

```
1710 Rust tests passed, 0 failed, 1 ignored (18 suites)
268/268 golden fixtures match the Java oracle
   (5 of them at a declared, guarded table tolerance — see Parity tolerance)
clippy -D warnings clean (host and wasm32-unknown-unknown)   cargo fmt clean
wasm 1866.6 KiB raw / 973.6 KiB gzipped   (budget 2048 KiB raw — 91.1% used)
web 336 tests / 35 files green, vite build green
Browser proof: an all-real-fluid Rankine cycle solved in-tab against the Java
   golden, water + R134a in the fluid picker, a T-s dome drawn from the linked
   tables, ZERO /api/ requests
```

Every number above was measured in this pass, raw, not carried forward. See
[Gate evidence](#gate-evidence).

---

## What Phase 5 delivers, by area

`crates/frees-core/src/props/` is **17,652 lines across 25 modules with 339
inline tests**, plus `tests/props_robustness.rs` (20 tests, ~2,600 hostile
documents).

| Area | Module(s) | LOC / tests | What works |
|---|---|---|---|
| **The property-function dispatcher** | `propfun.rs` | 1,735 / 20 | `PropertyFunctions.java` line-for-line: the 51-entry `FLUIDS` alias table, the hand-rolled `GLYCOL_MIX` grammar (`EG50` → `INCOMP::MEG[0.50]`), `OUTPUTS`/`INPUTS`/`HA_OUTPUTS`/`HA_INPUTS`, `plot_fluids`, `detect_fluid`, `nominal_enthalpy`/`nominal_pressure`, and `evaluate_with_tokens` **in the Java's exact branch order** — so `Enthalpy(N2, T=500)` stays an ideal-gas call and never becomes a real-fluid one. Plus the `RealFluid` trait that stands where `CoolProp.LIB` stands in the Java. |
| **The tabulated backend** | `phtable.rs`, `satsplit.rs`, `tables.rs` | 1,592 + 2,223 + 247 / 48 | `PhPropertyTable` (bicubic Hermite with *analytic* partials, nodal FD build) and `SaturationSplitTable` (saturation lines + exact two-phase relations + two dome-following single-phase pieces), both ported; a **`FRPHTAB1` reader** for the artifacts `tools/table-gen` produces from native CoolProp; and the wiring that decodes and installs them. Two fluids ship linked: water and R134a. |
| **Inverse lookups (no Java counterpart)** | `propfun.rs` `TableBackend` | — | `(P,T) → h` and `(P,s) → h` by monotone bisection on the tabulated surface, and `(T,x) → P` by bisection on the saturation line. The reference never needed these — it falls through to a native call for every non-`(P,h)` input — and they are what the Rankine and refrigeration documents actually call. |
| **Ideal gases + NASA-7** | `idealgas.rs`, `nasa.rs`, `periodic.rs` | 686 + 859 + 146 / 28 | Formation-reference enthalpy, the NASA-7 polynomial set with its 1000 K switch-over, the periodic table behind `MolarMass`. |
| **Chemistry / combustion** | `formula.rs`, `combustion.rs`, `thermochem.rs`, `equilibrium.rs` | 308 + 291 + 532 + 770 / 52 | The chemical-formula parser, heating values, stoichiometric AFR, mixture properties, `AdiabaticFlameTemp` both plain and equilibrium, `eq_molefraction`. |
| **Cubic equations of state** | `cubiceos.rs` | 1,542 / 20 | PR / SRK / RK / VDW: `eos_z`, `eos_volume`, `eos_density`, `eos_enthalpy`, `eos_entropy`, `eos_pressure`, `eos_psat`, with the Java's root selection. |
| **Compressible flow** | `compressible.rs` | 731 / 17 | Isentropic relations, normal and oblique shocks, Prandtl–Meyer, Fanno and Rayleigh lines, stagnation properties — including the bracketed inversions (`mach_a_astar`, `mach_prandtlmeyer`, `beta_oblique`) and their physical boundaries. |
| **Heat exchangers** | `hx.rs`, `hxcorr.rs` | 703 + 1,388 / 49 | The ε-NTU relations both ways for every arrangement, LMTD, fin efficiency, and the CoolProp-backed correlations (`htc_1phase`, `htc_evap`, `htc_cond`, `dp_*`) reaching the same backend a `prop$` call does. |
| **Convection, flow resistance, two-phase, pneumatics** | `convective.rs`, `flowresist.rs`, `twophase.rs`, `pneumatics.rs` | 339 + 274 + 430 + 209 / 43 | The Nusselt correlation family, Colebrook friction, Lockhart–Martinelli / Friedel / void-fraction models, ISO 6358. |
| **Transport, solids, Heisler** | `transport.rs`, `solids.rs`, `heisler.rs` | 320 + 486 + 445 / 40 | Mixture viscosity and conductivity, the bulk-material database (`k_`, `rho_`, `c_`, `E_`, `nu_`), the Heisler one-term charts for all three geometries. |
| **Diagrams and psychrometrics** | `diagrams.rs`, `psychro.rs` | 770 + 456 / 17 | `PropertyDiagrams.java` (dome, quality lines, isobars, isentropes, isotherms, markers) and the psychrometric chart. Both back the frontend's plot endpoints. |
| **Atmosphere** | `atmosphere.rs` | 127 / 5 | ISA / US-1976, `isa_t` / `isa_p` / `isa_rho` including the 11 km layer seam. |

### The one thing to know about the backend

`TableBackend` **declines by name rather than approximating**. It serves `T`,
`Dmass` and `Smass` from `(P,h)` and derives `Hmass`, `P`, `Umass` and `Q`; it
answers `Pcrit`, `Tcrit`, `Ttriple` and `ptriple` from the four constants the
artifact carries verbatim from CoolProp. Everything else — `Cpmass`, `Cvmass`,
viscosity, conductivity, speed of sound, `Z`, Prandtl, surface tension, humid
air, supercritical states, mixtures, incompressibles, and every fluid other than
water and R134a — comes back as an error that names what is missing. Nothing is
extrapolated past the served box.

---

## The measured table error

Ground truth throughout is **CoolProp 8.0.0** through the vendored
`libCoolProp.so`, reached by `tools/golden-dumper/run.sh` and `tools/table-gen`.
Raw numbers: `fixtures/proptables/ERROR-REPORT.json`, 4,000 fixed-seed samples
per fluid.

| Query form | Water max | Water rms | R134a max | R134a rms |
|---|---|---|---|---|
| `(P,h) → T` | 1.08e-05 | 3.23e-07 | 9.30e-06 | 3.43e-07 |
| `(P,h) → Dmass` | 4.54e-05 | 1.89e-06 | 5.89e-05 | 1.99e-06 |
| `(P,h) → Smass` | 2.08e-04 | 3.32e-06 | 6.99e-06 | 3.20e-07 |
| `(P,T) → h` *(inverse, new code)* | 2.02e-04 | 6.67e-06 | 9.44e-06 | 4.94e-07 |
| `(P,s) → h` *(inverse, new code)* | 2.14e-04 | 7.12e-06 | 8.56e-06 | 6.27e-07 |

By region, max over all three properties:

| | liquid | two-phase | vapor |
|---|---|---|---|
| Water | 2.08e-04 (n=224) | 3.88e-05 (n=3061) | 4.54e-05 (n=707) |
| R134a | 2.72e-06 (n=371) | 2.42e-05 (n=1007) | 5.89e-05 (n=584) |

**The single worst number in the set is located, not left as a mystery.** Water's
2.08e-04 is `Smass` in subcooled liquid at `P ≈ p_min` and `T = 288.6 K`, where
`s = 230.5 J/kg·K`. The **absolute** error there is `4.79e-02 J/kg·K`. Water's
entropy reference is zero at the triple point, so relative error is inflated
wherever `s` passes near zero; read that row as "0.05 J/kg·K". Both inverse forms
inherit the same corner through the same mechanism.

### What that means end-to-end, on a real document

Not the sampling band — the actual states the promoted fluid fixtures land on,
measured by replaying them through `frees-cli` and diffing against the Java
golden:

| Document | Worst variable | Worst relative error |
|---|---|---|
| `rankine-cycle` | `eta_th` | **6.41e-07** |
| `rankine-cycle-2` | `eta_th` | **6.42e-07** |
| `refrigeration-vcr` | `cop` | **1.53e-06** |
| `props_realfluid_water_states` | `d1_rank` (sat-liquid density at 10 kPa) | **7.20e-05** |
| `props_realfluid_r134a_states` | `d_pt` (`(P,T)` inverse) | **1.88e-06** |

The two cycle documents are three orders of magnitude better than the
aggregate worst case, because the states an engineer writes down are not the
corner of the sampling band.

### Coverage, stated plainly

`ERROR-REPORT.json` also records what the tables **cannot** serve, over the same
deliberately wide band:

* **Water: 3,992 of 4,000 samples covered, 3 true misses (0.1 %).**
* **R134a: 1,962 of 4,000 covered, 1,620 true misses (40.5 %).** Every one of
  those misses is superheat deeper than the served 132.7 kJ/kg — states CoolProp
  itself reaches only by extrapolating past its declared `Tmax` of 455 K for
  R134a. All eight R134a document states sit comfortably inside.

---

## Parity tolerance — the gate change, and its guards

`tests/parity.rs` compares variables at `1e-9` relative. The goldens hold
full-accuracy CoolProp values; the table path is `1e-7…1e-4`. **No table-backed
engine can pass a `1e-9` gate on a document that calls a real-fluid property
function.** D1 flagged this and named the owner as whoever promotes those
fixtures. This phase promoted them, so:

`fixtures/tolerances.json` declares a looser **relative** tolerance for five
named fixtures, each with its measured error and the mechanism that produces it.
Nothing else is relaxed — `display_names`, `block_count` and the error
classification stay exact for every fixture, and the other 263 are still held to
`1e-9`. Two guards, both enforced by the parity test itself:

* a fixture named in the file but **absent** from `fixtures/golden/` fails;
* a fixture named in the file that **passes at the default** fails, so a
  tolerance that is no longer needed cannot sit there pretending it is.

The test prints which fixtures used a declared tolerance on every run:

```
parity: 268 fixtures match the Java oracle (5 at a declared table tolerance:
  props_realfluid_r134a_states, props_realfluid_water_states, rankine-cycle,
  rankine-cycle-2, refrigeration-vcr)
```

**This is a real weakening of the gate, and it should be read as one.** The
alternative that does not weaken it is shipping `coolprop.wasm` as the accuracy
path with the tables as a Jacobian accelerator — which is what the Java does —
and that remains open (see [non-deliveries](#what-phase-5-did-not-deliver), 2).

---

## Robustness: what the fuzz found

`crates/frees-core/tests/props_robustness.rs` — 20 tests, roughly 2,600 hostile
documents plus ~8,000 direct backend probes — is Phase 5's half of
`robustness.rs`. The rule it enforces is stricter than "no panic":

> **a solved document never contains a non-finite value.**

A property function that returns `NaN` for a state it cannot serve has lied: the
solver propagates it, the residual is `NaN`, and the user gets a
converged-looking answer built on nothing.

The sweep covers every numeric property intrinsic (**71 functions × 9 hostile
scalars × every argument position, plus all-positions-at-once — 2,250 documents
from that test alone**), the whole phase
envelope (zero and negative absolute `T` and `P`, exactly on the critical point,
supercritical, exactly on and below the triple point, inside the dome, exactly on
and a hair outside `p_min` / `p_serve_max` / the saturation line), unknown and
malformed fluid names, malformed chemical formulas (`""`, `"3"`, `"H2O2X"`,
unbalanced brackets, a 6,000-character formula), degenerate equivalence ratios,
the ε-NTU removable singularities (`NTU = 0`, `Cr = 0`, `Cr = 1`, `ε ≥ 1`), the
bracketed inversions past their physical boundaries, the iterative correlations
in their non-convergent regime, and the tables at every grid edge — including
every truncation length and every header byte flipped two ways.

**It found two real defects, both now fixed and both regression-tested:**

1. **`SaturationSplitTable::eval` served `Some(NaN)`.** The Java returns
   `Double.NaN` for "not covered" and `PhTableRegistry` tests `isNaN` at every
   call site; this port replaced that convention with `Option`, so a `Some(NaN)`
   was the two conventions crossed — a decline wearing the costume of an answer.
   A non-finite `p` or `h` defeated every range check (all comparisons against
   NaN are false, so the point fell through to the liquid branch and interpolated
   at NaN) and `region()` reported it as `Liquid`. Fixed in `satsplit.rs`: an
   explicit finiteness screen on the inputs, and a finiteness screen on the
   result and both its partials before `Some` is returned.
2. **`TableBackend::props_si` carried a non-finite indicator into the
   interpolant.** `PropsSI("Hmass", "Hmass", NaN, "P", 101325, "Water")` returned
   `NaN`. The Newton solver probes with whatever the previous iterate produced,
   so this was reachable from an ordinary document. Fixed in `propfun.rs`: a
   non-finite input is an error naming the state, and the resolved enthalpy is
   re-checked before any output is read off it.

Everything else in the sweep was already correct. The slowest single document in
the whole corpus is well under the 20-second budget the harness asserts.

---

## Gate evidence

Every gate below was re-run raw in this pass. **Note for the next session:** the
`rtk` output filter rewrites `cargo`/`npx` invocations and *condenses* their
output — `cargo test` comes back as a one-line summary with no per-suite results,
and clippy/fmt warnings are swallowed entirely. Invoke the binary by absolute
path (`"$HOME/.cargo/bin/cargo"`, `./node_modules/.bin/vitest`) and redirect to a
file. `rtk`'s `find` also rejects `-exec`/`-not` and its `grep` chokes on `{`.

| Gate | Command | Result |
|---|---|---|
| Tests | `cargo test --release --workspace` | **1710 passed, 0 failed, 1 ignored** (18 suites) |
| Parity | `cargo test --release -p frees-core --test parity` | **268/268 fixtures match the Java oracle**, 5 at a declared tolerance |
| Property fuzz | `cargo test --release -p frees-core --test props_robustness` | **20 passed**, 0 failed |
| Clippy (host) | `cargo clippy --workspace --all-targets -- -D warnings` | exit **0**, no output |
| Clippy (wasm32) | same, `--target wasm32-unknown-unknown` | exit **0**, no output |
| Format | `cargo fmt --all --check` | exit **0**, no output |
| wasm bundle | `wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg` | **1,911,445 B = 1866.6 KiB raw**, **996,926 B = 973.6 KiB gzipped**. Budget 2048 KiB raw → **91.1 % used, 181.4 KiB headroom** |
| Web tests | `cd web && nvm use 22 && ./node_modules/.bin/vitest run` | **336 passed / 35 files**, 0 failed |
| Web build | `npm run build` | exit **0** (only the pre-existing rollup `/*#__PURE__*/` and chunk-size warnings from vendored deps) |

The one ignored test is still
`robustness.rs::the_slowest_quadrature_inputs_still_terminate`
(`#[ignore = "bounded but slow (~6 min)"]`), carried over from Phase 4.

### Browser proof

`web/dist` served by a static server **with an SPA fallback** (plain
`python3 -m http.server` 404s on `/help` and mis-types `.wasm`; the script is
`scratchpad/spa_server.py`). Driven with the Playwright MCP tools. The
CodeMirror handle in this build is `.cm-content`'s **`cmTile.view`**, not
`cmView`.

The document is `fixtures/corpus/rankine-cycle.frees` — an **all-real-fluid**
steam cycle that touches `(P,x)`, `(P,T)` and `(P,s)` inputs. F2 → **Solved**,
`12 eqns · 12 blocks · 17 iters · max residual 0`. Variable Explorer against the
Java golden (`fixtures/golden/rankine-cycle.json`, CoolProp 8.0.0):

| Variable | Browser | Java oracle | rel |
|---|---|---|---|
| `eta_th` | 0.39119741 | 0.39119716208990235 | 6.4e-07 |
| `h1` = `Enthalpy(Water, P=10 kPa, x=0)` | 191805.93 | 191805.94455889906 | 7.6e-08 |
| `h3` = `Enthalpy(Water, P=8 MPa, T=480 °C)` | 3349644.8 | 3349645.1659218343 | 1.1e-07 |
| `s3` = `Entropy(Water, P=8 MPa, T=480 °C)` | 6661.2607 | 6661.263743552311 | 4.6e-07 |
| `h4` = `Enthalpy(Water, P=10 kPa, s=s3)` | 2109392.1 | 2109393.1272094045 | 4.9e-07 |
| `w_turb` | 1240252.7 | 1240252.0387124298 | 5.3e-07 |
| `v1`, `w_pump`, `P_boiler`, `P_cond` | 0.0010102711 / 8072.0665 / 8000000 / 10000 | identical to display precision | — |

`browser_network_requests` filtered by `/api/`: **empty**, across the workspace,
the plot dialog and the Help page. The full unfiltered list is the static bundle
plus exactly two engine artefacts — `assets/engine.worker-*.js` and
`assets/frees_bg-*.wasm`. The only non-200 in the whole session was
`/build-info.js` (injected by nginx in the Docker deploy, absent from a bare
`dist`).

**Fluid list.** The property-diagram dialog's Fluid picker offers exactly
**`R134a`** and **`Water`** — read live from the engine, not a hardcoded list.
The Help page's *Reference · Supported Fluids* section lists the same two. This
is a deliberate narrowing of the Java, which returns all 36 canonical CoolProp
names because CoolProp serves all 36; a picker offering thirty-six fluids that
fail on thirty-four would be lying. `RealFluid::served_fluids()` returning `None`
gets the full Java list back verbatim, so a future `coolprop.wasm` needs no
change here.

**Property plot.** A water T–s diagram renders through Plotly with **17 traces**:
the saturation dome (278 of 400 points finite, `T` from 295.4 K to 618.5 K — the
gaps are the near-critical samples the table declines) and nine quality lines
`x = 0.1 … 0.9` (83 of 120 points each). The **seven isobars come back empty** —
see non-delivery 4.

---

## Fixtures

```
fixtures/corpus + fixtures/golden   268 promoted (was 204)   → 361 after Phase 6
fixtures/corpus-pending             29 staged                → 31  after Phase 6
```

### Promoted this phase (64)

Three came out of `corpus-pending` — `rankine-cycle`, `rankine-cycle-2`,
`refrigeration-vcr`, the first documents in the project's history to solve
against real-fluid property values. The other 61 are new, written to pin each
ported area against the oracle:

| Group | Fixtures |
|---|---|
| Real-fluid states | `props_realfluid_water_states`, `props_realfluid_r134a_states` |
| Chemistry / combustion | `chem_equilibrium`, `chem_errors`, `chem_flame_temp`, `chem_heating_value`, `chem_idealgas`, `chem_mixture`, `chem_molar_mass`, `chem_nasa7`, `adiabatic-flame-temp`, `props_combustion_phi_sweep`, `props_equilibrium_phi_sweep`, `err_equilibrium_h2_flametemp`, `err_equilibrium_h2_singular` |
| Compressible flow | `compressible-isentropic`, `compressible-normal-shock`, `compressible-oblique-expansion`, `compressible-rayleigh-fanno`, `props_compressible_sonic_point`, `karman-rocket`, + 6 `compressible_*_rejected` error fixtures |
| Cubic EOS | `cubic-eos-properties`, `eos-cubic-spot-probe`, `eos-cubic-sweep`, `props_cubiceos_root_selection` |
| Ideal gas / NASA-7 | `props_idealgas_range`, `props_idealgas_inverse`, `props_nasa7_range_ends`, `props_nasa7_switchover`, `err_idealgas_t_from_h_diverges` |
| Heat exchangers | `hx-correlations`, `props_hx_ntu_limits`, `hx_unknown_arrangement_rejected`, `hx_unreachable_effectiveness_rejected`, `err_hx_ntu_unbracketed` |
| Convection / flow / two-phase | `convective-correlations`, `flow-resistance-duct`, `flow-resistance-transition`, `twophase-lockhart-martinelli`, `twophase-void-friedel`, `twophase_martinelli_quality_rejected`, `pneumatics-iso6358`, `pneumatics_iso6358_bad_b_rejected` |
| Transport / solids / Heisler | `gas-transport-mixture`, `props_transport_sweep`, `solid-materials`, `props_solids_temperature`, `material-conduction`, `multi-objective-beam`, `solid_absent_property_rejected`, `solid_unknown_material_rejected`, `heisler-geometries`, `heisler_unknown_geometry_rejected` |
| Atmosphere | `isa-atmosphere`, `props_atmosphere_layer_seam` |

### Pending: 29, replayed document-by-document

> **Phase-6 annotation (2026-07-31).** The list is now **31** — see
> [`docs/status-phase6.md`](status-phase6.md#pending-31-replayed-document-by-document)
> for the current, re-replayed classification. Deltas: the three
> `COMPONENT`-blocked documents all get past the component layer now and have
> moved to other rows; two humid-air component documents were newly staged.
> Row-level annotations are inline below; nothing is deleted.

Every staged document was replayed through the current Rust engine and its
failure classified. **1 solves, 3 both-refuse-with-different-classification,
25 the Rust engine refuses.**

| # | Blocked on | Documents |
|---|---|---|
| 6 | **Phase-9 control-systems CALLs** (`lqr`, `lqe`, `c2d`, `routh`, `residue`, `tf2ss` — refused by name from `UNPORTED_CALL_INTRINSICS`) | `controller-design-lqr-pid`, `estimator-gramian-balreal`, `digital-control-c2d`, `routh-stability`, `inverse-laplace-residue`, `multi-output-destructuring` |
| 5 | **`PLOT` blocks** ~~(Phase 7)~~ → **Phase 9** *(label corrected 2026-07-31; see the note below the table)* | `control-analysis-report`, `cruise-control`, `nichols-chart`, `root-locus-analysis`, `step-impulse-response` |
| 5 | **`DYNAMIC` (ODE/DAE) blocks** ~~(Phase 8)~~ → **Phase 7** *(label corrected 2026-07-31)* · *Phase 6: now **7** — joined by `ev-battery-cooling-pid` and `pressure-cooker` from the row below* · **Phase 7: all seven resolved — five promoted, two moved on** | `damped-oscillator-ode`, `engine-cycle-wiebe`, `newton-cooling-transient`, `sounding-rocket-trajectory`, `transient-heat-rod` |
| 3 | ~~**`COMPONENT` instantiation** (Phase 6)~~ **Closed 2026-07-31 (Phase 6)** — all three now expand cleanly through the component layer and fail later: `ev-battery-cooling-pid` and `pressure-cooker` on `DYNAMIC` alone, `ev-thermal-management` on the missing `INCOMP::MEG[0.50]` property table (it reaches block 3 of 89 equations). None is promoted; each moved to a different blocker | `ev-battery-cooling-pid`, `ev-thermal-management`, `pressure-cooker` |
| 3 | **`PARAMETRIC` blocks** — error fixtures where the classifications still disagree: Java raises `SolverException` (underspecified when solved directly), Rust raises `ParseException` (block type unsupported). Both refuse; the gate compares classification | `damped-oscillator`, `driving-cycle-energy`, `projectile-trajectory` |
| 2 | **Transport properties the tables do not store** — `Viscosity`, `Conductivity`, `Cp`, `CompressibilityFactor`. These are now the *only* two blocked on Phase 5 itself · *Phase 6: now **5**, joined by `ev-thermal-management` (glycol) and the two newly staged humid-air documents `adv_moistair_W_passthrough` / `adv_moistair_dryair_three_way`, which need `HAPropsSI`. Read the row as "real-fluid coverage the tables do not have", not just transport* | `hx-correlations-fluid`, `thermo-compliance` |
| 1 | **`STATE TABLE` block type** (plus the transport properties above) | `state-tables-multifluid` |
| 1 | **`SYMBOLIC` / CAS** — Symja replacement undecided (Phase 9) | `partial-fractions` |
| 1 | **String variables** — `geom$ = 'wall'` not ported | `heisler-transient` |
| 1 | **`MODULE` inside `FOR`** — pipeline-ordering deviation carried from Phase 4 | `module_inside_for_loop` |
| 1 | **Ill-posed by construction — held deliberately.** Structurally square but rank-deficient, so the solution set is a *line*. It passes today (Rust within 6.6e-2 of the Java point) but promoting it would freeze an arbitrary point of a continuum into the gate | `solver_singular_linear_cycle` |

> **Two phase labels in this table were wrong; corrected 2026-07-31 (Phases 7–8).**
> Per [`PLAN.md`](../PLAN.md) §5, **Phase 7 is Dynamics** (`ode/`, `dae/`,
> `LINEARIZE`) and **Phase 8 is Analysis & design** (optimizer, NSGA-II,
> parametric sweeps, curve fitting, Monte Carlo, uncertainty). So:
>
> * the `DYNAMIC` row was labelled "Phase 8" and belongs to **Phase 7** — which
>   has now closed it: five of those documents were promoted and the other two
>   moved to other blockers;
> * the `PLOT` row was labelled "Phase 7", but the five documents in it are
>   **control-systems** documents. `PLOT` is not what is actually missing —
>   `rlocus`, `nichols`, `step`, `pole` and `tf` are, and those are
>   **Phase 9** (CAS & control systems). Phase 7 shipping did not move any of
>   them, which is the observable confirmation that the old label was wrong.

**29 total** *(31 after Phase 6 staged two humid-air component documents; no
pending document was promoted by Phase 6)*. Phase 4 listed 13 documents across
its "CoolProp-poisoned" (8) and
"property / material kernels" (5) groups. Phase 5 **promoted 8 of them**
(`rankine-cycle`, `rankine-cycle-2`, `refrigeration-vcr`,
`adiabatic-flame-temp`, `cubic-eos-properties`, `karman-rocket`,
`material-conduction`, `multi-objective-beam`). Of the 5 that remain, four are
now blocked on Phase 6/7/8 block types first (`ev-battery-cooling-pid`,
`ev-thermal-management`, `pressure-cooker`, `state-tables-multifluid`), and only
`thermo-compliance` is still blocked on Phase 5 itself. `hx-correlations-fluid`
is new this phase and joins it — **two documents wait on transport properties,
not on more table resolution.**

---

## What Phase 5 did **not** deliver

Ranked by how likely each is to bite the next session.

1. **The wasm bundle is at 91.1 % of its budget — 181 KiB of headroom left.**
   1147.7 KiB → 1866.6 KiB, of which 526 KB is the two linked `.phtab` files and
   the rest is the property code. **A third fluid costs another ~263 KB and does
   not fit**, and neither does Phase 6's component library at any meaningful
   size. This is the single hardest constraint the next phase inherits, and it
   has exactly three exits: raise the budget deliberately, move the tables out of
   the module (see 3), or drop to a coarser grid. Nothing in this phase chose
   between them.

2. **The tables are a *substitute* for CoolProp, not a wrapper around it, and
   the parity gate now says so.** Five fixtures compare at `1e-5`…`2e-4` instead
   of `1e-9`. The guards make the relaxation visible and self-expiring, but it is
   still a weaker gate than Phase 4 shipped. Shipping `coolprop.wasm` as the
   accuracy path — option A in D1, still open — would restore `1e-9` on all five
   and unblock transport properties, humid air, mixtures and supercritical
   states in one move. It was not attempted.

3. **The tables are linked into the wasm, which contradicts D1's own
   plan.** D1 says "lazily fetched per fluid, not linked into the wasm, so the
   wasm budget is untouched". They are linked, because a solve is synchronous —
   `PropsSI` is called from inside the Newton residual and there is no point in
   that stack where a Rust engine can await a `fetch`. The seam D1 wanted still
   exists (`props::tables::install_from_bytes` takes an arbitrary `FRPHTAB1`
   slice), but nothing in `web/` uses it, and no wasm export surfaces it. Doing
   it properly means pre-fetching at worker start-up — same bytes, different
   moment — or an async property source. **This is the direct cause of 1.**

4. **Property diagrams draw the dome and quality lines; the isobars, isentropes
   and isotherms are empty.** Measured in the browser: 9 quality lines and the
   saturation dome carry real points, all 7 isobars carry zero. The cause is a
   half-kelvin: `PropertyDiagrams.sweepEntropyAtPressure` starts its sweep at
   `T_triple + 0.5 = 273.66 K` and the generated water table's cold end is
   `t_low = 274.16 K`, so the first probe declines and the sweep produces
   nothing. The isotherm sweep needs `(T, D)` — an input pair with no pressure,
   which the split geometry cannot invert at all. Fixing the first is a
   generator change (`t_low`), not a diagram change; the second needs a real
   backend. **A user opening a P–h or T–s chart today sees a dome and quality
   lines on an otherwise empty grid.**

5. **The psychrometric chart cannot draw.** `HAPropsSI` is three-input on a
   different manifold and the tables have no humid-air piece, so
   `psychrometric_chart()` returns an honest error body for every request.
   `psychro.rs` is ported and tested against a stand-in backend; it has nothing
   to call. Every `AirH2O` document is refused by name.

6. **`v_crit` and `MolarMass` of a tabulated fluid go the long way round.**
   `FRPHTAB1` carries `p_crit`, `t_crit`, `p_triple` and `t_triple` but not
   `rhocrit` or `molar_mass`, so `v_crit(Water)` errors and `MolarMass(Water)`
   falls through to the formula parser (which gets 18.015 g/mol from `H2O` — a
   right answer by a different route, and a wrong one for any fluid whose alias
   is not a formula). Adding two `f64` to the generator's header would close it.

7. **The `(P,h)` inverse on a two-phase plateau is refused, and the refusal is
   correct but coarse.** `Enthalpy(Water, P, T=T_sat(P))` has a whole interval of
   answers, so the bisection declines. A real CoolProp answers it by convention
   (the liquid root). Any document that asks for enthalpy at exactly the
   saturation temperature gets an error where the reference gets a number.
   Pinned by
   `props_robustness.rs::an_inverse_lookup_on_a_two_phase_plateau_is_refused_rather_than_guessed`,
   which accepts either behaviour so a future backend does not break it.

8. **R134a's superheat ceiling declines 40.5 % of a wide sampling band.** Every
   miss is superheat deeper than the served 132.7 kJ/kg, where CoolProp itself is
   extrapolating past its declared `Tmax`. All current document states are
   inside, but a superheated-R134a document written next week may not be.

9. **The speed half of the D1 spike is still unmeasured.** D1 said so and it is
   still true: no end-to-end benchmark of a Rust table lookup versus a
   `coolprop.wasm` call across the JS boundary, and no solve-time comparison on a
   whole document. `PLAN.md` §4 asked for it. The tables are *assumed* fast
   because array interpolation is three to four orders below a 734 µs JNA call;
   that assumption has not been tested in a browser.

10. **`tools/table-gen` and `props/satsplit.rs` build the same object at
    different resolutions and cannot check each other.** The generator is Java on
    a 512/144/72 grid with a normalized liquid coordinate; `SaturationSplitTable::build`
    is the faithful Java port, hard-coded at 256/96/48 with the absolute
    coordinate, and its `FREESSP1` serialisation is a different format from the
    generator's `FRPHTAB1`. Both readers exist and both are tested, but **no test
    builds a table with `build()` and compares it to a generated one**, because
    they cannot be made to agree by construction. The cross-check that does exist
    (`TableGen` vs. the reference `SaturationSplitTable`, bit-identical) lives on
    the Java side.

11. **Everything Phase 4 did not deliver is still not delivered** — the 43
    refused CALL intrinsics, `MODULE` inside `FOR`, the SVD sign convention, the
    quadratic kernel-CALL memory, the ignored slow-quadrature test, and the
    newline tolerance inside `[...]`/`(...)`. See
    [`docs/status-phase4.md`](status-phase4.md#what-phase-4-did-not-deliver); none
    of them moved.

---

## Next

1. **Decide the bundle question before writing more code.** Non-delivery 1 is
   load-bearing for Phases 6–9. The cheapest honest answer is probably to move
   the `.phtab` files out of the module and pre-fetch them in the worker, which
   costs one JS change and buys back 526 KB.
2. **Phase 6 — the component/connect layer**, and with it the 295 library
   components as corpus. Three pending fixtures are waiting on it.
3. **Phase 8 — `DYNAMIC`.** Five pending fixtures, and it is the other half of
   the three component documents.
4. **Phase 9 — CAS + control systems**, the 43 refused CALL intrinsics, and the
   `SYMBOLIC` block. Seven pending fixtures.
5. **Opportunistic, and cheap**: two more `f64` in the `FRPHTAB1` header close
   non-delivery 6; lowering the generator's `t_low` by a kelvin closes half of
   non-delivery 4.
