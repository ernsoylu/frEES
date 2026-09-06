//! Component AST — `COMPONENT` templates, their instantiations and `connect`.
//!
//! Port of
//! `../frEES/backend/core/src/main/java/com/frees/backend/ast/ComponentDef.java`
//! (73 LOC, with its nested `Param` and `Variant` records), `ComponentInst.java`
//! (19) and `ConnectDecl.java` (17).
//!
//! These types are a **fixed contract**, exactly as [`crate::parser::defs`] is
//! for the Phase-4 procedural layer: [`crate::parser::toplevel`] fills them from
//! the grammar rules `componentDef`, `componentItem`, `componentVariant`,
//! `componentParam`, `componentInst`, `connectStmt`, `connectPort`,
//! `componentArgList` and `componentArg`, and the expander
//! ([`crate::components::expander`]) consumes them.
//!
//! # What this layer does *not* decide
//!
//! Everything here is syntax. The rules the Java engine enforces later, at
//! expansion time, are deliberately absent — mirroring where `ComponentExpander`
//! and its constructor put them, so the port cannot refuse a document the Java
//! accepts:
//!
//! * **Missing parameter values.** `componentParam : IDENT (EQ expr)?` makes a
//!   default optional at the *language* level. The shipped standard library
//!   nonetheless writes **no defaults for physical inputs** — "a model of an
//!   R134a system that forgets `fluid$` should error, not quietly run as water"
//!   (`ComponentLibrary`'s class doc). That rule is not a grammar rule and is
//!   not enforced here: `ComponentExpander.resolve` raises
//!   *"parameter 'x' has no value (give it a default or pass x=value)"* when a
//!   parameter is neither defaulted nor supplied at instantiation. The one place
//!   the library *does* use a default is the `model$` variant selector
//!   (`PARAM …, model$ = isentropic`), which the same code path reads.
//! * **Duplicate names.** Two `COMPONENT`s of one name, or two instances of one
//!   name, are hard errors raised by the `ComponentExpander` constructor — the
//!   parser appends both, so [`Components::def`] answers with the first.
//! * **Unknown / unselected variants**, `model$` resolution and the
//!   `REQUIRE`-scoped optionality of a parameter:
//!   `ComponentExpander.selectVariant`.
//! * **Domain separation** of a `connect` node: also the expander.

use crate::ast::{Equation, Expr};

/// A component parameter: `PARAM eta`, `PARAM eta = 0.7`, `PARAM fluid$`.
///
/// Port of `ComponentDef.Param`. `name` is lowercase; a trailing `$` marks a
/// string parameter (a fluid name, a table name, a `model$` variant selector),
/// whose value is baked into the encoded property-call names of the body at
/// expansion time.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// Lowercase name, including any trailing `$`.
    pub name: String,
    /// The `= expr` default, absent when the declaration carried none.
    pub default_value: Option<Expr>,
    /// `name.ends_with('$')`, stored because `ComponentExpander` branches on it.
    pub is_string: bool,
}

impl Param {
    /// A parameter declared by name, with `is_string` derived from the `$`
    /// suffix exactly as `AstBuilder.buildComponentDef` derives it.
    pub fn new(name: impl Into<String>, default_value: Option<Expr>) -> Param {
        let name = name.into();
        Param {
            is_string: name.ends_with('$'),
            name,
            default_value,
        }
    }
}

/// `VARIANT name [REQUIRE p, q] … END` — a selectable physics model.
///
/// Port of `ComponentDef.Variant`. The component's `model$` parameter picks one
/// variant by `name`; the chosen variant's `body` is expanded **alongside** the
/// shared [`ComponentDef::body`]. `require` lists the parameters that variant
/// needs — validated only when it is the selected one, so a parameter used
/// solely by an unselected variant need not be supplied.
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    /// Lowercase variant name.
    pub name: String,
    /// Lowercase `REQUIRE` parameter names, in declaration order.
    pub require: Vec<String>,
    /// The variant-only equations. The grammar admits equations only here — no
    /// `PARAM`, no nested `VARIANT`, no sub-instance.
    pub body: Vec<Equation>,
}

/// `COMPONENT Name(port1, port2, …) … END` — a reusable, parameterised set of
/// acausal equations with typed ports.
///
/// Port of `ComponentDef`. Body equations reference port members through a
/// dotted accessor (`in.P`, `out.h`, `out.mdot`), which the expression grammar
/// carries whole in an [`Expr::Var`] with a dotted name; a bare name in the body
/// is either a declared [`Param`] or a component-local variable / named output.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentDef {
    /// Lowercase component name.
    pub name: String,
    /// Lowercase port names, in declaration order — the order positional
    /// arguments of a [`ComponentInst`] bind to.
    pub ports: Vec<String>,
    /// Declared parameters, in declaration order, **followed by** the
    /// variant-scoped parameters synthesised from `REQUIRE` clauses (see
    /// [`ComponentDef::new`]).
    pub params: Vec<Param>,
    /// Equations shared by every variant.
    pub body: Vec<Equation>,
    pub variants: Vec<Variant>,
    /// Sub-instances, when this component is a hierarchical subsystem.
    pub sub_instances: Vec<ComponentInst>,
    /// `connect` declarations internal to a hierarchical subsystem.
    pub sub_connects: Vec<ConnectDecl>,
}

impl ComponentDef {
    /// Assemble a definition, applying the one piece of post-processing
    /// `AstBuilder.buildComponentDef` does after walking the items: **a name
    /// listed in a `VARIANT`'s `REQUIRE` clause is a variant-scoped parameter**,
    /// so any that is not already an explicit `PARAM` is appended with no
    /// default (it is required only when its variant is selected). A trailing
    /// `$` marks it a string parameter, exactly as for a `PARAM`.
    pub fn new(
        name: String,
        ports: Vec<String>,
        mut params: Vec<Param>,
        body: Vec<Equation>,
        variants: Vec<Variant>,
        sub_instances: Vec<ComponentInst>,
        sub_connects: Vec<ConnectDecl>,
    ) -> ComponentDef {
        for variant in &variants {
            for required in &variant.require {
                if params.iter().any(|p| p.name == *required) {
                    continue;
                }
                params.push(Param::new(required.clone(), None));
            }
        }
        ComponentDef {
            name,
            ports,
            params,
            body,
            variants,
            sub_instances,
            sub_connects,
        }
    }

    /// Whether this component is a hierarchical subsystem (built from
    /// sub-instances). Port of `ComponentDef.isHierarchical`.
    pub fn is_hierarchical(&self) -> bool {
        !self.sub_instances.is_empty()
    }

    /// The declared parameter with this (lowercase) name. Port of
    /// `ComponentDef.param`.
    pub fn param(&self, lowercase_name: &str) -> Option<&Param> {
        self.params.iter().find(|p| p.name == lowercase_name)
    }

    /// The variant with this (lowercase) name. Port of `ComponentDef.variant`.
    pub fn variant(&self, lowercase_name: &str) -> Option<&Variant> {
        self.variants.iter().find(|v| v.name == lowercase_name)
    }
}

/// The `name=value` parameter overrides of one instantiation.
///
/// Port of `ComponentInst`'s `Map<String, Expr>`, which the Java builds as a
/// **`LinkedHashMap`**: iteration follows first-insertion order, and re-assigning
/// a name replaces the value *in its original slot* rather than moving it to the
/// end. A `Vec` of pairs reproduces both without pulling in an ordered-map
/// dependency; the lists are a handful of entries long, so the linear lookup is
/// never on a hot path.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParamOverrides(Vec<(String, Expr)>);

impl ParamOverrides {
    pub fn new() -> ParamOverrides {
        ParamOverrides(Vec::new())
    }

    /// `LinkedHashMap.put` — returns the value it displaced, if any.
    pub fn put(&mut self, name: String, value: Expr) -> Option<Expr> {
        match self.0.iter_mut().find(|(key, _)| *key == name) {
            Some(slot) => Some(std::mem::replace(&mut slot.1, value)),
            None => {
                self.0.push((name, value));
                None
            }
        }
    }

    /// `Map.get`.
    pub fn get(&self, name: &str) -> Option<&Expr> {
        self.0
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Entries in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Expr)> {
        self.0.iter().map(|(key, value)| (key.as_str(), value))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// One instantiation: `Pump P1(s3, s4, eta=0.8, fluid$=Water)`.
///
/// Port of `ComponentInst`. `port_args` are the stream names bound to the
/// component's ports **in declaration order** (shared-name connection: two
/// instances naming the same stream are joined); `params` are `name=value`
/// overrides of the component's defaults.
#[derive(Debug, Clone)]
pub struct ComponentInst {
    /// Lowercase name of the [`ComponentDef`] being instantiated (`type` in the
    /// Java record — a Rust keyword).
    pub type_name: String,
    /// Lowercase instance name.
    pub name: String,
    /// Lowercase stream names bound positionally to the ports.
    pub port_args: Vec<String>,
    pub params: ParamOverrides,
    /// The instantiation as the user wrote it, verbatim, for diagnostics.
    pub source_text: String,
    /// 1-based source line of the instantiation, 0 when unknown (test fixtures).
    /// Navigation metadata: not part of structural equality, because the same
    /// library instance is line 221 in its file and line 564 in the concatenated
    /// source the Java parses as one string.
    pub line: usize,
}

impl PartialEq for ComponentInst {
    fn eq(&self, other: &Self) -> bool {
        self.type_name == other.type_name
            && self.name == other.name
            && self.port_args == other.port_args
            && self.params == other.params
            && self.source_text == other.source_text
    }
}

impl Eq for ComponentInst {}

/// One `connect(a.out, b.in, stream)` declaration.
///
/// Port of `ConnectDecl`. Each entry of `ports` is an endpoint written with a
/// dotted accessor — a port reference `instance.port` (`HP.out`) or a bare
/// stream name — kept as its lowercase dotted text, exactly as
/// `AstBuilder.buildConnect` keeps it. The expander ties the endpoints into one
/// node: across-variables equal, flow conserved.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectDecl {
    /// Lowercase dotted endpoint paths, in written order.
    pub ports: Vec<String>,
    /// The declaration as the user wrote it, verbatim, for diagnostics.
    pub source_text: String,
}

/// Everything the component layer of one document declared.
///
/// The three lists mirror the three fields `AstBuilder.ParseResult` carries
/// (`componentDefs`, `componentInsts`, `connects`); like the Java they are plain
/// append-ordered lists with no name-collision handling, because collisions are
/// the expander's error to raise (see the module docs).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Components {
    /// `COMPONENT … END` templates, in declaration order.
    pub defs: Vec<ComponentDef>,
    /// Top-level instantiations, in declaration order.
    pub instances: Vec<ComponentInst>,
    /// Top-level `connect` declarations, in declaration order.
    pub connects: Vec<ConnectDecl>,
}

impl Components {
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty() && self.instances.is_empty() && self.connects.is_empty()
    }

    /// The **first** definition with this (lowercase) name. A document that
    /// defines one name twice is rejected by the expander, not here.
    pub fn def(&self, lowercase_name: &str) -> Option<&ComponentDef> {
        self.defs.iter().find(|d| d.name == lowercase_name)
    }

    /// The **first** instance with this (lowercase) name. Duplicates are, again,
    /// the expander's error.
    pub fn instance(&self, lowercase_name: &str) -> Option<&ComponentInst> {
        self.instances.iter().find(|i| i.name == lowercase_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eq(text: &str) -> Equation {
        Equation::new(Expr::var("a"), Expr::num(1.0), text)
    }

    #[test]
    fn a_dollar_suffix_marks_a_string_parameter() {
        assert!(Param::new("fluid$", None).is_string);
        assert!(Param::new("model$", Some(Expr::var("isentropic"))).is_string);
        assert!(!Param::new("eta", None).is_string);
        // The `#` built-in-constant suffix is not the string marker.
        assert!(!Param::new("r#", None).is_string);
    }

    #[test]
    fn require_names_become_parameters_without_defaults() {
        let def = ComponentDef::new(
            "compressor".into(),
            vec!["in".into(), "out".into()],
            vec![Param::new("eta", Some(Expr::num(0.7)))],
            vec![eq("out.mdot = in.mdot")],
            vec![
                Variant {
                    name: "isentropic".into(),
                    require: vec!["eta".into()],
                    body: vec![],
                },
                Variant {
                    name: "map".into(),
                    require: vec!["map_eta$".into(), "rpm".into()],
                    body: vec![],
                },
            ],
            vec![],
            vec![],
        );
        // `eta` was already declared: it keeps its default and is not duplicated.
        assert_eq!(
            def.params
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            vec!["eta", "map_eta$", "rpm"]
        );
        assert_eq!(
            def.param("eta").unwrap().default_value,
            Some(Expr::num(0.7))
        );
        let synthesised = def.param("map_eta$").unwrap();
        assert_eq!(synthesised.default_value, None);
        assert!(synthesised.is_string, "the $ suffix survives the synthesis");
        assert!(!def.param("rpm").unwrap().is_string);
        assert_eq!(def.variant("map").unwrap().require.len(), 2);
        assert_eq!(def.variant("nope"), None);
    }

    #[test]
    fn a_name_required_by_two_variants_is_declared_once() {
        let def = ComponentDef::new(
            "c".into(),
            vec![],
            vec![],
            vec![],
            vec![
                Variant {
                    name: "a".into(),
                    require: vec!["k".into()],
                    body: vec![],
                },
                Variant {
                    name: "b".into(),
                    require: vec!["k".into(), "j".into()],
                    body: vec![],
                },
            ],
            vec![],
            vec![],
        );
        assert_eq!(
            def.params
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            vec!["k", "j"]
        );
    }

    #[test]
    fn a_component_is_hierarchical_only_when_it_has_sub_instances() {
        let leaf = ComponentDef::new("c".into(), vec![], vec![], vec![], vec![], vec![], vec![]);
        assert!(!leaf.is_hierarchical());

        let subsystem = ComponentDef::new(
            "chiller".into(),
            vec![],
            vec![],
            vec![],
            vec![],
            vec![ComponentInst {
                type_name: "pump".into(),
                name: "p1".into(),
                port_args: vec!["s1".into(), "s2".into()],
                params: ParamOverrides::new(),
                source_text: "Pump P1(s1, s2)".into(),
                line: 0,
            }],
            vec![],
        );
        assert!(subsystem.is_hierarchical());
    }

    #[test]
    fn param_overrides_keep_insertion_order_and_replace_in_place() {
        let mut params = ParamOverrides::new();
        assert!(params.is_empty());
        assert_eq!(params.put("eta".into(), Expr::num(1.0)), None);
        params.put("fluid$".into(), Expr::var("water"));
        // A second write to `eta` replaces the value without moving the slot,
        // exactly as `LinkedHashMap.put` does.
        assert_eq!(
            params.put("eta".into(), Expr::num(2.0)),
            Some(Expr::num(1.0))
        );
        assert_eq!(
            params.iter().map(|(k, _)| k).collect::<Vec<_>>(),
            vec!["eta", "fluid$"]
        );
        assert_eq!(params.get("eta"), Some(&Expr::num(2.0)));
        assert_eq!(params.get("nope"), None);
        assert!(params.contains_key("fluid$"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn components_lookups_answer_with_the_first_declaration() {
        let mut components = Components::default();
        assert!(components.is_empty());
        components.defs.push(ComponentDef::new(
            "pump".into(),
            vec!["in".into()],
            vec![Param::new("eta", None)],
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        components.defs.push(ComponentDef::new(
            "pump".into(),
            vec!["in".into(), "out".into()],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ));
        assert!(!components.is_empty());
        assert_eq!(components.def("pump").unwrap().ports.len(), 1);
        assert_eq!(components.def("turbine"), None);
        assert_eq!(components.instance("p1"), None);
    }
}
