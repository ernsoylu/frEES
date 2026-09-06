//! Acausal, multi-domain component system — the *pseudo bond graph* layer.
//!
//! Port of `parser/ComponentExpander.java` (1,656), `parser/ComponentLibrary.java`,
//! `ast/{ComponentDef,ComponentInst,ConnectDecl}.java`, `api/ComponentMetadata.java`
//! and `api/CyclePathResolver.java` — 2,667 LOC of Java in total.
//!
//! # The one architectural fact that governs everything here
//!
//! This is a **parser/expander layer, not a second solver**. Instantiating and
//! connecting components expands to flat scalar equations that flow through the
//! existing Newton/Tarjan pipeline unchanged. Nothing in this module may reach
//! into the solver.
//!
//! # Invariants carried from the parent engine
//!
//! * **Four domains**, each an `(across, flow)` pair with a junction rule:
//!   fluid `P`/`ṁ` (+ convective `h`), heat `T`/`Q̇` (`ΣQ̇=0`),
//!   electrical `V`/`I` (`ΣI=0`), mechanical `ω`/`τ` and `v`/`F`.
//! * A `connect` node must be a **single domain** — a violation is a hard
//!   `ParseException`, never a silent mis-solve.
//! * The fluid-family connector types that share the `(P,ṁ,h)` bond —
//!   thermofluid, pneumatic (`gas`), hydraulic (`oil`), humid air (`moistair`) —
//!   are separated by a reserved **`domain$` string parameter**.
//! * Humid air adds the humidity-ratio rider `W` on a dry-air mass basis:
//!   equal across a pass-through, flow-weighted only at an explicit mixer.
//! * `model$` selects a `VARIANT … REQUIRE …` body, validated per variant.
//! * Diagnostics are **component-named, never mangled scalars**.
//!
//! # The library is data
//!
//! All 312 shipped components are `.frees` DSL text (148 KB across 13 files in
//! the reference repo's `resources/components/`). They are embedded with
//! `include_str!` and parsed by the same front end as user text — there is no
//! second grammar and no hand-translated component.

pub mod cyclepath;
pub mod def;
pub mod domains;
pub mod expander;
pub mod library;
pub mod metadata;
pub mod variant;
