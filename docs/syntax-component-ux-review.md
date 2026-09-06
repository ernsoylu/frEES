# Frees syntax, components, and user experience: improvement report

Review date: 6 September 2026. Repository baseline: `2e79ff1`, with existing local solver changes present. Scope: this Rust/WebAssembly port and its browser interface.

**Recommendation:** retain the equation language and component architecture. Make the editor, wizard, documentation, schematic, and saved project agree on that language before introducing new syntax. The highest-value simplifications remove contradictory behavior and unnecessary decisions, rather than removing physics.

**Evidence and limits.** This review traces the parser, component expansion and library, generated catalogs, editor assistance, wizard generation, schematic interactions, project representation, and selected documentation. It includes executable native CLI probes, direct execution of frontend helpers, and 35 passing existing frontend tests. External research was limited to primary Modelica and W3C documentation. This is a source-based UX review, not a completed browser usability study, accessibility certification, or numerical audit of every component. Findings below distinguish reproduced behavior, source observations, and proposed improvements. No application code or frozen fixtures were changed.

## 1. What works and should remain

Frees has a strong small core: equations express relationships, components package equations, and connections generate conservation relationships. Plain equations and networks share a solver. The standard library is ordinary `.frees` source, making its physics inspectable and its authoring path available to users.

The following capabilities should be explicit preservation requirements:

- Order-independent, acausal equations and SI conversion of annotated literals.
- Components mixed with equations, member references, hierarchical composition, and user-defined components.
- Named physical parameters, per-model requirements, and explicit physical inputs rather than guessed defaults.
- Domain and fluid-family compatibility checks, conservation equations, and explicit mixing components.
- Both explicit connections and positional shared streams, including their different branching restrictions.
- Steady, transient, and linearized analyses; signal and physical connections remain distinct.
- Tables, maps, property functions, uncertainty, controller design, and advanced procedural/matrix features.
- Source-readable diagnostics, live Check, revision tracking, schematic navigation, and ordinary text emitted by UI helpers.
- Local browser execution, project storage, and preservation of legacy project data.

These are already implemented surfaces, not recommendations to build them again. In particular, the application already has live lint, autocomplete, signature infrastructure, a component wizard, click-to-wire schematic editing, project saving, and stale-response protection.

Evidence: [library loading](../crates/frees-core/src/components/library.rs), [expansion](../crates/frees-core/src/components/expander.rs), [component guide](../web/src/docs/components.md), [application](../web/src/App.tsx), [revision tracking](../web/src/modelRevision.ts), [project format](../web/src/project.ts).

## 2. Current syntax and interaction model

| Task | Current surface | Experience consequence |
|---|---|---|
| Express a relationship | `P * V = m * R * T` | Compact and familiar; users must learn that equality is not assignment. |
| Supply an engineering value | `P = 200 [kPa]` | Convenient SI conversion; units on arbitrary expressions are a different issue. |
| Name a string | `fluid$ = 'Water'`; component arguments also accept bare names | Useful compatibility, but inconsistent-looking examples increase learning effort. |
| Instantiate a device | `Pipe LINE(fluid$=Water, L=50, D=0.05, rough=0.0001)` | Named parameters are readable, but long argument lists need reliable assistance. |
| Wire devices | `connect(A.out, B.in)` | Ports are explicit and branching is supported. |
| Wire a series chain | `Pipe LINE(s1, s2, ...)` | Shorter, but port ordering and shared-stream restrictions become implicit. |
| Select physics | `model$=volumetric`, with `VARIANT ... REQUIRE ...` in the definition | A good existing mechanism; selection must match actual required inputs. |
| Package equations | `COMPONENT`, `MODULE`, `FUNCTION`, `PROCEDURE` | Several reusable forms exist for different jobs; a decision guide is needed. |
| Specify guesses | `GUESS x = 2 [0, 10]`, or Variable Information | Bounds use brackets too; component member paths are not accepted by this directive. |
| Run a transient | `DYNAMIC name(method=ida, time=0..600, points=601) ... END` | The same network can be reused, but initialization and solver choices need explanation. |
| Supply sampled data | Text `TABLE` or GUI function table | Useful dual authoring routes; precedence and portability must be visible. |
| Save work | Plain equation text and a JSON project both use `.frees` in this repository | The user needs to know which state travels with an export. |

Sources: [language fundamentals](../web/src/docs/language_fundamentals.md), [programming guide](../web/src/docs/programming_logic.md), [expression parser](../crates/frees-core/src/parser/expr.rs), [top-level parser](../crates/frees-core/src/parser/toplevel.rs), [project representation](../web/src/project.ts).

## 3. Prioritized improvement points

Priority definitions: **P0** prevents generated model corruption or misleading interpretation; **P1** removes recurring authoring and recovery friction; **P2** improves learning, discovery, and advanced workflows. Effort is relative: **S** is a localized change, **M** spans existing surfaces, **L** changes shared metadata or grammar. These are planning estimates, not measured delivery times.

### R1 — Preserve expressions and units in wizard output

**P0 · M · Reproduced.** The wizard advertises “value or variable,” but `generateComponentText` appends a unit to every non-string parameter with a known unit. Using the real `LiquidWallHX` catalog entry:

| UA field | Generated argument | Native Check outcome |
|---|---|---|
| `10` | `UA=10 [W/K]` | Intended literal annotation. |
| `conductance` | `UA=conductance [W/K]` | Introduces additional variables `w` and `k`; the bracket is interpreted through array syntax. |
| `10 [W/K]` | `UA=10 [W/K] [W/K]` | Syntax error: expected `)`, found `[`. |

The variable case is more serious than a rejected insertion: it changes the model's unknowns. The UA builder also passes a variable name into this generation path, so the shared generator is the right repair location.

**Change:** append the field's unit only to a plain numeric literal. Preserve an explicit unit, variable reference, or expression as source text, and validate using the existing parser. Keep engineering-value entry convenient without attempting a second expression parser in the form.

**Acceptance:** numeric, signed/offset-temperature, explicitly unit-bearing, variable, and arithmetic inputs round-trip with the intended value and unknown set. A Compute UA insertion checks without phantom unit variables. Keep validation distinct from whole-network solvability: an unwired component may legitimately be underdetermined.

Evidence: [generator](../web/src/componentText.ts), [wizard and UA callback](../web/src/ComponentWizardModal.tsx), [expression parser](../crates/frees-core/src/parser/expr.rs).

### R2 — Make the shipped engine authoritative for component discovery

**P1 · M · Measured and source-confirmed.** There are **312** component declarations in the embedded library and **295** entries in the generated wizard catalog. Seventeen moist-air additions are absent. Introductory documentation also says approximately 295. The documentation manifest reads component inventory from the sibling Java repository, so an upstream inventory cannot establish coverage of Rust-only additions.

**Change:** extend the existing catalog/manifest pipeline to take structural facts from this port's parsed library: names, ports, parameters, actual defaults, and variant requirements. Keep human descriptions in Markdown. Generate compact editor names and rich wizard data as separate outputs where that avoids loading unnecessary content; generated duplication is not itself a defect.

**Acceptance:** engine and catalog name sets agree; every shipped component is searchable, documented, and available to completion. The check runs without requiring a sibling Java checkout. Keep Java comparison as a separate compatibility check. Current missing names are listed in section 7.

Evidence: [library count and files](../crates/frees-core/src/components/library.rs), [catalog](../web/src/componentCatalog.ts), [manifest source selection](../web/scripts/build-doc-manifest.mjs), [documentation compiler](../web/scripts/compile-docs.js).

### R3 — Repair component signature help and contextual completion

**P1 · M · Reproduced/source-confirmed.** Executing `activeCallAt` on `Compressor CMP(fluid$=R134a, eta=` returns the callee `CMP`; signatures are keyed by `Compressor`. The helper also scans only the current line. The completion source offers a flat combined list of functions, variables, and component types, rather than parameter or port choices for the current context.

**Change:** resolve `Type Instance(...)` to its type, support multiline argument lists, and identify named arguments independently of comma position. Add completion for active parameters and `instance.port.member` paths using the same metadata as R2. Include local component definitions. Fix the existing highlighter's missing `COMPONENT`, `PARAM`, `VARIANT`, `REQUIRE`, `CONNECT`, `LINEARIZE`, and `GUESS` recognition; double-quoted comments should not appear as string literals.

**Acceptance:** `Compressor CMP(` shows compressor help, a continuation line retains it, and reordered named arguments highlight the correct parameter. Completing after a dot inserts only the missing path segment. Quoted strings and comments do not confuse argument counting. No language-server service is necessary for this first pass.

Evidence: [editor](../web/src/EquationEditor.tsx), [engine tokens](../crates/frees-core/src/token.rs), [lexer](../crates/frees-core/src/lexer.rs).

### R4 — Make variant defaults and inactive inputs explicit

**P1 · M · Source-confirmed; inactive-input behavior reproduced.** The UI assumes the first variant is the default, while the engine uses the declared `model$` default. `ComponentSpec` has no actual default-value field. The selector displays “default” without identifying the model. Documentation claims irrelevant variant inputs are not accepted; the expander accepts declared parameters even when the chosen variant does not use them. An isentropic compressor with `rpm=2900` produces no inactive-parameter warning in Check.

**Change:** show the engine's actual selected model, and explain which inputs become active when it changes. Warn about explicitly supplied inactive parameters without breaking old models. Do not silently change a model to the first catalog entry or delete user text. Form drafts may retain inactive values while omitting them from newly generated code, with that behavior made visible.

**Acceptance:** reordering variant documentation cannot change form behavior. UI and engine agree for a non-first default and for a component with no default. An unused `rpm` produces an actionable advisory. Numeric physical defaults remain explicit decisions, not a side effect of this cleanup.

Evidence: [variant resolution](../crates/frees-core/src/components/variant.rs), [parameter resolution](../crates/frees-core/src/components/expander.rs), [UI selection](../web/src/componentText.ts), [catalog schema](../web/src/componentCatalog.ts).

### R5 — Prevent avoidable wizard insertion errors

**P1 · M · Source-confirmed.** Instance-name validation checks identifier shape, not uniqueness in the document. `suggestInstanceName` returns the same base for repeated insertions. Add is gated by name shape and nonempty required fields, not valid Frees expressions. String parameters other than variant selectors are text inputs, despite the guide describing known-fluid choices.

**Change:** suggest an unused name with case-insensitive collision checks; show parser errors at the relevant field. Offer searchable fluid suggestions while retaining custom valid strings. Quote generated string literals where necessary. Preserve explicit physical inputs and the current text preview.

**Acceptance:** inserting two resistors yields distinct editable names; `R1` conflicts with `r1`; malformed expressions cannot be presented as valid additions. A fluid string such as `'INCOMP::MEG[0.50]'` survives generation. Custom fluid/table names remain possible.

Evidence: [wizard props and gating](../web/src/ComponentWizardModal.tsx), [name and text helpers](../web/src/componentText.ts).

### R6 — Validate schematic connections before committing text

**P1 · M · Source-confirmed.** `clickPort` checks whether endpoints belong to the same instance, then appends `connect(...)`. It does not check domain compatibility or whether the endpoints are already connected. Engine rejection exists downstream, but the UI announces “Wired” before that check succeeds.

**Change:** preview endpoint compatibility using existing domain metadata, and validate the proposed edit through Check where necessary. Distinguish “connection added; checking” from a validated connection. Reuse connection-set semantics for duplicate detection; do not compare raw strings alone. Keep the engine as the authority for signal writers, riders, families, and junction rules.

**Acceptance:** an electrical-to-fluid attempt names both incompatible endpoints; reconnecting an existing node does not add redundant text; valid branches and loops remain legal. One Undo removes the generated statement. Unknown/custom connector information does not trigger invented physics assumptions.

Evidence: [schematic interaction](../web/src/schematic/SchematicTab.tsx), [domain rules](../crates/frees-core/src/components/domains.rs), [connection expansion](../crates/frees-core/src/components/expander.rs).

### R7 — Support keyboard and touch access to schematic actions

**P1 · M · Source-confirmed gap; browser verification required.** Interactive SVG node/port groups have pointer handlers without focusability or keyboard handlers. The overall SVG is labeled as an image. Port circles have a normal radius of 3.2 SVG units, creating small targets at ordinary zoom.

**Change:** expose focusable node and port actions with accessible names, Enter/Space activation, Escape cancellation, visible focus, and a larger invisible hit area. A compact connection list with two endpoint selectors can provide an accessible alternative without rebuilding the diagram. Provide a simple non-dragging route for repositioning if manual layout remains an interactive feature.

**Acceptance:** a keyboard user can inspect a node, reveal its source, connect ports, and cancel selection. Touch users can select ports without precision tapping. Test zoomed and narrow viewports, focus restoration, and readable labels beyond color.

W3C treats keyboard operation and alternatives to dragging as separate requirements; keyboard support alone does not satisfy the non-dragging pointer requirement. See [WCAG 2.2 keyboard criterion](https://www.w3.org/TR/WCAG22/#keyboard) and [W3C explanation of dragging alternatives](https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements.html). This recommendation is not a claim that the whole application has been audited for conformance.

### R8 — Carry public component paths through guesses and source navigation

**P1 · L · Dotted-guess failure reproduced; source-navigation duplication confirmed.** The language allows `HX.in.P` in equations, but `GUESS HX.in.P = 200000` is rejected because the directive expects one identifier. The UI's guess reader similarly recognizes only simple names. Meanwhile schematic declaration lookup scans text because component declarations lack the source positions it needs.

**Change:** extend guesses/bounds to the existing member-reference path syntax, resolving through the same public-to-expanded mapping used by equations. Carry declaration spans and instance identity through Check metadata to gradually replace duplicate frontend scanning. Keep this additive; do not expose internal `$`-mangled names as the authoring API.

**Acceptance:** a pressure guess on a component port survives text export/import and agrees with Variable Information. Bounds address the same scalar used by the solver. Clicking a repeated local child name in a hierarchical model reveals the correct declaration/scope. Existing scalar guesses remain unchanged.

Evidence: [GUESS parser](../crates/frees-core/src/parser/toplevel.rs), [guess UI helpers](../web/src/guessDirectives.ts), [schematic declaration scanning](../web/src/schematic/declaration.ts).

### R9 — Expose definition provenance and case collisions

**P1 · M · Source-confirmed; case collision reproduced.** A user component silently shadows a built-in of the same name. This is useful customization, but a user can believe they are using the library's physics. Case insensitivity also means `T=300` and `t=2` describe one unknown with conflicting equations; Check reports overspecification without specifically explaining the case collision.

**Change:** show “local definition” versus “standard library” in component help and schematic inspection. Add a nonblocking shadowing advisory. Explain conflicting case-only spellings when they contribute to a declaration/equation conflict; avoid warning on every harmless case variation.

**Acceptance:** defining a local `Pipe` clearly identifies the override while preserving it. A `T`/`t` conflict points to both source lines and explains canonical naming. Local component help uses the local parameter schema.

Evidence: [library resolution contract](../crates/frees-core/src/components/library.rs), [parser](../crates/frees-core/src/parser/toplevel.rs).

### R10 — Make validity levels visible in results

**P0 · M · Unknown-unit behavior reproduced; presentation recommendation.** `x = 10 [bananas]` is reported as structurally solvable, with a warning that the value was left unconverted. This is compatible behavior, but the generic success message can be mistaken for physical validity. Equal equation/unknown counts also do not prove numerical convergence or a meaningful physical solution.

**Change:** distinguish syntax validity, structural solvability, unit status, and numerical convergence in the existing status/result surfaces. Keep actionable warnings attached to the model revision and visible in exported reports. Unknown units must not be summarized as verified SI values. Consider stricter unknown-unit rejection only as a deliberate compatibility decision, rather than changing legacy behavior accidentally.

**Acceptance:** the unknown-unit probe never presents an unqualified “validated” result. A unit warning links to its source. Under/overspecification, convergence failure, and worker failure have distinct next actions. Preserve existing revision protection; no duplicate request-state framework is needed.

Evidence: [unit parsing policy](../crates/frees-core/src/parser/expr.rs), [warning banner and request handling](../web/src/App.tsx), [revision tracking](../web/src/modelRevision.ts).

### R11 — Turn diagnostics into the next modeling action

**P1 · M · Existing foundation confirmed; proposed extension.** Check already reports free quantities, redundant relations, component member names, and explanations of common causes. Build on this instead of adding another general diagnostics screen.

**Change:** group free quantities by component/circuit, highlight the corresponding schematic items, and offer source navigation to the related equation. Provide context-specific explanations such as “pressure level is unpinned” or “this machine defines efficiency but not flow,” without automatically inserting a guessed boundary. Change “Could not reach the solver backend” to wording that accurately describes the local worker/engine failure.

**Acceptance:** a source–device–sink exercise with one missing boundary names the relevant free quantity; a duplicated boundary identifies the competing relation. A user can reach the source from the explanation. Automatic correction never chooses a physical value.

Evidence: [structural diagnostics](../crates/frees-core/src/engine.rs), [application error text](../web/src/App.tsx), [solve diagnostics UI](../web/src/SolveDiagnostics.tsx).

### R12 — Group components by modeling choice before merging them

**P2 · M · Physics differences source-confirmed.** The library mixes domain, device, application, and fidelity distinctions in names. Broad claims that fidelity is never duplicated overstate the current consistency.

**Change:** present related models together with columns for required data, ports, flow closure, energy closure, steady/transient use, and assumptions. A “fan” search should help users select a model from their available data. Use aliases/search tags for discoverability before introducing runtime aliases or renaming types.

**Acceptance:** users can distinguish `Fan`, `FanCurve`, and `FanMap` without reading all three equation bodies. Old type names remain searchable and loadable. Any later merger must pass the equivalence checklist in section 4.

Evidence: [fluid component equations](../crates/frees-core/src/components/library-data/fluid.frees), [catalog filtering](../web/src/ComponentWizardModal.tsx).

### R13 — Teach one default authoring style and retain advanced forms

**P2 · S · Documentation recommendation grounded in current grammar.** The shortest possible text is not always the easiest model to inspect. Multiple syntax forms are manageable when the introductory path is consistent.

**Change:** use named component parameters and explicit `connect` in introductory examples and UI output. Teach positional streams afterward as a compact series-chain alternative. Prefer single-quoted string literals in newly generated canonical examples, annotated numeric inputs, and display-unit controls for presentation. Provide a one-page guide distinguishing equation, function, procedure, module, and component use. Explain `time` versus temperature names, bracketed bounds versus units, and absolute temperatures versus differences.

**Acceptance:** a new user can complete one small network using a single connection style, then recognize the equivalent series form. No parser aliases are removed. Do not add arrow wiring, implicit multiplication, a second assignment syntax, or another block terminator merely to reduce character count.

Evidence: [language fundamentals](../web/src/docs/language_fundamentals.md), [programming guide](../web/src/docs/programming_logic.md), [component guide](../web/src/docs/components.md).

### R14 — Clarify ownership of text, form settings, and saved projects

**P1 · M · Existing precedence/persistence confirmed; UX recommendation.** Function tables may be authored in text or the GUI, with text taking precedence on name collisions. GUESS values can exist in text and Variable Information. Slider and REPL overrides also affect solving. Full projects already save substantially more than equation text.

**Change:** show the effective value's origin and precedence next to conflicting inputs. Make “Save project” and “Export equation text” explicit choices, with a short description of what travels. Offer existing text serialization where supported; do not force plots, layout, digitizer state, or legacy opaque data into the equation grammar.

**Acceptance:** a conflicting text/GUI table explains which is used. A project reload reproduces effective inputs and maps. Text export identifies any required project-only inputs. Legacy spreadsheet/analyzer slices remain preserved even though their UI is removed.

Evidence: [table guide](../web/src/docs/programming_logic.md), [guess helpers](../web/src/guessDirectives.ts), [override precedence](../web/src/App.tsx), [project schema and compatibility policy](../web/src/project.ts).

### R15 — Build learning around complete tasks, then verify with users

**P2 · M · Proposed validation program.** Existing getting-started cards, examples, reference pages, mobile layout, and command palette provide the necessary surfaces. More separate onboarding modals would fragment them.

**Change:** curate three short journeys: solve an equation with units; build and inspect a component chain; upgrade a steady model to a transient or map-driven model. Each should show an expected result, one deliberate error, and its recovery. Include explicit mixer and closed-loop examples after the first chain. Link to the authoritative reference rather than duplicating parameter tables in every tutorial.

**Acceptance:** representative users complete the tasks below without instructor intervention. Measure a baseline first; target values are product goals, not results established by this review.

| Task | Measure | Initial target |
|---|---|---|
| First scalar solve with units | Time and unaided completion | At least 4 of 5 pilot users within 5 minutes. |
| Source–pipe–sink network | Wrong insertions/connections and completion | At least 4 of 5 within 10 minutes. |
| Recover from missing boundary | Correct diagnosis and source navigation | At least 4 of 5 within 3 minutes. |
| Change compressor fidelity | Awareness of new inputs and preserved wiring | No participant silently runs the wrong variant. |
| Enter variable-backed UA | Correct generated source | Every test case preserves the expression and unknown set. |
| Save/reopen model with a map | Effective input/result agreement | No missing table or hidden override. |
| Keyboard/touch inspection and wiring | Task completion, focus, target selection | No required pointer-only step in the keyboard journey. |

Use separate novice and experienced engineering participants when possible. A five-person pilot finds friction; it does not establish population-level performance or numerical correctness.

## 4. Duplication audit: what to consolidate and what to preserve

| Apparent duplication | Finding | Lean action |
|---|---|---|
| Engine library, Markdown, wizard catalog, completion names | Actual structural drift: 312 versus 295. Separate generated payload sizes are useful. | One authoritative structural inventory, multiple generated views; R2 owns this work. |
| UI variant rules versus engine rules | Different default-selection representations and unclear inactive inputs. | Actual defaults and requirements from shared metadata; R4. |
| Frontend declaration/guess/call scanners | Several partial understandings of the same language. | Share semantic identity/spans incrementally; fix immediate local errors first; R3/R8. |
| `Compressor(model$=map)` and `CompressorMap` | Partial conceptual overlap, not drop-in equivalents: the former calls `map_eta$(PR, rpm)`; the latter uses `map_eta$(PR)` and can also constrain mass flow. | Group in discovery. Merge only with explicit map dimensions and flow-closure preservation. |
| `Fan`, `FanCurve`, `FanMap` | Similar pressure-rise task; `Fan` also computes an enthalpy rise using fluid properties and efficiency. The curve/map forms use supplied density and do not impose that energy equation. | Preserve physical distinctions; show assumptions before considering variants. |
| `Pump` and `PumpMap` | Efficiency/energy relation versus a head–flow characteristic. | Do not delete either as redundant. Explain whether flow is actually determined. |
| Domain-specific pipes, sources, and valves | Repeated shapes can encode different connector guards, state bases, and constitutive laws. | Keep domain safety; compare equations before reducing internal repetition. |
| `connect` and shared streams | Two useful authoring styles, but shared streams have a two-port restriction. | Canonical introductory style, advanced shorthand retained; R13. |
| Raw boundary equations and source components | Equivalent constraints in some cases, but useful at different levels of reuse/discovery. | Keep both; diagnose when both constrain the same quantity; R11. |
| `FUNCTION`, `PROCEDURE`, `MODULE`, `COMPONENT` | Shared reuse intent does not make evaluation or connection semantics interchangeable. | A task-based decision guide, not a universal new block construct. |
| Text tables, GUI tables, digitized maps | Different ways to acquire the same callable data; source precedence matters. | Shared table semantics and visible ownership; R14. |
| Signal math blocks and scalar equations | Algebra can overlap, but explicit signal topology is useful for control diagrams. | Retain both; do not require a block for a one-off arithmetic expression. |
| Legacy project slices with removed UI | Compatibility data, not an active duplicate feature. | Preserve opaque data; do not revive removed UI or delete user data. |

**Merger gate:** before replacing two component types with variants, compare port identities and families; parameter meaning and units; table arity; energy/mass/species equations; equation/unknown balance; public outputs; dynamic states and initialization; sign/reversal behavior; supported operating range; and existing documents. An adapter or retained old type must preserve these. Matching one nominal output is insufficient evidence of equivalence.

External comparison reinforces this caution: Modelica's derivation of stream equations treats convective mixing as a mass/energy-balance problem with special handling, rather than ordinary equality of connector values. The recommendation here is to preserve and explain Frees's explicit-mixer contract, not to import Modelica's entire stream system. See [Modelica stream-equation derivation](https://specification.modelica.org/master/derivation-of-stream-equations.html); the fetched page is a development specification and is used for conceptual comparison only.

## 5. A lean syntax target without a language rewrite

The target should be **fewer rules a user must remember**, while keeping advanced expressiveness available.

This example uses existing syntax; quoting and formatting represent a recommended canonical style, not a proposed new grammar:

```frees
Source SUP(fluid$='Water', mdot=2 [kg/s], P=300 [kPa], T=298 [K])
Pipe LINE(fluid$='Water', L=50 [m], D=0.05 [m], rough=0.0001 [m])
Sink RET()

connect(SUP.out, LINE.in)
connect(LINE.out, RET.in)

dP = SUP.out.P - RET.in.P
```

For a variable-backed parameter, the correct intended pattern is already concise:

```frees
conductance = 800 [W/K]
LiquidWallHX HX(fluid$='Water', UA=conductance)
```

The second fragment illustrates parameter binding only; it needs boundary conditions and connections to form a solvable network. The UI should not append `[W/K]` to `conductance`.

Recommended policy:

1. Keep `=` as the declarative relationship operator and named component parameters as the default.
2. Keep explicit `connect` as the general wiring form; do not add arrows or variadic shorthand that suggests series connection but actually creates one node.
3. Keep `$` strings and case-insensitive lookup for compatibility; make quoting and name collisions predictable through tooling.
4. Keep literal units, display units, and temperature conversions semantically distinct. Do not shorten them by silently changing what a value means.
5. Use the existing `model$` mechanism where models share a stable interface. Do not force physically distinct devices into a single type to reduce catalog count.
6. Make the one justified additive grammar improvement—public member paths in guesses—reuse existing path parsing and resolution.
7. Make generated text readable and undoable. A whole-document formatter can wait until source spans and comment preservation are reliable.

## 6. Delivery sequence and acceptance gates

**First: fix incorrect assistance and missing discovery.** Deliver R1, R2, the signature-help portion of R3, R4, and R5. They address demonstrable defects without requiring new modeling semantics. Update the contradictory guide statements at the same time. Add focused regression coverage for generated text passed to the real parser, rather than checking strings alone.

**Second: make building and recovering reliable.** Deliver R6, R7, R10, and R11; then R8/R9 as shared identity and source metadata become available. Contextual completion can expand from that same information. Do not block the generator fix on a comprehensive parser-metadata redesign.

**Third: reduce choice overload.** Deliver R12–R15, measure the pilot journeys, and revisit component mergers only where users remain confused and the merger gate can be satisfied.

Release acceptance should require:

- No removal of existing syntax or physics capabilities as a side effect of UX cleanup.
- Passing relevant parser/component regressions and native parity checks for semantic changes; retain frozen oracle inputs unchanged.
- Representative browser/WASM checks for wizard-generated component models, custom definitions, variants, maps, and steady/transient behavior. Native CLI evidence alone does not establish browser parity.
- Exact engine/catalog inventory agreement and correct actual defaults.
- No phantom variables from units or malformed generated strings.
- No duplicate instance names or misleading success after invalid wiring.
- Keyboard and touch completion of the focused schematic journeys.
- Save/reopen preservation of effective settings, table data, and existing compatibility payloads.
- A measured before/after user-task comparison. Use completion and error recovery as success measures; fewer tokens or fewer component names alone are not success.

## 7. Evidence record and reproducibility

**Checks performed during this review:**

- Built the current native CLI with `cargo build -q -p frees-cli`.
- Solved the canonical source–pipe–sink example in section 5 successfully through the native CLI, with no diagnostics; checked every local report link for an existing target.
- Executed the real `generateComponentText` against the `LiquidWallHX` catalog entry for literals, references, and annotated values.
- Executed the editor's actual `activeCallAt` helper: the component example returned `{name: 'CMP', argIndex: 1}`, while `sin(` correctly returned `sin`.
- Compared component declarations in the 13 embedded domain files with generated `ComponentSpec` entries: 312 versus 295.
- Ran `componentText.test.ts`, `schematic/declaration.test.ts`, and `modelRevision.test.ts` under Node 22: **3 files, 35 tests passed**. Those existing tests do not disprove the additional reproduced failures.
- Probed native Check for unit suffixes, dotted guesses, case collisions, unknown units, and inactive variant parameters.

**Repeat the most important engine probes:**

```sh
cargo build -q -p frees-cli

# Compare the unknown set with and without the trailing [W/K].
target/debug/frees-cli check <<'EOF'
conductance = 10 [W/K]
LiquidWallHX HX(fluid$=Water, UA=conductance [W/K])
EOF

target/debug/frees-cli check <<'EOF'
conductance = 10 [W/K]
LiquidWallHX HX(fluid$=Water, UA=conductance)
EOF

# Existing dotted-member GUESS limitation.
target/debug/frees-cli check <<'EOF'
GUESS HX.in.P = 200000
EOF

# Structurally solvable, with an unconverted-unit warning.
target/debug/frees-cli check <<'EOF'
x = 10 [bananas]
EOF
```

The UA probes intentionally leave the network unwired. Both are underspecified; the relevant difference is **13 versus 11 unknowns**, with unexpected `w` and `k` only in the suffixed form.

**Missing catalog names at review time:** `ApparatusDewPointCoil`, `ChilledBeam`, `DOAS`, `DesiccantWheel`, `Economizer`, `FaceAndBypassCoil`, `FanCoilUnit`, `FanPoweredBox`, `HeatPipeWrapAround`, `IndirectDirectEvaporativeCooler`, `IndirectEvaporativeCooler`, `InductionUnit`, `LiquidDesiccantContactor`, `RadiantPanel`, `SensibleAirToAirHX`, `SteamHumidifier`, and `TotalEnergyExchanger`.

**Evidence boundary:** existing local changes in `crates/frees-core/src/dae/solver.rs` and `crates/frees-core/src/solver/newton.rs` were left untouched and included in the native build. No complete numerical suite or live browser interaction audit was run for this report. Full-browser reproduction, broader accessibility testing, and participant measurements remain acceptance work for the proposed implementation, not claimed accomplishments of this investigation.
