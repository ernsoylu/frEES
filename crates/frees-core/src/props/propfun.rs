//! Real-fluid property functions — the CoolProp-facing surface.
//!
//! Port of
//! `../frEES/backend/core/src/main/java/com/frees/backend/props/PropertyFunctions.java`
//! (537 LOC) together with the four-function façade it calls through,
//! `props/CoolProp.java`.
//!
//! # What is here, and what cannot be
//!
//! Everything in `PropertyFunctions.java` that is *dispatch* — the fluid alias
//! table, the glycol-mixture grammar, the output/indicator key maps, the branch
//! order of `evaluate`, the humid-air arity rules, the seeding helpers — is
//! ported line for line. It is what decides **which** CoolProp call a
//! document's `Enthalpy(R134a, T=…, x=…)` becomes, and it has to agree with the
//! Java exactly or a browser solve silently answers a different question.
//!
//! What cannot be ported is CoolProp itself. So the four calls the Java makes
//! into the native library become one trait, [`RealFluid`], and the engine
//! holds **at most one** installed implementation ([`install`], [`backend`]),
//! exactly as the Java holds at most one loaded `LIB`:
//!
//! ```text
//!   Java                                    Rust
//!   CoolProp.LIB != null                    propfun::is_available()
//!   CoolProp.propsSI(...)       throws      propfun::props_si        -> Result
//!   CoolProp.propsSIOrNaN(...)  NaN         propfun::props_si_or_nan -> f64
//!   CoolProp.props1SI(...)      throws      propfun::props1_si       -> Result
//!   CoolProp.haPropsSI(...)     throws      propfun::ha_props_si     -> Result
//! ```
//!
//! **With no backend installed, every real-fluid call fails with an error that
//! names the fluid, both indicators and their values.** That is the one
//! behaviour this module will not compromise on: a build that cannot reach a
//! property must say which property it could not reach — never extrapolate,
//! never quietly return a plausible number.
//!
//! # Backends
//!
//! * [`TableBackend`] — serves `(P, h)` flashes from a
//!   [`crate::props::satsplit::SaturationSplitTable`] per fluid, the browser's
//!   intended hot path (decision D1). Its accuracy is the table's
//!   (`~1e-5…1e-4` relative); it declines every state outside its box.
//! * Anything else a host installs — a `coolprop.wasm`, a JS bridge, a test
//!   double — by implementing [`RealFluid`].
//!
//! # Not ported, deliberately
//!
//! `LENIENT` / `enterLenient` / `exitLenient` / `guardedPropsSI` and the
//! `sanitize` clamps exist only for the SUNDIALS IDA transient residual path
//! (Phase 8). There is no transient solver in this build to enable them, and a
//! clamp that is on when nothing sets it would silently rescue a *steady* solve
//! from an out-of-range state — the precise failure mode this module exists to
//! prevent. They land with the DAE integrator, not before.
//!
//! `PhTableRegistry` (the per-fluid lazy cache and its 300-sample validation
//! gate) is likewise absent: its whole design is "intercept `(P,Hmass)`, else
//! fall through to the native call", and in the browser there is nothing to
//! fall through to. [`TableBackend`] is the browser's answer to the same
//! question and says so when it cannot serve a state.

// PARITY RULE. `nominalEnthalpy`'s guard is written `!(P > 0)` in the Java, and
// the negation is load-bearing: it is what makes a NaN pressure fail the test.
// Rewriting it as `P <= 0` (what `neg_cmp_op_on_partial_ord` asks for) would
// let NaN through and seed the solver with a garbage enthalpy. The negated form
// stays, and the lint is silenced here rather than at each site so the shape is
// obviously deliberate — same treatment as `eval.rs`, `linalg.rs`, `nasa.rs`.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use std::sync::{Arc, Mutex, RwLock};

use crate::diag::{FreesError, Result};
use crate::props::{combustion, idealgas, solids};

/// The `prop$` prefix every encoded property call carries
/// (`PropertyFunctions.PREFIX`).
pub const PREFIX: &str = "prop$";

const WATER: &str = "Water";
const CO2: &str = "CO2";
/// CoolProp key for mass density, also used for the `v`/`volume` indicators.
const DMASS: &str = "Dmass";
const VOLUME: &str = "volume";

/// Output function name -> CoolProp `PropsSI` output key (`OUTPUTS`).
const OUTPUTS: &[(&str, &str)] = &[
    ("enthalpy", "Hmass"),
    ("entropy", "Smass"),
    ("temperature", "T"),
    ("pressure", "P"),
    ("density", DMASS),
    (VOLUME, DMASS),
    ("intenergy", "Umass"),
    ("quality", "Q"),
    ("cp", "Cpmass"),
    ("specheat", "Cpmass"),
    ("cv", "Cvmass"),
    ("viscosity", "viscosity"),
    ("conductivity", "conductivity"),
    ("soundspeed", "speed_of_sound"),
    ("compressibility", "Z"),
    ("compressibilityfactor", "Z"),
    ("prandtl", "Prandtl"),
    ("volexpcoef", "isobaric_expansion_coefficient"),
    ("gibbs", "Gmass"),
];

/// Indicator letter -> CoolProp `PropsSI` input key (`INPUTS`).
const INPUTS: &[(&str, &str)] = &[
    ("t", "T"),
    ("p", "P"),
    ("h", "Hmass"),
    ("s", "Smass"),
    ("u", "Umass"),
    ("x", "Q"),
    ("q", "Q"),
    ("v", DMASS),
    ("d", DMASS),
    ("rho", DMASS),
];

/// Lowercased accepted fluid spellings -> CoolProp canonical names (`FLUIDS`).
///
/// Declaration order is the Java source order. Java's `Map.ofEntries` is
/// unordered so nothing there depends on it — except [`detect_fluid`], which is
/// documented at its definition.
const FLUIDS: &[(&str, &str)] = &[
    ("water", WATER),
    ("steam", WATER),
    ("steam_iapws", WATER),
    ("air", "Air"),
    ("airh2o", "HumidAir"),
    ("r134a", "R134a"),
    ("r12", "R12"),
    ("r22", "R22"),
    ("r32", "R32"),
    ("r123", "R123"),
    ("r245fa", "R245fa"),
    ("r404a", "R404A"),
    ("r407c", "R407C"),
    ("r410a", "R410A"),
    // Low-GWP replacement blends. Unlike the legacy blends above, these are
    // keyed as "<NAME>.mix" in the predefined-mixture registry (the bare name
    // only searches the pure-fluid library and fails), so the lowercased
    // document spelling maps to the exact-case .mix key.
    ("r448a", "R448A.mix"),
    ("r449a", "R449A.mix"),
    ("r452a", "R452A.mix"),
    ("r452b", "R452B.mix"),
    ("r454a", "R454A.mix"),
    ("r454b", "R454B.mix"),
    ("r454c", "R454C.mix"),
    ("r455a", "R455A.mix"),
    ("r513a", "R513A.mix"),
    ("r515b", "R515B.mix"),
    ("r1234yf", "R1234yf"),
    ("r1234ze", "R1234ze(E)"),
    ("ammonia", "Ammonia"),
    ("r717", "Ammonia"),
    // Spelled formulas (CO2, N2, CH4, ...) are ideal gases with
    // formation-reference enthalpy (see IdealGas); only full names select the
    // CoolProp real fluids.
    ("carbondioxide", CO2),
    ("r744", CO2),
    ("nitrogen", "Nitrogen"),
    ("oxygen", "Oxygen"),
    ("hydrogen", "Hydrogen"),
    ("helium", "Helium"),
    ("argon", "Argon"),
    ("methane", "Methane"),
    ("ethane", "Ethane"),
    ("propane", "Propane"),
    ("r290", "Propane"),
    ("isobutane", "IsoButane"),
    ("r600a", "IsoButane"),
    ("butane", "n-Butane"),
    ("r600", "n-Butane"),
    // Chemical formula aliases for real fluid properties/constants lookups
    ("n2", "Nitrogen"),
    ("o2", "Oxygen"),
    ("co2", CO2),
    ("co", "CarbonMonoxide"),
    ("h2o", WATER),
    ("h2", "Hydrogen"),
    ("ch4", "Methane"),
    ("c2h6", "Ethane"),
    ("c3h8", "Propane"),
    ("c4h10", "n-Butane"),
];

/// Humid-air output function name -> `HAPropsSI` output key (`HA_OUTPUTS`).
const HA_OUTPUTS: &[(&str, &str)] = &[
    ("enthalpy", "H"),
    ("entropy", "S"),
    ("temperature", "T"),
    (VOLUME, "V"),
    ("humrat", "W"),
    ("relhum", "R"),
    ("wetbulb", "B"),
    ("dewpoint", "D"),
    ("cp", "C"),
    ("specheat", "C"),
    ("gibbs", "G"),
];

/// Humid-air indicator -> `HAPropsSI` input key (`HA_INPUTS`).
const HA_INPUTS: &[(&str, &str)] = &[
    ("t", "T"),
    ("p", "P"),
    ("h", "H"),
    ("s", "S"),
    ("v", "V"),
    ("w", "W"),
    ("r", "R"),
    ("rh", "R"),
    ("b", "B"),
    ("twb", "B"),
    ("d", "D"),
    ("tdp", "D"),
];

fn lookup<'a>(table: &'a [(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    table.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// The sorted, comma-joined key list an "unknown …" message quotes — the port
/// of `String.join(", ", MAP.keySet().stream().sorted().toList())`.
fn sorted_keys(table: &[(&str, &str)]) -> String {
    let mut keys: Vec<&str> = table.iter().map(|(k, _)| *k).collect();
    keys.sort_unstable();
    keys.join(", ")
}

// ---------------------------------------------------------------------------
// The backend seam (the Java's `props/CoolProp.java`)
// ---------------------------------------------------------------------------

/// The four CoolProp calls the engine makes, as one trait.
///
/// Each method returns [`Result`] rather than a NaN sentinel: the Java has two
/// entry points per call (`propsSI` throws, `propsSIOrNaN` returns NaN) over
/// one native function, and an `Err` maps onto either at the call site — the
/// `*_or_nan` helpers below do exactly that.
///
/// Implementations must **decline** (`Err`) rather than extrapolate. A backend
/// that answers outside its valid range is worse than no backend, because a
/// wrong answer reaches the user as a solved variable.
pub trait RealFluid: Send + Sync {
    /// `CoolProp.propsSI(output, name1, value1, name2, value2, fluid)`.
    /// `fluid` is already alias-resolved by [`resolve_fluid`].
    fn props_si(
        &self,
        output: &str,
        name1: &str,
        value1: f64,
        name2: &str,
        value2: f64,
        fluid: &str,
    ) -> Result<f64>;

    /// `CoolProp.props1SI(fluid, param)` — the state-free constants
    /// (`Tcrit`, `Pcrit`, `Ttriple`, `rhocrit`, `molar_mass`).
    fn props1_si(&self, fluid: &str, param: &str) -> Result<f64>;

    /// `CoolProp.haPropsSI(...)` — humid air, three input pairs.
    ///
    /// The default declines, so a `(P,h)`-table backend does not have to
    /// pretend it can do psychrometrics.
    #[allow(clippy::too_many_arguments)]
    fn ha_props_si(
        &self,
        output: &str,
        name1: &str,
        value1: f64,
        name2: &str,
        value2: f64,
        name3: &str,
        value3: f64,
    ) -> Result<f64> {
        let _ = (value1, value2, value3);
        Err(FreesError::property(format!(
            "humid-air property '{output}' at ({name1}, {name2}, {name3}) is not available: \
             the installed property backend does not implement HAPropsSI."
        )))
    }

    /// The canonical fluid names this backend can actually serve, or [`None`]
    /// when it serves everything the alias table knows (which is what a real
    /// CoolProp does).
    ///
    /// Not in the Java, which never needed it: `CoolProp.isAvailable()` was
    /// the whole question, because a loaded library serves every fluid in
    /// `PropertyFunctions.FLUIDS`. A tabulated backend serves two, and a fluid
    /// picker that offered thirty-six would be lying about thirty-four of them.
    fn served_fluids(&self) -> Option<Vec<String>> {
        None
    }

    /// A short human name for the backend, quoted in diagnostics.
    fn describe(&self) -> String {
        "property backend".to_string()
    }
}

/// The one installed backend, or `None`. Mirrors `CoolProp.LIB`.
static BACKEND: RwLock<Option<Arc<dyn RealFluid>>> = RwLock::new(None);

/// Installs `backend` as the engine's real-fluid source, replacing any previous
/// one. Returns the backend that was installed before.
pub fn install(backend: Arc<dyn RealFluid>) -> Option<Arc<dyn RealFluid>> {
    let mut slot = BACKEND.write().unwrap_or_else(|e| e.into_inner());
    let previous = slot.replace(backend);
    cache::clear();
    previous
}

/// Removes the installed backend; every real-fluid call then fails honestly.
pub fn uninstall() -> Option<Arc<dyn RealFluid>> {
    let mut slot = BACKEND.write().unwrap_or_else(|e| e.into_inner());
    let previous = slot.take();
    cache::clear();
    previous
}

/// The installed backend, if any.
pub fn backend() -> Option<Arc<dyn RealFluid>> {
    BACKEND
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map(Arc::clone)
}

/// Port of `CoolProp.isAvailable()`.
pub fn is_available() -> bool {
    BACKEND.read().unwrap_or_else(|e| e.into_inner()).is_some()
}

/// A one-line description of the property backend, for `getReference`-style
/// surfaces and for diagnostics that need to say *why* a call failed.
pub fn backend_description() -> String {
    match backend() {
        Some(be) => be.describe(),
        None => "none (no real-fluid property backend installed)".to_string(),
    }
}

// ---------------------------------------------------------------------------
// The property caches — `CoolProp.PROPS_CACHE` and `CoolProp.HA_CACHE`
// ---------------------------------------------------------------------------

/// The memo the Java façade keeps in front of the native library, ported at the
/// one depth this port can prove correct.
///
/// # What the Java has, and why this port needs it
///
/// `props/CoolProp.java` holds `PROPS_CACHE` and `HA_CACHE`: two
/// `LinkedHashMap(16, 0.75f, true)` — access-ordered, evicting the eldest past
/// 20 000 entries — keyed by the *whole* call, `(output, name1, prop1, name2,
/// prop2, fluid)` for `PropsSI` and `(output, name1, prop1, name2, prop2,
/// name3, prop3)` for `HAPropsSI`. Its own comment says why: "PropsSI is a pure
/// function of its arguments: repeated states — iterative solves circling a
/// fixed point … skip the global-lock native call entirely." This module was
/// ported without them, and CLAUDE.md's dependency table asked for them by name
/// ("keep the same four-call façade, **including the existing LRU caches**").
///
/// What that costs is measured rather than guessed. Replaying
/// `fixtures/corpus/ev-battery-cooling-pid.frees` — the document that is 61 %
/// of the parity gate's wall clock — makes **5 539 832** `props_si` calls for
/// **162 893** distinct argument tuples. 97.1 % of every property call in that
/// solve repeats a tuple already answered, and **84.1 % repeat the immediately
/// preceding call verbatim**. That shape is exactly the one the Java comment
/// describes: a finite-difference Jacobian re-evaluates a block's residuals once
/// per variable, and a property call whose two arguments do not involve the
/// perturbed variable is asked the identical question every time. Four of the
/// six call shapes that document uses have **one or two** distinct tuples across
/// the whole 4000-second transient — `Hmass(Water, P=200 kPa, T=305 K)` is asked
/// 54 757 times and has exactly one answer.
///
/// # Why the capacity is one, and not the Java's 20 000
///
/// Under the Java the cache was pure bookkeeping: CoolProp's `PropsSI` *is* a
/// pure function of its arguments, so a hit and a miss return the same double at
/// any capacity. Here it is not, quite. [`super::rustprop_warm`] seeds each
/// `(P,Hmass)`/`(P,Smass)` flash from the previous answer, so the answer to a
/// call is a function of its arguments **and of the seed left by whatever was
/// asked before it**. A cache entry is only honest while that seed still says
/// what it said when the entry was written.
///
/// Measured on the document above, replaying the call stream through LRUs of
/// eight capacities and comparing every hit against the value the live call
/// actually returned:
///
/// ```text
///   capacity   hits (% of all calls)   bit-identical to the live answer
///          1     4 659 582  (84.11 %)                          99.72 %
///          2     5 182 716  (93.55 %)                          65.11 %
///          4     5 205 035  (93.96 %)                          46.96 %
///          8     5 294 997  (95.58 %)                          17.30 %
///         16     5 325 438  (96.13 %)                          17.06 %
///         32     5 372 342  (96.98 %)                          16.78 %
///        128     5 375 918  (97.04 %)                          16.65 %
///     20 000     5 376 906  (97.06 %)                          16.60 %
/// ```
///
/// The knee is not a matter of taste. At capacity **1** a hit means the
/// *immediately preceding* property call asked this same question, so the warm
/// adapter's seed is still the state that call converged on and a live recompute
/// starts where it stopped. At capacity 2 something else has been asked in
/// between, the seed has moved, and fidelity collapses by a third immediately
/// and to a sixth by capacity 8. Nine more percentage points of hit rate cost
/// five sixths of the guarantee.
///
/// So the port takes the Java's mechanism at the depth where it can still make
/// the Java's promise — a hit returns what the call would have returned — and
/// says out loud that it is leaving 13 points of hit rate on the table.
///
/// # The seed-identity tag was built and measured. It does not pay.
///
/// This section used to end "lifting the cap needs the entry to carry the seed
/// identity it was written under; that is the recorded next step", with an
/// estimate of "roughly another 1.6x on this document". Both halves were tested
/// (2026-08-25) and the estimate was wrong by an order of magnitude, so the
/// next step is recorded here as **taken and declined** rather than left open.
///
/// The tag that works is the seed's *content* — the `(p, x, T, rho, cp)` bits
/// [`super::rustprop_warm`] left in its slot after the call an entry was
/// written from — not a generation counter, and not the seed that call
/// *started* from. Content is what a deterministic function reproduces from: a
/// repeat served warm out of that same slot state returns the same double, and
/// a hit is honest exactly while the slot still says it. Replaying the same
/// call stream under that rule, with a hit modelled as a real cache's hit (the
/// backend does not run, so the seed does **not** advance):
///
/// ```text
///   capacity   hits (% of all calls)   bit-identical   tag rejects
///          1     4 659 582  (84.11 %)         99.72 %            0
///          2     4 660 182  (84.12 %)         99.72 %      522 534
///          4     4 661 729  (84.15 %)         99.72 %      543 306
///          8     4 678 695  (84.46 %)         99.72 %      616 302
///         16     4 696 720  (84.78 %)         99.72 %      628 718
///         32     4 734 041  (85.45 %)         99.72 %      638 301
///        128     4 747 469  (85.70 %)         98.18 %      628 449
///     20 000     4 747 474  (85.70 %)         98.18 %      629 432
/// ```
///
/// The tag does exactly what it was meant to: not one seeded hit at any
/// capacity is anything but bit-identical, where the untagged table loses five
/// sixths of them, and capacity 1 reproduces line for line — the harness's own
/// control. **But the thirteen points were never there to recover.** They were
/// the dishonest hits, and tagging turns them into 629 432 rejects instead. The
/// extra hit rate a tagged cache buys is **1.34 points** at the widest capacity
/// that still holds capacity 1's fidelity (32), and 1.59 above it — where
/// fidelity drops to 98.18 %, because those rows keep a *cold* entry alive long
/// enough to serve it where a live call would now answer warm.
///
/// Speed, measured rather than extrapolated, by installing a real hashed LRU in
/// this position and timing `ev-battery-cooling-pid`. Paired and alternated on
/// a quiet box, three runs each, one binary:
///
/// ```text
///   the shipped one-slot cache            45.71 45.27 45.22   mean 45.40 s
///   hashed LRU, capacity 1, tagged        48.33 47.58 49.10   mean 48.34 s  +6.5 %
///   hashed LRU, capacity 32, tagged       48.24 47.76 48.38   mean 48.13 s  +5.1 %
///   hashed LRU, capacity 20 000, tagged   44.58 44.12 44.53   mean 44.41 s  -2.2 %
/// ```
///
/// The capacity-1 row settles it. It answers byte-identically to the shipped
/// slot — same solution digest — and costs **6.5 %** to do it, because a hash
/// plus an LRU list plus a seed-slot read is more work than four short string
/// comparisons, five and a half million times. The 1.34 points the tag earns do
/// not cover that. The one configuration that comes out ahead is capacity
/// 20 000 at 2.2 %, under the 3 % bar this repo reverts at, and it is the
/// configuration that *fails* the fidelity claim the tag was built for.
///
/// So the cache stays one slot, and "leaving 13 points on the table" is now
/// known to be leaving 1.6.
///
/// # What the 0.28 % actually are
///
/// R2 recorded that they "move *towards* the oracle", reasoning that the cold
/// path is `HSU_P_flash`, bit-identical to `rustprop::props_si` "and therefore
/// to CoolProp 8.0.0". The first half is true; the *therefore* does not follow.
/// Asked R2's own probe state, `T(R134a, P = 3.5e5, Hmass = 1.0e5)`, a live
/// CoolProp 8.0.0 returns `193.726008250251_43` — the warm answer to `1.3e-14`,
/// and the cold answer to `1.694e-10`. At that state the cache serves the
/// *further* of the two from the oracle.
///
/// It is not systematically the further one either: over 297 states fed
/// byte-identical inputs from the same library, cold is nearer in 174 and warm
/// in 61. Neither ordering is the adapter's doing — rustprop's own `Hmass(T,P)`
/// is up to `5.051e-10` from CoolProp's before any flash runs — and the whole
/// spread sits inside `eps_tolerance<double>(30)`, the `1.86e-9` relative
/// bracket both libraries' `(P,X)` temperature solve stops on.
/// `tests/rustprop_warm.rs::neither_warm_nor_cold_is_uniformly_the_coolprop_answer`
/// pins it with the oracle's own literals.
///
/// None of which argues for changing what this cache serves: what took
/// `components_g4_radiator` from `1.2042e-6` to `1.2947e-14` was returning the
/// *identical* double for a repeated call, not which of the two doubles it was.
/// Consistency is the property being bought here, and it is orthogonal to the
/// paragraph above.
///
/// # Failures are not cached
///
/// Deliberately, and for the Java's stated reason — "so the error string stays
/// fresh". Only a value the backend returned as `Ok` is stored, so a refused
/// state is refused again, with a message describing the call actually made.
///
/// One Java behaviour is *not* reproduced, and it is a speed difference rather
/// than a value one: `propsSIOrNaN` caches its `NaN` and the throwing `propsSI`
/// then skips such an entry (`cached != null && !cached.isNaN()`). Here
/// [`props_si_or_nan`] delegates to [`props_si`], so a repeatedly-failing state
/// is re-asked rather than served a stored `NaN` — the same value, one backend
/// call later.
mod cache {
    use super::Mutex;

    /// One remembered call: the last one this façade answered.
    ///
    /// The strings are owned and **overwritten in place** rather than
    /// reallocated, because a miss happens hundreds of thousands of times in a
    /// transient solve and four allocations apiece would be a real fraction of
    /// what the cache saves.
    #[derive(Default)]
    struct Slot {
        names: [String; 4],
        /// Input values, bitwise. Bits rather than `f64` so the key is `Eq`,
        /// which also matches the Java: its record equality is `Double.equals`,
        /// under which `NaN` matches itself and `+0.0` does not match `-0.0`.
        /// (No backend stores an entry for a non-finite input, but the key type
        /// should not be the reason.)
        values: [u64; 3],
        value: f64,
        /// `false` until the first successful call is remembered — a `Slot` of
        /// empty names would otherwise match a call with empty names, which
        /// [`props1_si`](super::props1_si)'s shape makes reachable.
        occupied: bool,
    }

    impl Slot {
        fn matches(&self, names: [&str; 4], values: [u64; 3]) -> bool {
            self.occupied
                && self.values == values
                && self.names[0] == names[0]
                && self.names[1] == names[1]
                && self.names[2] == names[2]
                && self.names[3] == names[3]
        }

        fn remember(&mut self, names: [&str; 4], values: [u64; 3], value: f64) {
            for (slot, name) in self.names.iter_mut().zip(names) {
                slot.clear();
                slot.push_str(name);
            }
            self.values = values;
            self.value = value;
            self.occupied = true;
        }
    }

    static PROPS: Mutex<Option<Slot>> = Mutex::new(None);
    static HA: Mutex<Option<Slot>> = Mutex::new(None);

    fn with<T>(which: &Mutex<Option<Slot>>, f: impl FnOnce(&mut Slot) -> T) -> T {
        let mut guard = which.lock().unwrap_or_else(|e| e.into_inner());
        f(guard.get_or_insert_with(Slot::default))
    }

    /// `PropsSI`: `(output, name1, value1, name2, value2, fluid)`.
    pub(super) fn props_get(
        output: &str,
        name1: &str,
        value1: f64,
        name2: &str,
        value2: f64,
        fluid: &str,
    ) -> Option<f64> {
        let names = [output, name1, name2, fluid];
        let values = [value1.to_bits(), value2.to_bits(), 0];
        with(&PROPS, |s| s.matches(names, values).then_some(s.value))
    }

    pub(super) fn props_put(
        output: &str,
        name1: &str,
        value1: f64,
        name2: &str,
        value2: f64,
        fluid: &str,
        value: f64,
    ) {
        let names = [output, name1, name2, fluid];
        let values = [value1.to_bits(), value2.to_bits(), 0];
        with(&PROPS, |s| s.remember(names, values, value));
    }

    /// `HAPropsSI`: `(output, name1, value1, name2, value2, name3, value3)`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn ha_get(
        output: &str,
        name1: &str,
        value1: f64,
        name2: &str,
        value2: f64,
        name3: &str,
        value3: f64,
    ) -> Option<f64> {
        let names = [output, name1, name2, name3];
        let values = [value1.to_bits(), value2.to_bits(), value3.to_bits()];
        with(&HA, |s| s.matches(names, values).then_some(s.value))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn ha_put(
        output: &str,
        name1: &str,
        value1: f64,
        name2: &str,
        value2: f64,
        name3: &str,
        value3: f64,
        value: f64,
    ) {
        let names = [output, name1, name2, name3];
        let values = [value1.to_bits(), value2.to_bits(), value3.to_bits()];
        with(&HA, |s| s.remember(names, values, value));
    }

    /// Forgets both remembered calls.
    ///
    /// Called whenever the installed backend changes: two backends answer the
    /// same question differently by construction — that is what installing one
    /// *means* — so an entry must not outlive the backend that wrote it.
    pub(super) fn clear() {
        with(&PROPS, |s| s.occupied = false);
        with(&HA, |s| s.occupied = false);
    }
}

/// Forgets the last property call each façade remembers — see [`mod@cache`].
///
/// [`install`] and [`uninstall`] already do this. A caller needs it only to take
/// a measurement against a cold cache: a benchmark, or a test that asserts on
/// [`super::rustprop_warm::stats`] through this façade rather than against the
/// backend directly.
pub fn clear_cache() {
    cache::clear();
}

fn no_backend(what: &str) -> FreesError {
    FreesError::property(format!(
        "{what} needs a real-fluid property backend and none is installed. \
         Build with the `rustprop-backend` feature (what ships), or \
         `linked-tables` for the D1 (P,h) tables; see \
         docs/decisions/0009-rustprop-backend.md."
    ))
}

/// [`RealFluid::props_si`] through the installed backend, or an error naming the
/// state. Port of `CoolProp.propsSI`.
pub fn props_si(
    output: &str,
    name1: &str,
    value1: f64,
    name2: &str,
    value2: f64,
    fluid: &str,
) -> Result<f64> {
    let Some(be) = backend() else {
        return Err(no_backend(&format!(
            "{output}({fluid}, {name1}={value1}, {name2}={value2})"
        )));
    };
    // `CoolProp.propsSI`'s LRU, in the Java's position: in front of the
    // backend, consulted before the call and written only on success. See
    // [`mod@cache`] for the measurement that says why it is worth having.
    if let Some(value) = cache::props_get(output, name1, value1, name2, value2, fluid) {
        return Ok(value);
    }
    let value = be.props_si(output, name1, value1, name2, value2, fluid)?;
    cache::props_put(output, name1, value1, name2, value2, fluid, value);
    Ok(value)
}

/// Port of `CoolProp.propsSIOrNaN` — the diagram sweeps' entry point.
pub fn props_si_or_nan(
    output: &str,
    name1: &str,
    value1: f64,
    name2: &str,
    value2: f64,
    fluid: &str,
) -> f64 {
    props_si(output, name1, value1, name2, value2, fluid).unwrap_or(f64::NAN)
}

/// Port of `CoolProp.props1SI`.
pub fn props1_si(fluid: &str, param: &str) -> Result<f64> {
    let Some(be) = backend() else {
        return Err(no_backend(&format!("{param} of {fluid}")));
    };
    be.props1_si(fluid, param)
}

/// `props1SI` returning NaN instead of erroring.
pub fn props1_si_or_nan(fluid: &str, param: &str) -> f64 {
    props1_si(fluid, param).unwrap_or(f64::NAN)
}

/// Port of `CoolProp.haPropsSI`.
#[allow(clippy::too_many_arguments)]
pub fn ha_props_si(
    output: &str,
    name1: &str,
    value1: f64,
    name2: &str,
    value2: f64,
    name3: &str,
    value3: f64,
) -> Result<f64> {
    let Some(be) = backend() else {
        return Err(no_backend(&format!(
            "{output}(AirH2O, {name1}={value1}, {name2}={value2}, {name3}={value3})"
        )));
    };
    // `CoolProp.haPropsSI`'s own LRU — the Java keeps a second one, on the
    // stated grounds that "psychrometric sweeps and coil iterations repeat
    // states heavily". Nothing in the humid-air path carries state between
    // calls (the warm adapter never sees it), so a hit here is bit-identical to
    // the call it replaces.
    if let Some(value) = cache::ha_get(output, name1, value1, name2, value2, name3, value3) {
        return Ok(value);
    }
    let value = be.ha_props_si(output, name1, value1, name2, value2, name3, value3)?;
    cache::ha_put(output, name1, value1, name2, value2, name3, value3, value);
    Ok(value)
}

/// Port of `CoolProp.haPropsSIOrNaN` — the psychrometric sweeps' entry point.
#[allow(clippy::too_many_arguments)]
pub fn ha_props_si_or_nan(
    output: &str,
    name1: &str,
    value1: f64,
    name2: &str,
    value2: f64,
    name3: &str,
    value3: f64,
) -> f64 {
    ha_props_si(output, name1, value1, name2, value2, name3, value3).unwrap_or(f64::NAN)
}

// ---------------------------------------------------------------------------
// Fluid resolution
// ---------------------------------------------------------------------------

/// Matches the Java `GLYCOL_MIX` pattern
/// `(eg|meg|ethyleneglycol|pg|mpg|propyleneglycol)_?(\d{1,3})` under
/// `Matcher.matches()` (whole-string). Returns `(base, percent digits)`.
///
/// Hand-rolled rather than pulling in a regex crate: the grammar is six
/// alternatives, an optional underscore and one-to-three ASCII digits. Java's
/// `\d` is ASCII-only by default (no `UNICODE_CHARACTER_CLASS`), so ASCII
/// digits are the exact semantics, not an approximation.
fn glycol_parts(token: &str) -> Option<(&'static str, &str)> {
    // Longest-first so "mpg"/"meg" are not mis-split; under `matches()` the
    // Java alternation order is irrelevant because the whole string must be
    // consumed, and longest-first reproduces that with a single scan.
    const BASES: &[&str] = &[
        "ethyleneglycol",
        "propyleneglycol",
        "meg",
        "mpg",
        "eg",
        "pg",
    ];
    let base = BASES.iter().copied().find(|b| token.starts_with(b))?;
    let mut rest = &token[base.len()..];
    if let Some(stripped) = rest.strip_prefix('_') {
        rest = stripped;
    }
    if rest.is_empty() || rest.len() > 3 || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((base, rest))
}

/// Whether the (lowercased) token names a CoolProp fluid alias or glycol mix.
/// Port of `isKnownFluid`.
pub fn is_known_fluid(token: &str) -> bool {
    lookup(FLUIDS, token).is_some() || glycol_parts(token).is_some()
}

/// Maps a written fluid token to a CoolProp fluid name: a known alias, an
/// aqueous glycol mixture (EG50, PG30, …), or the token unchanged so the
/// backend can report an unknown fluid itself. Port of `resolveFluid`.
///
/// The Java throws `IllegalStateException` on an out-of-range glycol
/// concentration; here that is a [`FreesError::Property`], which the engine
/// classifies the same way it classifies `PropertyEvaluationException`.
pub fn resolve_fluid(token: &str) -> Result<String> {
    if let Some(alias) = lookup(FLUIDS, token) {
        return Ok(alias.to_string());
    }
    if let Some((base, digits)) = glycol_parts(token) {
        // Three ASCII digits at most, so this cannot overflow.
        let percent: i32 = digits.parse().unwrap_or(-1);
        if percent <= 0 || percent >= 100 {
            return Err(FreesError::property(format!(
                "Glycol mixture concentration must be between 1 and 99 mass-%, got {percent}%. \
                 Example: EG50 for a 50 % ethylene-glycol / 50 % water coolant."
            )));
        }
        let coolprop_base = if base.starts_with('p') || base.starts_with("mp") {
            "MPG"
        } else {
            "MEG"
        };
        // Java: `BigDecimal.valueOf(percent).movePointLeft(2).toPlainString()`
        // — an exact decimal shift, so 50 -> "0.50", 5 -> "0.05".
        let fraction = format!("0.{percent:02}");
        return Ok(format!("INCOMP::{coolprop_base}[{fraction}]"));
    }
    Ok(token.to_string())
}

/// Canonical CoolProp fluid names available for property diagrams.
/// Port of `plotFluids()`.
pub fn plot_fluids() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for (_, canonical) in FLUIDS {
        if *canonical != "HumidAir" && !out.contains(canonical) {
            out.push(canonical);
        }
    }
    // Java `Stream.sorted()` on String is `compareTo`, i.e. UTF-16 code-unit
    // order; every canonical name here is ASCII, where that is byte order.
    out.sort_unstable();
    out
}

/// [`plot_fluids`] narrowed to what the installed backend can actually serve.
///
/// The Java has no counterpart because it did not need one — with CoolProp
/// loaded, `plotFluids()` *is* the served set. A tabulated backend serves a
/// subset, and `GET /api/plot/fluids` feeds a picker: offering a fluid whose
/// every plot point would fail is worse than a short list. Backends that serve
/// everything ([`RealFluid::served_fluids`] returning [`None`]) get the full
/// Java list back, unchanged and in the Java's order.
pub fn plot_fluids_available() -> Vec<&'static str> {
    let all = plot_fluids();
    let Some(be) = backend() else {
        return Vec::new();
    };
    match be.served_fluids() {
        None => all,
        Some(served) => all
            .into_iter()
            .filter(|name| served.iter().any(|s| s.eq_ignore_ascii_case(name)))
            .collect(),
    }
}

/// Every accepted document spelling of a fluid with its canonical name, sorted
/// by spelling — the surface the language reference and the editor's completion
/// list want. Not in the Java, which never needed the alias side of the map.
pub fn fluid_aliases() -> Vec<(&'static str, &'static str)> {
    let mut out: Vec<(&'static str, &'static str)> = FLUIDS.to_vec();
    out.sort_unstable_by(|a, b| a.0.cmp(b.0));
    out
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `\b<needle>\b` over an already-lowercased haystack — the one regex feature
/// [`detect_fluid`] uses.
fn contains_word(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let hay = haystack.as_bytes();
    let ned = needle.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = haystack[from..].find(needle) {
        let start = from + rel;
        let end = start + ned.len();
        let left_ok = start == 0 || !(is_word_byte(hay[start - 1]) && is_word_byte(ned[0]));
        let right_ok =
            end == hay.len() || !(is_word_byte(hay[end]) && is_word_byte(ned[ned.len() - 1]));
        if left_ok && right_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

/// Scans equation text (case-insensitively) for any mention of a known fluid,
/// returning the canonical CoolProp name or `"Water"`. Port of `detectFluid`.
///
/// **Documented divergence, unavoidable.** The Java sorts `FLUIDS.keySet()` by
/// length descending with a *stable* sort — but `Map.ofEntries` iterates in an
/// order Java randomises per JVM (the immutable-map SALT), so the relative order
/// of two same-length keys is not defined by the Java source and is not
/// reproducible run to run (`co2`/`ch4`/`h2o` are three such keys). This port
/// sorts by length descending, stable, over the **source declaration order**,
/// which is deterministic. Text naming two same-length fluids can therefore
/// pick a different one than a given JVM run did; text naming one fluid — every
/// real document — cannot.
pub fn detect_fluid(text: &str) -> &'static str {
    if text.trim().is_empty() {
        return WATER;
    }
    let lower = text.to_lowercase();
    let mut keys: Vec<(&str, &'static str)> = FLUIDS.to_vec();
    // Stable sort by descending key length: the Java's
    // `sortedKeys.sort((a, b) -> Integer.compare(b.length(), a.length()))`.
    keys.sort_by_key(|(key, _)| std::cmp::Reverse(key.len()));
    for (key, canonical) in keys {
        if contains_word(&lower, key) {
            return canonical;
        }
    }
    WATER
}

// ---------------------------------------------------------------------------
// Solver seeding (Phase-A consistent-state init)
// ---------------------------------------------------------------------------

/// A thermodynamically consistent nominal specific enthalpy [J/kg] for
/// `fluid_token` at pressure `p`, for **seeding** the solver's initial guess.
/// Port of `nominalEnthalpy`. Returns NaN if nothing resolves; never errors.
pub fn nominal_enthalpy(fluid_token: &str, p: f64) -> f64 {
    // Java: `!CoolProp.isAvailable() || !(P > 0) || !Double.isFinite(P)`. The
    // negated comparison is load-bearing — it is what rejects NaN.
    if !is_available() || !(p > 0.0) || !p.is_finite() {
        return f64::NAN;
    }
    if fluid_token.eq_ignore_ascii_case("airh2o") || fluid_token.eq_ignore_ascii_case("humidair") {
        return 5.0e4; // moist air per kg dry air ~50 kJ/kg near ambient
    }
    let Ok(fluid) = resolve_fluid(fluid_token) else {
        return f64::NAN;
    };
    let h = props_si_or_nan("H", "P", p, "Q", 0.5, &fluid); // mid-dome (condensable)
    if h.is_finite() {
        return h;
    }
    props_si_or_nan("H", "P", p, "T", 300.0, &fluid) // incompressible / single-phase
}

/// A nominal sub-critical operating pressure [Pa] for seeding a condensable
/// fluid's pressure. Port of `nominalPressure`. Never errors.
pub fn nominal_pressure(fluid_token: &str) -> f64 {
    if !is_available()
        || fluid_token.eq_ignore_ascii_case("airh2o")
        || fluid_token.eq_ignore_ascii_case("humidair")
    {
        return f64::NAN;
    }
    if let Ok(fluid) = resolve_fluid(fluid_token) {
        if let Ok(pcrit) = props1_si(&fluid, "Pcrit") {
            if pcrit.is_finite() && pcrit > 1.0e6 && pcrit < 2.0e7 {
                return 0.35 * pcrit;
            }
        }
    }
    f64::NAN
}

// ---------------------------------------------------------------------------
// evaluate — the encoded-call dispatcher
// ---------------------------------------------------------------------------

/// One resolved CoolProp input pair.
#[derive(Debug, Clone, Copy)]
struct Input {
    key: &'static str,
    value: f64,
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Java would index `parts[i]` and throw `ArrayIndexOutOfBoundsException`; a
/// wasm build compiles `panic = "abort"`, so an out-of-bounds index would take
/// the module down instead of returning a diagnostic. Both engines refuse the
/// same calls; only the message differs.
fn part<'a>(parts: &[&'a str], i: usize, encoded: &str) -> Result<&'a str> {
    parts.get(i).copied().ok_or_else(|| {
        FreesError::property(format!(
            "Malformed property call '{encoded}': expected at least {} '$'-separated fields.",
            i + 1
        ))
    })
}

/// Evaluates an encoded property call against the indicator values.
/// Port of `evaluate(encoded, values)`.
pub fn evaluate(encoded: &str, values: &[f64]) -> Result<f64> {
    evaluate_with_tokens(encoded, values, &[])
}

/// Evaluates an encoded property call. Real-fluid calls use `values` (the state
/// indicators); chemistry calls (MolarMass / HeatingValue / StoichAFR) use
/// `tokens` (fluid/formula/mode strings, case-preserved).
///
/// Port of `evaluate(encoded, values, tokens)`. **The branch order below is the
/// Java's, exactly** — in particular the ideal-gas test sits after the solid /
/// saturation / humid-air branches and before the `OUTPUTS` lookup, so
/// `Enthalpy(N2, T=500)` is an ideal-gas call and never a real-fluid one.
pub fn evaluate_with_tokens(encoded: &str, values: &[f64], tokens: &[String]) -> Result<f64> {
    let parts: Vec<&str> = encoded.split('$').collect();
    let output = part(&parts, 1, encoded)?;

    // 1. Chemistry + state-free fluid constants, keyed on the output alone.
    let token0 = || -> Result<&str> {
        tokens.first().map(String::as_str).ok_or_else(|| {
            FreesError::property(format!(
                "{}(...) needs a fluid or formula token, e.g. {}(Water).",
                capitalize(output),
                capitalize(output)
            ))
        })
    };
    match output {
        "molarmass" => return combustion::molar_mass_with(token0()?, coolprop_molar_mass),
        "heatingvalue" => {
            let mode = tokens.get(1).map(String::as_str).unwrap_or("lhv");
            return combustion::heating_value(token0()?, mode);
        }
        "stoichafr" => return combustion::stoich_afr(token0()?),
        "t_crit" => return props1_si(&resolve_fluid(token0()?)?, "Tcrit"),
        "p_crit" => return props1_si(&resolve_fluid(token0()?)?, "Pcrit"),
        "t_triple" => return props1_si(&resolve_fluid(token0()?)?, "Ttriple"),
        "v_crit" => return Ok(1.0 / props1_si(&resolve_fluid(token0()?)?, "rhocrit")?),
        _ => {}
    }

    // 2. Bulk solid materials.
    if matches!(output, "k_" | "rho_" | "c_" | "e_" | "nu_") {
        return evaluate_solid(output, &parts, values, tokens, encoded);
    }

    // 3. Saturation-line properties (one indicator + an implied quality).
    if matches!(output, "p_sat" | "t_sat" | "surfacetension") {
        return evaluate_saturation(output, &parts, values, encoded);
    }

    let fluid_token = part(&parts, 2, encoded)?;

    // 4. Humid air -> HAPropsSI.
    if fluid_token == "airh2o" || fluid_token == "humidair" {
        return evaluate_humid_air(output, &parts, values, encoded);
    }

    // 5. Ideal gases (formation-reference enthalpy) — BEFORE the OUTPUTS map.
    if idealgas::is_ideal_gas(fluid_token) {
        return idealgas::evaluate(output, &parts, values);
    }

    // 6. Real fluids.
    let Some(output_key) = lookup(OUTPUTS, output) else {
        return Err(FreesError::property(format!(
            "Unknown property function: {output}. Supported: {}",
            sorted_keys(OUTPUTS)
        )));
    };
    if parts.len() != 5 || values.len() != 2 {
        let cap = capitalize(output);
        return Err(FreesError::property(format!(
            "{cap} requires a fluid and exactly two property indicators, \
             e.g. {cap}(R134a, T=300, x=1)"
        )));
    }
    let fluid = resolve_fluid(fluid_token)?;
    let first = to_input(parts[3], values[0], output)?;
    let second = to_input(parts[4], values[1], output)?;
    let raw = props_si(
        output_key,
        first.key,
        first.value,
        second.key,
        second.value,
        &fluid,
    )?;
    // Volume is reported as specific volume = 1/density.
    Ok(if output == VOLUME { 1.0 / raw } else { raw })
}

/// The middle stage of `Combustion.molarMass`: a CoolProp real fluid's
/// `molar_mass`, or `None` so the formula parser gets its turn.
///
/// The Java guards with `isKnownFluid(lower) && CoolProp.isAvailable()` and
/// swallows every `RuntimeException`; all three exits are reproduced here.
fn coolprop_molar_mass(lower: &str) -> Option<f64> {
    if !is_known_fluid(lower) || !is_available() {
        return None;
    }
    let fluid = resolve_fluid(lower).ok()?;
    let m = props1_si(&fluid, "molar_mass").ok()?;
    (m.is_finite() && m > 0.0).then_some(m)
}

/// Bulk solid-material properties. Accepts both the no-argument form
/// `k_(Steel)` (material in `tokens`) and the temperature form
/// `k_(Steel, T=400)` (material in `parts[2]`, T in `values`).
/// Port of `evaluateSolid`.
fn evaluate_solid(
    output: &str,
    parts: &[&str],
    values: &[f64],
    tokens: &[String],
    encoded: &str,
) -> Result<f64> {
    let material;
    let mut temp_k = None;
    if let Some(first) = tokens.first() {
        material = first.as_str();
    } else {
        material = part(parts, 2, encoded)?;
        if parts.len() > 3 && parts[3] != "t" {
            return Err(FreesError::property(format!(
                "{} accepts only a temperature indicator T, got '{}'.",
                capitalize(output),
                parts[3]
            )));
        }
        if let Some(v) = values.first() {
            temp_k = Some(*v);
        }
    }
    solids::lookup_at(material, output, temp_k)
}

/// Saturation-line properties that take a single indicator and an implied
/// quality: `P_sat(fluid, T=…)`, `T_sat(fluid, P=…)`, `SurfaceTension`.
/// Port of `evaluateSaturation`.
fn evaluate_saturation(output: &str, parts: &[&str], values: &[f64], encoded: &str) -> Result<f64> {
    if parts.len() != 4 || values.len() != 1 {
        return Err(FreesError::property(format!(
            "{} takes a fluid and one indicator, e.g. P_sat(Water, T=373.15), \
             T_sat(Water, P=101325), SurfaceTension(Water, T=300)",
            capitalize(output)
        )));
    }
    let fluid = resolve_fluid(part(parts, 2, encoded)?)?;
    let input = to_input(parts[3], values[0], output)?;
    let key = match output {
        "p_sat" => "P",
        "t_sat" => "T",
        "surfacetension" => "surface_tension",
        // The Java's `default -> 0.0`; unreachable, the caller matched already.
        _ => return Ok(0.0),
    };
    props_si(key, input.key, input.value, "Q", 0.0, &fluid)
}

/// `AirH2O` calls map to `HAPropsSI` and need three indicators (e.g. T, P, R).
/// Port of `evaluateHumidAir`.
fn evaluate_humid_air(output: &str, parts: &[&str], values: &[f64], encoded: &str) -> Result<f64> {
    let Some(output_key) = lookup(HA_OUTPUTS, output) else {
        return Err(FreesError::property(format!(
            "Unknown humid-air function: {output}. Supported: {}",
            sorted_keys(HA_OUTPUTS)
        )));
    };
    if parts.len() != 6 || values.len() != 3 {
        let cap = capitalize(output);
        return Err(FreesError::property(format!(
            "{cap}(AirH2O, ...) requires exactly three property indicators, \
             e.g. {cap}(AirH2O, T=300, P=101325, R=0.5)"
        )));
    }
    let mut keys = ["", "", ""];
    for (i, slot) in keys.iter_mut().enumerate() {
        let indicator = part(parts, i + 3, encoded)?;
        let Some(key) = lookup(HA_INPUTS, indicator) else {
            return Err(FreesError::property(format!(
                "Unknown humid-air indicator '{indicator}' in {}(AirH2O, ...). Supported: {}",
                capitalize(output),
                sorted_keys(HA_INPUTS)
            )));
        };
        *slot = key;
    }
    ha_props_si(
        output_key, keys[0], values[0], keys[1], values[1], keys[2], values[2],
    )
}

/// Port of `toInput`.
fn to_input(indicator: &str, value: f64, output: &str) -> Result<Input> {
    let Some(key) = lookup(INPUTS, indicator) else {
        return Err(FreesError::property(format!(
            "Unknown property indicator '{indicator}' in {}(...). Supported: {}",
            capitalize(output),
            sorted_keys(INPUTS)
        )));
    };
    // The v indicator is specific volume; CoolProp expects density.
    if indicator == "v" {
        if value == 0.0 {
            return Err(FreesError::property(format!(
                "Specific volume must be nonzero in {}(...)",
                capitalize(output)
            )));
        }
        return Ok(Input {
            key,
            value: 1.0 / value,
        });
    }
    Ok(Input { key, value })
}

// ---------------------------------------------------------------------------
// The tabulated backend (decision D1)
// ---------------------------------------------------------------------------

use crate::props::auxtable::{AuxKind, AuxTable};
use crate::props::satsplit::{Output as SplitOutput, SaturationSplitTable};

/// A [`RealFluid`] served by per-fluid `(P, h)` split tables.
///
/// This is the browser's intended hot path: [`SaturationSplitTable`] answers
/// `T`, `Dmass` and `Smass` from `(P, h)` at the table's accuracy and declines
/// everything outside its box. Inputs other than `(P, Hmass)` are inverted by
/// bisection **on the tabulated surface itself**, which is what the pending
/// Rankine/refrigeration documents need (`Enthalpy(Water, P=…, T=…)`).
///
/// It is **not** a CoolProp substitute and does not pretend to be: `Cpmass`,
/// `Cvmass`, transport properties, sound speed, `Z`, Prandtl, surface tension,
/// humid air, supercritical states, mixtures and incompressibles are all
/// declined **by name**, never approximated.
pub struct TableBackend {
    tables: Vec<SaturationSplitTable>,
    aux: Vec<AuxTable>,
}

impl TableBackend {
    /// A backend over already-decoded split tables, keyed by each table's own
    /// `fluid()` name (the canonical CoolProp spelling).
    pub fn new(tables: Vec<SaturationSplitTable>) -> TableBackend {
        TableBackend {
            tables,
            aux: Vec::new(),
        }
    }

    /// A backend over split tables **and** the `FRAUX1` grids that cover what
    /// the split geometry cannot: the incompressible glycols, single-phase air
    /// transport, and transport on the saturation line.
    pub fn with_aux(tables: Vec<SaturationSplitTable>, aux: Vec<AuxTable>) -> TableBackend {
        TableBackend { tables, aux }
    }

    /// The canonical fluid names this backend can serve a full `(P,h)` flash
    /// for — the split-table fluids, and not the aux grids, which serve
    /// individual properties rather than states.
    pub fn fluids(&self) -> Vec<&str> {
        self.tables.iter().map(|t| t.fluid()).collect()
    }

    /// Every fluid this backend can answer *something* for, split tables and
    /// aux grids together. This is what a fluid picker should offer.
    pub fn all_served(&self) -> Vec<String> {
        let mut out: Vec<String> = self.fluids().iter().map(|s| (*s).to_string()).collect();
        for a in &self.aux {
            let name = a.name().to_string();
            if !out.iter().any(|s| s.eq_ignore_ascii_case(&name)) {
                out.push(name);
            }
        }
        out
    }

    fn table(&self, fluid: &str) -> Option<&SaturationSplitTable> {
        self.tables
            .iter()
            .find(|t| t.fluid().eq_ignore_ascii_case(fluid))
    }

    /// The incompressible grid and mass fraction behind an `INCOMP::MEG[0.50]`.
    fn incomp(&self, fluid: &str) -> Option<(&AuxTable, f64)> {
        let (family, x) = incomp_parts(fluid)?;
        let t = self.aux.iter().find(|a| {
            a.kind() == AuxKind::Incompressible && a.name().eq_ignore_ascii_case(family)
        })?;
        Some((t, x))
    }

    fn aux_of(&self, fluid: &str, kind: AuxKind) -> Option<&AuxTable> {
        self.aux
            .iter()
            .find(|a| a.kind() == kind && a.name().eq_ignore_ascii_case(fluid))
    }

    /// `h` such that `output(P, h) = target`, by bisection between the table's
    /// own enthalpy bounds at this pressure.
    ///
    /// `T` and `Smass` both rise monotonically with `h` at fixed `P` (flat, for
    /// `T`, across the dome), so bisection is well posed except exactly on a
    /// two-phase temperature plateau — where the inverse genuinely is not
    /// unique, and the caller gets a declined lookup rather than an arbitrary
    /// point of the plateau.
    fn invert(
        &self,
        table: &SaturationSplitTable,
        output: SplitOutput,
        p: f64,
        target: f64,
    ) -> Option<f64> {
        let hf = table.hf_at(p);
        let hg = table.hg_at(p);
        if !hf.is_finite() || !hg.is_finite() {
            return None;
        }
        // Stay a hair inside the served box: the outermost cells are exactly
        // where the fit is weakest, and `value` declines beyond them anyway.
        // `h_liquid_min_at` is the only honest way to ask for the liquid floor —
        // `dh_liquid_max` is [J/kg] on an absolute table and dimensionless on a
        // normalized one.
        let h_floor = table.h_liquid_min_at(p);
        let mut lo = hf - 0.999 * (hf - h_floor);
        let mut hi = hg + 0.999 * table.dh_vapor_max();
        let mut f_lo = table.value(output, p, lo)? - target;
        let f_hi = table.value(output, p, hi)? - target;
        if f_lo == 0.0 {
            return Some(lo);
        }
        if f_hi == 0.0 {
            return Some(hi);
        }
        if f_lo * f_hi > 0.0 {
            return None; // target outside the tabulated band at this pressure
        }
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            let f_mid = table.value(output, p, mid)? - target;
            if f_mid == 0.0 || (hi - lo).abs() <= 1e-9 * hi.abs().max(1.0) {
                return Some(mid);
            }
            if f_lo * f_mid <= 0.0 {
                hi = mid;
            } else {
                lo = mid;
                f_lo = f_mid;
            }
        }
        Some(0.5 * (lo + hi))
    }

    /// `P` such that `T_sat(P) = t`, by bisection on the tabulated saturation
    /// line — the inverse of [`SaturationSplitTable::tsat_at`].
    ///
    /// `T_sat` rises strictly with pressure over the whole subcritical band, so
    /// this is single-valued wherever it exists at all. A temperature off the
    /// served line (below the triple point, at or above critical, or past
    /// `p_serve_max`) gets [`None`] rather than a clamped endpoint.
    fn saturation_pressure(&self, table: &SaturationSplitTable, t: f64) -> Option<f64> {
        if !t.is_finite() {
            return None;
        }
        let (mut lo, mut hi) = (table.p_min(), table.p_serve_max());
        let (t_lo, t_hi) = (table.tsat_at(lo), table.tsat_at(hi));
        if !t_lo.is_finite() || !t_hi.is_finite() || !(t_lo <= t && t <= t_hi) {
            return None;
        }
        for _ in 0..200 {
            let mid = (lo.ln() + hi.ln()).mul_add(0.5, 0.0).exp();
            if !(mid > lo) || !(mid < hi) {
                break;
            }
            if table.tsat_at(mid) <= t {
                lo = mid;
            } else {
                hi = mid;
            }
            if hi - lo <= 1e-12 * hi {
                break;
            }
        }
        Some(0.5 * (lo + hi))
    }
}

/// Splits `INCOMP::MEG[0.50]` into its family and mass fraction.
///
/// This is the exact spelling [`resolve_fluid`] produces and the only one that
/// reaches a backend, so the parse is deliberately strict — anything else is
/// not an incompressible and must fall through to the ordinary lookup.
fn incomp_parts(fluid: &str) -> Option<(&str, f64)> {
    let rest = fluid.strip_suffix(']')?;
    let (family, frac) = rest.rsplit_once('[')?;
    if !family.starts_with("INCOMP::") {
        return None;
    }
    let x: f64 = frac.parse().ok()?;
    (x.is_finite() && (0.0..=1.0).contains(&x)).then_some((family, x))
}

impl TableBackend {
    /// Serves one property of an incompressible mixture.
    ///
    /// # Why this is exact in pressure
    ///
    /// CoolProp's incompressible model makes `Dmass`, `Cpmass`, `viscosity` and
    /// `conductivity` **exactly** pressure-independent, and `Hmass`/`Smass`
    /// **exactly linear** in pressure — verified at generation time at every
    /// node, with the run failing rather than writing a table that quietly is
    /// not the library. So a `(x, tau)` grid plus the stored pressure slopes
    /// reproduces CoolProp with no error beyond the grid interpolation.
    #[allow(clippy::too_many_arguments)]
    fn incomp_props(
        &self,
        table: &AuxTable,
        x: f64,
        output: &str,
        p: f64,
        other_key: &str,
        other: f64,
        fluid: &str,
    ) -> Result<f64> {
        let Some((t_lo, t_hi)) = table.band_at(x) else {
            return Err(FreesError::property(format!(
                "{output}({fluid}, …) is outside the generated table for {}: \
                 a mass fraction of {x} is not tabulated ({:.0} % to {:.0} % is).",
                table.name(),
                table.axis1_span().0 * 100.0,
                table.axis1_span().1 * 100.0
            )));
        };
        let span = t_hi - t_lo;

        // Resolve the state to a normalized temperature first, then read the
        // output off it — the same shape the split-table path uses with `h`.
        let tau = match other_key {
            "T" => (other - t_lo) / span,
            "Hmass" | "Smass" => {
                let k = if other_key == "Hmass" {
                    "Hmass"
                } else {
                    "Smass"
                };
                let Some(tau) = self.incomp_invert(table, x, p, k, other) else {
                    return Err(FreesError::property(format!(
                        "{output}({fluid}, P={p}, {other_key}={other}) is outside the generated \
                         table for {}: no temperature in {t_lo:.2} K to {t_hi:.2} K has that \
                         {other_key}.",
                        table.name()
                    )));
                };
                tau
            }
            _ => {
                return Err(FreesError::property(format!(
                    "{output}({fluid}, P={p}, {other_key}={other}) is not tabulated: an \
                     incompressible mixture is a function of temperature, so the second input \
                     must be T, Hmass or Smass."
                )));
            }
        };
        if !(0.0..=1.0).contains(&tau) {
            return Err(FreesError::property(format!(
                "{output}({fluid}, P={p}, {other_key}={other}) is outside the generated table \
                 for {}: its tabulated band is {t_lo:.2} K to {t_hi:.2} K.",
                table.name()
            )));
        }

        match output {
            "T" => Ok(t_lo + tau * span),
            "P" => Ok(p),
            "Dmass" | "Cpmass" | "viscosity" | "conductivity" => {
                self.incomp_read(table, x, tau, output, fluid, p)
            }
            "Hmass" | "Smass" => self.incomp_with_pressure(table, x, tau, output, fluid, p),
            "Umass" => {
                let h = self.incomp_with_pressure(table, x, tau, "Hmass", fluid, p)?;
                let d = self.incomp_read(table, x, tau, "Dmass", fluid, p)?;
                Ok(h - p / d)
            }
            _ => Err(FreesError::property(format!(
                "'{output}' is not a tabulated output for '{fluid}'. The incompressible grid \
                 stores Dmass, Cpmass, viscosity, conductivity, Hmass and Smass and derives T, \
                 P and Umass; a mixture with no vapour phase has no Q, and quantities like \
                 speed_of_sound or Z are not defined for CoolProp's incompressible model."
            ))),
        }
    }

    /// A pressure-independent output straight off the grid.
    fn incomp_read(
        &self,
        table: &AuxTable,
        x: f64,
        tau: f64,
        output: &str,
        fluid: &str,
        p: f64,
    ) -> Result<f64> {
        let k = table
            .output(output)
            .ok_or_else(|| uncovered(output, fluid, p, "tau", tau))?;
        table
            .value(k, x, tau)
            .ok_or_else(|| uncovered(output, fluid, p, "tau", tau))
    }

    /// `Hmass`/`Smass`, reconstructed from the reference-pressure column and the
    /// stored (constant) pressure slope.
    fn incomp_with_pressure(
        &self,
        table: &AuxTable,
        x: f64,
        tau: f64,
        output: &str,
        fluid: &str,
        p: f64,
    ) -> Result<f64> {
        let base = self.incomp_read(table, x, tau, output, fluid, p)?;
        let slope_name = if output == "Hmass" {
            "dHmass_dP"
        } else {
            "dSmass_dP"
        };
        let slope = self.incomp_read(table, x, tau, slope_name, fluid, p)?;
        Ok(base + slope * (p - table.ref_pressure()))
    }

    /// The `tau` whose `Hmass`/`Smass` at this pressure is `target`.
    ///
    /// Both rise monotonically with temperature (`cp > 0`), so bisection is well
    /// posed across the whole band. There is no two-phase plateau here — an
    /// incompressible mixture has no dome — which is exactly why this is
    /// simpler than the split table's inverse.
    fn incomp_invert(
        &self,
        table: &AuxTable,
        x: f64,
        p: f64,
        output: &str,
        target: f64,
    ) -> Option<f64> {
        if !target.is_finite() {
            return None;
        }
        let at = |tau: f64| -> Option<f64> {
            let k = table.output(output)?;
            let slope = table.output(if output == "Hmass" {
                "dHmass_dP"
            } else {
                "dSmass_dP"
            })?;
            Some(table.value(k, x, tau)? + table.value(slope, x, tau)? * (p - table.ref_pressure()))
        };
        let (lo_v, hi_v) = (at(0.0)?, at(1.0)?);
        if !(lo_v..=hi_v).contains(&target) {
            return None;
        }
        let (mut lo, mut hi) = (0.0f64, 1.0f64);
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            match at(mid) {
                Some(v) if v <= target => lo = mid,
                Some(_) => hi = mid,
                None => return None,
            }
            if hi - lo <= 1e-13 {
                break;
            }
        }
        Some(0.5 * (lo + hi))
    }

    /// Transport (`viscosity`, `conductivity`, `Cpmass`) that the `(P,h)` split
    /// table does not store, from whichever aux grid covers this fluid.
    ///
    /// Two shapes, and between them they are every transport lookup the
    /// correlation toolkit makes:
    ///
    /// * `(P, Q)` with `Q` exactly 0 or 1 — `htc_evap`, `htc_cond`,
    ///   `htc_liquid_only` and `dp_2phase` ask *only* on the dome, never off it.
    /// * `(P, T)` single-phase — `htc_1phase` and `htc_extair`.
    fn aux_transport(
        &self,
        output: &str,
        name1: &str,
        value1: f64,
        name2: &str,
        value2: f64,
        fluid: &str,
    ) -> Option<Result<f64>> {
        if !matches!(output, "viscosity" | "conductivity" | "Cpmass") {
            return None;
        }
        if let Some((p, q)) = pair(name1, value1, name2, value2, "P", "Q") {
            let table = self.aux_of(fluid, AuxKind::SaturationLine)?;
            // Only the two dome edges are tabulated. A wet state's transport is
            // not one number — it depends on the flow regime, which is what the
            // correlations exist to model — so this is refused on its own terms
            // rather than falling through to "not a tabulated output", which
            // would name the wrong cause.
            if q != 0.0 && q != 1.0 {
                return Some(Err(FreesError::property(format!(
                    "{output}({fluid}, P={p}, Q={q}) is not tabulated: transport is carried on \
                     the saturation line at Q=0 and Q=1 only. Inside the dome it is not a single \
                     property — it depends on the flow regime, which is what htc_evap / htc_cond \
                     / dp_2phase compute from the two edge values."
                ))));
            }
            let k = table.output(output)?;
            if !p.is_finite() || p <= 0.0 {
                return Some(Err(uncovered(output, fluid, p, "Q", q)));
            }
            return Some(
                table
                    .value(k, libm::log(p), q)
                    .ok_or_else(|| uncovered(output, fluid, p, "Q", q)),
            );
        }
        if let Some((p, t)) = pair(name1, value1, name2, value2, "P", "T") {
            let table = self.aux_of(fluid, AuxKind::PressureTemperature)?;
            let k = table.output(output)?;
            if !p.is_finite() || p <= 0.0 {
                return Some(Err(uncovered(output, fluid, p, "T", t)));
            }
            return Some(
                table
                    .value(k, libm::log(p), t)
                    .ok_or_else(|| uncovered(output, fluid, p, "T", t)),
            );
        }
        None
    }

    /// `Dmass` for a fluid that has only a `(P,T)` aux grid — air, which no
    /// split table covers.
    fn aux_pt_density(
        &self,
        output: &str,
        name1: &str,
        value1: f64,
        name2: &str,
        value2: f64,
        fluid: &str,
    ) -> Option<Result<f64>> {
        if output != "Dmass" {
            return None;
        }
        // Only for a fluid with no split table of its own. A `(P,T)` aux grid
        // is a coarse 24x64 convenience for the transport a correlation needs;
        // a split table is the accurate state surface. If a fluid ever has
        // both, the split table must win, and it must win here rather than by
        // ordering accident at the call site.
        if self.table(fluid).is_some() {
            return None;
        }
        let (p, t) = pair(name1, value1, name2, value2, "P", "T")?;
        let table = self.aux_of(fluid, AuxKind::PressureTemperature)?;
        let k = table.output("Dmass")?;
        if !p.is_finite() || p <= 0.0 {
            return Some(Err(uncovered(output, fluid, p, "T", t)));
        }
        Some(
            table
                .value(k, libm::log(p), t)
                .ok_or_else(|| uncovered(output, fluid, p, "T", t)),
        )
    }
}

/// Matches an unordered input pair against `(want1, want2)`, returning the two
/// values in the wanted order.
fn pair(
    name1: &str,
    value1: f64,
    name2: &str,
    value2: f64,
    want1: &str,
    want2: &str,
) -> Option<(f64, f64)> {
    if name1 == want1 && name2 == want2 {
        Some((value1, value2))
    } else if name1 == want2 && name2 == want1 {
        Some((value2, value1))
    } else {
        None
    }
}

impl RealFluid for TableBackend {
    fn props_si(
        &self,
        output: &str,
        name1: &str,
        value1: f64,
        name2: &str,
        value2: f64,
        fluid: &str,
    ) -> Result<f64> {
        // A non-finite input is refused before any dispatch: the Newton solver
        // probes with whatever the previous iterate produced, and a NaN that
        // reaches an interpolant comes back as a NaN *answer* — a wrong number
        // wearing the shape of a right one. Regression:
        // `tests/props_robustness.rs::the_installed_backend_answers_or_errors_for_every_key_combination`.
        if !value1.is_finite() || !value2.is_finite() {
            return Err(FreesError::property(format!(
                "{output}({fluid}, {name1}={value1}, {name2}={value2}) is not a state: \
                 a property indicator is not a finite number."
            )));
        }

        // An incompressible mixture is served entirely by its own grid — it has
        // no dome, so none of the split-table machinery below applies to it.
        if let Some((aux, x)) = self.incomp(fluid) {
            let (p, other_key, other) = if name1 == "P" {
                (value1, name2, value2)
            } else if name2 == "P" {
                (value2, name1, value1)
            } else {
                return Err(FreesError::property(format!(
                    "{output}({fluid}, {name1}={value1}, {name2}={value2}) is not tabulated: \
                     the incompressible grid needs pressure as one of the two inputs."
                )));
            };
            return self.incomp_props(aux, x, output, p, other_key, other, fluid);
        }

        // Transport the split table does not store, and `Dmass` for a fluid
        // that has no split table at all (air). Both are checked before the
        // split-table lookup so a tabulated fluid can still get its viscosity.
        if let Some(v) = self.aux_transport(output, name1, value1, name2, value2, fluid) {
            return v;
        }
        if let Some(v) = self.aux_pt_density(output, name1, value1, name2, value2, fluid) {
            return v;
        }

        let Some(table) = self.table(fluid) else {
            // A fluid can be aux-served without having a split table — air is.
            // Saying "no property table for Air" and then listing Air among the
            // served fluids would be both true and useless, so the two cases
            // get different diagnostics.
            if let Some(aux) = self.aux_of(fluid, AuxKind::PressureTemperature) {
                let outputs: Vec<&str> = ["viscosity", "conductivity", "Cpmass", "Dmass"]
                    .into_iter()
                    .filter(|o| aux.output(o).is_some())
                    .collect();
                return Err(FreesError::property(format!(
                    "{output}({fluid}, {name1}={value1}, {name2}={value2}) is not available: \
                     this build carries a (P,T) transport grid for '{fluid}' — {} — but no \
                     (P,h) state table, so enthalpy, entropy and saturation states have no \
                     source. Generate one with tools/table-gen.",
                    outputs.join(", ")
                )));
            }
            // States and transport-only are listed separately: a build that
            // says it "tabulates Air" and then declines `Enthalpy(Air, …)` has
            // told the reader nothing useful.
            let states = self.fluids();
            let transport_only: Vec<&str> = self
                .aux
                .iter()
                .map(AuxTable::name)
                .filter(|n| self.table(n).is_none())
                .collect();
            let state_list = if states.is_empty() {
                "(none)".to_string()
            } else {
                states.join(", ")
            };
            let mut msg = format!(
                "no property table for fluid '{fluid}'. \
                 This build tabulates full states for: {state_list}"
            );
            if !transport_only.is_empty() {
                msg.push_str(&format!(
                    ", and transport/incompressible properties only for: {}",
                    transport_only.join(", ")
                ));
            }
            msg.push_str(
                ". Generate more with tools/table-gen (states) or tools/aux-gen (transport, \
                 incompressibles).",
            );
            return Err(FreesError::property(msg));
        };
        // Everything this backend can do needs a pressure; orient the pair.
        let (p, other_key, other) = if name1 == "P" {
            (value1, name2, value2)
        } else if name2 == "P" {
            (value2, name1, value1)
        } else if let Some((t, q)) = pair(name1, value1, name2, value2, "T", "Q") {
            // `(T, Q)` is the one pair without a pressure the split geometry can
            // still answer, because the saturation line *is* the P↔T map inside
            // the dome. D1 measured this form at 2.4e-06 (water) / 2.5e-06
            // (R134a) and the property diagrams are built entirely out of it —
            // without it every dome and quality line is a row of gaps.
            let Some(p) = self.saturation_pressure(table, t) else {
                return Err(FreesError::property(format!(
                    "{output}({fluid}, T={t}, Q={q}) is outside the generated property table \
                     for {fluid}: T is not on the tabulated saturation line \
                     ({:.2} K to {:.2} K).",
                    table.tsat_at(table.p_min()),
                    table.tsat_at(table.p_serve_max())
                )));
            };
            (p, "Q", q)
        } else {
            return Err(FreesError::property(format!(
                "{output}({fluid}, {name1}={value1}, {name2}={value2}) is not tabulated: \
                 the (P,h) split table needs pressure as one of the two inputs (or the \
                 saturation pair (T, Q))."
            )));
        };

        // Resolve the state to an enthalpy first, then read the output off it.
        let h = match other_key {
            "Hmass" => other,
            "Q" => {
                let hf = table.hf_at(p);
                let hg = table.hg_at(p);
                if !hf.is_finite() || !hg.is_finite() || !(0.0..=1.0).contains(&other) {
                    return Err(uncovered(output, fluid, p, other_key, other));
                }
                hf + other * (hg - hf)
            }
            "T" => self
                .invert(table, SplitOutput::Temperature, p, other)
                .ok_or_else(|| uncovered(output, fluid, p, other_key, other))?,
            "Smass" => self
                .invert(table, SplitOutput::Entropy, p, other)
                .ok_or_else(|| uncovered(output, fluid, p, other_key, other))?,
            _ => {
                return Err(FreesError::property(format!(
                    "{output}({fluid}, P={p}, {other_key}={other}) is not tabulated: \
                     the (P,h) split table accepts Hmass, T, Smass or Q as the second input."
                )));
            }
        };

        // Every path below reads off `(p, h)`; if the state resolution produced
        // something that is not a state, stop here rather than interpolating on
        // it.
        if !h.is_finite() {
            return Err(uncovered(output, fluid, p, other_key, other));
        }
        match output {
            "Hmass" => Ok(h),
            "P" => Ok(p),
            "T" | "Dmass" | "Smass" => {
                let kind = SplitOutput::from_key(output).expect("matched arm");
                table
                    .value(kind, p, h)
                    .ok_or_else(|| uncovered(output, fluid, p, "Hmass", h))
            }
            "Umass" => {
                let d = table
                    .value(SplitOutput::Density, p, h)
                    .ok_or_else(|| uncovered(output, fluid, p, "Hmass", h))?;
                Ok(h - p / d)
            }
            "Q" => {
                let hf = table.hf_at(p);
                let hg = table.hg_at(p);
                if !hf.is_finite() || !hg.is_finite() || hg <= hf {
                    return Err(uncovered(output, fluid, p, "Hmass", h));
                }
                Ok((h - hf) / (hg - hf))
            }
            _ => Err(FreesError::property(format!(
                "'{output}' is not a tabulated output for '{fluid}'. The (P,h) split table \
                 stores T, Dmass and Smass and derives Hmass, P, Umass and Q; everything else \
                 (Cpmass, Cvmass, viscosity, conductivity, speed_of_sound, Z, Prandtl, \
                 surface_tension, Gmass) needs a full property backend."
            ))),
        }
    }

    fn props1_si(&self, fluid: &str, param: &str) -> Result<f64> {
        // The grid geometry is *not* the fluid's constants — `p_max` is
        // 0.75·p_crit by construction, and answering "Pcrit" from it would be a
        // wrong number. A generated `FRPHTAB1` artifact carries the four real
        // ones separately, exactly as CoolProp reported them to the generator;
        // those are answered and nothing else is.
        let table = self.table(fluid).ok_or_else(|| {
            FreesError::property(format!(
                "no property table for fluid '{fluid}'. This build tabulates: {}.",
                if self.fluids().is_empty() {
                    "(none)".to_string()
                } else {
                    self.fluids().join(", ")
                }
            ))
        })?;
        let constants = table.constants().ok_or_else(|| {
            FreesError::property(format!(
                "constant '{param}' of '{fluid}' is not carried by this (P,h) split table."
            ))
        })?;
        match param {
            "Pcrit" | "pcrit" | "P_critical" => Ok(constants.p_crit),
            "Tcrit" | "tcrit" | "T_critical" => Ok(constants.t_crit),
            "Ttriple" | "T_triple" => Ok(constants.t_triple),
            "ptriple" | "P_triple" => Ok(constants.p_triple),
            _ => Err(FreesError::property(format!(
                "constant '{param}' of '{fluid}' is not carried by the (P,h) split table; \
                 it stores Pcrit, Tcrit, Ttriple and ptriple only. Everything else \
                 (rhocrit, molar_mass, Tmax, …) needs a full property backend."
            ))),
        }
    }

    /// The split-table fluids **only**, deliberately not [`all_served`].
    ///
    /// This feeds `plot_fluids_available`, which feeds the property-diagram
    /// fluid picker, and a diagram needs full states — a dome, an entropy axis,
    /// an enthalpy axis. Air has a `(P,T)` transport grid and no state table, so
    /// offering it here would put a fluid in the picker whose every plot point
    /// fails. The doc comment on the trait method already stated the rule ("a
    /// fluid picker that offered thirty-six would be lying about thirty-four of
    /// them"); returning everything the backend can answer *something* for
    /// broke it, and `frees`'s dome test caught it.
    fn served_fluids(&self) -> Option<Vec<String>> {
        Some(self.fluids().iter().map(|s| (*s).to_string()).collect())
    }

    fn describe(&self) -> String {
        format!("(P,h) split tables [{}]", self.fluids().join(", "))
    }
}

fn uncovered(output: &str, fluid: &str, p: f64, k2: &str, v2: f64) -> FreesError {
    FreesError::property(format!(
        "{output}({fluid}, P={p}, {k2}={v2}) is outside the generated property table for \
         {fluid}. The table covers a subcritical (P,h) box only; this state is not in it, and \
         extrapolating would be a wrong answer rather than a missing one."
    ))
}

// ---------------------------------------------------------------------------
// The hxcorr::Fluids seam
// ---------------------------------------------------------------------------

/// The installed backend as `hxcorr`'s `Fluids`, so the CoolProp-querying
/// heat-exchanger correlations resolve through exactly the same path a `prop$`
/// call does — same aliases, same backend, same honest refusals.
#[derive(Debug, Clone, Copy, Default)]
pub struct InstalledFluids;

impl crate::props::hxcorr::Fluids for InstalledFluids {
    fn resolve_fluid(&self, token: &str) -> Result<String> {
        resolve_fluid(token)
    }

    fn props_si(
        &self,
        output: &str,
        name1: &str,
        value1: f64,
        name2: &str,
        value2: f64,
        fluid: &str,
    ) -> Result<f64> {
        props_si(output, name1, value1, name2, value2, fluid)
    }

    fn props1_si(&self, fluid: &str, param: &str) -> Result<f64> {
        props1_si(fluid, param)
    }
}

/// Serialises the tests that swap the process-global backend.
///
/// The slot is global — exactly like the Java's `CoolProp.LIB` — so any test in
/// any module that installs or uninstalls a backend must hold this first.
/// `cargo test` is multi-threaded by default, and without the lock a
/// `propfun` test and a `diagrams` test racing on the slot fail
/// non-deterministically.
#[cfg(test)]
pub(crate) fn test_swap_guard() -> std::sync::MutexGuard<'static, ()> {
    static SWAP: std::sync::Mutex<()> = std::sync::Mutex::new(());
    SWAP.lock().unwrap_or_else(|e| e.into_inner())
}

/// Runs `body` with **no** property backend installed, restoring whatever was
/// there before.
///
/// Since [`crate::props::tables::install_builtin_once`] installs the linked
/// tables from the public entry points, "nothing installed" is no longer the
/// resting state of a test binary — any test that asserts the honest
/// no-backend diagnostic has to ask for it, and hold the swap lock while it
/// does.
#[cfg(test)]
pub(crate) fn test_without_backend<T>(body: impl FnOnce() -> T) -> T {
    let _guard = test_swap_guard();
    let previous = uninstall();
    let out = body();
    if let Some(p) = previous {
        install(p);
    }
    out
}

/// Runs `body` with the linked tables installed, restoring whatever was there
/// before.
#[cfg(test)]
pub(crate) fn test_with_builtin_tables<T>(body: impl FnOnce() -> T) -> T {
    let _guard = test_swap_guard();
    let previous = backend();
    crate::props::tables::install_builtin().expect("linked tables must decode");
    let out = body();
    match previous {
        Some(p) => {
            install(p);
        }
        None => {
            uninstall();
        }
    }
    out
}

/// Runs `body` with **rustprop** installed, restoring whatever was there
/// before — the D9 counterpart of [`test_with_builtin_tables`].
///
/// Both exist for the same reason: a test that cares which backend answered
/// must say so. Since D9 the two disagree about what is *serveable* (rustprop
/// answers transport at `(P,T)`, a `(P,h)` table does not), so leaving the
/// question to whatever the global slot happens to hold makes the assertion
/// depend on test order.
#[cfg(all(test, feature = "rustprop-backend"))]
pub(crate) fn test_with_rustprop<T>(body: impl FnOnce() -> T) -> T {
    let _guard = test_swap_guard();
    let previous = backend();
    install(Arc::new(crate::props::rustprop_backend::RustpropBackend));
    let out = body();
    match previous {
        Some(p) => {
            install(p);
        }
        None => {
            uninstall();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `INCOMP::MEG[0.50]` spelling is the only one that reaches a backend,
    /// and the split has to be exact — a family that fell through would send a
    /// glycol to the `(P,h)` path, which has no geometry for it.
    #[test]
    fn incomp_parts_splits_only_the_spelling_resolve_fluid_produces() {
        assert_eq!(
            incomp_parts("INCOMP::MEG[0.50]"),
            Some(("INCOMP::MEG", 0.50))
        );
        assert_eq!(
            incomp_parts("INCOMP::MPG[0.05]"),
            Some(("INCOMP::MPG", 0.05))
        );
        for not_one in [
            "Water",
            "R134a",
            "MEG[0.50]",
            "INCOMP::MEG",
            "INCOMP::MEG[]",
            "INCOMP::MEG[abc]",
            "INCOMP::MEG[0.50",
            "INCOMP::MEG[1.50]",
            "INCOMP::MEG[-0.1]",
            "INCOMP::MEG[NaN]",
        ] {
            assert_eq!(incomp_parts(not_one), None, "{not_one}");
        }
    }

    /// The aux grids answer the calls the `(P,h)` table declines, and decline
    /// the ones nothing can answer. Values are CoolProp 8.0.0 ground truth.
    #[test]
    fn the_aux_grids_serve_what_the_split_table_cannot() {
        let _guard = test_swap_guard();
        let previous = backend();
        crate::props::tables::install_builtin().expect("install");

        // Glycol: no dome, so the split geometry never applies to it.
        let h = evaluate("prop$enthalpy$eg50$p$t", &[200_000.0, 305.0]).unwrap();
        assert!((h - 39_687.033).abs() / 39_687.033 < 1e-3, "h = {h}");
        let mu = evaluate("prop$viscosity$eg50$p$t", &[200_000.0, 305.0]).unwrap();
        assert!(
            (mu - 0.002_592_678_9).abs() / 0.002_592_678_9 < 5e-3,
            "mu = {mu}"
        );
        // Round-trips through the (P,h) inverse the wall-HX components use.
        let t = evaluate("prop$temperature$eg50$p$h", &[200_000.0, h]).unwrap();
        assert!((t - 305.0).abs() < 1e-6, "T = {t}");

        // Transport on the dome — the whole reason htc_evap was blocked.
        let mu = evaluate("prop$viscosity$r134a$p$x", &[350_000.0, 0.0]).unwrap();
        assert!(mu > 0.0 && mu < 1e-2, "mu_f(R134a) = {mu}");
        // Air transport, for htc_extair.
        let k = evaluate("prop$conductivity$air$p$t", &[101_325.0, 313.0]).unwrap();
        assert!((k - 0.027).abs() < 0.003, "k_air = {k}");

        // Inside the dome transport is not one number, and says so by name
        // rather than through the split table's "not a tabulated output".
        let err = evaluate("prop$viscosity$r134a$p$x", &[350_000.0, 0.5])
            .unwrap_err()
            .to_string_message();
        assert!(err.contains("saturation line"), "{err}");

        // Air has a transport grid but no state table, and the diagnostic must
        // not claim air is untabulated while listing it among the served.
        let err = evaluate("prop$enthalpy$air$p$t", &[101_325.0, 300.0])
            .unwrap_err()
            .to_string_message();
        assert!(err.contains("no (P,h) state table"), "{err}");

        // A concentration CoolProp does not model is declined, not extrapolated.
        let err = evaluate("prop$enthalpy$eg90$p$t", &[200_000.0, 305.0])
            .unwrap_err()
            .to_string_message();
        assert!(!err.is_empty(), "{err}");

        match previous {
            Some(p) => {
                install(p);
            }
            None => {
                uninstall();
            }
        }
    }

    /// A backend that replays recorded answers, so the dispatch can be tested
    /// without a property library.
    struct Recorded(Vec<(String, f64)>);

    impl Recorded {
        fn new(rows: &[(&str, f64)]) -> Recorded {
            Recorded(rows.iter().map(|(k, v)| (k.to_string(), *v)).collect())
        }

        fn get(&self, key: &str) -> Result<f64> {
            self.0
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| *v)
                .ok_or_else(|| FreesError::property(format!("not recorded: {key}")))
        }
    }

    impl RealFluid for Recorded {
        fn props_si(
            &self,
            output: &str,
            name1: &str,
            value1: f64,
            name2: &str,
            value2: f64,
            fluid: &str,
        ) -> Result<f64> {
            self.get(&format!(
                "{output}|{name1}={value1}|{name2}={value2}|{fluid}"
            ))
        }

        fn props1_si(&self, fluid: &str, param: &str) -> Result<f64> {
            self.get(&format!("{param}|{fluid}"))
        }

        fn ha_props_si(
            &self,
            output: &str,
            name1: &str,
            value1: f64,
            name2: &str,
            value2: f64,
            name3: &str,
            value3: f64,
        ) -> Result<f64> {
            self.get(&format!(
                "HA:{output}|{name1}={value1}|{name2}={value2}|{name3}={value3}"
            ))
        }
    }

    /// Installs `rows` for the duration of `body`, restoring whatever was there
    /// before. Holds [`test_swap_guard`] — see its doc comment.
    fn with_backend<T>(rows: &[(&str, f64)], body: impl FnOnce() -> T) -> T {
        let _guard = test_swap_guard();
        let previous = install(Arc::new(Recorded::new(rows)));
        let out = body();
        restore(previous);
        out
    }

    /// Runs `body` with no backend installed.
    fn without_backend<T>(body: impl FnOnce() -> T) -> T {
        let _guard = test_swap_guard();
        let previous = uninstall();
        let out = body();
        restore(previous);
        out
    }

    fn restore(previous: Option<Arc<dyn RealFluid>>) {
        match previous {
            Some(p) => {
                install(p);
            }
            None => {
                uninstall();
            }
        }
    }

    // -----------------------------------------------------------------------
    // The property caches (`mod@cache`)
    // -----------------------------------------------------------------------

    /// A backend that counts every call and answers `value1 + value2` (or the
    /// three-value sum for humid air), so a test can see whether the façade
    /// reached it. `"boom"` as the output is a refusal, for the
    /// failures-are-not-cached rule.
    #[derive(Default)]
    struct Counting {
        props: std::sync::atomic::AtomicU64,
        ha: std::sync::atomic::AtomicU64,
    }

    impl RealFluid for Counting {
        fn props_si(
            &self,
            output: &str,
            _name1: &str,
            value1: f64,
            _name2: &str,
            value2: f64,
            _fluid: &str,
        ) -> Result<f64> {
            self.props
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if output == "boom" {
                return Err(FreesError::property("no"));
            }
            Ok(value1 + value2)
        }

        fn props1_si(&self, _fluid: &str, _param: &str) -> Result<f64> {
            Ok(1.0)
        }

        fn ha_props_si(
            &self,
            output: &str,
            _name1: &str,
            value1: f64,
            _name2: &str,
            value2: f64,
            _name3: &str,
            value3: f64,
        ) -> Result<f64> {
            self.ha.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if output == "boom" {
                return Err(FreesError::property("no"));
            }
            Ok(value1 + value2 + value3)
        }
    }

    fn with_counting<T>(body: impl FnOnce(&Counting) -> T) -> T {
        let _guard = test_swap_guard();
        let backend = Arc::new(Counting::default());
        let previous = install(backend.clone());
        let out = body(&backend);
        restore(previous);
        out
    }

    fn counts(backend: &Counting) -> (u64, u64) {
        use std::sync::atomic::Ordering::Relaxed;
        (backend.props.load(Relaxed), backend.ha.load(Relaxed))
    }

    /// The whole point: the immediately repeated call does not reach the
    /// backend, and answers the same double.
    #[test]
    fn an_immediately_repeated_call_is_served_without_the_backend() {
        with_counting(|be| {
            let first = props_si("T", "P", 3.5e5, "Hmass", 1.0e5, "R134a").unwrap();
            assert_eq!(counts(be).0, 1);
            for _ in 0..99 {
                let again = props_si("T", "P", 3.5e5, "Hmass", 1.0e5, "R134a").unwrap();
                assert_eq!(again.to_bits(), first.to_bits());
            }
            assert_eq!(counts(be).0, 1, "99 repeats must not reach the backend");

            let ha = ha_props_si("H", "T", 298.15, "P", 101_325.0, "R", 0.5).unwrap();
            assert_eq!(counts(be).1, 1);
            for _ in 0..99 {
                assert_eq!(
                    ha_props_si("H", "T", 298.15, "P", 101_325.0, "R", 0.5)
                        .unwrap()
                        .to_bits(),
                    ha.to_bits()
                );
            }
            assert_eq!(counts(be).1, 1);
        });
    }

    /// Capacity is **one**, deliberately — `mod@cache` has the measurement. A
    /// call in between evicts, and this test is what says so out loud: raising
    /// the capacity is not a tuning knob, it is a change of contract.
    #[test]
    fn one_intervening_call_evicts_the_entry() {
        with_counting(|be| {
            props_si("T", "P", 3.5e5, "Hmass", 1.0e5, "R134a").unwrap();
            props_si("T", "P", 3.5e5, "Hmass", 1.0e5, "Water").unwrap();
            props_si("T", "P", 3.5e5, "Hmass", 1.0e5, "R134a").unwrap();
            assert_eq!(counts(be).0, 3);
        });
    }

    /// Every component of the key discriminates — a cache that confused two of
    /// these would answer a different question than the one asked.
    #[test]
    fn every_part_of_the_key_discriminates() {
        with_counting(|be| {
            let base = || props_si("T", "P", 3.5e5, "Hmass", 1.0e5, "R134a").unwrap();
            let mut expected = 0;
            for call in [
                ("Dmass", "P", 3.5e5, "Hmass", 1.0e5, "R134a"), // output
                ("T", "Q", 3.5e5, "Hmass", 1.0e5, "R134a"),     // name1
                ("T", "P", 3.6e5, "Hmass", 1.0e5, "R134a"),     // value1
                ("T", "P", 3.5e5, "Smass", 1.0e5, "R134a"),     // name2
                ("T", "P", 3.5e5, "Hmass", 1.1e5, "R134a"),     // value2
                ("T", "P", 3.5e5, "Hmass", 1.0e5, "Water"),     // fluid
            ] {
                base();
                let (o, n1, v1, n2, v2, f) = call;
                props_si(o, n1, v1, n2, v2, f).unwrap();
                expected += 2;
                assert_eq!(
                    counts(be).0,
                    expected,
                    "{call:?} was confused with the base"
                );
            }
            // `+0.0` and `-0.0` are different keys, as they are in the Java
            // (whose record equality is `Double.equals`).
            base();
            props_si("T", "P", 0.0, "Hmass", 1.0e5, "R134a").unwrap();
            props_si("T", "P", -0.0, "Hmass", 1.0e5, "R134a").unwrap();
            assert_eq!(counts(be).0, expected + 3);
        });
    }

    /// "Failures are not cached so the error string stays fresh" — the Java's
    /// rule, and the reason a refused state is refused again by the backend
    /// rather than by a stale memo.
    #[test]
    fn a_refusal_is_never_remembered() {
        with_counting(|be| {
            for _ in 0..5 {
                assert!(props_si("boom", "P", 1.0, "Hmass", 2.0, "Water").is_err());
            }
            assert_eq!(counts(be).0, 5);
            for _ in 0..5 {
                assert!(ha_props_si("boom", "T", 1.0, "P", 2.0, "R", 3.0).is_err());
            }
            assert_eq!(counts(be).1, 5);
        });
    }

    /// An entry must not outlive the backend that wrote it: installing a new
    /// one is a statement that the same question now has a different answer.
    #[test]
    fn changing_the_backend_forgets_the_entry() {
        let _guard = test_swap_guard();
        let previous = install(Arc::new(Recorded::new(&[("T|P=1|Hmass=2|Water", 300.0)])));
        assert_eq!(
            props_si("T", "P", 1.0, "Hmass", 2.0, "Water").unwrap(),
            300.0
        );
        install(Arc::new(Recorded::new(&[("T|P=1|Hmass=2|Water", 400.0)])));
        assert_eq!(
            props_si("T", "P", 1.0, "Hmass", 2.0, "Water").unwrap(),
            400.0
        );
        // …and uninstalling leaves nothing behind to answer with.
        uninstall();
        assert!(props_si("T", "P", 1.0, "Hmass", 2.0, "Water").is_err());
        restore(previous);
    }

    /// [`clear_cache`] is the seam a measurement needs, so it has to work from
    /// outside the module.
    #[test]
    fn clear_cache_forces_the_next_call_through() {
        with_counting(|be| {
            props_si("T", "P", 1.0, "Hmass", 2.0, "Water").unwrap();
            props_si("T", "P", 1.0, "Hmass", 2.0, "Water").unwrap();
            assert_eq!(counts(be).0, 1);
            clear_cache();
            props_si("T", "P", 1.0, "Hmass", 2.0, "Water").unwrap();
            assert_eq!(counts(be).0, 2);
        });
    }

    #[test]
    fn fluid_aliases_resolve_to_the_java_canonical_names() {
        assert_eq!(resolve_fluid("water").unwrap(), "Water");
        assert_eq!(resolve_fluid("steam_iapws").unwrap(), "Water");
        assert_eq!(resolve_fluid("r717").unwrap(), "Ammonia");
        assert_eq!(resolve_fluid("r1234ze").unwrap(), "R1234ze(E)");
        assert_eq!(resolve_fluid("r454b").unwrap(), "R454B.mix");
        assert_eq!(resolve_fluid("c4h10").unwrap(), "n-Butane");
        // Unknown tokens pass through unchanged so the backend names them.
        assert_eq!(resolve_fluid("unobtainium").unwrap(), "unobtainium");
    }

    #[test]
    fn glycol_mixtures_follow_the_java_grammar() {
        assert_eq!(resolve_fluid("eg50").unwrap(), "INCOMP::MEG[0.50]");
        assert_eq!(resolve_fluid("eg_50").unwrap(), "INCOMP::MEG[0.50]");
        assert_eq!(resolve_fluid("meg30").unwrap(), "INCOMP::MEG[0.30]");
        assert_eq!(
            resolve_fluid("ethyleneglycol60").unwrap(),
            "INCOMP::MEG[0.60]"
        );
        assert_eq!(resolve_fluid("pg40").unwrap(), "INCOMP::MPG[0.40]");
        assert_eq!(resolve_fluid("mpg25").unwrap(), "INCOMP::MPG[0.25]");
        assert_eq!(
            resolve_fluid("propyleneglycol5").unwrap(),
            "INCOMP::MPG[0.05]"
        );
        // Out of range -> the Java's IllegalStateException text.
        let err = resolve_fluid("eg0").unwrap_err().to_string();
        assert!(err.contains("between 1 and 99 mass-%"), "{err}");
        let err = resolve_fluid("eg100").unwrap_err().to_string();
        assert!(err.contains("got 100%"), "{err}");
        // Not a glycol at all -> pass-through, not an error.
        assert_eq!(resolve_fluid("eg").unwrap(), "eg");
        assert_eq!(resolve_fluid("eg1234").unwrap(), "eg1234");
        assert!(is_known_fluid("eg50"));
        assert!(!is_known_fluid("eg"));
    }

    #[test]
    fn plot_fluids_is_the_distinct_sorted_non_humid_list() {
        let fluids = plot_fluids();
        assert!(!fluids.contains(&"HumidAir"));
        assert!(fluids.windows(2).all(|w| w[0] < w[1]), "{fluids:?}");
        // Distinct: Water appears under water/steam/steam_iapws/h2o.
        assert_eq!(fluids.iter().filter(|f| **f == "Water").count(), 1);
        for expected in ["Water", "Air", "R134a", "CO2", "n-Butane", "R1234ze(E)"] {
            assert!(fluids.contains(&expected), "missing {expected}");
        }
        // 36 distinct canonical names once HumidAir is dropped. Java sorts
        // uppercase before lowercase, so "Water" precedes "n-Butane".
        assert_eq!(fluids.len(), 36, "{fluids:?}");
        assert_eq!(fluids.last(), Some(&"n-Butane"));
    }

    #[test]
    fn detect_fluid_finds_whole_words_longest_first() {
        assert_eq!(detect_fluid("h = Enthalpy(R134a, T=300, x=1)"), "R134a");
        assert_eq!(detect_fluid(""), "Water");
        assert_eq!(detect_fluid("   "), "Water");
        assert_eq!(detect_fluid("no fluid here"), "Water");
        // "airh2o" wins over "air" because it is longer.
        assert_eq!(detect_fluid("w = HumRat(AirH2O, T=300)"), "HumidAir");
        assert_eq!(detect_fluid("rho = Density(Air, T=300, P=1e5)"), "Air");
        // Word boundaries: "airfoil" is not "air".
        assert_eq!(detect_fluid("airfoil_lift = 3"), "Water");
    }

    #[test]
    fn without_a_backend_every_real_fluid_call_names_the_state() {
        without_backend(|| {
            let err = evaluate("prop$enthalpy$water$t$p", &[300.0, 101325.0])
                .unwrap_err()
                .to_string();
            assert!(err.contains("Water"), "{err}");
            assert!(err.contains("T=300"), "{err}");
            assert!(err.contains("P=101325"), "{err}");
            assert!(err.contains("none is installed"), "{err}");
            assert!(!is_available());
            assert!(backend_description().starts_with("none"));
        });
    }

    #[test]
    fn enthalpy_of_water_matches_the_java_oracle() {
        // Oracle (CoolProp 8.0.0, via tools/golden-dumper):
        //   h = Enthalpy(Water, T=300 [K], P=101325 [Pa]) -> 112654.89965464505
        with_backend(
            &[("Hmass|T=300|P=101325|Water", 112_654.899_654_645_05)],
            || {
                let h = evaluate("prop$enthalpy$water$t$p", &[300.0, 101325.0]).unwrap();
                assert_eq!(h, 112_654.899_654_645_05);
            },
        );
    }

    #[test]
    fn volume_is_reported_as_the_reciprocal_of_density() {
        with_backend(&[("Dmass|T=300|P=101325|Water", 996.556_340_388_5)], || {
            let v = evaluate("prop$volume$water$t$p", &[300.0, 101325.0]).unwrap();
            assert_eq!(v, 1.0 / 996.556_340_388_5);
        });
    }

    #[test]
    fn the_v_indicator_is_specific_volume_and_is_inverted() {
        with_backend(&[("T|P=101325|Dmass=2|Water", 500.0)], || {
            let t = evaluate("prop$temperature$water$p$v", &[101325.0, 0.5]).unwrap();
            assert_eq!(t, 500.0);
        });
        let err = evaluate("prop$temperature$water$p$v", &[101325.0, 0.0])
            .unwrap_err()
            .to_string();
        assert!(err.contains("Specific volume must be nonzero"), "{err}");
    }

    #[test]
    fn an_ideal_gas_token_never_becomes_a_real_fluid_call() {
        // With no backend at all: if the branch order were wrong this would
        // fail with "no backend" instead of answering.
        without_backend(|| {
            let h = evaluate("prop$enthalpy$n2$t", &[500.0]).unwrap();
            assert!(h.is_finite() && h > 0.0, "{h}");
            // Z of an ideal gas is exactly 1, never a table lookup.
            assert_eq!(
                evaluate("prop$compressibility$n2$t$p", &[500.0, 1e5]).unwrap(),
                1.0
            );
        });
    }

    #[test]
    fn solid_material_calls_reach_the_material_table() {
        without_backend(|| {
            // k_(Steel) — no indicator, material carried as a token.
            let k = evaluate_with_tokens("prop$k_", &[], &["Steel".to_string()]).unwrap();
            assert_eq!(k, solids::lookup("Steel", "k_").unwrap());
            // k_(Steel, T=400) — material in parts[2], T in values.
            let k400 = evaluate("prop$k_$steel$t", &[400.0]).unwrap();
            assert_eq!(k400, solids::lookup_at("steel", "k_", Some(400.0)).unwrap());
            // A non-temperature indicator is refused by name.
            let err = evaluate("prop$k_$steel$p", &[1e5]).unwrap_err().to_string();
            assert!(err.contains("only a temperature indicator T"), "{err}");
            // An unknown material lists what is known.
            let err = evaluate("prop$e_$vibranium$t", &[300.0])
                .unwrap_err()
                .to_string();
            assert!(err.contains("Unknown material 'vibranium'"), "{err}");
        });
    }

    #[test]
    fn saturation_calls_pin_quality_zero() {
        with_backend(&[("P|T=373.15|Q=0|Water", 101_417.977_284_361_9)], || {
            let p = evaluate("prop$p_sat$water$t", &[373.15]).unwrap();
            assert_eq!(p, 101_417.977_284_361_9);
        });
        let err = evaluate("prop$p_sat$water$t$p", &[373.15, 1e5])
            .unwrap_err()
            .to_string();
        assert!(err.contains("takes a fluid and one indicator"), "{err}");
    }

    #[test]
    fn humid_air_needs_exactly_three_indicators() {
        with_backend(&[("HA:W|T=300|P=101325|R=0.5", 0.011_140_2)], || {
            let w = evaluate("prop$humrat$airh2o$t$p$r", &[300.0, 101325.0, 0.5]).unwrap();
            assert_eq!(w, 0.011_140_2);
        });
        let err = evaluate("prop$humrat$airh2o$t$p", &[300.0, 101325.0])
            .unwrap_err()
            .to_string();
        assert!(err.contains("requires exactly three"), "{err}");
        let err = evaluate("prop$humrat$airh2o$t$p$zz", &[300.0, 101325.0, 0.5])
            .unwrap_err()
            .to_string();
        assert!(err.contains("Unknown humid-air indicator 'zz'"), "{err}");
        let err = evaluate("prop$soundspeed$airh2o$t$p$r", &[300.0, 101325.0, 0.5])
            .unwrap_err()
            .to_string();
        assert!(err.contains("Unknown humid-air function"), "{err}");
    }

    #[test]
    fn unknown_outputs_and_indicators_list_what_is_supported() {
        let err = evaluate("prop$bogus$water$t$p", &[300.0, 1e5])
            .unwrap_err()
            .to_string();
        assert!(err.contains("Unknown property function: bogus"), "{err}");
        assert!(
            err.contains("compressibility, compressibilityfactor"),
            "{err}"
        );
        let err = evaluate("prop$enthalpy$water$t$zz", &[300.0, 1e5])
            .unwrap_err()
            .to_string();
        assert!(err.contains("Unknown property indicator 'zz'"), "{err}");
        // The sorted key list, exactly as Java joins it.
        assert!(err.contains("d, h, p, q, rho, s, t, u, v, x"), "{err}");
    }

    #[test]
    fn arity_violations_quote_the_example_call() {
        let err = evaluate("prop$enthalpy$water$t", &[300.0])
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("Enthalpy requires a fluid and exactly two property indicators"),
            "{err}"
        );
        assert!(err.contains("Enthalpy(R134a, T=300, x=1)"), "{err}");
    }

    #[test]
    fn malformed_encodings_are_errors_not_panics() {
        // Java throws ArrayIndexOutOfBounds here; a wasm build would abort.
        let err = evaluate("prop$enthalpy", &[300.0]).unwrap_err().to_string();
        assert!(err.contains("Malformed property call"), "{err}");
        let err = evaluate("prop", &[]).unwrap_err().to_string();
        assert!(err.contains("Malformed property call"), "{err}");
    }

    #[test]
    fn state_free_constants_route_through_props1si() {
        with_backend(
            &[
                ("Tcrit|Water", 647.096),
                ("Pcrit|Water", 22_064_000.0),
                ("Ttriple|Water", 273.16),
                ("rhocrit|Water", 322.0),
            ],
            || {
                let t = evaluate_with_tokens("prop$t_crit", &[], &["Water".to_string()]).unwrap();
                assert_eq!(t, 647.096);
                let v = evaluate_with_tokens("prop$v_crit", &[], &["Water".to_string()]).unwrap();
                assert_eq!(v, 1.0 / 322.0);
                let tt =
                    evaluate_with_tokens("prop$t_triple", &[], &["Water".to_string()]).unwrap();
                assert_eq!(tt, 273.16);
            },
        );
    }

    #[test]
    fn molar_mass_prefers_the_ideal_gas_table_then_the_backend_then_the_formula() {
        without_backend(|| {
            // Ideal-gas species: no backend needed.
            let m = evaluate_with_tokens("prop$molarmass", &[], &["CO2".to_string()]).unwrap();
            assert!((m - 0.044_009_5).abs() < 1e-6, "{m}");
            // Formula: no backend needed either.
            let m = evaluate_with_tokens("prop$molarmass", &[], &["C8H18".to_string()]).unwrap();
            assert!((m - 0.114_23).abs() < 1e-4, "{m}");
            // "Water" with no backend falls through to the formula parser and
            // fails on the element "Wa" — exactly the Java on a machine where
            // CoolProp.isAvailable() is false.
            assert!(evaluate_with_tokens("prop$molarmass", &[], &["Water".to_string()]).is_err());
        });
        // With a backend the middle stage answers.
        with_backend(&[("molar_mass|Water", 0.018_015_268)], || {
            let m = evaluate_with_tokens("prop$molarmass", &[], &["Water".to_string()]).unwrap();
            assert_eq!(m, 0.018_015_268);
        });
    }

    #[test]
    fn seeding_helpers_degrade_to_nan_rather_than_erroring() {
        without_backend(|| {
            assert!(nominal_enthalpy("water", 1e5).is_nan());
            assert!(nominal_pressure("r134a").is_nan());
        });
        // The negated guard must reject NaN even though a row is recorded.
        with_backend(&[("H|P=NaN|Q=0.5|Water", 1.0)], || {
            assert!(nominal_enthalpy("water", f64::NAN).is_nan());
            assert!(nominal_enthalpy("water", -1.0).is_nan());
        });
        with_backend(&[("H|P=100000|Q=0.5|Water", 500_000.0)], || {
            assert_eq!(nominal_enthalpy("water", 1e5), 500_000.0);
        });
        // Mid-dome fails -> the single-phase fallback is tried.
        with_backend(&[("H|P=100000|T=300|Water", 1_234.0)], || {
            assert_eq!(nominal_enthalpy("water", 1e5), 1_234.0);
        });
        with_backend(&[("Pcrit|R134a", 4_059_280.0)], || {
            assert_eq!(nominal_pressure("r134a"), 0.35 * 4_059_280.0);
        });
        // Out of the refrigerant band -> NaN, keep the generic nominal.
        with_backend(&[("Pcrit|Water", 22_064_000.0)], || {
            assert!(nominal_pressure("water").is_nan());
        });
        // Humid air short-circuits before any backend call.
        with_backend(&[], || {
            assert_eq!(nominal_enthalpy("AirH2O", 1e5), 5.0e4);
            assert!(nominal_pressure("AirH2O").is_nan());
        });
    }

    #[test]
    fn a_backend_that_cannot_serve_a_state_says_which_state() {
        let backend = TableBackend::new(Vec::new());
        let err = backend
            .props_si("Hmass", "T", 300.0, "P", 1e5, "Water")
            .unwrap_err()
            .to_string();
        assert!(err.contains("no property table for fluid 'Water'"), "{err}");
        assert!(err.contains("(none)"), "{err}");
        assert_eq!(backend.describe(), "(P,h) split tables []");
        assert_eq!(backend.served_fluids(), Some(Vec::new()));
        // A constant of a fluid it does not tabulate at all fails on the fluid,
        // not on the parameter.
        let err = backend.props1_si("Water", "Pcrit").unwrap_err().to_string();
        assert!(err.contains("no property table for fluid 'Water'"), "{err}");
    }

    /// The four constants a `FRPHTAB1` artifact carries are answered from the
    /// artifact; everything else is refused by name rather than approximated
    /// from the grid.
    #[test]
    fn a_generated_table_answers_only_the_constants_it_actually_carries() {
        let table = crate::props::satsplit::SaturationSplitTable::decode_generated(
            &crate::props::tables::water_phtab().unwrap(),
        )
        .unwrap();
        let backend = TableBackend::new(vec![table]);
        // CoolProp 8.0.0: Pcrit = 22.064 MPa, Tcrit = 647.096 K, Ttriple = 273.16 K.
        assert_eq!(
            backend.props1_si("Water", "Pcrit").unwrap(),
            22_063_999.999_997_754
        );
        assert_eq!(
            backend.props1_si("Water", "Tcrit").unwrap(),
            647.095_999_999_987_3
        );
        assert_eq!(backend.props1_si("Water", "Ttriple").unwrap(), 273.16);
        // p_max is 0.75*p_crit by construction — the grid must never be mistaken
        // for the constant.
        assert!(backend.props1_si("Water", "rhocrit").is_err());
        assert!(backend.props1_si("Water", "molar_mass").is_err());
        let err = backend
            .props1_si("Water", "molar_mass")
            .unwrap_err()
            .to_string();
        assert!(err.contains("needs a full property backend"), "{err}");
    }
}
