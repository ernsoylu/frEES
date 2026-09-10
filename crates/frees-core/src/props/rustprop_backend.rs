//! [`RustpropBackend`] — the accuracy path of decision D8
//! (`docs/decisions/0008-coolprop-wasm.md`), served by **rustprop**, the
//! pure-Rust CoolProp 8 port, as a direct cargo dependency rather than an
//! emscripten `coolprop.wasm` blob.
//!
//! The four [`RealFluid`] calls forward 1:1 to `rustprop::props_si` /
//! `rustprop::ha_props_si`; every rustprop error becomes a
//! [`FreesError::Property`] carrying the rustprop message, which the engine
//! classifies exactly as it classifies the Java's
//! `PropertyEvaluationException`.
//!
//! Fluids go to rustprop **under their plain names, never an `IF97::`
//! prefix**: the frees parity oracle is the native CoolProp default backend,
//! i.e. IAPWS-95 HEOS for water, and IF97 diverges from it at ~1e-6 — two
//! orders looser than the tightest fixture tolerances.

use crate::diag::{FreesError, Result};
use crate::props::propfun::RealFluid;
use crate::props::rustprop_warm;

/// A [`RealFluid`] answered by the rustprop engines linked into this build:
/// HEOS (Water, R134a, R1234yf, Air per the `rustprop-data` features in
/// `Cargo.toml`), the incompressible backend (the MEG/MPG glycols), and
/// humid air.
pub struct RustpropBackend;

/// The property-error mapping: message = the rustprop error's message
/// (rustprop's `Display` is upstream `what()`, message-only).
fn property_err(e: rustprop::Error) -> FreesError {
    FreesError::property(e.message())
}

/// Refuses a state whose numbers are not numbers, before rustprop sees it.
///
/// A `NaN` or infinite input is not a thermodynamic state, so nothing downstream
/// can do anything useful with it — but "useful" is not the reason this guard
/// exists. rustprop is a faithful port of a C++ library that reaches for
/// `assert` where upstream does, and a bracketing assertion is exactly what a
/// `NaN` defeats: `PropsSI("Dmass", "Smass", NaN, "Hmass", 101325, "Water")`
/// trips `assert!(fa * fb <= 0.0)` in the Chebyshev root finder. In the shipped
/// wasm that is not a recoverable error — `panic = "abort"`, so the engine
/// worker dies and `engineClient` has to respawn it — and the inputs to a
/// property call are *document* values, which a diverging solve or a
/// `props_si_or_nan` sweep produces as `NaN` routinely.
///
/// So the boundary is here, where the values are still data.
fn finite_inputs(named: &[(&str, f64)], call: impl FnOnce() -> String) -> Result<()> {
    for (name, value) in named {
        if !value.is_finite() {
            return Err(FreesError::property(format!(
                "{}: the input {name}={value} is not a finite value, so this is not a \
                 state the property backend can be asked about.",
                call()
            )));
        }
    }
    Ok(())
}

/// Turns a non-finite answer into a refusal that names the call.
///
/// [`RealFluid`]'s contract is stricter than "returns a number": an
/// implementation must **decline** rather than serve something the caller cannot
/// use, because "a backend that answers outside its valid range is worse than no
/// backend". A `NaN` is the worst case of that — it reaches the Newton residual
/// as data, and a NaN residual converges nothing while looking like progress.
///
/// Upstream leaves a few degenerate states returning `NaN` instead of throwing,
/// and rustprop reproduces that faithfully; `props_robustness`'s exhaustive key
/// sweep is what finds them (`PropsSI("Hmass", "T", 0, "Smass", 101325,
/// "Water")` is one). Converting here costs nothing on the sweep paths —
/// `props_si_or_nan` turns the `Err` straight back into `NaN` — and on the solve
/// path it replaces a silent poison value with a named property error.
fn finite(value: f64, what: impl FnOnce() -> String) -> Result<f64> {
    if value.is_finite() {
        return Ok(value);
    }
    Err(FreesError::property(format!(
        "{} is {value}, not a finite value. The property backend has no answer at \
         this state, and a non-finite one would propagate into the solve as data.",
        what()
    )))
}

impl RealFluid for RustpropBackend {
    fn props_si(
        &self,
        output: &str,
        name1: &str,
        value1: f64,
        name2: &str,
        value2: f64,
        fluid: &str,
    ) -> Result<f64> {
        let call = || format!("{output}({fluid}, {name1}={value1}, {name2}={value2})");
        // The input guard runs FIRST, before any rustprop code — including the
        // warm adapter. That ordering is the whole point of it: a `NaN` input
        // is what trips a bracketing `assert!` inside rustprop, and the shipped
        // wasm is `panic = "abort"`.
        finite_inputs(&[(name1, value1), (name2, value2)], call)?;
        // The warm-state adapter: a `(P,Hmass)`/`(P,Smass)` query a hair from
        // the last one this fluid answered is served by a two-or-three-step
        // Newton over rustprop's guessed-density solve instead of a fresh
        // bisecting flash. It declines — returning `None` — for every input
        // pair, output and fluid it does not own, for every dome-region
        // state, and whenever its locality or stability gate is not
        // satisfied, and then this call proceeds exactly as it did before the
        // adapter existed. See `props::rustprop_warm`.
        if let Some(v) = rustprop_warm::try_props_si(output, name1, value1, name2, value2, fluid) {
            // The warm path answers off a converged state, so this should never
            // fire — but `finite` is an invariant of this backend, not of one
            // code path, and a guard that only covers the slow route is not one.
            return finite(v, call);
        }
        let value = rustprop::props_si(output, name1, value1, name2, value2, fluid)
            .map_err(property_err)?;
        finite(value, call)
    }

    fn props1_si(&self, fluid: &str, param: &str) -> Result<f64> {
        // rustprop's trivial route: a trivial output never needs a state
        // update, so the empty input names are deliberately never parsed —
        // this is exactly upstream's `Props1SI` shape.
        let value = rustprop::props_si(param, "", 0.0, "", 0.0, fluid).map_err(property_err)?;
        finite(value, || format!("{param} of {fluid}"))
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
        let call =
            || format!("{output}(AirH2O, {name1}={value1}, {name2}={value2}, {name3}={value3})");
        finite_inputs(&[(name1, value1), (name2, value2), (name3, value3)], call)?;
        let value = rustprop::ha_props_si(output, name1, value1, name2, value2, name3, value3)
            .map_err(property_err)?;
        finite(value, call)
    }

    /// The fluids this build's `rustprop-data` features can serve **full
    /// states** for, spelled as [`crate::props::propfun::resolve_fluid`]
    /// produces them. The glycols are listed by family: `resolve_fluid`
    /// appends the mass fraction (`INCOMP::MEG[0.50]`), so the family is the
    /// serveable identity — the same spelling [`super::propfun::TableBackend`]
    /// keys its incompressible aux grids by.
    ///
    /// `Air` is on this list, and since **Wave-3 D6** it is served by rustprop
    /// alone. Its history is worth one paragraph, because the entry outlived
    /// the reason it was added.
    ///
    /// F3 put Air here because [`crate::props::rustprop_warm`] answered
    /// `(P,Hmass)`/`(P,Smass)` for it by Newton over `(T,p)` density solves —
    /// at a time when rustprop served the pseudo-pure fluids at
    /// `(P,T)`/`(Q,T)`/`(P,Q)` alone and those two pairs were a loud
    /// `NotImplemented`. Wave-2 R6/R7 then ported the pseudo-pure `HSU_P` and
    /// `(D,P)` flashes, so rustprop serves them itself, at every temperature,
    /// matching the CoolProp 8.0.0 wheel to ~1e-12 relative (measured over nine
    /// states in `tests/rustprop_warm.rs::air_p_hmass_matches_the_coolprop_wheel`).
    /// Measured against that, the adapter bought Air ~1.1x, so D6 retired its
    /// pseudo-pure path: `rustprop_warm` now declines Air at the door and the
    /// call delegates.
    ///
    /// The list is unchanged by that and still honest — `(P,T)`, `(P,Hmass)`,
    /// `(P,Smass)`, transport and `Z` are all served, and rustprop draws Air's
    /// pseudo-pure dome too (both branches across 65-130 K, glide intact),
    /// which is what the diagram picker in `frees` needs.
    ///
    /// Since **Wave-3 F8** the claim this entry makes about `Air` is checked by
    /// the parity gate and not only by unit tests: `sysdesign-ex06-pneumatic`,
    /// `-2` and `sysdesign-ex07-pneumatic-servo` are in `fixtures/corpus/` and
    /// drive `Temperature(Air, P, h)` at 1-7 bar, landing 13 ulp from the Java
    /// oracle's 300 K (2.46e-15 relative). Note that this list itself gates only
    /// the diagram picker — `props_si` never consults it — so those fixtures
    /// verify the *premise*, not the list.
    ///
    /// One caveat that is *not* on this list's conscience: inside Air's dome,
    /// rustprop's `(P,h)` flash refuses a low-quality sliver on the bubble side
    /// (measured at 1 bar: `q` from ~1e-4 to ~0.01 raises "unable to bracket
    /// the (p,X) solution … the derivative path is not ported"). That is a
    /// rustprop gap, recorded in the second Wave-3 amendment to
    /// `docs/decisions/0009-rustprop-backend.md`. It does not make `Air`
    /// unserved — a fluid whose every state failed would not belong here, and
    /// this is a sliver at 79 K.
    ///
    /// `CO2` joined the list on **2026-08-24 (Wave C1), by owner request**. Its
    /// data had been linked since Wave G2 — `MolarMass(CarbonDioxide)` answers,
    /// and `fixtures/corpus/docs_fluids_materials_03.frees` has proved it since
    /// — but the fluid was held off this list because linking data and serving
    /// *full states* are different claims. The claim is now measured, not
    /// assumed: the constants land on CoolProp (`Tcrit` 304.1282 K, `pcrit`
    /// 7.377298 MPa, `Ttriple` 216.592 K, `ptriple` 517.964 kPa), the dome draws
    /// 400/400 points across 216.592-304.128 K, all nine quality lines and all
    /// seven isentropes are complete, and `(P,h)`/`(P,s)`/`(T,D)` round-trip to
    /// machine precision over saturated, subcooled, superheated, near-critical
    /// (304.2 K / 7.4 MPa, where `cp` reaches 76.2 kJ/kg-K) and supercritical
    /// (to 400 K / 20 MPa) states.
    ///
    /// Note the spelling: the canonical name here is `CO2`, which is what
    /// [`super::propfun::plot_fluids`] emits and therefore what
    /// [`super::propfun::plot_fluids_available`] filters against —
    /// `"CarbonDioxide"` is a document *alias* and would match nothing.
    ///
    /// One limitation, and it is CoolProp's rather than this port's: CO2's
    /// melting line is steep out of a 0.518 MPa triple point, so the diagram
    /// generator's fixed cold anchor lands in the **solid** region above ~2.9
    /// MPa and CoolProp refuses it by name. See
    /// [`super::diagrams`]'s `sweep_entropy_at_pressure` for what this port does
    /// about it (ledger item 38) and its `isotherms` for the one curve it leaves
    /// absent on purpose.
    fn served_fluids(&self) -> Option<Vec<String>> {
        Some(
            [
                "Water",
                "R134a",
                "R1234yf",
                "Air",
                "CO2",
                // 2026-09-09, by owner request, after measuring rather than
                // assuming: each of these four draws a complete 400-point dome
                // on every `diagrams::Kind`, a full nine-line quality set, and
                // seven isobars on T-s — where CO2, on this list since Wave C1,
                // manages three. The coverage table is in
                // `crates/frees/src/lib.rs`'s
                // `the_newly_served_fluids_draw_every_diagram_kind`.
                "Ammonia",
                "Nitrogen",
                "Methane",
                "Propane",
                // 2026-09-10, with the budget at 5120: every remaining fluid
                // the alias table names, measured the same way — all seventeen
                // draw a full 400-point dome on all six `diagrams::Kind`s with
                // isoline counts matching or beating Water's. The two that did
                // not were `R1234ze(E)` and `n-Butane`, and that was the
                // lower-casing bug in `property_diagram`, not the data.
                //
                // `R410A` is the one asterisk: 399 dome points rather than 400,
                // which is the pseudo-pure blend's glide at one end, not a
                // failure.
                "R12",
                "R22",
                "R32",
                "R123",
                "R245fa",
                "R404A",
                "R407C",
                "R410A",
                "R1234ze(E)",
                "Argon",
                "CarbonMonoxide",
                "Ethane",
                "Helium",
                "Hydrogen",
                "IsoButane",
                "n-Butane",
                "Oxygen",
                "INCOMP::MEG",
                "INCOMP::MPG",
            ]
            .map(String::from)
            .to_vec(),
        )
    }

    fn describe(&self) -> String {
        format!("rustprop (CoolProp {})", rustprop::UPSTREAM_VERSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const B: RustpropBackend = RustpropBackend;

    /// The repo's own documented CoolProp ground truth (CLAUDE.md):
    /// `h = Enthalpy(Water, T=300 [K], P=101325 [Pa])` -> 112654.89965464505.
    #[test]
    fn water_enthalpy_matches_documented_ground_truth() {
        let h = B
            .props_si("Hmass", "T", 300.0, "P", 101325.0, "Water")
            .unwrap();
        let expected = 112654.89965464505;
        assert!(
            ((h - expected) / expected).abs() < 1e-9,
            "Hmass(Water, 300 K, 101325 Pa) = {h}, want {expected}"
        );
    }

    #[test]
    fn t_of_p_h_roundtrips() {
        for fluid in ["Water", "R134a", "R1234yf"] {
            let (t0, p) = (300.0, 101325.0);
            let h = B.props_si("Hmass", "T", t0, "P", p, fluid).unwrap();
            let t = B.props_si("T", "P", p, "Hmass", h, fluid).unwrap();
            assert!(
                (t - t0).abs() < 1e-6,
                "{fluid}: T(P, Hmass(T0, P)) = {t}, want {t0}"
            );
        }
    }

    #[test]
    fn saturated_water_viscosity_is_finite_and_positive() {
        let mu = B
            .props_si("viscosity", "P", 101325.0, "Q", 0.0, "Water")
            .unwrap();
        assert!(mu.is_finite() && mu > 0.0, "viscosity = {mu}");
    }

    /// Air transport and `Z` at `(T,P)` answer identically to rustprop asked
    /// directly, because the forward is 1:1. (This used to open by noting that
    /// Air was *not* in [`RealFluid::served_fluids`]; F3 added it, and D6 left
    /// it there while retiring the warm path that first put it on the list.)
    #[test]
    fn air_transport_and_z_match_rustprop_directly() {
        for output in ["viscosity", "Z"] {
            let via_backend = B
                .props_si(output, "T", 300.0, "P", 101325.0, "Air")
                .unwrap();
            let direct = rustprop::props_si(output, "T", 300.0, "P", 101325.0, "Air").unwrap();
            assert!(via_backend.is_finite() && via_backend > 0.0);
            assert_eq!(via_backend, direct, "{output}(Air, 300 K, 101325 Pa)");
        }
    }

    /// The fluids linked on 2026-09-09 and 2026-09-10 answer real states.
    ///
    /// The ideal-gas leg covers only the permanent gases and light
    /// hydrocarbons: at 300 K / 1 atm those are dilute. The refrigerants get
    /// their own leg below, because several (R123 most obviously) are *liquid*
    /// at that state, where the ideal-gas limit says nothing at all.
    ///
    /// These assertions are **physics, not captured output**. There is no
    /// CoolProp oracle on this checkout — `tools/golden-dumper` needs the
    /// sibling reference repo, which is absent — and quoting numbers this code
    /// produced as though they were references would prove nothing. So the
    /// checks are ones that fail if the data is wrong: the ideal-gas limit at
    /// low density, `T(P, h(T, P))` round-tripping, and saturated liquid
    /// denser and colder-enthalpy than saturated vapour. Exact-value goldens
    /// belong with the next oracle run.
    #[test]
    fn the_newly_linked_fluids_answer_states_that_obey_the_physics() {
        const R: f64 = 8.314_462_618_153_24;
        for fluid in [
            "Ammonia",
            "Nitrogen",
            "Methane",
            "Propane",
            "Argon",
            "CarbonMonoxide",
            "Ethane",
            "Helium",
            "Hydrogen",
            "Oxygen",
        ] {
            let (t, p) = (300.0, 101_325.0);

            // Ideal-gas limit. At 300 K and 1 atm every one of these is a dilute
            // gas, so rho must sit within 2% of PM/(RT) — close enough to catch
            // a wrong fluid's data, loose enough for the real compressibility.
            let m = B.props_si("M", "T", t, "P", p, fluid).unwrap();
            let rho = B.props_si("Dmass", "T", t, "P", p, fluid).unwrap();
            let ideal = p * m / (R * t);
            assert!(
                ((rho - ideal) / ideal).abs() < 0.02,
                "{fluid}: rho = {rho}, ideal gas = {ideal}"
            );

            // The (P, h) inverse is the one every solve path leans on.
            let h = B.props_si("Hmass", "T", t, "P", p, fluid).unwrap();
            let back = B.props_si("T", "P", p, "Hmass", h, fluid).unwrap();
            assert!((back - t).abs() < 1e-6, "{fluid}: T round-trip gave {back}");

            // Transport properties are a separate table from the EoS and do
            // not come with it. CarbonMonoxide is the one fluid on the served
            // list with no viscosity or conductivity model upstream — it gets
            // density, enthalpy and entropy and refuses the other two with
            // "Viscosity model is not available for this fluid". Pinned as a
            // known limitation so it is a documented absence rather than a
            // surprise, and so the day upstream adds one, this test says so.
            let transport_expected = fluid != "CarbonMonoxide";
            for output in ["viscosity", "conductivity"] {
                match B.props_si(output, "T", t, "P", p, fluid) {
                    Ok(value) => {
                        assert!(
                            transport_expected,
                            "{fluid}: {output} now answers ({value})"
                        );
                        assert!(
                            value.is_finite() && value > 0.0,
                            "{fluid}: {output} = {value}"
                        );
                    }
                    Err(e) => assert!(!transport_expected, "{fluid}: {output} failed: {e}"),
                }
            }
        }

        // The refrigerants: no ideal-gas leg (R123 is liquid at 300 K / 1 atm,
        // rho = 1459 kg/m3), but the (P,h) inverse still has to invert and the
        // density still has to be finite and positive.
        for fluid in [
            "R12",
            "R22",
            "R32",
            "R123",
            "R245fa",
            "R404A",
            "R407C",
            "R410A",
            "R1234ze(E)",
            "IsoButane",
            "n-Butane",
        ] {
            let (t, p) = (300.0, 101_325.0);
            let rho = B.props_si("Dmass", "T", t, "P", p, fluid).unwrap();
            assert!(rho.is_finite() && rho > 0.0, "{fluid}: rho = {rho}");
            let h = B.props_si("Hmass", "T", t, "P", p, fluid).unwrap();
            let back = B.props_si("T", "P", p, "Hmass", h, fluid).unwrap();
            assert!((back - t).abs() < 1e-6, "{fluid}: T round-trip gave {back}");
        }

        // Saturation, at a pressure comfortably inside every one of their
        // two-phase regions.
        for (fluid, pressure) in [
            ("Ammonia", 1.0e6),
            ("Nitrogen", 1.0e6),
            ("Methane", 1.0e6),
            ("Propane", 1.0e6),
            ("R12", 5.0e5),
            ("R22", 1.0e6),
            ("R32", 1.0e6),
            ("R410A", 1.0e6),
            ("Ethane", 1.0e6),
            ("Oxygen", 1.0e6),
            ("Argon", 1.0e6),
        ] {
            let rho_f = B.props_si("Dmass", "P", pressure, "Q", 0.0, fluid).unwrap();
            let rho_g = B.props_si("Dmass", "P", pressure, "Q", 1.0, fluid).unwrap();
            let h_f = B.props_si("Hmass", "P", pressure, "Q", 0.0, fluid).unwrap();
            let h_g = B.props_si("Hmass", "P", pressure, "Q", 1.0, fluid).unwrap();
            assert!(rho_f > rho_g, "{fluid}: rho_f {rho_f} <= rho_g {rho_g}");
            assert!(h_g > h_f, "{fluid}: h_g {h_g} <= h_f {h_f}");
        }
    }

    /// The refrigerant designations reach the same data, not a second copy of
    /// it — `props/propfun.rs` already carried both aliases while the data
    /// behind them was missing.
    ///
    /// `resolve_fluid` takes the lowercased spelling, which is the alphabet the
    /// AST stores and therefore what the document path hands it.
    #[test]
    fn the_refrigerant_aliases_reach_the_same_fluids() {
        for (alias, canonical) in [("r717", "Ammonia"), ("r290", "Propane")] {
            let resolved = crate::props::propfun::resolve_fluid(alias).unwrap();
            let via_alias = B
                .props_si("Hmass", "T", 300.0, "P", 101_325.0, &resolved)
                .unwrap();
            let direct = B
                .props_si("Hmass", "T", 300.0, "P", 101_325.0, canonical)
                .unwrap();
            assert_eq!(via_alias, direct, "{alias} -> {resolved} vs {canonical}");
        }
    }

    #[test]
    fn glycol_cp_and_viscosity_answer() {
        let fluid = "INCOMP::MEG[0.50]";
        let cp = B
            .props_si("Cpmass", "T", 300.0, "P", 101325.0, fluid)
            .unwrap();
        let mu = B
            .props_si("viscosity", "T", 300.0, "P", 101325.0, fluid)
            .unwrap();
        assert!(cp.is_finite() && cp > 0.0, "Cpmass = {cp}");
        assert!(mu.is_finite() && mu > 0.0, "viscosity = {mu}");
    }

    #[test]
    fn humid_air_enthalpy_answers() {
        let h = B
            .ha_props_si("H", "T", 298.15, "P", 101325.0, "R", 0.5)
            .unwrap();
        assert!(h.is_finite(), "HAPropsSI H = {h}");
    }

    #[test]
    fn props1_si_answers_water_tcrit() {
        let t = B.props1_si("Water", "Tcrit").unwrap();
        // rustprop reports the numerical critical point for superancillary
        // fluids, so compare loosely against IAPWS-95's 647.096 K.
        assert!((t - 647.096).abs() < 0.1, "Tcrit(Water) = {t}");
    }

    /// The `finite` guard: a rustprop call that *succeeds* with a non-finite
    /// answer must reach the engine as a refusal, never as data.
    ///
    /// RE-PINNED 2026-08-18, at Wave-2 integration. The original witness was
    /// `PropsSI("Hmass", "T", 0, "Smass", 101325, "Water")`, which rustprop
    /// answered `Ok(NaN)`. Wave-2 R7 ported upstream's `post_update` validity
    /// gate onto the HEOS `update()` choke point, so rustprop now refuses that
    /// state with "rhomolar is not a valid number" — which is the *correct*
    /// movement: the CoolProp 8.0.0 wheel refuses it too, with "rhomolar is
    /// less than zero". The old pin fired exactly as it was written to.
    ///
    /// The guard itself is not obsolete — a scan of ~1.8 million
    /// (output, pair, value, fluid) combinations at integration still found
    /// 1 754 calls that return `Ok` with a non-finite value. This one is a
    /// representative: transport at an absurd temperature, where rustprop's
    /// ideal-gas derivative underflows to `NaN` and rustprop hands it back.
    ///
    /// Note what the oracle does here, because it is NOT parity: the wheel
    /// *raises* ("calc_alpha0_deriv_nocache returned invalid number ...
    /// tau: 6.47096e-28"). So this pin witnesses a remaining rustprop-side
    /// gap — the missing `ValidNumber` check on the ideal-gas derivative —
    /// and this guard is what keeps that gap out of the frees engine. If
    /// rustprop closes it, this assertion fires again and wants a new witness
    /// from the same scan; the guard stays either way.
    ///
    /// RE-PINNED AGAIN 2026-08-18, at Wave-3 integration — and this time there
    /// is no live witness left to pin to, so the guard is exercised directly.
    /// Wave-3 R11 closed exactly the gap the paragraph above names: it ported
    /// upstream's two `ValidNumber` gates (`calc_alpha0_deriv_nocache`'s throw
    /// and the nanobind layer's `_raise_if_invalid`), so rustprop now refuses
    /// `L(Water, T=1e30, P=101325)` with the wheel's message verbatim. That
    /// comment said this assertion would fire again and want a new witness
    /// from the same scan. It fired.
    ///
    /// There is no new witness. A 1,088,640-call sweep at this integration —
    /// 30 outputs x 21 input pairs x 12 values x 12 fluids, spanning HEOS,
    /// pseudo-pure, SRK, PR, IF97, PC-SAFT and incompressible — returned
    /// 30,482 `Ok` and **zero** non-finite among them, against the ~1,754 such
    /// calls the Wave-2 scan found. The class is closed, not just this state.
    ///
    /// So the guard is now tested for what it *is*, rather than through a
    /// backend state that no longer produces one. It is NOT obsolete:
    /// [`RealFluid`]'s contract is frees-side, `finite` is the only thing
    /// enforcing it, and a backend regression — or a future backend that is
    /// not rustprop — must still meet a refusal rather than put a `NaN` into a
    /// Newton residual. Deleting a guard because its trigger was fixed
    /// elsewhere is how the trigger comes back unnoticed.
    #[test]
    fn a_non_finite_answer_becomes_a_refusal() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let err =
                finite(bad, || "L(Water, T=1e30, P=101325)".to_string()).expect_err("must refuse");
            assert!(matches!(err, FreesError::Property { .. }));
            let msg = err.to_string_message();
            assert!(
                msg.contains("Water") && msg.contains("not a finite value"),
                "{msg}"
            );
        }
        // A finite answer passes through untouched, bitwise.
        let good = finite(0.609_499_858_485_592_3, || "x".to_string()).unwrap();
        assert_eq!(good, 0.609_499_858_485_592_3);
    }

    /// The state the guard above was pinned to until Wave-3, kept as a
    /// regression witness for the fidelity gain R11 landed: rustprop refuses it
    /// now, with the CoolProp 8.0.0 wheel's own message, instead of answering
    /// `NaN`. The sibling below does the same job for the Wave-2 R7 witness.
    #[test]
    fn the_alpha0_nan_state_is_now_refused_by_rustprop_itself() {
        let raw = rustprop::props_si("L", "T", 1e30, "P", 101_325.0, "Water");
        let msg = raw
            .expect_err("R11 gated the ideal-gas derivative")
            .to_string();
        assert!(
            msg.contains("calc_alpha0_deriv_nocache returned invalid number"),
            "expected upstream's verbatim alpha0 text, got: {msg}"
        );
        // And it still reaches the engine as a property error.
        let err = B
            .props_si("L", "T", 1e30, "P", 101_325.0, "Water")
            .unwrap_err();
        assert!(matches!(err, FreesError::Property { .. }));
    }

    /// The state the guard above used to be pinned to, kept as a regression
    /// witness for the fidelity gain Wave-2 R7 landed: rustprop refuses it now,
    /// as the CoolProp 8.0.0 wheel does, instead of answering `NaN`. Either way
    /// the caller gets a property error — this asserts the *route* changed from
    /// the `finite` guard to rustprop's own `post_update`.
    #[test]
    fn the_old_nan_state_is_now_refused_by_rustprop_itself() {
        let raw = rustprop::props_si("Hmass", "T", 0.0, "Smass", 101_325.0, "Water");
        assert!(
            raw.is_err(),
            "rustprop should refuse T=0 K here (upstream: 'rhomolar is less than zero')"
        );
        let err = B
            .props_si("Hmass", "T", 0.0, "Smass", 101_325.0, "Water")
            .unwrap_err();
        assert!(matches!(err, FreesError::Property { .. }));
    }

    /// The `finite_inputs` guard. This one is not a nicety: without it the call
    /// below trips a bracketing `assert!` inside rustprop's Chebyshev root
    /// finder, and the shipped wasm is `panic = "abort"` — so a `NaN` that a
    /// diverging solve produced would take the engine worker down rather than
    /// return an error. `props_robustness`'s key sweep is what found it.
    #[test]
    fn a_non_finite_input_is_refused_before_it_reaches_rustprop() {
        for (k1, v1, k2, v2) in [
            ("Smass", f64::NAN, "Hmass", 101_325.0),
            ("T", f64::INFINITY, "P", 101_325.0),
            ("P", 101_325.0, "T", f64::NEG_INFINITY),
        ] {
            let err = B.props_si("Dmass", k1, v1, k2, v2, "Water").unwrap_err();
            assert!(matches!(err, FreesError::Property { .. }));
            let msg = err.to_string_message();
            assert!(msg.contains("not a finite value"), "{msg}");
        }
        let err = B
            .ha_props_si("H", "T", 298.15, "P", 101_325.0, "R", f64::NAN)
            .unwrap_err();
        assert!(err.to_string_message().contains("not a finite value"));
    }

    /// D3 slice (2026-08-22): the humid-air **error shapes**, pinned.
    ///
    /// The success path is graded against 912 recorded CoolProp points in
    /// `tests/humidair_grading.rs`; nothing graded the refusal path, and a
    /// rustprop tag bump that changed these messages would have slipped
    /// through with every gate green. Each message below is upstream CoolProp
    /// 8.0.0's `HAPropsSI` text verbatim (rustprop's `Display` is upstream's
    /// `what()`), probed 2026-08-22 against the v0.1.0 tag — so this test is
    /// the tripwire: if a bump changes one, the change is a fidelity question
    /// to answer, not a silent drift.
    #[test]
    fn humid_air_error_shapes_are_upstreams_verbatim() {
        type Case = (
            &'static str,
            &'static str,
            f64,
            &'static str,
            f64,
            &'static str,
            f64,
            &'static str,
        );
        let cases: &[Case] = &[
            // An unknown output key names the parser and lists the vocabulary.
            (
                "Banana", "T", 298.15, "P", 101_325.0, "R", 0.5,
                "your input [Banana] was not understood to Name2Type",
            ),
            // Relative humidity outside [0, 1], either side, names key + range.
            (
                "H", "T", 298.15, "P", 101_325.0, "R", 1.5,
                "The input for key (Rh) with value (1.5) is outside the range of validity: (0) to (1)",
            ),
            (
                "H", "T", 298.15, "P", 101_325.0, "R", -0.2,
                "The input for key (Rh) with value (-0.2) is outside the range of validity: (0) to (1)",
            ),
            // Dry-bulb below the 130 K floor of the RP-1485 model.
            (
                "H", "T", 100.0, "P", 101_325.0, "R", 0.5,
                "The input for key (T) with value (100) is outside the range of validity",
            ),
            // A trio without pressure — duplicate keys included — is refused
            // with upstream's fixed sentence, not a generic parse error.
            (
                "H", "T", 298.15, "T", 300.0, "R", 0.5,
                "Pressure must be one of the inputs to HAPropsSI",
            ),
            (
                "H", "T", 298.15, "R", 0.5, "W", 0.01,
                "Pressure must be one of the inputs to HAPropsSI",
            ),
            // Humidity ratio below zero names upstream's HumRat spelling.
            (
                "H", "T", 298.15, "P", 101_325.0, "W", -0.01,
                "The input for key (HumRat) with value (-0.01) is outside the range of validity: (0) to (10)",
            ),
            // A non-positive pressure is a range refusal of its own.
            ("H", "T", 298.15, "P", 0.0, "R", 0.5, "Pressure out of range"),
        ];
        for (out, n1, v1, n2, v2, n3, v3, want) in cases {
            let err = B
                .ha_props_si(out, n1, *v1, n2, *v2, n3, *v3)
                .expect_err(want);
            assert!(matches!(err, FreesError::Property { .. }));
            let msg = err.to_string_message();
            assert!(
                msg.contains(want),
                "{out}({n1}={v1}, {n2}={v2}, {n3}={v3}): expected {want:?} in {msg:?}"
            );
        }
    }

    /// Two edges the refusal pins above must NOT grow guards for, recorded so
    /// nobody "fixes" them on the frees side: upstream accepts `W` at its
    /// range ceiling of 10 kg/kg, and it computes from a wet-bulb *above* the
    /// dry-bulb without any consistency check. Both answer finite numbers in
    /// CoolProp 8.0.0 and therefore must answer here — an invented refusal
    /// would be a parity break, not a safety net.
    #[test]
    fn humid_air_upstream_permissiveness_is_preserved() {
        let at_w_ceiling = B
            .ha_props_si("H", "T", 298.15, "P", 101_325.0, "W", 10.0)
            .unwrap();
        assert!(at_w_ceiling.is_finite() && at_w_ceiling > 0.0);
        let wetbulb_above_drybulb = B
            .ha_props_si("H", "T", 298.15, "P", 101_325.0, "B", 320.0)
            .unwrap();
        assert!(wetbulb_above_drybulb.is_finite() && wetbulb_above_drybulb > 0.0);
    }

    #[test]
    fn errors_classify_as_property_errors() {
        let unknown = B
            .props_si("Hmass", "T", 300.0, "P", 101325.0, "NotAFluid")
            .unwrap_err();
        assert!(matches!(unknown, FreesError::Property { .. }));

        let bad_pair = B
            .props_si("Hmass", "T", 300.0, "T", 300.0, "Water")
            .unwrap_err();
        assert!(matches!(bad_pair, FreesError::Property { .. }));
    }
}
