//! Transient integration — the `DYNAMIC` block.
//!
//! Port of `../frEES/backend/core/src/main/java/com/frees/backend/core/ode/`
//! (17 files, 2,892 LOC).
//!
//! # Port frees' own integrators, not a crate
//!
//! `BdfMethod` is 99 lines, `RosenbrockMethod` 95, `RungeKuttaMethod` 131.
//! Substituting a third-party solver changes numerical behaviour for no gain
//! and breaks parity with the oracle (`PLAN.md` §5, Phase-7 note).
//!
//! # Steady <-> transient from one network
//!
//! Storage components emit `der(member) = …` plus initials, which route into a
//! `DYNAMIC` block's algebraic body. The steady limit must still recover the
//! Phase-1 operating point — that is the invariant tying this module to the
//! component layer Phase 6 shipped.
pub mod accessors;
pub mod analysis;
pub mod deadline;
pub mod dynamic;
pub mod events;
pub(crate) mod hermite_root;
pub mod integrator;
pub mod methods;
pub mod problem;
