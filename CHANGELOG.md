# Changelog

All notable changes to `frees` are documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

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
