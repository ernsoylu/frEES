# Changelog

All notable changes to `frees` are documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0](https://github.com/ernsoylu/frees/compare/frees-v0.1.0...frees-v0.2.0) (2026-09-11)


### Features

* **engine:** complete capabilities and make usage trustworthy (Phase 6) ([#10](https://github.com/ernsoylu/frees/issues/10)) ([4cd949e](https://github.com/ernsoylu/frees/commit/4cd949ec6899bcbe9b138595bdc7fa916f9709f7))
* **engine:** implement Phase 4 prepare once solve repeatedly with PreparedDocument ([#8](https://github.com/ernsoylu/frees/issues/8)) ([0bdce8d](https://github.com/ernsoylu/frees/commit/0bdce8d1e8980fd60945835fb3b0fcb195a6caef))
* **engine:** scaled Newton GE, sparse LU scratch, and opt-in Radau (Phase 8) ([33ea6db](https://github.com/ernsoylu/frees/commit/33ea6db5f5722e85bdc4389034b1a01c35a11725))
* implement worker pool for independent sweep rows (Phase 7) ([#19](https://github.com/ernsoylu/frees/issues/19)) ([f83b564](https://github.com/ernsoylu/frees/commit/f83b56470b418a499e8a3513eedbf5a5a42f3269))
* **inspection:** complete Phase 10D engineering inspection across tables and plots ([c30a48d](https://github.com/ernsoylu/frees/commit/c30a48de62a7c9b9228aee29f7a490d10d5c0120))
* **optimizer:** solve each optimization candidate once (Phase 3) ([#7](https://github.com/ernsoylu/frees/issues/7)) ([730c063](https://github.com/ernsoylu/frees/commit/730c063c737a2e5a0427106640b140f3fb0a611f))
* **perf:** Phase 10E preparation, rendering, and persistence optimizations ([#17](https://github.com/ernsoylu/frees/issues/17)) ([6c8d84f](https://github.com/ernsoylu/frees/commit/6c8d84f82b2ef4f3e40b4a35a5a6ce1803785dae))
* **phase-0:** establish browser correctness and performance gates ([#11](https://github.com/ernsoylu/frees/issues/11)) ([68e14ac](https://github.com/ernsoylu/frees/commit/68e14ac52170b479c2bd841835954294e1231cc0))
* **phase-1:** execute Phase 1 operational wins and governance ([41b2ec3](https://github.com/ernsoylu/frees/commit/41b2ec3f6e516d3ce9bbbd688da425a62038118d))
* **plots:** complete Phase 10C plot presentation reproducibility and view persistence ([e670eb5](https://github.com/ernsoylu/frees/commit/e670eb5306f3bd47f0492e2b6a5d7f968c8c5db5))
* **ux:** correct generated source and component assistance (Phase 9A) ([b338eda](https://github.com/ernsoylu/frees/commit/b338edac289cbcaeee615bce882654591c281774))
* **ux:** group related models and teach ownership (Phase 9D) ([6f13c78](https://github.com/ernsoylu/frees/commit/6f13c784db8f2d759c37497457fe95ec52c60a36))
* **ux:** interactive Pareto front point inspection and document loading (Phase 2.5) ([9795e2c](https://github.com/ernsoylu/frees/commit/9795e2c3869c51cb68809b47e46bebf47780ddcd))
* **ux:** share public member paths and component identity (Phase 9C) ([245afc0](https://github.com/ernsoylu/frees/commit/245afc0a3e328662b4ddcf2b91eaa4403ad66099))
* **ux:** validate schematic wiring and distinguish diagnostic levels (Phase 9B) ([2151581](https://github.com/ernsoylu/frees/commit/21515816b42b97e5d5998d4068ef9498a1b96ae1))


### Bug Fixes

* **ci:** allow Unicode-3.0 in cargo-deny licenses ([38eea5a](https://github.com/ernsoylu/frees/commit/38eea5a9d86ef94811fab73c2f3603317a58d7dc))
* **ci:** calibrate CAS test depth and set RUST_MIN_STACK for macOS runner ([8b15ef4](https://github.com/ernsoylu/frees/commit/8b15ef4c527fc4b255b92c8e90aeb635a6e756fe))
* **ci:** correct action SHAs, job-level permissions, and offline route filtering ([7db5eb8](https://github.com/ernsoylu/frees/commit/7db5eb86da8be73f386e3ddb2b09a76db52ffe4b))
* **ci:** sanitize deny.toml schema and make release workflow resilient ([cfd2494](https://github.com/ernsoylu/frees/commit/cfd24941c34e2c48c3c9be710a0d32aa245c7e2b))
* **code-quality:** resolve Sonar quality gate issues across tables and plots ([32a8774](https://github.com/ernsoylu/frees/commit/32a8774436b56932ffd5837a00ca24efa9d8e414))
* **docs:** stop claiming live Graph tags in the calculation report ([9ba22c7](https://github.com/ernsoylu/frees/commit/9ba22c7aa7a74c97e432988f7c406effd97a93f0))
* **engine:** stop Check identity walk on hierarchical cycles ([8b64478](https://github.com/ernsoylu/frees/commit/8b64478e4ab749883ec97772d2edea00db32cfdf))
* **plots:** diagnose unsupported PLOT kinds, types, booleans and slices ([23199d2](https://github.com/ernsoylu/frees/commit/23199d244c2f0ab3ffdaf821e6be704d43f94bcf))
* **plots:** persist explicit sources and require unambiguous legacy bindings ([b81bbd6](https://github.com/ernsoylu/frees/commit/b81bbd6f3704c9fd3be4bf44be42cec11dfdfa6e))
* **plots:** preserve code ownership and offer collision-free editable copies ([df31880](https://github.com/ernsoylu/frees/commit/df31880be3a1525ffd6290778add83418a2233f8))
* **plots:** preserve ordered gaps and report filtered sample counts ([224a9ea](https://github.com/ernsoylu/frees/commit/224a9ea4221cf6e13d94c872d08fe44b583468f2))
* **plots:** resolve ODE solution tables, component variable demangling, and table preservation ([cac191e](https://github.com/ernsoylu/frees/commit/cac191ee8b5450c324db916c45430e9ea0b7c906))
* **plots:** restrict thermodynamic requests to applicable kinds ([91bc170](https://github.com/ernsoylu/frees/commit/91bc170ffa9f42d92210826a0d74b17e0e4e68f6))
* **plots:** sanitize Plotly traces against undefined properties and add Phase 9 Playwright journeys ([#18](https://github.com/ernsoylu/frees/issues/18)) ([85e4ff8](https://github.com/ernsoylu/frees/commit/85e4ff8fe4a704628836f6eb1e44c87a96a38fd8))
* **plots:** validate histogram mesh and bubble channels ([831e4ea](https://github.com/ernsoylu/frees/commit/831e4eab734a0784857e3a57772e48bcf389a7df))
* resolve prepared solver lowering, worker pool lifecycle, and modal render validation ([#20](https://github.com/ernsoylu/frees/issues/20)) ([5211687](https://github.com/ernsoylu/frees/commit/52116877b7ed5565d98fd510683e83924fc71d73))
* **tables:** carry units through functions and convert known knots to SI ([d450795](https://github.com/ernsoylu/frees/commit/d450795b40d2131be73afe81641427535408063a))
* **tables:** classify edit batches atomically and scope undo to user fields ([48afd82](https://github.com/ernsoylu/frees/commit/48afd82757043b73bf6b16114d76d0c3eb2f27d9))
* **tables:** detach legacy formulas across structural edits ([2bb155c](https://github.com/ernsoylu/frees/commit/2bb155ca73f2a57452fb253d2999e2b126ed25ba))
* **tables:** expose sweep convergence passes and provisional values ([d273c84](https://github.com/ernsoylu/frees/commit/d273c840b2aa101a8cc366817c3209f765552256))
* **tables:** own user revisions and reject obsolete check and solve results ([3a9c5d0](https://github.com/ernsoylu/frees/commit/3a9c5d00af6ac8b8e7f830d2e59642fa941c3d33))
* **tables:** preserve full precision and index exact curve knots ([3998ae8](https://github.com/ernsoylu/frees/commit/3998ae89f1cedce69cbe45a7f8ecae3d6b531d49))
* **tables:** preview CSV dialects and keep generated names unique ([741c693](https://github.com/ernsoylu/frees/commit/741c6930c81718dbc819ef5c17deca04bc2c49e6))
* **tables:** require an explicit choice before lookup reduction ([c3b7727](https://github.com/ernsoylu/frees/commit/c3b772785c2b5d6e7157089ab19a275510e3ef0e))
* **tables:** resolve duplicateAsSnapshot typing and usedRows reference ([1ace413](https://github.com/ernsoylu/frees/commit/1ace41334c5a231be8bcb21edd270ad19544fe02))
* **tables:** retain deadline results and distinguish worker cancellation ([7f10226](https://github.com/ernsoylu/frees/commit/7f10226478cca44dcef60225be2dadf6d8aceb8d))
* **tables:** separate exact CSV from formatted export and snapshot copies ([e02d339](https://github.com/ernsoylu/frees/commit/e02d339b24d58c93850f150c2a4dcbde8dfffd53))
* **tables:** validate drafts and capacities while preserving malformed imports ([abd5f38](https://github.com/ernsoylu/frees/commit/abd5f388301d164d74d3fd5a409d87ca65c42c3b))
* **tables:** validate log fill domains and record interpolation ([e6b5ff2](https://github.com/ernsoylu/frees/commit/e6b5ff2212af4087604f8bf65ad62a80eac31ce4))
* **ui:** do not classify successful checks as syntax errors ([f97abef](https://github.com/ernsoylu/frees/commit/f97abefc7744c6d4c85bc2060f3b1da02a940606))
* **ux:** drive family rows and generator cases from one table each ([6907d60](https://github.com/ernsoylu/frees/commit/6907d60963b22f53186cb12c5c2a9508c0f59306))
* **ux:** ignore instance line in structural equality ([13c0fe8](https://github.com/ernsoylu/frees/commit/13c0fe82de6c4013a2f5ea675226f28370b2bac3))
* **ux:** narrow selectedPointIndex type and add error field in Pareto mock ([e26ad5d](https://github.com/ernsoylu/frees/commit/e26ad5d1fd0c8834ada3b30869aaec95be680b23))
* **ux:** store related-model rows without duplicated object literals ([d8864b8](https://github.com/ernsoylu/frees/commit/d8864b87c26c3c33ca61d67925768bd0978e5bbd))


### Performance Improvements

* **solver:** reduce numerical memory and measured overhead (Phase 5) ([#9](https://github.com/ernsoylu/frees/issues/9)) ([91fc99d](https://github.com/ernsoylu/frees/commit/91fc99d61513e89f3a12f29771c00fe47d074519))

## [0.1.0] - 2026-09-07

### Added
- **Pure-Rust Target-Agnostic Core (`frees-core`)**:
  - Declarative bidirectional equation solver with automatic causality resolution.
  - Bipartite matching for structural degree-of-freedom validation.
  - Tarjan's Strongly Connected Components (SCC) topological decomposition.
  - Scaled Newton-Raphson with diagonal preconditioning, Armijo line search, and Powell hybrid dogleg.
  - Adaptive Dormand-Prince 5(4) (`ode45`) and 5th-order implicit Radau IIA (`radau5`) ODE integrators.
  - Variable-coefficient BDF / IDA index-1 DAE integrator with Brent zero-crossing event localization.
  - Pure-Rust `rustprop` CoolProp 8.0.0 thermodynamic backbone (Helmholtz equations of state, incompressibles, ASHRAE psychrometrics).
  - Exact rational symbolic CAS for closed-form simplification, differentiation, and Riccati CARE control synthesis.
  - Standard acausal component library with 295+ components spanning fluid networks, thermal systems, moist air HVAC, mechanics, and electrical circuits.
- **WebAssembly Engine Bridge (`crates/frees`)**:
  - Structured JSON RPC interface compiling to `wasm32-unknown-unknown`.
  - Reusable `PreparedDocument` structural compilation cache with in-place parameter mutations.
  - Strict size budget enforcement ($\le 4,096\text{ KiB}$ raw ceiling).
- **Web Workbench (`web`)**:
  - React 19 / TypeScript modern scientific computing interface.
  - High-performance virtualized workbook powered by Glide Data Grid with CSV import/export.
  - Plotly.js scientific charting with automatic thermodynamic state diagram overlays ($T$-$s$, $P$-$h$, Mollier, psychrometric).
  - Web Worker concurrency pool executing parallel parametric sweeps across up to 4 worker instances.
  - Monotonic revision coordinator (`ModelRevisionTracker`) eliminating race conditions.
  - Full offline PWA support via Workbox service worker caching and IndexedDB project library.
  - Shareable URL document links via `#share=<lz-string>`.
- **Command-Line Interface (`crates/frees-cli`)**:
  - Headless native binary for `solve`, `check`, and JSON batch export.
- **Quality & Parity Assurance**:
  - 1,308-fixture frozen regression test corpus matching declared tolerances.
  - Strict linting with `-D warnings` on native and `wasm32-unknown-unknown`.
  - Zero-network offline Playwright automated verification.
