//! `VARIANT … REQUIRE …` — the `model$` physics-variant selector.
//!
//! Port of the variant half of
//! `../frEES/backend/core/src/main/java/com/frees/backend/parser/ComponentExpander.java`
//! — `VARIANT_SELECTOR`, `selectVariant`, `variantNames`, `variantHint`,
//! `stringToken` and `ResolvedInstance.effectiveBody`.
//!
//! # One component, many models
//!
//! A single `COMPONENT` can ship several physics bodies at different fidelities
//! — a compressor as isentropic-η, as volumetric-η, or as a measured map — and
//! the instantiation picks one with a reserved string parameter:
//!
//! ```text
//! COMPONENT Compressor(in, out)
//!   PARAM model$ = isentropic, fluid$
//!   out.mdot = in.mdot                     ← shared by every variant
//!   VARIANT isentropic REQUIRE eta
//!     out.h = in.h + (h_s - in.h) / eta
//!   END
//!   VARIANT map REQUIRE map_mdot, map_eta
//!     out.mdot = map_mdot(out.P / in.P, rpm)
//!   END
//! END
//! ```
//!
//! Three rules govern it, and all three are load-bearing:
//!
//! 1. **Equations outside any `VARIANT` are shared.** The selected variant's
//!    body is expanded *alongside* [`ComponentDef::body`], never instead of it
//!    ([`VariantScope::effective_body`]). Mass conservation written once holds
//!    for every fidelity.
//! 2. **`REQUIRE` is validated per variant, not per component.** A parameter
//!    listed by some variant's `REQUIRE` but not the selected one's is
//!    *optional* — it need not be supplied and is silently dropped
//!    ([`VariantScope::is_optional`]). That is what lets a map-based compressor
//!    not demand the isentropic variant's `eta`, and vice versa.
//! 3. **A missing selection is an error, never a default.** A component that
//!    declares variants must declare a `PARAM model$`; an unknown variant name
//!    is rejected with the list of valid choices. Guessing a physics model is
//!    exactly the kind of silent wrong answer this engine refuses.
//!
//! Every rejection here names the **component and its instance**
//! (`Component 'x' (compressor): …`), per the parent engine's diagnostics
//! contract — never a mangled scalar.

use std::collections::BTreeSet;

use crate::ast::{Equation, Expr};
use crate::components::def::{ComponentDef, ComponentInst, Variant};
use crate::diag::{FreesError, Result};

/// The string parameter that selects a component's physics variant (§5.5).
///
/// Transcribed from `ComponentExpander.VARIANT_SELECTOR`.
pub const VARIANT_SELECTOR: &str = "model$";

/// The value of a string parameter, decoded from its parameter expression.
///
/// Port of `ComponentExpander.stringToken`. Two spellings are accepted and one
/// is not:
///
/// * a quoted string — lowercased, because the language is case-insensitive;
/// * a bare name (`fluid$ = Water`, `model$ = isentropic`) — already lowercase,
///   since [`Expr::var`] lowercases on construction;
/// * anything else (a number, an arithmetic expression) is a hard error.
///
/// It lives here rather than in the expander because the `model$` selector is
/// its first consumer, and every other string parameter — `fluid$`, `domain$`,
/// `map$`, `arr$` — is decoded by exactly the same two rules.
pub fn string_token(instance_name: &str, param_name: &str, value: &Expr) -> Result<String> {
    match value {
        Expr::Str(v) => Ok(v.to_ascii_lowercase()),
        Expr::Var(name) => Ok(name.clone()), // already lowercased by `Expr::var`
        _ => Err(FreesError::parse(format!(
            "Component '{instance_name}': string parameter '{param_name}' must be a \
             name or quoted string."
        ))),
    }
}

/// Which physics variant an instance selected, and what that implies for
/// parameter validation.
///
/// Built by [`VariantScope::resolve`], which is the port of
/// `ComponentExpander.selectVariant` plus the two `Set` computations
/// (`variantParams` / `selectedRequire`) that `resolve` derives from it.
#[derive(Debug, Clone)]
pub struct VariantScope<'d> {
    /// The chosen variant, or `None` when the component declares none.
    selected: Option<&'d Variant>,
    /// The union of **every** variant's `REQUIRE` names — the parameters that
    /// exist only because some variant asked for them.
    variant_params: BTreeSet<&'d str>,
}

impl<'d> VariantScope<'d> {
    /// Resolves the `model$` selection for one instantiation.
    ///
    /// Port of `selectVariant`. A component with no `VARIANT` blocks yields an
    /// empty scope; otherwise the selector parameter must exist and must resolve
    /// to a declared variant name.
    pub fn resolve(inst: &ComponentInst, def: &'d ComponentDef) -> Result<VariantScope<'d>> {
        let variant_params: BTreeSet<&'d str> = def
            .variants
            .iter()
            .flat_map(|v| v.require.iter().map(String::as_str))
            .collect();

        if def.variants.is_empty() {
            return Ok(VariantScope {
                selected: None,
                variant_params,
            });
        }

        let Some(selector) = def.param(VARIANT_SELECTOR) else {
            return Err(FreesError::parse(format!(
                "Component '{}' ({}): declares VARIANT blocks but no 'PARAM \
                 {VARIANT_SELECTOR}' selector to choose between them.",
                inst.name, inst.type_name
            )));
        };
        let selection = inst
            .params
            .get(VARIANT_SELECTOR)
            .or(selector.default_value.as_ref());
        let Some(selection) = selection else {
            return Err(FreesError::parse(format!(
                "Component '{}' ({}): no variant selected — give '{VARIANT_SELECTOR}' \
                 a default or pass {VARIANT_SELECTOR}=<variant>. Variants: {}.",
                inst.name,
                inst.type_name,
                variant_names(def)
            )));
        };
        let name = string_token(&inst.name, VARIANT_SELECTOR, selection)?;
        let Some(variant) = def.variant(&name) else {
            return Err(FreesError::parse(format!(
                "Component '{}' ({}): unknown {VARIANT_SELECTOR} '{name}'. Variants: {}.",
                inst.name,
                inst.type_name,
                variant_names(def)
            )));
        };
        Ok(VariantScope {
            selected: Some(variant),
            variant_params,
        })
    }

    /// The selected variant, or `None` when the component declares none.
    pub fn selected(&self) -> Option<&'d Variant> {
        self.selected
    }

    /// The selected variant's name, for diagnostics.
    pub fn selected_name(&self) -> Option<&'d str> {
        self.selected.map(|v| v.name.as_str())
    }

    /// Whether a declared parameter may be left unsupplied.
    ///
    /// Port of the `optional` flag in `ComponentExpander.resolve`: a parameter
    /// that exists only because *some* variant requires it, but which the
    /// **selected** variant does not, is optional — it is skipped silently
    /// rather than demanded. Ordinary `PARAM`s are never optional, even when a
    /// variant also lists them in `REQUIRE`, as long as that variant is the one
    /// selected.
    pub fn is_optional(&self, param_name: &str) -> bool {
        self.variant_params.contains(param_name) && !self.requires(param_name)
    }

    /// Whether the **selected** variant lists this parameter in its `REQUIRE`.
    pub fn requires(&self, param_name: &str) -> bool {
        self.selected
            .is_some_and(|v| v.require.iter().any(|r| r == param_name))
    }

    /// The clause appended to a missing-parameter error to point at the
    /// selected variant.
    ///
    /// Port of `variantHint`. Empty when the parameter is not the selected
    /// variant's doing, so an ordinary missing `PARAM` reads unchanged.
    pub fn hint(&self, param_name: &str) -> String {
        match self.selected {
            Some(v) if self.requires(param_name) => {
                format!(" (required by the selected '{}' variant).", v.name)
            }
            _ => String::new(),
        }
    }

    /// The equations to expand: the shared body **plus** the selected variant's.
    ///
    /// Port of `ResolvedInstance.effectiveBody`. Shared equations come first, in
    /// declaration order, then the variant's — the order the expanded equation
    /// list carries into the solver.
    pub fn effective_body<'a>(&self, def: &'a ComponentDef) -> Vec<&'a Equation>
    where
        'd: 'a,
    {
        let mut all: Vec<&'a Equation> = def.body.iter().collect();
        if let Some(v) = self.selected {
            all.extend(v.body.iter());
        }
        all
    }
}

/// The declared variant names, comma-separated, for the "Variants: …" tail of a
/// rejection. Port of `variantNames`.
pub fn variant_names(def: &ComponentDef) -> String {
    def.variants
        .iter()
        .map(|v| v.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::def::{Param, ParamOverrides};

    fn eq(text: &str) -> Equation {
        Equation::new(Expr::var("lhs"), Expr::var("rhs"), text)
    }

    fn variant(name: &str, require: &[&str], body: &[&str]) -> Variant {
        Variant {
            name: name.into(),
            require: require.iter().map(|r| (*r).to_string()).collect(),
            body: body.iter().map(|b| eq(b)).collect(),
        }
    }

    /// A compressor with two fidelities and a shared mass balance — the shape
    /// the module doc describes.
    fn compressor(default_model: Option<&str>) -> ComponentDef {
        ComponentDef::new(
            "compressor".into(),
            vec!["in".into(), "out".into()],
            vec![
                Param::new("model$", default_model.map(Expr::var)),
                Param::new("eta", None),
            ],
            vec![eq("out.mdot = in.mdot")],
            vec![
                variant("isentropic", &["eta"], &["out.h = in.h + dhs / eta"]),
                variant("map", &["map_eta$", "rpm"], &["out.h = map_eta$(rpm)"]),
            ],
            vec![],
            vec![],
        )
    }

    fn inst(name: &str, type_name: &str, overrides: &[(&str, Expr)]) -> ComponentInst {
        let mut params = ParamOverrides::new();
        for (k, v) in overrides {
            params.put((*k).to_string(), v.clone());
        }
        ComponentInst {
            type_name: type_name.into(),
            name: name.into(),
            port_args: vec![],
            params,
            source_text: format!("{type_name} {name}()"),
            line: 0,
        }
    }

    fn err(e: FreesError) -> String {
        match e {
            FreesError::Parse { message, .. } => message,
            other => panic!("expected a parse error, got {other:?}"),
        }
    }

    // ── stringToken ─────────────────────────────────────────────────────────

    #[test]
    fn a_string_parameter_is_a_bare_name_or_a_quoted_string() {
        assert_eq!(
            string_token("c1", "fluid$", &Expr::var("Water")).unwrap(),
            "water"
        );
        assert_eq!(
            string_token("c1", "fluid$", &Expr::Str("R134a".into())).unwrap(),
            "r134a"
        );
    }

    #[test]
    fn a_numeric_string_parameter_is_rejected_by_instance_name() {
        let e = string_token("c1", "model$", &Expr::num(2.0)).unwrap_err();
        assert_eq!(
            err(e),
            "Component 'c1': string parameter 'model$' must be a name or quoted string."
        );
    }

    // ── selection ───────────────────────────────────────────────────────────

    #[test]
    fn no_variants_means_no_selection_and_the_plain_body() {
        let def = ComponentDef::new(
            "pipe".into(),
            vec!["in".into(), "out".into()],
            vec![Param::new("k", Some(Expr::num(1.0)))],
            vec![eq("out.mdot = in.mdot"), eq("out.h = in.h")],
            vec![],
            vec![],
            vec![],
        );
        let scope = VariantScope::resolve(&inst("a", "pipe", &[]), &def).unwrap();
        assert_eq!(scope.selected(), None);
        assert_eq!(scope.selected_name(), None);
        assert_eq!(scope.effective_body(&def).len(), 2);
        assert!(!scope.is_optional("k"));
        assert_eq!(scope.hint("k"), "");
    }

    #[test]
    fn the_default_selector_picks_a_variant_without_an_override() {
        let def = compressor(Some("isentropic"));
        let scope = VariantScope::resolve(&inst("c1", "compressor", &[]), &def).unwrap();
        assert_eq!(scope.selected_name(), Some("isentropic"));
    }

    #[test]
    fn an_override_beats_the_default() {
        let def = compressor(Some("isentropic"));
        let scope = VariantScope::resolve(
            &inst("c1", "compressor", &[("model$", Expr::var("map"))]),
            &def,
        )
        .unwrap();
        assert_eq!(scope.selected_name(), Some("map"));
    }

    #[test]
    fn the_shared_body_is_expanded_alongside_the_selected_variant() {
        let def = compressor(Some("isentropic"));
        let scope = VariantScope::resolve(&inst("c1", "compressor", &[]), &def).unwrap();
        let body: Vec<&str> = scope
            .effective_body(&def)
            .iter()
            .map(|e| e.source_text.as_str())
            .collect();
        // Shared first, then the variant's — and the *unselected* variant's
        // equation is nowhere in sight.
        assert_eq!(body, vec!["out.mdot = in.mdot", "out.h = in.h + dhs / eta"]);
    }

    #[test]
    fn switching_the_variant_switches_the_physics_body() {
        let def = compressor(Some("isentropic"));
        let scope = VariantScope::resolve(
            &inst("c1", "compressor", &[("model$", Expr::var("map"))]),
            &def,
        )
        .unwrap();
        let body: Vec<&str> = scope
            .effective_body(&def)
            .iter()
            .map(|e| e.source_text.as_str())
            .collect();
        assert_eq!(body, vec!["out.mdot = in.mdot", "out.h = map_eta$(rpm)"]);
    }

    #[test]
    fn a_quoted_selector_value_works_too() {
        let def = compressor(None);
        let scope = VariantScope::resolve(
            &inst("c1", "compressor", &[("model$", Expr::Str("MAP".into()))]),
            &def,
        )
        .unwrap();
        assert_eq!(scope.selected_name(), Some("map"));
    }

    // ── per-variant REQUIRE validation ──────────────────────────────────────

    #[test]
    fn an_unselected_variants_parameter_is_optional() {
        let def = compressor(Some("isentropic"));
        let scope = VariantScope::resolve(&inst("c1", "compressor", &[]), &def).unwrap();
        // `map_eta$` and `rpm` belong to the unselected `map` variant.
        assert!(scope.is_optional("map_eta$"));
        assert!(scope.is_optional("rpm"));
        // `eta` is required by the selected variant, so it is NOT optional…
        assert!(!scope.is_optional("eta"));
        assert!(scope.requires("eta"));
        // …and a plain parameter no variant mentions is never optional.
        assert!(!scope.is_optional("model$"));
    }

    #[test]
    fn selecting_the_other_variant_flips_which_parameters_are_optional() {
        let def = compressor(Some("isentropic"));
        let scope = VariantScope::resolve(
            &inst("c1", "compressor", &[("model$", Expr::var("map"))]),
            &def,
        )
        .unwrap();
        assert!(scope.is_optional("eta"));
        assert!(!scope.is_optional("map_eta$"));
        assert!(!scope.is_optional("rpm"));
    }

    #[test]
    fn the_missing_parameter_hint_names_the_selected_variant() {
        let def = compressor(Some("isentropic"));
        let scope = VariantScope::resolve(&inst("c1", "compressor", &[]), &def).unwrap();
        assert_eq!(
            scope.hint("eta"),
            " (required by the selected 'isentropic' variant)."
        );
        // A parameter the selected variant does not require gets no hint.
        assert_eq!(scope.hint("rpm"), "");
        assert_eq!(scope.hint("model$"), "");
    }

    // ── rejections (verbatim against the Java oracle) ───────────────────────

    #[test]
    fn an_unknown_model_lists_the_valid_choices() {
        let def = compressor(Some("zzz"));
        let e = VariantScope::resolve(&inst("x", "compressor", &[]), &def).unwrap_err();
        assert_eq!(
            err(e),
            "Component 'x' (compressor): unknown model$ 'zzz'. Variants: isentropic, map."
        );
    }

    #[test]
    fn variants_without_a_selector_parameter_are_rejected() {
        let def = ComponentDef::new(
            "c".into(),
            vec!["in".into(), "out".into()],
            vec![],
            vec![],
            vec![variant("a", &["r"], &["out.P = in.P * r"])],
            vec![],
            vec![],
        );
        let e = VariantScope::resolve(&inst("x", "c", &[]), &def).unwrap_err();
        assert_eq!(
            err(e),
            "Component 'x' (c): declares VARIANT blocks but no 'PARAM model$' \
             selector to choose between them."
        );
    }

    #[test]
    fn a_selector_with_neither_default_nor_override_is_rejected() {
        let def = compressor(None);
        let e = VariantScope::resolve(&inst("x", "compressor", &[]), &def).unwrap_err();
        assert_eq!(
            err(e),
            "Component 'x' (compressor): no variant selected — give 'model$' a default \
             or pass model$=<variant>. Variants: isentropic, map."
        );
    }

    #[test]
    fn variant_names_are_listed_in_declaration_order() {
        assert_eq!(variant_names(&compressor(None)), "isentropic, map");
        let no_variants =
            ComponentDef::new("p".into(), vec![], vec![], vec![], vec![], vec![], vec![]);
        assert_eq!(variant_names(&no_variants), "");
    }
}
