//! Instantiation and connection — components to flat scalar equations.
//!
//! Port of
//! `../frEES/backend/core/src/main/java/com/frees/backend/parser/ComponentExpander.java`
//! (1,656 LOC), minus the domain/junction rules that live in
//! [`crate::components::domains`] and the `model$` variant selector that lives
//! in [`crate::components::variant`].
//!
//! # The one architectural fact
//!
//! This is a **parser/expander, not a second solver**. Every `COMPONENT`
//! instance is cloned into flat scalar equations that flow through the existing
//! Newton/Tarjan pipeline unchanged; `connect(...)` adds a handful more. Nothing
//! here reaches into the solver, and nothing here evaluates a number.
//!
//! # Names, and why they are the whole contract
//!
//! A **stream** is a bundle of solver variables. Its members flatten with a `$`:
//! `s2.P` is the variable `s2$p`, displayed back to the user as `s2.p`. That
//! mapping is what makes the terse connection style work at all — two
//! components that name the same stream *are* connected, because they read and
//! write the same scalars, with no equations in between.
//!
//! | written | solver variable | displayed |
//! |---|---|---|
//! | `s2.P` (shared-name stream) | `s2$p` | `s2.p` |
//! | `CHLR.in.P` (free port wired by `connect`) | `chlr$in$p` | `chlr.in.p` |
//! | `P1.W` (named output) | `p1$w` | `p1.w` |
//! | `D.R1.in.P` (sub-instance of a subsystem) | `d.r1$in$p` | `d.r1.in.p` |
//!
//! Every one of those spellings is user-visible in the result table, so they are
//! pinned by tests against the Java oracle rather than inferred.
//!
//! # Two connection syntaxes, one expansion
//!
//! * **Shared stream name** — `Pump P1(s3, s4)` binds ports positionally to
//!   stream names. Two instances naming `s4` share every `s4$…` variable, so a
//!   series chain conserves mass and energy with *zero* extra equations. Terse,
//!   but it cannot express a branch and cannot express a closed loop.
//! * **`connect(a, b, …)`** — an instance written with no positional arguments
//!   gets *free* ports, bound to synthetic per-instance streams `inst$port`;
//!   `connect` then ties any number of them into one node. A node emits the
//!   domain's across equalities plus one flow rule.
//!
//! The second is not a different mechanism, only a different way of deciding
//! which streams are the same one. Both end in the same flat scalars.
//!
//! # Union-find, and why a loop is not over-determined
//!
//! A closed cycle written with shared names is over-determined: the last link
//! restates what the chain already forced. `connect` fixes that with a
//! union-find over the connection graph, seeded with each fluid two-port's
//! internal in↔out link:
//!
//! * an across equality whose endpoints are **already** connected is dropped —
//!   only spanning-tree edges are emitted;
//! * a `connect` that closes a loop emits **no mass balance** either, because
//!   the loop `Σṁ` is cyclically dependent on the rest of the loop.
//!
//! A *capacitive* volume (one with a `der(port.P)` pressure state) is
//! deliberately **not** seeded: mass accumulates in it and its pressure is a
//! state, so it breaks both cycles. Seeding it would make a closed C-R-C-R loop
//! look fully connected and silently drop the closing node's equations —
//! the non-square closed-refrigerant-cycle bug. The fluid-identity propagation
//! *does* seed it, because the fluid really is the same through a volume.
//!
//! # Diagnostics
//!
//! Non-negotiable, and inherited from the parent engine: **every error names the
//! component and its instance**, never a mangled scalar. `Component 'p1' (pump):
//! parameter 'eta' has no value…`, not `p1$eta is unmatched`. The two places
//! where an internal name would otherwise leak — the high-index guard and the
//! `connect` equality source text — go through the display-name map.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::ast::{Equation, Expr, Statement};
use crate::components::def::{ComponentDef, ComponentInst, ConnectDecl, ParamOverrides};
use crate::components::domains::{
    self, ConnectorTypes, Domain, InstanceView, JunctionRule, StreamMembers,
};
use crate::components::variant::{string_token, VariantScope};
use crate::diag::{FreesError, Result};

/// Stream member → property-function name for **derived** state properties.
///
/// Transcribed from `ComponentExpander.DERIVED_PROPS`. A stream carries the
/// canonical members `(P, h, mdot)`; any other thermodynamic property named at
/// top level (`s3.T`, `s1.x`) is rewritten to the matching property call on
/// `(P, h)`, so a user can pin a state naturally and let Newton invert it.
///
/// `p`, `h` and `mdot` are deliberately absent — they are the solver's own
/// variables. So is every member on a *fluid-less* stream: without a fluid there
/// is nothing to evaluate, and a rider called `.x` must not be mistaken for
/// thermodynamic quality (see [`Network::top_stream_member`]).
const DERIVED_PROPS: [(&str, &str); 9] = [
    ("t", "temperature"),
    ("s", "entropy"),
    ("x", "quality"),
    ("v", "volume"),
    ("rho", "density"),
    ("d", "density"),
    ("u", "intenergy"),
    ("cp", "cp"),
    ("cv", "cv"),
];

fn derived_prop(member: &str) -> Option<&'static str> {
    DERIVED_PROPS
        .iter()
        .find(|(m, _)| *m == member)
        .map(|(_, prop)| *prop)
}

// ── Public results ──────────────────────────────────────────────────────────

/// An `init(member) = …` line lifted out of a component body.
///
/// Port of the `DynamicSystem.InitialCondition` values `componentInitials()`
/// hands back. It declares a transient state's initial value and is **not** a
/// solver equation. The component layer never produces array indices, so the
/// Java's always-empty `indices` list has no counterpart here.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentInitial {
    /// The flat state variable (`tk$t`), already namespaced.
    pub state: String,
    pub value: Expr,
}

/// One connection-topology edge for the schematic payload.
///
/// Port of `ComponentExpander.Connection`. `domain` is the bond-graph domain and
/// `endpoints` the refs the node joins, exactly as written. The rest is what a
/// *drawn* schematic needs and the coarse domain cannot supply:
///
/// * `connector` — the fluid connector type (`liquid`, `twophase`, `gas`, `oil`,
///   `moistair`, `fluid`); `None` outside the fluid domain. A coolant line and a
///   refrigerant line are both `domain = fluid`, so without this a renderer
///   cannot tell two circuits apart.
/// * `fluid` — the working fluid the node carries, so each gets its own style.
/// * `streams` — per endpoint, the display prefix its member variables use
///   (`CHLR.in` for a connect-wired free port, `s2` for a shared-name stream),
///   so solved values can be looked up without re-deriving the binding.
#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
    /// The Java record carries `Domain.name().toLowerCase()`; the typed value is
    /// lossless and [`Domain::as_str`] recovers that spelling for the wire.
    pub domain: Domain,
    pub endpoints: Vec<String>,
    pub connector: Option<String>,
    pub fluid: Option<String>,
    pub streams: Vec<String>,
}

// ── Insertion-ordered map ───────────────────────────────────────────────────

/// A minimal insertion-ordered string map — the port of the `LinkedHashMap`s
/// whose **iteration order the Java depends on**:
///
/// * `streamFluid` decides, per connected set, which known fluid propagates:
///   the first in insertion order wins (`rootFluid.putIfAbsent`);
/// * the capacitive-node map decides which offending node the C-C rejection
///   names;
/// * the shared-stream junction map fixes the order of [`Connection`]s in the
///   schematic payload.
///
/// A `BTreeMap` would silently reorder all three.
#[derive(Debug, Clone)]
struct OrderedMap<V> {
    order: Vec<String>,
    values: HashMap<String, V>,
}

impl<V> OrderedMap<V> {
    fn new() -> OrderedMap<V> {
        OrderedMap {
            order: Vec::new(),
            values: HashMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<&V> {
        self.values.get(key)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        self.values.get_mut(key)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// `LinkedHashMap.putIfAbsent`, reporting whether it inserted.
    fn put_if_absent(&mut self, key: &str, value: V) -> bool {
        if self.values.contains_key(key) {
            return false;
        }
        self.order.push(key.to_string());
        self.values.insert(key.to_string(), value);
        true
    }

    /// Entries in insertion order.
    fn iter(&self) -> impl Iterator<Item = (&str, &V)> {
        self.order
            .iter()
            .filter_map(|k| self.values.get(k.as_str()).map(|v| (k.as_str(), v)))
    }
}

// ── Union-find ──────────────────────────────────────────────────────────────

/// Minimal union-find over stream names, for connection-graph cycle detection.
///
/// Port of `ComponentExpander.UnionFind`, with the same semantics — an unseen
/// node is its own root (`putIfAbsent(x, x)`), and `union(a, b)` points `a`'s
/// root at `b`'s. The Java's `find` recurses; this iterates, because a long
/// series chain would otherwise recurse once per element.
#[derive(Debug, Default)]
struct UnionFind {
    parent: HashMap<String, String>,
}

impl UnionFind {
    fn new() -> UnionFind {
        UnionFind::default()
    }

    fn find(&mut self, x: &str) -> String {
        let mut root = x.to_string();
        while let Some(p) = self.parent.get(&root) {
            if *p == root {
                break;
            }
            root = p.clone();
        }
        self.parent
            .entry(root.clone())
            .or_insert_with(|| root.clone());
        // Path compression: everything walked now points straight at the root.
        let mut cur = x.to_string();
        while cur != root {
            match self.parent.insert(cur, root.clone()) {
                Some(next) => cur = next,
                None => break,
            }
        }
        root
    }

    fn union(&mut self, a: &str, b: &str) {
        let ra = self.find(a);
        let rb = self.find(b);
        self.parent.insert(ra, rb);
    }

    fn connected(&mut self, a: &str, b: &str) -> bool {
        self.find(a) == self.find(b)
    }
}

// ── Resolved instances ──────────────────────────────────────────────────────

/// A fully resolved instance: its definition, port→stream binding, parameter
/// values, and the selected physics variant.
///
/// Port of `ComponentExpander.ResolvedInstance`. The parameter lists stay in
/// declaration order — [`domains::port_fluid`] takes the *first* string
/// parameter whose base name prefixes a port, which is how a two-fluid heat
/// exchanger's `hot$`/`cold$` split works.
#[derive(Debug)]
struct ResolvedInstance<'d> {
    /// Owned, because hierarchy flattening mints new instances that exist in no
    /// source list.
    inst: ComponentInst,
    def: &'d ComponentDef,
    /// Port name → bound stream, in port-declaration order.
    port_to_stream: Vec<(String, String)>,
    numeric_params: Vec<(String, Expr)>,
    string_params: Vec<(String, String)>,
    variant: VariantScope<'d>,
}

impl<'d> ResolvedInstance<'d> {
    fn numeric(&self, name: &str) -> Option<&Expr> {
        self.numeric_params
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
    }

    fn string(&self, name: &str) -> Option<&str> {
        self.string_params
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    fn stream_of_port(&self, port: &str) -> Option<&str> {
        domains::port_stream(&self.port_to_stream, port)
    }

    fn streams(&self) -> impl Iterator<Item = &str> {
        self.port_to_stream.iter().map(|(_, s)| s.as_str())
    }

    /// The shared body plus the selected variant's — the equations to expand.
    fn effective_body(&self) -> Vec<&'d Equation> {
        self.variant.effective_body(self.def)
    }

    fn view(&self) -> InstanceView<'_> {
        InstanceView {
            string_params: &self.string_params,
            ports: &self.port_to_stream,
        }
    }
}

// ── The expander ────────────────────────────────────────────────────────────

/// Expands acausal `COMPONENT`/instantiation pairs and `connect(...)`
/// declarations into flat scalar equations.
///
/// Construction does all the resolution — hierarchy flattening, instantiation,
/// stream member collection, fluid inference, connector typing — and reports
/// every structural error. [`expand`](ComponentExpander::expand) then emits the
/// equations and [`rewrite_statements`](ComponentExpander::rewrite_statements)
/// rewrites the document's own dotted references onto the same flat names.
///
/// `display_names` is borrowed for the expander's lifetime and accumulated
/// in place, exactly as the Java passes its `Map<String, String>` around: the
/// mangled `s2$p` gets its `s2.p` spelling the moment it is minted, so no later
/// pass has to reverse-engineer one.
#[derive(Debug)]
pub struct ComponentExpander<'d, 'n> {
    net: Network<'d>,
    display_names: &'n mut BTreeMap<String, String>,
    initials: Vec<ComponentInitial>,
    has_storage: bool,
    /// Nonblocking advisories: a declared parameter was supplied but is inactive
    /// for the selected `model$` variant. Source is not rewritten.
    inactive_warnings: Vec<String>,
}

/// The resolved network — everything the rewrites read but never mutate.
///
/// Split out from [`ComponentExpander`] so a rewrite can hold the network
/// immutably while appending to the display-name map.
#[derive(Debug)]
struct Network<'d> {
    instances: Vec<ResolvedInstance<'d>>,
    /// Instance name → index into `instances`.
    instance_index: HashMap<String, usize>,
    /// Top-level `connect`s plus every one lifted out of a subsystem.
    connects: Vec<ConnectDecl>,
    /// Stream → its fluid, from the attached components' fluid parameters.
    stream_fluid: OrderedMap<String>,
    /// Stream → display prefix. A synthetic free-port stream `inst$port` shows
    /// as `inst.port` so member references read naturally.
    stream_display: BTreeMap<String, String>,
    stream_members: StreamMembers,
    connector_types: ConnectorTypes,
    /// `"sub.port"` → the stream that port is bound to, for the boundary ports
    /// of a flattened subsystem.
    port_alias: BTreeMap<String, String>,
    /// Whether any component definition is visible (built-in or user-declared).
    has_defs: bool,
}

impl<'d, 'n> ComponentExpander<'d, 'n> {
    /// Resolves a document's component layer.
    ///
    /// Port of the `ComponentExpander` constructor. Built-in standard-library
    /// components are curated, so a user definition of the same name overrides
    /// one silently; two *user* definitions of one name collide.
    /// ([`crate::components::library::Library::resolve`] implements the same
    /// rule and can feed this.)
    pub fn new(
        builtin_defs: &'d [ComponentDef],
        user_defs: &'d [ComponentDef],
        component_insts: &[ComponentInst],
        connects: &[ConnectDecl],
        display_names: &'n mut BTreeMap<String, String>,
    ) -> Result<ComponentExpander<'d, 'n>> {
        let defs_by_name = resolve_defs(builtin_defs, user_defs)?;

        // Hierarchy: flatten subsystem instances (a COMPONENT built from
        // sub-instances + internal connects) into leaf instances and connects
        // before resolving, so the rest of the expander sees a flat network.
        let mut flat_insts: Vec<ComponentInst> = Vec::new();
        let mut flat_conns: Vec<ConnectDecl> = connects.to_vec();
        let mut port_alias: BTreeMap<String, String> = BTreeMap::new();
        let mut stream_display: BTreeMap<String, String> = BTreeMap::new();
        for inst in component_insts {
            let mut stack: Vec<String> = Vec::new();
            flatten_instance(
                inst.clone(),
                &defs_by_name,
                &mut flat_insts,
                &mut flat_conns,
                &mut port_alias,
                &mut stream_display,
                &mut stack,
            )?;
        }

        let mut instances: Vec<ResolvedInstance<'d>> = Vec::with_capacity(flat_insts.len());
        let mut instance_index: HashMap<String, usize> = HashMap::new();
        let mut inactive_warnings = Vec::new();
        for inst in flat_insts {
            let resolved = resolve(
                inst,
                &defs_by_name,
                &mut stream_display,
                &mut inactive_warnings,
            )?;
            let name = resolved.inst.name.clone();
            if instance_index
                .insert(name.clone(), instances.len())
                .is_some()
            {
                return Err(FreesError::parse(format!(
                    "Component instance '{name}' is declared more than once."
                )));
            }
            instances.push(resolved);
        }

        let stream_members = build_stream_members(&instances);
        let stream_fluid = build_stream_fluid_map(&instances);
        let views: Vec<InstanceView<'_>> = instances.iter().map(ResolvedInstance::view).collect();
        let connector_types = domains::build_connector_types(&views, &stream_members)?;
        drop(views);

        let mut net = Network {
            instances,
            instance_index,
            connects: flat_conns,
            stream_fluid,
            stream_display,
            stream_members,
            connector_types,
            port_alias,
            has_defs: !defs_by_name.is_empty(),
        };
        net.propagate_fluid_across_connects()?;

        Ok(ComponentExpander {
            net,
            display_names,
            initials: Vec::new(),
            has_storage: false,
            inactive_warnings,
        })
    }

    /// Advisories collected while resolving instances. Empty when every supplied
    /// parameter is active for the selected variant.
    pub fn inactive_warnings(&self) -> &[String] {
        &self.inactive_warnings
    }

    /// Whether any component definitions or instances are present. Port of
    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        !self.net.has_defs && self.net.instances.is_empty()
    }

    /// The fluid inferred for a stream, if any.
    pub fn stream_fluid(&self, stream: &str) -> Option<&str> {
        self.net.stream_fluid.get(stream).map(String::as_str)
    }

    /// Stream → fluid, in inference order. Port of `streamFluids`.
    pub fn stream_fluids(&self) -> impl Iterator<Item = (&str, &str)> {
        self.net.stream_fluid.iter().map(|(k, v)| (k, v.as_str()))
    }

    /// Initial conditions collected from `init(member) = …` body lines. Valid
    /// after [`expand`](ComponentExpander::expand). Port of
    /// `componentInitials`.
    pub fn component_initials(&self) -> &[ComponentInitial] {
        &self.initials
    }

    /// Whether any component body declares a transient state with
    /// `der(member) = …`. Valid after [`expand`](ComponentExpander::expand);
    /// such a network is routed into a `DYNAMIC` block rather than the steady
    /// equation list. Port of `hasStorage`.
    pub fn has_storage(&self) -> bool {
        self.has_storage
    }

    /// SI units of each stream's canonical members, keyed by flat solver name
    /// (`s2$p` → `Pa`). Port of `memberUnits`.
    pub fn member_units(&self) -> BTreeMap<String, &'static str> {
        domains::member_units(&self.net.stream_members)
    }

    /// Expands every instance body — and every `connect(...)` node — into flat
    /// scalar equations.
    ///
    /// Port of `expand`. A component's `der(member) = …` line marks a transient
    /// state and stays as a state-derivative equation; an `init(member) = …`
    /// line declares that state's initial value and is lifted into
    /// [`component_initials`](ComponentExpander::component_initials) rather than
    /// becoming a solver equation.
    pub fn expand(&mut self) -> Result<Vec<Equation>> {
        let mut out: Vec<Equation> = Vec::new();
        let mut der_states: BTreeSet<String> = BTreeSet::new();

        for ri in &self.net.instances {
            let prefix = format!("COMPONENT {} {}: ", ri.def.name, ri.inst.name);
            for eq in ri.effective_body() {
                let lhs = self.net.rewrite_body(&eq.lhs, ri, self.display_names)?;
                let rhs = self.net.rewrite_body(&eq.rhs, ri, self.display_names)?;
                if let Expr::Call { function, args } = &lhs {
                    if args.len() == 1 {
                        if let Expr::Var(state) = &args[0] {
                            if function == "init" {
                                self.initials.push(ComponentInitial {
                                    state: state.clone(),
                                    value: rhs,
                                });
                                continue; // an initial condition, not an equation
                            }
                            if function == "der" {
                                self.has_storage = true;
                                der_states.insert(state.clone());
                            }
                        }
                    }
                }
                out.push(Equation::new(
                    lhs,
                    rhs,
                    format!("{prefix}{}", eq.source_text),
                ));
            }
        }

        self.net.expand_connects(&mut out, self.display_names)?;
        check_high_index(&out, &der_states, self.display_names)?;
        Ok(out)
    }

    /// Rewrites the dotted member references in top-level statements to flat
    /// names. Port of `rewriteStatements`.
    pub fn rewrite_statements(&mut self, statements: Vec<Statement>) -> Result<Vec<Statement>> {
        statements
            .into_iter()
            .map(|s| self.rewrite_statement(s))
            .collect()
    }

    /// Rewrites the dotted component references in a `DYNAMIC`-block body
    /// equation (e.g. a time-scheduled input `RIN.out.mdot = f(time)`) to their
    /// flat solver names — the *same* rewrite top-level statements get.
    ///
    /// Port of `rewriteTopEquation`. Without it a `DYNAMIC` body referencing
    /// `RIN.out.mdot` would target a variable distinct from the component's
    /// expanded `rin$out$mdot`, leaving the port unconstrained and the DAE
    /// non-square. This is what lets an acausal component take a
    /// scheduled/controlled transient input.
    pub fn rewrite_top_equation(&mut self, eq: &Equation) -> Result<Equation> {
        Ok(Equation::new(
            self.rewrite_top_expr(&eq.lhs)?,
            self.rewrite_top_expr(&eq.rhs)?,
            eq.source_text.clone(),
        ))
    }

    /// Rewrites dotted component references in an arbitrary expression (a
    /// `DYNAMIC` initial value or event guard) to flat solver names. Port of
    /// `rewriteTopExpr`.
    pub fn rewrite_top_expr(&mut self, e: &Expr) -> Result<Expr> {
        // No "no instances, no defs" short-circuit here: the Java's
        // `rewriteTop` runs unconditionally and mangles dotted names
        // (`a.b.c` -> `a$b$c`, display spelling recorded) even on an empty
        // network, and the component-free rewrite in
        // `engine::expand_component_layer` depends on exactly that (the
        // 2026-08-22 docs harvest's `docs_components_05`).
        self.net.rewrite_top(e, self.display_names)
    }

    /// The document's connection topology — the data layer of the rendered
    /// schematic.
    ///
    /// Port of `connections`. Explicit `connect(...)` nodes keep their endpoints
    /// as written; shared-stream junctions (two instance ports naming one
    /// stream) are reported as `instance.port` pairs. Domains reuse the node
    /// classification the expander already applies, so the payload can never
    /// disagree with the solve.
    pub fn connections(&self) -> Vec<Connection> {
        let mut out = self.net.explicit_connections();
        out.extend(self.net.shared_stream_junctions());
        out
    }

    fn rewrite_statement(&mut self, s: Statement) -> Result<Statement> {
        match s {
            Statement::Eq(eq) => Ok(Statement::Eq(Equation::new(
                self.net.rewrite_top(&eq.lhs, self.display_names)?,
                self.net.rewrite_top(&eq.rhs, self.display_names)?,
                eq.source_text,
            ))),
            Statement::For {
                var_name,
                start,
                end,
                body,
            } => {
                let mut rewritten = Vec::with_capacity(body.len());
                for b in body {
                    rewritten.push(self.rewrite_statement(b)?);
                }
                Ok(Statement::For {
                    var_name,
                    start: self.net.rewrite_top(&start, self.display_names)?,
                    end: self.net.rewrite_top(&end, self.display_names)?,
                    body: rewritten,
                })
            }
            // `SYMBOLIC` and `CALL` carry no dotted component references, and
            // the Java's `default -> s` leaves them alone.
            other => Ok(other),
        }
    }
}

// ── Construction helpers ────────────────────────────────────────────────────

/// Built-ins first, then user definitions layered over the top.
///
/// **The single implementation of the shadowing rule.** It lives here because
/// this is the copy on the solve path — the `ComponentExpander` constructor
/// calls it directly, exactly where the Java constructor builds its own
/// `defsByName`. [`crate::components::library::Library::resolve`] is a thin
/// wrapper over it, so the two can never drift; before this they were two
/// hand-written copies of the same twenty lines.
pub(crate) fn resolve_defs<'d>(
    builtin_defs: &'d [ComponentDef],
    user_defs: &'d [ComponentDef],
) -> Result<HashMap<&'d str, &'d ComponentDef>> {
    let mut by_name: HashMap<&'d str, &'d ComponentDef> =
        HashMap::with_capacity(builtin_defs.len() + user_defs.len());
    for d in builtin_defs {
        by_name.insert(d.name.as_str(), d);
    }
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for d in user_defs {
        if !seen.insert(d.name.as_str()) {
            return Err(FreesError::parse(format!(
                "COMPONENT '{}' is defined more than once.",
                d.name
            )));
        }
        by_name.insert(d.name.as_str(), d);
    }
    Ok(by_name)
}

/// Maximum nesting of hierarchical subsystems (a `COMPONENT` whose body
/// instantiates a `COMPONENT` whose body instantiates a `COMPONENT` …).
///
/// [`flatten_instance`] calls itself once per level, so a tower of wrappers
/// recurses once per wrapper and overflows the stack — an **abort**, not an
/// `Err`, that no caller can catch and that takes the whole wasm module with it.
/// Measured before this guard existed: a debug build died between 400 and 600
/// levels on a 2 MiB test-thread stack, and the browser's stack is smaller
/// still. The self-instantiation check above catches *cycles*; it cannot catch a
/// finite tower, because every level is a different name.
///
/// The shipped library's deepest subsystem is **depth 1** (15 hierarchical
/// components, none of which nests another subsystem), and a hand-written model
/// nests a handful. 64 is far beyond both and far below the ceiling — the same
/// number, chosen the same way, as [`crate::parser::toplevel`]'s
/// `MAX_BLOCK_DEPTH` and for the same reason.
///
/// **This has no Java counterpart.** `ComponentExpander.flattenInstance`
/// recurses unguarded and dies with `StackOverflowError`; the reference engine
/// survives that only because a JVM turns it into a catchable `Error` on a
/// thread it can abandon. A wasm module cannot.
const MAX_HIERARCHY_DEPTH: usize = 64;

/// The "binds N ports but declares M" rejection, shared by [`resolve`] and
/// [`flatten_instance`].
fn arity_error(inst: &ComponentInst, def: &ComponentDef) -> FreesError {
    FreesError::parse(format!(
        "Component '{}' ({}) binds {} port(s) but COMPONENT {} declares {} ({}). \
         Bind every port to a stream, or none and wire them with connect(...).",
        inst.name,
        inst.type_name,
        inst.port_args.len(),
        def.name,
        def.ports.len(),
        def.ports.join(", ")
    ))
}

/// Recursively flattens a (possibly hierarchical) instance into leaf instances
/// and connects.
///
/// Port of `flattenInstance`. A leaf is appended as-is. A hierarchical subsystem
/// expands its sub-instances (namespaced `outer.sub`) and rewrites its internal
/// connects: a reference to an outer port resolves to the stream that port is
/// bound to; a `sub.port` reference is namespaced; bare internal stream names
/// are namespaced too.
///
/// **One deliberate divergence.** The Java indexes `portArgs` positionally here
/// with no arity guard, so a hierarchical instance bound to too few streams dies
/// with `IndexOutOfBoundsException: Index 1 out of bounds for length 1` (checked
/// against the oracle). A panic is not an acceptable port of that, and the
/// engine's own contract says a diagnostic names the component and its instance
/// — so this raises the same [`arity_error`] a *leaf* instance of the same shape
/// already raises, which is what the Java's own `resolve` would have said one
/// step later.
fn flatten_instance<'d>(
    inst: ComponentInst,
    defs: &HashMap<&'d str, &'d ComponentDef>,
    out_insts: &mut Vec<ComponentInst>,
    out_conns: &mut Vec<ConnectDecl>,
    port_alias: &mut BTreeMap<String, String>,
    stream_display: &mut BTreeMap<String, String>,
    stack: &mut Vec<String>,
) -> Result<()> {
    let def = defs.get(inst.type_name.as_str()).copied();
    let Some(def) = def.filter(|d| d.is_hierarchical()) else {
        out_insts.push(inst);
        return Ok(());
    };
    if stack.contains(&inst.type_name) {
        return Err(FreesError::parse(format!(
            "COMPONENT '{}' instantiates itself (hierarchical cycle).",
            inst.type_name
        )));
    }
    if stack.len() >= MAX_HIERARCHY_DEPTH {
        return Err(FreesError::parse(format!(
            "COMPONENT '{}' is nested more than {MAX_HIERARCHY_DEPTH} subsystems deep \
             (via {}). Flatten the hierarchy — a model this deep is almost always a \
             wrapper chain that can be collapsed.",
            inst.type_name,
            stack.join(" > ")
        )));
    }
    stack.push(inst.type_name.clone());

    let free_ports = inst.port_args.is_empty() && !def.ports.is_empty();
    if !free_ports && inst.port_args.len() != def.ports.len() {
        return Err(arity_error(&inst, def));
    }
    let mut port_map: Vec<(String, String)> = Vec::with_capacity(def.ports.len());
    if free_ports {
        for port in &def.ports {
            let stream = format!("{}${port}", inst.name);
            port_alias.insert(format!("{}.{port}", inst.name), stream.clone());
            stream_display.insert(stream.clone(), format!("{}.{port}", inst.name));
            port_map.push((port.clone(), stream));
        }
    } else {
        for (port, stream) in def.ports.iter().zip(&inst.port_args) {
            port_alias.insert(format!("{}.{port}", inst.name), stream.clone());
            port_map.push((port.clone(), stream.clone()));
        }
    }

    // Resolve the subsystem's own parameter values (override or default) so they
    // can be substituted into the sub-instances' parameter expressions — a
    // cell's UA/fluid can reference the subsystem's UA/fluid.
    let mut outer_params: Vec<(String, Expr)> = Vec::new();
    for p in &def.params {
        if let Some(v) = inst.params.get(&p.name).or(p.default_value.as_ref()) {
            outer_params.push((p.name.clone(), v.clone()));
        }
    }

    for sub in &def.sub_instances {
        let sub_name = format!("{}.{}", inst.name, sub.name);
        let sub_ports: Vec<String> = sub
            .port_args
            .iter()
            .map(|pa| rewrite_sub_ref(pa, &inst.name, def, &port_map))
            .collect();
        let mut sub_params = ParamOverrides::new();
        for (key, value) in sub.params.iter() {
            sub_params.put(key.to_string(), substitute_params(value, &outer_params));
        }
        flatten_instance(
            ComponentInst {
                type_name: sub.type_name.clone(),
                name: sub_name,
                port_args: sub_ports,
                params: sub_params,
                source_text: sub.source_text.clone(),
            },
            defs,
            out_insts,
            out_conns,
            port_alias,
            stream_display,
            stack,
        )?;
    }
    for sc in &def.sub_connects {
        out_conns.push(ConnectDecl {
            ports: sc
                .ports
                .iter()
                .map(|r| rewrite_sub_ref(r, &inst.name, def, &port_map))
                .collect(),
            source_text: sc.source_text.clone(),
        });
    }
    stack.pop();
    Ok(())
}

/// Rewrites a reference inside a subsystem body: an outer port → its bound
/// stream; otherwise (a `sub.port` or an internal bare stream) → namespaced with
/// the subsystem instance name. Port of `rewriteSubRef`.
fn rewrite_sub_ref(
    reference: &str,
    prefix: &str,
    def: &ComponentDef,
    port_map: &[(String, String)],
) -> String {
    let r = reference.to_ascii_lowercase();
    if !r.contains('.') && def.ports.contains(&r) {
        if let Some(stream) = domains::port_stream(port_map, &r) {
            return stream.to_string();
        }
    }
    format!("{prefix}.{r}")
}

/// Substitutes a subsystem's parameter values into a sub-instance's parameter
/// expression (e.g. a cell's `UA = UA/2` where the outer `UA` is a subsystem
/// parameter).
///
/// Port of `substituteParams` — including its reach: only `Var`, `Neg`, `BinOp`
/// and `Call` are descended into, so an outer parameter used inside an array
/// index, a comparison or a matrix literal is *not* substituted.
fn substitute_params(e: &Expr, params: &[(String, Expr)]) -> Expr {
    match e {
        Expr::Var(name) => params
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| e.clone()),
        Expr::Neg(inner) => Expr::Neg(Box::new(substitute_params(inner, params))),
        Expr::BinOp { op, left, right } => Expr::BinOp {
            op: *op,
            left: Box::new(substitute_params(left, params)),
            right: Box::new(substitute_params(right, params)),
        },
        Expr::Call { function, args } => Expr::Call {
            function: function.clone(),
            args: args.iter().map(|a| substitute_params(a, params)).collect(),
        },
        other => other.clone(),
    }
}

/// Resolves one leaf instance against its definition. Port of `resolve`.
fn resolve<'d>(
    inst: ComponentInst,
    defs: &HashMap<&'d str, &'d ComponentDef>,
    stream_display: &mut BTreeMap<String, String>,
    inactive_warnings: &mut Vec<String>,
) -> Result<ResolvedInstance<'d>> {
    let Some(def) = defs.get(inst.type_name.as_str()).copied() else {
        return Err(FreesError::parse(format!(
            "Unknown component type '{}' for instance '{}'. Define it with COMPONENT {}(...).",
            inst.type_name, inst.name, inst.type_name
        )));
    };
    // Two instantiation styles: shared-name (every port bound positionally to a
    // stream) or connector (no positional args — ports are "free", bound to
    // synthetic per-instance streams `inst$port` that connect(...) ties).
    let free_ports = inst.port_args.is_empty() && !def.ports.is_empty();
    if !free_ports && inst.port_args.len() != def.ports.len() {
        return Err(arity_error(&inst, def));
    }
    let mut port_to_stream: Vec<(String, String)> = Vec::with_capacity(def.ports.len());
    if free_ports {
        for port in &def.ports {
            let synthetic = format!("{}${port}", inst.name);
            stream_display.insert(synthetic.clone(), format!("{}.{port}", inst.name));
            port_to_stream.push((port.clone(), synthetic));
        }
    } else {
        for (port, stream) in def.ports.iter().zip(&inst.port_args) {
            port_to_stream.push((port.clone(), stream.clone()));
        }
    }

    // Validate parameter overrides against the declared parameters.
    for (key, _) in inst.params.iter() {
        if def.param(key).is_none() {
            return Err(FreesError::parse(format!(
                "Component '{}' ({}): unknown parameter '{key}'.",
                inst.name, inst.type_name
            )));
        }
    }

    // Physics-variant selection (§5.5): the `model$` parameter picks one VARIANT
    // body to expand, and scopes which parameters are actually required.
    let variant = VariantScope::resolve(&inst, def)?;

    let mut numeric_params: Vec<(String, Expr)> = Vec::new();
    let mut string_params: Vec<(String, String)> = Vec::new();
    for p in &def.params {
        let supplied = inst.params.contains_key(&p.name);
        if supplied && variant.is_optional(&p.name) {
            let model = variant.selected_name().unwrap_or("default");
            inactive_warnings.push(format!(
                "Component '{}' ({}): parameter '{}' is not used by the selected \
                 '{model}' variant.",
                inst.name, inst.type_name, p.name
            ));
        }
        let value = inst.params.get(&p.name).or(p.default_value.as_ref());
        let Some(value) = value else {
            // A parameter listed in some variant's REQUIRE but not the selected
            // one's is optional — skip it silently when unsupplied.
            if variant.is_optional(&p.name) {
                continue;
            }
            let (kind, example) = if p.is_string {
                ("string parameter", "Name")
            } else {
                ("parameter", "value")
            };
            return Err(FreesError::parse(format!(
                "Component '{}' ({}): {kind} '{}' has no value (give it a default or \
                 pass {}={example}).{}",
                inst.name,
                inst.type_name,
                p.name,
                p.name,
                variant.hint(&p.name)
            )));
        };
        if p.is_string {
            string_params.push((p.name.clone(), string_token(&inst.name, &p.name, value)?));
        } else {
            numeric_params.push((p.name.clone(), value.clone()));
        }
    }

    Ok(ResolvedInstance {
        inst,
        def,
        port_to_stream,
        numeric_params,
        string_params,
        variant,
    })
}

/// Records, per stream, the member names its components reference in their
/// bodies. Port of `buildStreamMembers`.
fn build_stream_members(instances: &[ResolvedInstance<'_>]) -> StreamMembers {
    let mut members = StreamMembers::new();
    for ri in instances {
        for eq in ri.effective_body() {
            members.collect_from_equation(eq, &ri.port_to_stream);
        }
    }
    members
}

/// Whether the component declares a fluid for any of its ports. Port of
/// `definesFluid`.
fn defines_fluid(ri: &ResolvedInstance<'_>) -> bool {
    ri.def
        .ports
        .iter()
        .any(|port| domains::port_fluid(port, &ri.string_params).is_some())
}

/// Associates each stream with its fluid.
///
/// Port of `buildStreamFluidMap`. A fluid-bearing component assigns its ports
/// directly (per port: a multi-fluid HX maps hot ports → `hot$`, cold ports →
/// `cold$`). A fluid-less pass-through component (Boiler, Condenser, Throttle,
/// Splitter, Mixer) carries the same fluid on all its ports, so it propagates a
/// neighbour's fluid to its other streams — iterated to a fixpoint so the fluid
/// flows the length of a circuit.
fn build_stream_fluid_map(instances: &[ResolvedInstance<'_>]) -> OrderedMap<String> {
    let mut stream_fluid: OrderedMap<String> = OrderedMap::new();
    for ri in instances {
        for (port, stream) in &ri.port_to_stream {
            if let Some(fluid) = domains::port_fluid(port, &ri.string_params) {
                stream_fluid.put_if_absent(stream, fluid.to_string());
            }
        }
    }
    let mut changed = true;
    while changed {
        changed = false;
        for ri in instances {
            // Skip fluid-bearing components — the pass above already assigned
            // their streams (and a multi-fluid component must not
            // cross-contaminate).
            if defines_fluid(ri) {
                continue;
            }
            let known = ri.streams().find_map(|s| stream_fluid.get(s).cloned());
            let Some(known) = known else { continue };
            for (_, stream) in &ri.port_to_stream {
                if stream_fluid.put_if_absent(stream, known.clone()) {
                    changed = true;
                }
            }
        }
    }
    stream_fluid
}

/// High-index DAE guard (§8.9): two storage states forced equal by an algebraic
/// equation (two thermal masses tied to one node — `m1.T = m2.T` while both
/// carry `der`) make the system index ≥ 2 and singular. Reject it with an
/// actionable message rather than failing later with a singular matrix.
///
/// Port of `checkHighIndex`. The message quotes **display** names, so the user
/// reads `m1.p.t`, never `m1$p$t`.
fn check_high_index(
    equations: &[Equation],
    der_states: &BTreeSet<String>,
    display_names: &BTreeMap<String, String>,
) -> Result<()> {
    if der_states.len() < 2 {
        return Ok(());
    }
    for eq in equations {
        let (Expr::Var(a), Expr::Var(b)) = (&eq.lhs, &eq.rhs) else {
            continue;
        };
        if der_states.contains(a) && der_states.contains(b) && a != b {
            let show = |n: &String| {
                display_names
                    .get(n)
                    .cloned()
                    .unwrap_or_else(|| n.replace('$', "."))
            };
            return Err(FreesError::parse(format!(
                "High-index DAE: storage states '{}' and '{}' are rigidly coupled \
                 (directly equated) — index ≥ 2. Lump them into one storage element, \
                 or insert a small resistance/compliance between them.",
                show(a),
                show(b)
            )));
        }
    }
    Ok(())
}

/// Whether a component fixes a pressure state — has a `der(port.P)` in its body
/// or any variant (the marker of a capacitive volume). Port of
/// `isPressureCapacitive`.
fn is_pressure_capacitive(def: &ComponentDef) -> bool {
    body_has_pressure_der(&def.body, &def.ports)
        || def
            .variants
            .iter()
            .any(|v| body_has_pressure_der(&v.body, &def.ports))
}

fn body_has_pressure_der(body: &[Equation], ports: &[String]) -> bool {
    body.iter()
        .any(|eq| has_pressure_der(&eq.lhs, ports) || has_pressure_der(&eq.rhs, ports))
}

/// Port of `hasPressureDer` — including its reach: only `Call`, `BinOp` and
/// `Neg` are descended into.
fn has_pressure_der(e: &Expr, ports: &[String]) -> bool {
    match e {
        Expr::Call { function, args } => {
            if function.eq_ignore_ascii_case("der") && args.len() == 1 {
                if let Expr::Var(name) = &args[0] {
                    if let Some(dot) = name.rfind('.') {
                        if dot > 0 && name[dot + 1..].eq_ignore_ascii_case("p") {
                            let port = &name[..dot];
                            if ports.iter().any(|p| p.eq_ignore_ascii_case(port)) {
                                return true;
                            }
                        }
                    }
                }
            }
            args.iter().any(|a| has_pressure_der(a, ports))
        }
        Expr::BinOp { left, right, .. } => {
            has_pressure_der(left, ports) || has_pressure_der(right, ports)
        }
        Expr::Neg(inner) => has_pressure_der(inner, ports),
        _ => false,
    }
}

// ── The network ─────────────────────────────────────────────────────────────

impl Network<'_> {
    fn instance(&self, name: &str) -> Option<&ResolvedInstance<'_>> {
        self.instance_index.get(name).map(|&i| &self.instances[i])
    }

    /// A stream's display prefix: the `inst.port` spelling for a synthetic
    /// free-port stream, otherwise the stream name as the document wrote it.
    /// Port of `displayStream`.
    fn display_stream<'s>(&'s self, stream: &'s str) -> &'s str {
        self.stream_display
            .get(stream)
            .map(String::as_str)
            .unwrap_or(stream)
    }

    /// Mints the flat solver name for a stream member and registers its display
    /// spelling. Port of the `displayNames.putIfAbsent` inside `streamMember`.
    fn register_member(
        &self,
        stream: &str,
        member: &str,
        display_names: &mut BTreeMap<String, String>,
    ) -> String {
        let flat = domains::stream_member_name(stream, member);
        if !display_names.contains_key(&flat) {
            display_names.insert(
                flat.clone(),
                format!("{}.{member}", self.display_stream(stream)),
            );
        }
        flat
    }

    /// Port of `streamMember`.
    fn stream_member(
        &self,
        stream: &str,
        member: &str,
        display_names: &mut BTreeMap<String, String>,
    ) -> Expr {
        Expr::Var(self.register_member(stream, member, display_names))
    }

    /// Port of `equality`, with the display registration the pure
    /// [`domains::equality`] deliberately leaves to the expander.
    fn equality(
        &self,
        stream_a: &str,
        member: &str,
        stream_b: &str,
        prefix: &str,
        display_names: &mut BTreeMap<String, String>,
    ) -> Equation {
        self.register_member(stream_a, member, display_names);
        self.register_member(stream_b, member, display_names);
        domains::equality(stream_a, member, stream_b, prefix)
    }

    /// Resolves a connect endpoint (`instance.port` or a bare stream name) to
    /// its stream. Port of `streamOf`.
    fn stream_of<'s>(&'s self, reference: &'s str, c: &ConnectDecl) -> Result<&'s str> {
        // A flattened subsystem boundary port (e.g. "loop.b" where `loop` is a
        // hierarchical instance now expanded away) resolves via the alias map.
        if let Some(alias) = self.port_alias.get(reference) {
            return Ok(alias);
        }
        if reference.contains('$') {
            // Already a flat synthetic stream (a nested subsystem boundary).
            return Ok(reference);
        }
        // Last dot: instance names may themselves be dotted (`sub.sub`).
        let Some(dot) = reference.rfind('.') else {
            return Ok(reference); // a bare stream name
        };
        let (instance_name, port) = (&reference[..dot], &reference[dot + 1..]);
        match self
            .instance(instance_name)
            .and_then(|ri| ri.stream_of_port(port))
        {
            Some(stream) => Ok(stream),
            None => Err(FreesError::parse(format!(
                "connect(...): '{reference}' is not a port (instance.port) or a stream \
                 name. {}",
                c.source_text
            ))),
        }
    }

    /// Every endpoint of every connect, resolved once.
    fn resolved_connect_streams(&self) -> Result<Vec<Vec<String>>> {
        let mut out = Vec::with_capacity(self.connects.len());
        for c in &self.connects {
            let mut sts = Vec::with_capacity(c.ports.len());
            for reference in &c.ports {
                sts.push(self.stream_of(reference, c)?.to_string());
            }
            out.push(sts);
        }
        Ok(out)
    }

    /// Seeds in↔out links for fluid two-port components.
    ///
    /// Port of `seedComponentLinks`. Only a *fluid* two-port carries its members
    /// port→port (a series pass-through); a heat two-port (conduction) does not
    /// equate its ports — its two ends are at different temperatures — so it must
    /// not seed a loop link.
    ///
    /// `exclude_capacitive` drops the link for a capacitive volume; see the
    /// module docs for why that is what keeps a closed C-R-C-R loop square.
    fn seed_component_links(&self, uf: &mut UnionFind, exclude_capacitive: bool) {
        for ri in &self.instances {
            let streams: Vec<&str> = ri.streams().collect();
            if streams.len() != 2
                || !domains::is_fluid_stream(streams[0], &self.stream_members)
                || !domains::is_fluid_stream(streams[1], &self.stream_members)
            {
                continue;
            }
            if exclude_capacitive && is_pressure_capacitive(ri.def) {
                continue;
            }
            uf.union(streams[0], streams[1]);
        }
    }

    /// Connected streams share a fluid (they share `P` and `h`), so a fluid known
    /// on one endpoint of a `connect` flows to the others — letting derived
    /// properties resolve on the synthetic free-port streams of fluid-less
    /// components (Boiler/Condenser/…) in a connector-style flowsheet.
    ///
    /// Port of `propagateFluidAcrossConnects`. This is also where an unresolvable
    /// `connect` endpoint is first rejected, exactly as in the Java (the
    /// constructor runs it, so the error surfaces before `expand`).
    fn propagate_fluid_across_connects(&mut self) -> Result<()> {
        if self.connects.is_empty() {
            return Ok(());
        }
        let mut uf = UnionFind::new();
        self.seed_component_links(&mut uf, false);
        let per_connect = self.resolved_connect_streams()?;
        for sts in &per_connect {
            for st in &sts[1.min(sts.len())..] {
                uf.union(&sts[0], st);
            }
        }
        // Fluid known anywhere in a connected set → assign it to the whole set.
        let mut root_fluid: OrderedMap<String> = OrderedMap::new();
        for (stream, fluid) in self.stream_fluid.iter() {
            let root = uf.find(stream);
            root_fluid.put_if_absent(&root, fluid.clone());
        }
        let mut assignments: Vec<(String, String)> = Vec::new();
        for sts in &per_connect {
            for stream in sts {
                let root = uf.find(stream);
                if let Some(fluid) = root_fluid.get(&root) {
                    assignments.push((stream.clone(), fluid.clone()));
                }
            }
        }
        for (stream, fluid) in assignments {
            self.stream_fluid.put_if_absent(&stream, fluid);
        }
        Ok(())
    }

    /// Enforces the *never C-C, always C-R-C* index-1 discipline: two
    /// **capacitive** volumes (each fixing a pressure state via `der(port.P)`)
    /// connected directly at one node both assert `P` there — the index-2 trap.
    /// A resistive flow element must sit between them.
    ///
    /// Port of `checkNoCapacitiveCapacitive`. A connect-only union-find gives
    /// true node granularity: the loop-detection union-find of
    /// [`Network::expand_connects`] collapses whole series chains through each
    /// two-port's internal link, so it cannot be used here.
    fn check_no_capacitive_capacitive(&self) -> Result<()> {
        let mut node_uf = UnionFind::new();
        for sts in self.resolved_connect_streams()? {
            for st in &sts[1.min(sts.len())..] {
                node_uf.union(&sts[0], st);
            }
        }
        let mut node_caps: OrderedMap<Vec<String>> = OrderedMap::new();
        for ri in &self.instances {
            if !is_pressure_capacitive(ri.def) {
                continue;
            }
            for stream in ri.streams() {
                if !domains::is_fluid_stream(stream, &self.stream_members) {
                    continue;
                }
                let root = node_uf.find(stream);
                if !node_caps.contains_key(&root) {
                    node_caps.put_if_absent(&root, Vec::new());
                }
                if let Some(caps) = node_caps.get_mut(&root) {
                    if !caps.contains(&ri.inst.name) {
                        caps.push(ri.inst.name.clone());
                    }
                }
            }
        }
        for (_, caps) in node_caps.iter() {
            if caps.len() >= 2 {
                return Err(FreesError::parse(format!(
                    "connect(...): capacitive volumes [{}] are connected directly with no \
                     resistance between them (C-C). Two pressure-storage volumes at one \
                     node make the DAE index-2; interpose a resistive flow element between \
                     them (the C-R-C rule).",
                    caps.join(", ")
                )));
            }
        }
        Ok(())
    }

    /// Emits the node equations for each `connect(...)`.
    ///
    /// Port of `expandConnects`: the domain's across variables equal across all
    /// endpoints (spanning-tree edges only) plus one flow rule. See the module
    /// docs for the union-find loop-closure argument.
    fn expand_connects(
        &self,
        out: &mut Vec<Equation>,
        display_names: &mut BTreeMap<String, String>,
    ) -> Result<()> {
        if self.connects.is_empty() {
            return Ok(());
        }
        self.check_no_capacitive_capacitive()?;
        let mut uf = UnionFind::new();
        // Capacitive volumes break the cycle (they are states, not pass-throughs).
        self.seed_component_links(&mut uf, true);

        for c in &self.connects {
            let refs = &c.ports;
            if refs.len() < 2 {
                return Err(FreesError::parse(format!(
                    "connect(...) needs at least two endpoints: {}",
                    c.source_text
                )));
            }
            let mut sts: Vec<String> = Vec::with_capacity(refs.len());
            for reference in refs {
                sts.push(self.stream_of(reference, c)?.to_string());
            }
            let prefix = format!("CONNECT {}: ", refs.join(", "));
            domains::check_single_domain(&sts, refs, &self.stream_members)?;
            let dom = domains::node_domain(&sts, &self.stream_members);
            if dom == Domain::Fluid {
                domains::check_fluid_connector_type(&sts, refs, &self.connector_types)?;
            }

            // Loop closure: do two endpoints already share a connection set?
            let mut loop_closing = false;
            'outer: for (i, a) in sts.iter().enumerate() {
                for b in &sts[i + 1..] {
                    if uf.connected(a, b) {
                        loop_closing = true;
                        break 'outer;
                    }
                }
            }

            // Across variables — equal across the node, emitted as spanning-tree
            // equalities (cycle-closing edges are redundant).
            let across = domains::across_members_for_node(
                dom,
                &sts,
                &self.stream_members,
                &self.connector_types,
            );
            let root = &sts[0];
            for st in &sts[1..] {
                if !uf.connected(root, st) {
                    for member in &across {
                        out.push(self.equality(root, member, st, &prefix, display_names));
                    }
                    uf.union(root, st);
                }
            }

            // Flow conservation.
            match dom.junction_rule() {
                JunctionRule::Kirchhoff(flow) => {
                    for st in &sts {
                        self.register_member(st, flow, display_names);
                    }
                    out.push(domains::kirchhoff_balance(&sts, flow, &prefix));
                }
                // Causal broadcast: across equality only, no flow.
                JunctionRule::Broadcast => {}
                JunctionRule::FluidMass => {
                    // ṁ passes through (2-way equality / signed Σ at a branch),
                    // skipped when this connect closes a loop — the loop ṁ
                    // balance is then cyclically dependent on the rest of it.
                    if loop_closing {
                        continue;
                    }
                    if sts.len() == 2 {
                        out.push(self.equality(&sts[0], "mdot", &sts[1], &prefix, display_names));
                    } else {
                        for st in &sts {
                            self.register_member(st, "mdot", display_names);
                        }
                        out.push(domains::mass_conservation(
                            &sts,
                            refs,
                            &prefix,
                            &c.source_text,
                        )?);
                    }
                }
            }
        }
        Ok(())
    }

    // ── Body rewriting (port → stream, local → instance$local, params) ───────

    /// Port of `rewriteBody`.
    fn rewrite_body(
        &self,
        e: &Expr,
        ri: &ResolvedInstance<'_>,
        display_names: &mut BTreeMap<String, String>,
    ) -> Result<Expr> {
        Ok(match e {
            Expr::Num { .. } | Expr::Str(_) => e.clone(),
            Expr::Var(name) => self.rewrite_body_var(name, ri, display_names)?,
            Expr::Neg(inner) => Expr::Neg(Box::new(self.rewrite_body(inner, ri, display_names)?)),
            Expr::Not(inner) => Expr::Not(Box::new(self.rewrite_body(inner, ri, display_names)?)),
            Expr::BinOp { op, left, right } => Expr::BinOp {
                op: *op,
                left: Box::new(self.rewrite_body(left, ri, display_names)?),
                right: Box::new(self.rewrite_body(right, ri, display_names)?),
            },
            Expr::Compare { op, left, right } => Expr::Compare {
                op: *op,
                left: Box::new(self.rewrite_body(left, ri, display_names)?),
                right: Box::new(self.rewrite_body(right, ri, display_names)?),
            },
            Expr::Logical { op, left, right } => Expr::Logical {
                op: *op,
                left: Box::new(self.rewrite_body(left, ri, display_names)?),
                right: Box::new(self.rewrite_body(right, ri, display_names)?),
            },
            Expr::Range { start, end } => Expr::Range {
                start: Box::new(self.rewrite_body(start, ri, display_names)?),
                end: Box::new(self.rewrite_body(end, ri, display_names)?),
            },
            Expr::ArrayLiteral(elements) => {
                let mut out = Vec::with_capacity(elements.len());
                for el in elements {
                    out.push(self.rewrite_body(el, ri, display_names)?);
                }
                Expr::ArrayLiteral(out)
            }
            Expr::ArrayAccess { name, indices } => {
                let mut out = Vec::with_capacity(indices.len());
                for i in indices {
                    out.push(self.rewrite_body(i, ri, display_names)?);
                }
                Expr::ArrayAccess {
                    name: namespace_local(name, ri, display_names),
                    indices: out,
                }
            }
            Expr::Call { function, args } => {
                let mut out = Vec::with_capacity(args.len());
                for a in args {
                    out.push(self.rewrite_body(a, ri, display_names)?);
                }
                // A string parameter used as a *function/table name* (`PARAM map$`
                // with a body call `map$(x)`) bakes to the parameter's value, so a
                // map-driven component resolves to the globally-declared
                // TABLE/FUNCTION of that name. Ordinary function names are never
                // string-parameter keys, so this never collides.
                match ri.string(function) {
                    Some(table) => Expr::call(table, out),
                    None => Expr::call(bake_fluid(function, &ri.string_params), out),
                }
            }
        })
    }

    /// Port of `rewriteBodyVar`.
    fn rewrite_body_var(
        &self,
        name: &str,
        ri: &ResolvedInstance<'_>,
        display_names: &mut BTreeMap<String, String>,
    ) -> Result<Expr> {
        if name.contains('.') {
            let mut segs = name.split('.');
            let port = segs.next().unwrap_or_default();
            let Some(stream) = ri.stream_of_port(port) else {
                return Err(FreesError::parse(format!(
                    "Component '{}': '{name}' references unknown port '{port}'. Ports: {}.",
                    ri.def.name,
                    ri.def.ports.join(", ")
                )));
            };
            // `"in."` splits to one segment in Java (trailing empties dropped),
            // so an empty member is the same "needs a member" error.
            let Some(member) = segs.next().filter(|m| !m.is_empty()) else {
                return Err(FreesError::parse(format!(
                    "Component '{}': port reference '{name}' needs a member (e.g. {port}.P).",
                    ri.def.name
                )));
            };
            return Ok(self.stream_member(stream, member, display_names));
        }
        if let Some(value) = ri.numeric(name) {
            return Ok(value.clone());
        }
        // A string parameter used as a *fluid* argument is already baked into the
        // encoded property-call name (it never reaches here as a bare Var).
        // Anywhere else — an arrangement string `hx_effectiveness(arr$, …)` — it
        // substitutes to its literal value.
        if let Some(value) = ri.string(name) {
            return Ok(Expr::Str(value.to_string()));
        }
        // Reserved global: `time` in a component body is the simulation time,
        // never a per-instance local. This is what lets time-driven source blocks
        // (Step/Ramp/Sine/drive cycles) exist as library components at all.
        if name == "time" {
            return Ok(Expr::Var("time".to_string()));
        }
        Ok(Expr::Var(namespace_local(name, ri, display_names)))
    }

    // ── Top-level rewriting ─────────────────────────────────────────────────

    /// Port of `rewriteTop`.
    fn rewrite_top(&self, e: &Expr, display_names: &mut BTreeMap<String, String>) -> Result<Expr> {
        Ok(match e {
            Expr::Num { .. } | Expr::Str(_) => e.clone(),
            Expr::Var(name) => self.rewrite_top_var(name, display_names)?,
            Expr::Neg(inner) => Expr::Neg(Box::new(self.rewrite_top(inner, display_names)?)),
            Expr::Not(inner) => Expr::Not(Box::new(self.rewrite_top(inner, display_names)?)),
            Expr::BinOp { op, left, right } => Expr::BinOp {
                op: *op,
                left: Box::new(self.rewrite_top(left, display_names)?),
                right: Box::new(self.rewrite_top(right, display_names)?),
            },
            Expr::Compare { op, left, right } => Expr::Compare {
                op: *op,
                left: Box::new(self.rewrite_top(left, display_names)?),
                right: Box::new(self.rewrite_top(right, display_names)?),
            },
            Expr::Logical { op, left, right } => Expr::Logical {
                op: *op,
                left: Box::new(self.rewrite_top(left, display_names)?),
                right: Box::new(self.rewrite_top(right, display_names)?),
            },
            Expr::Range { start, end } => Expr::Range {
                start: Box::new(self.rewrite_top(start, display_names)?),
                end: Box::new(self.rewrite_top(end, display_names)?),
            },
            Expr::ArrayLiteral(elements) => {
                let mut out = Vec::with_capacity(elements.len());
                for el in elements {
                    out.push(self.rewrite_top(el, display_names)?);
                }
                Expr::ArrayLiteral(out)
            }
            Expr::ArrayAccess { name, indices } => {
                let mut out = Vec::with_capacity(indices.len());
                for i in indices {
                    out.push(self.rewrite_top(i, display_names)?);
                }
                // The Java leaves the array's base name alone here.
                Expr::ArrayAccess {
                    name: name.clone(),
                    indices: out,
                }
            }
            Expr::Call { function, args } => {
                let mut out = Vec::with_capacity(args.len());
                for a in args {
                    out.push(self.rewrite_top(a, display_names)?);
                }
                Expr::call(function, out)
            }
        })
    }

    /// Port of `rewriteTopVar`: `instance.port.member`, `instance.output` and
    /// `stream.member` all land on the same flat names the bodies produced.
    fn rewrite_top_var(
        &self,
        name: &str,
        display_names: &mut BTreeMap<String, String>,
    ) -> Result<Expr> {
        if !name.contains('.') {
            return Ok(Expr::Var(name.to_string()));
        }
        let segs: Vec<&str> = name.split('.').collect();
        if let Some(ri) = self.instance(segs[0]) {
            if segs.len() >= 2 {
                if let Some(stream) = ri.stream_of_port(segs[1]) {
                    if segs.len() < 3 {
                        return Err(FreesError::parse(format!(
                            "Reference '{name}' to port '{}' of component '{}' needs a \
                             member (e.g. {name}.P).",
                            segs[1], segs[0]
                        )));
                    }
                    return Ok(self.top_stream_member(stream, segs[2], display_names));
                }
            }
            // instance output / local: inst.output
            if segs.len() != 2 {
                return Err(FreesError::parse(format!(
                    "Reference '{name}' to component '{}' is not a port member or named \
                     output.",
                    segs[0]
                )));
            }
            let flat = format!("{}${}", segs[0], segs[1]);
            if !display_names.contains_key(&flat) {
                display_names.insert(flat.clone(), format!("{}.{}", segs[0], segs[1]));
            }
            return Ok(Expr::Var(flat));
        }
        // A stream member: stream.member
        if segs.len() == 2 {
            return Ok(self.top_stream_member(segs[0], segs[1], display_names));
        }
        let flat = segs.join("$");
        if !display_names.contains_key(&flat) {
            display_names.insert(flat.clone(), segs.join("."));
        }
        Ok(Expr::Var(flat))
    }

    /// Resolves a top-level `stream.member`.
    ///
    /// Port of `topStreamMember`. On a stream that has a fluid, the canonical
    /// members (`P`, `h`, `mdot`) stay flat solver variables while a derived
    /// state property (`.T`, `.s`, `.x`, `.v`, `.rho`, `.cp`, …) is rewritten to
    /// the matching property call on the stream's `(P, h)` — so the user can
    /// write `s3.T = 753 [K]` and let the solver invert it for the enthalpy. On
    /// a fluid-less stream *every* member is an opaque rider variable, so a name
    /// like `.x` is never mistaken for thermodynamic quality.
    fn top_stream_member(
        &self,
        stream: &str,
        member: &str,
        display_names: &mut BTreeMap<String, String>,
    ) -> Expr {
        if let (Some(prop), Some(fluid)) = (derived_prop(member), self.stream_fluid.get(stream)) {
            let p = self.stream_member(stream, "p", display_names);
            let h = self.stream_member(stream, "h", display_names);
            return Expr::call(
                format!("prop${prop}${}$p$h", fluid.to_ascii_lowercase()),
                vec![p, h],
            );
        }
        self.stream_member(stream, member, display_names)
    }

    // ── Schematic payload ───────────────────────────────────────────────────

    /// `connect(...)` declarations, endpoints kept as written. Port of
    /// `explicitConnections`.
    fn explicit_connections(&self) -> Vec<Connection> {
        let mut out = Vec::new();
        for c in &self.connects {
            // The streams the endpoints name, skipping unresolvable ones. Fewer
            // than two means the expansion already rejected this declaration with
            // a real error; there is nothing to draw.
            let streams: Vec<String> = c
                .ports
                .iter()
                .filter_map(|r| self.stream_of(r, c).ok().map(str::to_string))
                .collect();
            if streams.len() < 2 {
                continue;
            }
            // Per endpoint, the display prefix its member variables use — aligned
            // with `c.ports` so index i describes endpoint i. An endpoint the
            // expansion could not resolve keeps its written ref, which is the best
            // available guess and never a wrong lookup key.
            let endpoint_streams: Vec<String> = c
                .ports
                .iter()
                .map(|r| match self.stream_of(r, c) {
                    Ok(stream) => self.display_stream(stream).to_string(),
                    Err(_) => r.clone(),
                })
                .collect();
            out.push(self.connection(&streams, c.ports.clone(), endpoint_streams));
        }
        out
    }

    /// Ports of different instances naming one stream — the terse connection
    /// style — reported as the `instance.port` pairs they join. Port of
    /// `sharedStreamJunctions`.
    fn shared_stream_junctions(&self) -> Vec<Connection> {
        let mut by_stream: OrderedMap<Vec<String>> = OrderedMap::new();
        for ri in &self.instances {
            for (port, stream) in &ri.port_to_stream {
                if !by_stream.contains_key(stream) {
                    by_stream.put_if_absent(stream, Vec::new());
                }
                if let Some(list) = by_stream.get_mut(stream) {
                    list.push(format!("{}.{port}", ri.inst.name));
                }
            }
        }
        let mut out = Vec::new();
        for (stream, endpoints) in by_stream.iter() {
            if endpoints.len() < 2 {
                continue;
            }
            // One shared stream names every endpoint's variables, so the display
            // prefix repeats across the node.
            let display = self.display_stream(stream).to_string();
            let streams = vec![stream.to_string()];
            let repeated = vec![display; endpoints.len()];
            out.push(self.connection(&streams, endpoints.clone(), repeated));
        }
        out
    }

    fn connection(
        &self,
        streams: &[String],
        endpoints: Vec<String>,
        endpoint_streams: Vec<String>,
    ) -> Connection {
        let domain = domains::node_domain(streams, &self.stream_members);
        // The fluid connector type of a fluid node; `None` outside the fluid
        // domain, where the concept does not apply. This is what separates a
        // coolant line from a refrigerant line — both are `domain = fluid`.
        let connector = (domain == Domain::Fluid)
            .then(|| domains::node_fluid_type(streams, &self.connector_types).to_string());
        // The working fluid a node carries, from its first stream that has one.
        // `build_stream_fluid_map` tags *every* port of a fluid-bearing component
        // — a wall port included, since it cannot know which ports are thermal —
        // so a heat node between a coolant HX and a thermal mass would otherwise
        // report itself as carrying the coolant. It does not: it carries heat.
        let fluid = (domain == Domain::Fluid)
            .then(|| {
                streams
                    .iter()
                    .find_map(|st| self.stream_fluid.get(st).cloned())
            })
            .flatten();
        Connection {
            domain,
            endpoints,
            connector,
            fluid,
            streams: endpoint_streams,
        }
    }
}

/// Port of `namespaceLocal`: a bare local/output name becomes
/// `<instance>$<name>`, exactly like a `MODULE`'s per-instance namespacing.
fn namespace_local(
    name: &str,
    ri: &ResolvedInstance<'_>,
    display_names: &mut BTreeMap<String, String>,
) -> String {
    if name.contains('.') || ri.numeric(name).is_some() || ri.string(name).is_some() {
        // Dotted names are handled elsewhere; parameters are substituted elsewhere.
        return name.to_string();
    }
    let flat = format!("{}${name}", ri.inst.name);
    if !display_names.contains_key(&flat) {
        display_names.insert(flat.clone(), format!("{}.{name}", ri.inst.name));
    }
    flat
}

/// Bakes a string (fluid) parameter into an encoded property-call function name.
///
/// Port of `bakeFluid`. The expression parser encodes
/// `Enthalpy(fluid$, P=.., h=..)` as `prop$enthalpy$fluid$$p$h` — the
/// parameter's trailing `$` yields an empty segment after it. If the fluid
/// segment matches one of this instance's string parameters, rebuild with the
/// concrete value; otherwise leave it for the global string-variable pass (a
/// document-level `R$`).
fn bake_fluid(function: &str, string_params: &[(String, String)]) -> String {
    if !function.starts_with("prop$") {
        return function.to_string();
    }
    // Rust's `split` keeps trailing empties, matching Java's `split("\\$", -1)`.
    let parts: Vec<&str> = function.split('$').collect();
    if parts.len() < 4 || !parts[3].is_empty() {
        return function.to_string();
    }
    let fluid_var = format!("{}$", parts[2]);
    let Some((_, value)) = string_params.iter().find(|(k, _)| *k == fluid_var) else {
        return function.to_string();
    };
    let mut rebuilt = format!("prop${}${}", parts[1], value.to_ascii_lowercase());
    for part in &parts[4..] {
        rebuilt.push('$');
        rebuilt.push_str(part);
    }
    rebuilt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::def::{Param, Variant};
    use crate::lexer::tokenize;
    use crate::parser::{parse_expr, Cursor};

    // ── Building documents by hand ───────────────────────────────────────────
    //
    // The front end does not parse `COMPONENT` blocks yet, so these tests build
    // the AST directly — but every *expression* goes through the real parser, so
    // precedence, member accessors and property-call encoding are the shipping
    // ones and not a test-only approximation.

    fn expr(src: &str) -> Expr {
        let tokens = tokenize(src).unwrap_or_else(|e| panic!("lex {src:?}: {e}"));
        let mut cursor = Cursor::new(&tokens, src);
        parse_expr(&mut cursor).unwrap_or_else(|e| panic!("parse {src:?}: {e}"))
    }

    /// Splits on the top-level `=` (never one that is part of `<=`/`>=`/`<>`).
    fn eq(src: &str) -> Equation {
        let bytes = src.as_bytes();
        let split = (0..bytes.len())
            .find(|&i| {
                bytes[i] == b'='
                    && !matches!(bytes.get(i.wrapping_sub(1)), Some(b'<' | b'>' | b'='))
                    && !matches!(bytes.get(i + 1), Some(b'=' | b'>'))
            })
            .unwrap_or_else(|| panic!("not an equation: {src:?}"));
        Equation::new(
            expr(src[..split].trim()),
            expr(src[split + 1..].trim()),
            src.replace(' ', ""), // ANTLR's getText() drops the whitespace
        )
    }

    /// `COMPONENT name(ports) PARAM … <body> END`, with `param` defaults given as
    /// expression source (`None` for a bare `PARAM k`).
    fn comp(
        name: &str,
        ports: &[&str],
        params: &[(&str, Option<&str>)],
        body: &[&str],
    ) -> ComponentDef {
        comp_full(name, ports, params, body, vec![], vec![], vec![])
    }

    fn comp_full(
        name: &str,
        ports: &[&str],
        params: &[(&str, Option<&str>)],
        body: &[&str],
        variants: Vec<Variant>,
        subs: Vec<ComponentInst>,
        sub_connects: Vec<ConnectDecl>,
    ) -> ComponentDef {
        ComponentDef::new(
            name.to_ascii_lowercase(),
            ports.iter().map(|p| p.to_ascii_lowercase()).collect(),
            params
                .iter()
                .map(|(n, d)| Param::new(n.to_ascii_lowercase(), d.map(expr)))
                .collect(),
            body.iter().map(|b| eq(b)).collect(),
            variants,
            subs,
            sub_connects,
        )
    }

    fn variant(name: &str, require: &[&str], body: &[&str]) -> Variant {
        Variant {
            name: name.to_ascii_lowercase(),
            require: require.iter().map(|r| r.to_ascii_lowercase()).collect(),
            body: body.iter().map(|b| eq(b)).collect(),
        }
    }

    /// `Type Name(port, port, param=value, …)`.
    fn inst(type_name: &str, name: &str, ports: &[&str], params: &[(&str, &str)]) -> ComponentInst {
        let mut overrides = ParamOverrides::new();
        for (k, v) in params {
            overrides.put(k.to_ascii_lowercase(), expr(v));
        }
        ComponentInst {
            type_name: type_name.to_ascii_lowercase(),
            name: name.to_ascii_lowercase(),
            port_args: ports.iter().map(|p| p.to_ascii_lowercase()).collect(),
            params: overrides,
            source_text: format!("{type_name} {name}(…)"),
        }
    }

    fn conn(refs: &[&str]) -> ConnectDecl {
        ConnectDecl {
            ports: refs.iter().map(|r| r.to_ascii_lowercase()).collect(),
            source_text: format!("connect({})", refs.join(",")),
        }
    }

    // ── Rendering, so assertions read like the equations they check ──────────

    fn num(v: f64) -> String {
        if v.fract() == 0.0 && v.abs() < 1e15 {
            format!("{}", v as i64)
        } else {
            format!("{v}")
        }
    }

    fn show(e: &Expr) -> String {
        match e {
            Expr::Num { value, .. } => num(*value),
            Expr::Str(s) => format!("'{s}'"),
            Expr::Var(n) => n.clone(),
            Expr::Neg(i) => format!("-{}", show(i)),
            Expr::Not(i) => format!("not {}", show(i)),
            Expr::BinOp { op, left, right } => {
                format!("({} {} {})", show(left), op.as_str(), show(right))
            }
            Expr::Compare { op, left, right } => {
                format!("({} {} {})", show(left), op.as_str(), show(right))
            }
            Expr::Logical { op, left, right } => {
                format!("({} {} {})", show(left), op.as_str(), show(right))
            }
            Expr::Range { start, end } => format!("{}:{}", show(start), show(end)),
            Expr::ArrayLiteral(els) => {
                format!("[{}]", els.iter().map(show).collect::<Vec<_>>().join(", "))
            }
            Expr::ArrayAccess { name, indices } => format!(
                "{name}[{}]",
                indices.iter().map(show).collect::<Vec<_>>().join(", ")
            ),
            Expr::Call { function, args } => format!(
                "{function}({})",
                args.iter().map(show).collect::<Vec<_>>().join(", ")
            ),
        }
    }

    fn show_eq(e: &Equation) -> String {
        format!("{} = {}", show(&e.lhs), show(&e.rhs))
    }

    // ── The harness ─────────────────────────────────────────────────────────

    struct Expanded {
        equations: Vec<Equation>,
        statements: Vec<Statement>,
        display: BTreeMap<String, String>,
        units: BTreeMap<String, &'static str>,
        connections: Vec<Connection>,
        initials: Vec<ComponentInitial>,
        has_storage: bool,
        fluids: BTreeMap<String, String>,
    }

    impl Expanded {
        /// The fluid inferred for a stream, if any.
        fn stream_fluid_of(&self, stream: &str) -> Option<&str> {
            self.fluids.get(stream).map(String::as_str)
        }

        /// Every expanded equation, rendered.
        fn eqs(&self) -> Vec<String> {
            self.equations.iter().map(show_eq).collect()
        }

        /// Every rewritten top-level statement, rendered.
        fn tops(&self) -> Vec<String> {
            self.statements
                .iter()
                .map(|s| match s {
                    Statement::Eq(e) => show_eq(e),
                    other => format!("{other:?}"),
                })
                .collect()
        }

        fn source_texts(&self) -> Vec<&str> {
            self.equations
                .iter()
                .map(|e| e.source_text.as_str())
                .collect()
        }

        fn display_of(&self, flat: &str) -> Option<&str> {
            self.display.get(flat).map(String::as_str)
        }
    }

    fn expand(
        defs: &[ComponentDef],
        insts: &[ComponentInst],
        connects: &[ConnectDecl],
        top: &[&str],
    ) -> Result<Expanded> {
        let builtins: Vec<ComponentDef> = Vec::new();
        let mut display: BTreeMap<String, String> = BTreeMap::new();
        let statements: Vec<Statement> = top.iter().map(|s| Statement::Eq(eq(s))).collect();
        let (equations, statements, units, connections, initials, has_storage, fluids) = {
            let mut ex = ComponentExpander::new(&builtins, defs, insts, connects, &mut display)?;
            let equations = ex.expand()?;
            let statements = ex.rewrite_statements(statements)?;
            let fluids = ex
                .stream_fluids()
                .map(|(s, f)| (s.to_string(), f.to_string()))
                .collect();
            (
                equations,
                statements,
                ex.member_units(),
                ex.connections(),
                ex.component_initials().to_vec(),
                ex.has_storage(),
                fluids,
            )
        };
        Ok(Expanded {
            equations,
            statements,
            display,
            units,
            connections,
            initials,
            has_storage,
            fluids,
        })
    }

    fn expand_ok(
        defs: &[ComponentDef],
        insts: &[ComponentInst],
        connects: &[ConnectDecl],
        top: &[&str],
    ) -> Expanded {
        expand(defs, insts, connects, top).unwrap_or_else(|e| panic!("expected success: {e}"))
    }

    fn expand_err(
        defs: &[ComponentDef],
        insts: &[ComponentInst],
        connects: &[ConnectDecl],
        top: &[&str],
    ) -> String {
        match expand(defs, insts, connects, top) {
            Err(FreesError::Parse { message, .. }) => message,
            Err(other) => panic!("expected a parse error, got {other:?}"),
            Ok(_) => panic!("expected a parse error, but the document expanded"),
        }
    }

    // ── Shared fixtures, mirroring the oracle documents ─────────────────────

    fn pipe() -> ComponentDef {
        comp(
            "Pipe",
            &["in", "out"],
            &[("k", Some("2"))],
            &[
                "out.mdot = in.mdot",
                "out.P = in.P - k * in.mdot",
                "out.h = in.h",
                "dP = in.P - out.P",
            ],
        )
    }

    fn res() -> ComponentDef {
        comp(
            "Res",
            &["in", "out"],
            &[("k", Some("2"))],
            &[
                "out.mdot = in.mdot",
                "out.P = in.P - k * in.mdot",
                "out.h = in.h",
            ],
        )
    }

    fn src() -> ComponentDef {
        comp(
            "Src",
            &["out"],
            &[("p0", Some("400")), ("h0", Some("50")), ("m0", Some("2"))],
            &["out.P = p0", "out.h = h0", "out.mdot = m0"],
        )
    }

    fn snk() -> ComponentDef {
        comp(
            "Snk",
            &["in"],
            &[("c", Some("0"))],
            &["W = in.mdot * in.h + c"],
        )
    }

    /// The boundary pair the hierarchy documents in the oracle corpus use
    /// (`f3_hier` / `k2_hier_shared` / `k3_nested`): P = 400, h = 5, ṁ = 2.
    fn src_h5() -> ComponentDef {
        comp(
            "Src",
            &["out"],
            &[],
            &["out.P = 400", "out.h = 5", "out.mdot = 2"],
        )
    }

    /// `f3_hier.frees`'s sink.
    fn snk_f3() -> ComponentDef {
        comp("Snk", &["in"], &[], &["W = in.mdot * in.h + in.P"])
    }

    /// `k2_hier_shared.frees` / `k3_nested.frees`'s sink.
    fn snk_sum() -> ComponentDef {
        comp("Snk", &["in"], &[], &["W = in.mdot + in.P + in.h"])
    }

    // =====================================================================
    //  1. The shared-stream chain — oracle: corpus/c1_chain.frees
    // =====================================================================

    #[test]
    fn a_two_port_chain_flattens_to_stream_scalars() {
        let out = expand_ok(
            &[pipe()],
            &[
                inst("Pipe", "A", &["s1", "s2"], &[]),
                inst("Pipe", "B", &["s2", "s3"], &[("k", "5")]),
            ],
            &[],
            &["s1.P = 500", "s1.mdot = 3", "s1.h = 100"],
        );
        assert_eq!(
            out.eqs(),
            vec![
                "s2$mdot = s1$mdot",
                "s2$p = (s1$p - (2 * s1$mdot))",
                "s2$h = s1$h",
                "a$dp = (s1$p - s2$p)",
                "s3$mdot = s2$mdot",
                "s3$p = (s2$p - (5 * s2$mdot))",
                "s3$h = s2$h",
                "b$dp = (s2$p - s3$p)",
            ]
        );
        // The terse style needs *no* junction equations: the two instances write
        // the same `s2$…` scalars, so mass and energy are conserved by naming.
        assert_eq!(out.equations.len(), 8);
        // Top-level `s1.P = 500` lands on the same flat name the bodies produced.
        assert_eq!(out.tops(), vec!["s1$p = 500", "s1$mdot = 3", "s1$h = 100"]);
    }

    #[test]
    fn stream_members_and_locals_get_their_user_visible_display_names() {
        // Pinned against the Java oracle's `display_names` for c1_chain.frees.
        let out = expand_ok(
            &[pipe()],
            &[
                inst("Pipe", "A", &["s1", "s2"], &[]),
                inst("Pipe", "B", &["s2", "s3"], &[("k", "5")]),
            ],
            &[],
            &[],
        );
        for (flat, shown) in [
            ("s1$p", "s1.p"),
            ("s1$h", "s1.h"),
            ("s1$mdot", "s1.mdot"),
            ("s2$p", "s2.p"),
            ("s3$mdot", "s3.mdot"),
            ("a$dp", "a.dp"),
            ("b$dp", "b.dp"),
        ] {
            assert_eq!(out.display_of(flat), Some(shown), "display of {flat}");
        }
    }

    #[test]
    fn each_expanded_equation_quotes_its_component_and_instance() {
        let out = expand_ok(
            &[pipe()],
            &[inst("Pipe", "A", &["s1", "s2"], &[])],
            &[],
            &[],
        );
        assert_eq!(
            out.source_texts(),
            vec![
                "COMPONENT pipe a: out.mdot=in.mdot",
                "COMPONENT pipe a: out.P=in.P-k*in.mdot",
                "COMPONENT pipe a: out.h=in.h",
                "COMPONENT pipe a: dP=in.P-out.P",
            ]
        );
    }

    // =====================================================================
    //  2. connect(...) with free ports — oracle: corpus/c2_connect.frees
    // =====================================================================

    #[test]
    fn free_ports_are_wired_by_connect_into_synthetic_streams() {
        let out = expand_ok(
            &[src(), res(), snk()],
            &[
                inst("Src", "S", &[], &[]),
                inst("Res", "L", &[], &[]),
                inst("Snk", "K", &[], &[]),
            ],
            &[conn(&["S.out", "L.in"]), conn(&["L.out", "K.in"])],
            &[],
        );
        assert_eq!(
            out.eqs(),
            vec![
                "s$out$p = 400",
                "s$out$h = 50",
                "s$out$mdot = 2",
                "l$out$mdot = l$in$mdot",
                "l$out$p = (l$in$p - (2 * l$in$mdot))",
                "l$out$h = l$in$h",
                "k$w = ((k$in$mdot * k$in$h) + 0)",
                // node 1: across (P, h) then the two-way mass pass-through
                "s$out$p = l$in$p",
                "s$out$h = l$in$h",
                "s$out$mdot = l$in$mdot",
                // node 2
                "l$out$p = k$in$p",
                "l$out$h = k$in$h",
                "l$out$mdot = k$in$mdot",
            ]
        );
        // A free port displays as `inst.port`, not as the mangled stream.
        assert_eq!(out.display_of("s$out$p"), Some("s.out.p"));
        assert_eq!(out.display_of("k$in$mdot"), Some("k.in.mdot"));
        assert_eq!(out.display_of("k$w"), Some("k.w"));
    }

    #[test]
    fn a_connect_equality_quotes_the_written_endpoints_not_the_mangled_streams() {
        let out = expand_ok(
            &[src(), snk()],
            &[inst("Src", "S", &[], &[]), inst("Snk", "K", &[], &[])],
            &[conn(&["S.out", "K.in"])],
            &[],
        );
        let junction: Vec<&str> = out
            .source_texts()
            .into_iter()
            .filter(|t| t.starts_with("CONNECT"))
            .collect();
        assert_eq!(
            junction,
            vec![
                "CONNECT s.out, k.in: s$out.p = k$in.p",
                "CONNECT s.out, k.in: s$out.h = k$in.h",
                "CONNECT s.out, k.in: s$out.mdot = k$in.mdot",
            ]
        );
    }

    // =====================================================================
    //  3. Native branching — oracle: corpus/c3_branch.frees
    // =====================================================================

    #[test]
    fn a_three_way_fluid_node_emits_a_signed_mass_balance() {
        let src3 = comp(
            "Src",
            &["out"],
            &[("p0", Some("400")), ("h0", Some("50")), ("m0", Some("6"))],
            &["out.P = p0", "out.h = h0", "out.mdot = m0"],
        );
        let snk3 = comp(
            "Snk",
            &["in"],
            &[("pset", Some("100"))],
            &["in.P = pset", "W = in.mdot * in.h"],
        );
        let out = expand_ok(
            &[src3, res(), snk3],
            &[
                inst("Src", "S", &[], &[]),
                inst("Res", "R1", &[], &[]),
                inst("Res", "R2", &[], &[("k", "4")]),
                inst("Snk", "K1", &[], &[]),
                inst("Snk", "K2", &[], &[("pset", "120")]),
            ],
            &[
                conn(&["S.out", "R1.in", "R2.in"]),
                conn(&["R1.out", "K1.in"]),
                conn(&["R2.out", "K2.in"]),
            ],
            &[],
        );
        let node: Vec<String> = out
            .equations
            .iter()
            .zip(out.eqs())
            .filter(|(e, _)| e.source_text.starts_with("CONNECT s.out,"))
            .map(|(_, shown)| shown)
            .collect();
        assert_eq!(
            node.iter().map(String::as_str).collect::<Vec<_>>(),
            vec![
                "s$out$p = r1$in$p",
                "s$out$h = r1$in$h",
                "s$out$p = r2$in$p",
                "s$out$h = r2$in$h",
                // Σ(outlet ṁ) = Σ(inlet ṁ) — direction read off the port names.
                "s$out$mdot = (r1$in$mdot + r2$in$mdot)",
            ]
        );
        assert!(out
            .source_texts()
            .contains(&"CONNECT s.out, r1.in, r2.in: sum(mdot_out) = sum(mdot_in)"));
        // The Java oracle counts 24 equations against 23 variables here.
        assert_eq!(out.equations.len(), 24);
    }

    #[test]
    fn a_branch_whose_direction_cannot_be_read_is_rejected_by_endpoint_name() {
        // Oracle: corpus/e15_mass_dir_unknown.frees
        let s = comp("Src", &["a"], &[], &["a.P = 100", "a.h = 5", "a.mdot = 1"]);
        let k = comp("Snk", &["b"], &[], &["W = b.mdot * b.h + b.P"]);
        let msg = expand_err(
            &[s, k],
            &[
                inst("Src", "S", &[], &[]),
                inst("Snk", "K1", &[], &[]),
                inst("Snk", "K2", &[], &[]),
            ],
            &[conn(&["S.a", "K1.b", "K2.b"])],
            &[],
        );
        assert_eq!(
            msg,
            "connect(...): cannot tell whether 's.a' is an inlet or an outlet for the \
             mass balance — name the port with 'in'/'out', or split the flow with a \
             Splitter/Mixer component. connect(S.a,K1.b,K2.b)"
        );
    }

    // =====================================================================
    //  4. Loop closure — oracle: corpus/c5_loop.frees
    // =====================================================================

    #[test]
    fn a_loop_closing_connect_emits_nothing_at_all() {
        let pump = comp(
            "Pump",
            &["in", "out"],
            &[("dp", Some("100"))],
            &["out.mdot = in.mdot", "out.P = in.P + dp", "out.h = in.h"],
        );
        let out = expand_ok(
            &[pump, res()],
            &[inst("Pump", "P", &[], &[]), inst("Res", "R", &[], &[])],
            &[conn(&["P.out", "R.in"]), conn(&["R.out", "P.in"])],
            &["P.in.P = 200", "P.in.h = 50", "P.in.mdot = 4"],
        );
        // Six body equations, three from the first node, ZERO from the second:
        // both endpoints are already connected through the two pass-throughs, so
        // the across equalities are redundant and the loop Σṁ is cyclically
        // dependent. The Java oracle solves exactly 12 variables here.
        assert_eq!(
            out.eqs(),
            vec![
                "p$out$mdot = p$in$mdot",
                "p$out$p = (p$in$p + 100)",
                "p$out$h = p$in$h",
                "r$out$mdot = r$in$mdot",
                "r$out$p = (r$in$p - (2 * r$in$mdot))",
                "r$out$h = r$in$h",
                "p$out$p = r$in$p",
                "p$out$h = r$in$h",
                "p$out$mdot = r$in$mdot",
            ]
        );
    }

    #[test]
    fn a_capacitive_volume_does_not_seed_the_loop_and_keeps_the_cycle_square() {
        // A C-R-C-R ring: without the capacitive exclusion the closing connect
        // would look redundant and its equations would vanish, leaving the ring
        // non-square. Each volume carries `der(a.P)`, so it is *not* a
        // pass-through for loop detection.
        let vol = comp(
            "Vol",
            &["a", "b"],
            &[("V", Some("1"))],
            &[
                "der(a.P) = (a.mdot + b.mdot) / V",
                "b.P = a.P",
                "b.h = a.h",
                "b.mdot = -a.mdot",
            ],
        );
        let out = expand_ok(
            &[vol, res()],
            &[
                inst("Vol", "C1", &[], &[]),
                inst("Res", "R1", &[], &[]),
                inst("Vol", "C2", &[], &[]),
                inst("Res", "R2", &[], &[]),
            ],
            &[
                conn(&["C1.b", "R1.in"]),
                conn(&["R1.out", "C2.a"]),
                conn(&["C2.b", "R2.in"]),
                conn(&["R2.out", "C1.a"]),
            ],
            &[],
        );
        // Every one of the four nodes emits its full (P, h, ṁ) triple: 4 × 3.
        let junctions = out
            .source_texts()
            .into_iter()
            .filter(|t| t.starts_with("CONNECT"))
            .count();
        assert_eq!(junctions, 12);
        assert!(out.has_storage);
    }

    // =====================================================================
    //  5. Physics variants — oracle: corpus/c4_variant.frees
    // =====================================================================

    #[test]
    fn the_model_selector_picks_a_body_and_shares_the_equations_outside_it() {
        let c = comp_full(
            "Comp",
            &["in", "out"],
            &[("model$", Some("simple"))],
            &["out.mdot = in.mdot"],
            vec![
                variant(
                    "simple",
                    &["ratio"],
                    &["out.P = in.P * ratio", "out.h = in.h * 1.1"],
                ),
                variant(
                    "detailed",
                    &["ratio", "eta"],
                    &[
                        "out.P = in.P * ratio",
                        "out.h = in.h + (in.h * (ratio - 1)) / eta",
                    ],
                ),
            ],
            vec![],
            vec![],
        );
        let out = expand_ok(
            &[c],
            &[
                inst("Comp", "C1", &["s1", "s2"], &[("ratio", "3")]),
                inst(
                    "Comp",
                    "C2",
                    &["s2", "s3"],
                    &[("model$", "detailed"), ("ratio", "2"), ("eta", "0.8")],
                ),
            ],
            &[],
            &[],
        );
        assert_eq!(
            out.eqs(),
            vec![
                // shared, then the selected variant's — C1 is `simple`…
                "s2$mdot = s1$mdot",
                "s2$p = (s1$p * 3)",
                "s2$h = (s1$h * 1.1)",
                // …and C2 is `detailed`, with `eta` supplied only by C2.
                "s3$mdot = s2$mdot",
                "s3$p = (s2$p * 2)",
                "s3$h = (s2$h + ((s2$h * (2 - 1)) / 0.8))",
            ]
        );
    }

    #[test]
    fn a_parameter_only_an_unselected_variant_needs_may_be_omitted() {
        // C1 above supplies no `eta`, which only `detailed` requires. That is the
        // whole point of per-variant REQUIRE, so it is pinned on its own.
        let c = comp_full(
            "Comp",
            &["in", "out"],
            &[("model$", Some("simple"))],
            &["out.mdot = in.mdot"],
            vec![
                variant("simple", &["ratio"], &["out.P = in.P * ratio"]),
                variant(
                    "detailed",
                    &["ratio", "eta"],
                    &["out.P = in.P * ratio / eta"],
                ),
            ],
            vec![],
            vec![],
        );
        expand_ok(
            &[c],
            &[inst("Comp", "C1", &["s1", "s2"], &[("ratio", "3")])],
            &[],
            &[],
        );
    }

    // =====================================================================
    //  6. Heat / electrical / mechanical / translational / signal nodes
    //     oracle: corpus/c6_heat_elec.frees, f6_signal, f7_mech, g6_transl
    // =====================================================================

    #[test]
    fn a_heat_node_emits_a_kirchhoff_balance_and_a_temperature_equality() {
        let tsrc = comp("TSrc", &["p"], &[("tset", Some("400"))], &["p.T = tset"]);
        let cond = comp(
            "Cond",
            &["a", "b"],
            &[("UA", Some("10"))],
            &["a.Qdot = UA * (a.T - b.T)", "b.Qdot = -a.Qdot"],
        );
        let out = expand_ok(
            &[tsrc, cond],
            &[inst("TSrc", "HOT", &[], &[]), inst("Cond", "W", &[], &[])],
            &[conn(&["HOT.p", "W.a"])],
            &[],
        );
        assert_eq!(
            out.eqs(),
            vec![
                "hot$p$t = 400",
                "w$a$qdot = (10 * (w$a$t - w$b$t))",
                "w$b$qdot = -w$a$qdot",
                "hot$p$t = w$a$t",
                "(hot$p$qdot + w$a$qdot) = 0",
            ]
        );
        assert!(out
            .source_texts()
            .contains(&"CONNECT hot.p, w.a: sum(qdot) = 0"));
        // A heat two-port is NOT a pass-through — its ends sit at different
        // temperatures — so it never seeds the loop union-find.
        assert_eq!(out.units.get("hot$p$t"), Some(&"K"));
        assert_eq!(out.units.get("w$a$qdot"), Some(&"W"));
    }

    #[test]
    fn an_electrical_node_sums_currents_and_equates_potentials() {
        let vsrc = comp(
            "VSrc",
            &["p", "n"],
            &[("vset", Some("12"))],
            &["p.V - n.V = vset", "p.I + n.I = 0"],
        );
        let rst = comp(
            "Rst",
            &["p", "n"],
            &[("R", Some("4"))],
            &["p.V - n.V = R * p.I", "p.I + n.I = 0"],
        );
        let gnd = comp("Gnd", &["p"], &[], &["p.V = 0"]);
        let out = expand_ok(
            &[vsrc, rst, gnd],
            &[
                inst("VSrc", "B", &[], &[]),
                inst("Rst", "R1", &[], &[]),
                inst("Gnd", "G", &[], &[]),
            ],
            &[conn(&["B.p", "R1.p"]), conn(&["R1.n", "B.n", "G.p"])],
            &[],
        );
        let junctions: Vec<String> = out
            .equations
            .iter()
            .zip(out.eqs())
            .filter(|(e, _)| e.source_text.starts_with("CONNECT"))
            .map(|(_, shown)| shown)
            .collect();
        assert_eq!(
            junctions,
            vec![
                "b$p$v = r1$p$v",
                "(b$p$i + r1$p$i) = 0",
                // A three-endpoint electrical node: two spanning-tree equalities…
                "r1$n$v = b$n$v",
                "r1$n$v = g$p$v",
                // …and one Kirchhoff current law over all three.
                "((r1$n$i + b$n$i) + g$p$i) = 0",
            ]
        );
        assert_eq!(out.units.get("b$p$v"), Some(&"V"));
        assert_eq!(out.units.get("b$p$i"), Some(&"A"));
    }

    #[test]
    fn rotational_and_translational_nodes_use_their_own_pairs() {
        let shaft = comp(
            "Shaft",
            &["a", "b"],
            &[],
            &["a.w = b.w", "a.tau + b.tau = 0"],
        );
        let msrc = comp("MSrc", &["p"], &[("tq", Some("10"))], &["p.tau = -tq"]);
        let rot = expand_ok(
            &[shaft, msrc],
            &[inst("MSrc", "M", &[], &[]), inst("Shaft", "SH", &[], &[])],
            &[conn(&["M.p", "SH.a"])],
            &[],
        );
        assert!(rot
            .source_texts()
            .contains(&"CONNECT m.p, sh.a: sum(tau) = 0"));
        assert!(rot.eqs().contains(&"m$p$w = sh$a$w".to_string()));
        // Units come from the members the *bodies* name, so the shaft's `w` is
        // an angular speed while the source's port only ever named `tau`.
        assert_eq!(rot.units.get("sh$a$w"), Some(&"rad/s"));
        assert_eq!(rot.units.get("m$p$tau"), Some(&"N-m"));
        assert_eq!(rot.units.get("m$p$w"), None);

        let damper = comp(
            "Damper",
            &["a", "b"],
            &[("c", Some("4"))],
            &["a.F = c * (a.vel - b.vel)", "b.F = -a.F"],
        );
        let fsrc = comp("FSrc", &["p"], &[("fs", Some("20"))], &["p.F = -fs"]);
        let trans = expand_ok(
            &[damper, fsrc],
            &[inst("FSrc", "F", &[], &[]), inst("Damper", "D", &[], &[])],
            &[conn(&["F.p", "D.a"])],
            &[],
        );
        assert!(trans
            .source_texts()
            .contains(&"CONNECT f.p, d.a: sum(f) = 0"));
        assert!(trans.eqs().contains(&"f$p$vel = d$a$vel".to_string()));
        assert_eq!(trans.units.get("d$a$vel"), Some(&"m/s"));
        assert_eq!(trans.units.get("f$p$f"), Some(&"N"));
    }

    #[test]
    fn a_signal_node_broadcasts_with_no_flow_equation() {
        let step = comp("Step", &["out"], &[("v", Some("5"))], &["out.sig = v"]);
        let gain = comp(
            "Gain",
            &["in", "out"],
            &[("g", Some("2"))],
            &["out.sig = g * in.sig"],
        );
        let probe = comp("Probe", &["in"], &[], &["y = in.sig"]);
        let out = expand_ok(
            &[step, gain, probe],
            &[
                inst("Step", "S", &[], &[]),
                inst("Gain", "G", &[], &[]),
                inst("Probe", "P1", &[], &[]),
                inst("Probe", "P2", &[], &[]),
            ],
            &[conn(&["S.out", "G.in"]), conn(&["G.out", "P1.in", "P2.in"])],
            &[],
        );
        let junctions: Vec<String> = out
            .equations
            .iter()
            .zip(out.eqs())
            .filter(|(e, _)| e.source_text.starts_with("CONNECT"))
            .map(|(_, shown)| shown)
            .collect();
        // One writer, two readers, no conservation law at all.
        assert_eq!(
            junctions,
            vec![
                "s$out$sig = g$in$sig",
                "g$out$sig = p1$in$sig",
                "g$out$sig = p2$in$sig",
            ]
        );
        // A `sig` value carries no canonical unit.
        assert!(out.units.is_empty());
    }

    // =====================================================================
    //  7. Domain and connector-type separation (hard errors, by design)
    // =====================================================================

    #[test]
    fn a_fluid_port_cannot_be_connected_to_a_heat_port() {
        // Oracle: corpus/e10_domain_mix.frees
        let s = comp(
            "Src",
            &["out"],
            &[],
            &["out.P = 100", "out.h = 5", "out.mdot = 1"],
        );
        let t = comp("TSrc", &["p"], &[], &["p.T = 400", "p.Qdot = 10"]);
        let msg = expand_err(
            &[s, t],
            &[inst("Src", "S", &[], &[]), inst("TSrc", "H", &[], &[])],
            &[conn(&["S.out", "H.p"])],
            &[],
        );
        assert_eq!(
            msg,
            "connect(s.out, h.p): cannot connect a fluid port (s.out) to a heat port \
             (h.p) — different physical domains. Couple domains through a transducer \
             component (a motor, pump, heating resistor, …), not a direct connect."
        );
    }

    #[test]
    fn pneumatic_and_hydraulic_lines_cannot_be_connected() {
        // Oracle: corpus/e11_connector_type.frees
        let g = comp(
            "GSrc",
            &["out"],
            &[("domain$", Some("gas"))],
            &["out.P = 100", "out.h = 5", "out.mdot = 1"],
        );
        let o = comp(
            "OSnk",
            &["in"],
            &[("domain$", Some("oil"))],
            &["W = in.mdot * in.h + in.P"],
        );
        let msg = expand_err(
            &[g.clone(), o.clone()],
            &[inst("GSrc", "G", &[], &[]), inst("OSnk", "O", &[], &[])],
            &[conn(&["G.out", "O.in"])],
            &[],
        );
        assert!(
            msg.starts_with(
                "connect(g.out, o.in): cannot connect a 'gas' line (g.out) to a 'oil' \
                 line (o.in)."
            ),
            "{msg}"
        );

        // The same mistake made with the terse shared-name style is caught while
        // the streams are being tagged, not at a node. Oracle: e12.
        let msg = expand_err(
            &[g, o],
            &[
                inst("GSrc", "G", &["s1"], &[]),
                inst("OSnk", "O", &["s1"], &[]),
            ],
            &[],
            &[],
        );
        assert_eq!(
            msg,
            "Incompatible fluid connector types on stream 's1': 'gas' and 'oil' bound \
             to the same stream. Pneumatic ('gas'), hydraulic ('oil') and thermofluid \
             ('fluid') lines are different connector types and cannot share a port."
        );
    }

    // =====================================================================
    //  8. Riders: moist-air W, blend z, gas species
    //     oracle: corpus/f5_moistair.frees, h2_zrider, h1_species
    // =====================================================================

    #[test]
    fn a_moist_air_node_carries_the_humidity_ratio_across() {
        let msrc = comp(
            "MSrc",
            &["out"],
            &[("domain$", Some("moistair"))],
            &[
                "out.P = 101325",
                "out.h = 50000",
                "out.mdot = 1",
                "out.W = 0.008",
            ],
        );
        let msnk = comp(
            "MSnk",
            &["in"],
            &[("domain$", Some("moistair"))],
            &["Q = in.mdot * in.h + in.P + in.W"],
        );
        let out = expand_ok(
            &[msrc, msnk],
            &[inst("MSrc", "A", &[], &[]), inst("MSnk", "C", &[], &[])],
            &[conn(&["A.out", "C.in"])],
            &[],
        );
        let junctions: Vec<String> = out
            .equations
            .iter()
            .zip(out.eqs())
            .filter(|(e, _)| e.source_text.starts_with("CONNECT"))
            .map(|(_, shown)| shown)
            .collect();
        assert_eq!(
            junctions,
            vec![
                "a$out$p = c$in$p",
                "a$out$h = c$in$h",
                // W rides with the across variables on a pass-through node…
                "a$out$w = c$in$w",
                // …and ṁ is still the through variable.
                "a$out$mdot = c$in$mdot",
            ]
        );
        // The moist-air `w` is a ratio, not the mechanical angular speed.
        assert_eq!(out.units.get("a$out$w"), Some(&"-"));
    }

    #[test]
    fn blend_composition_and_gas_species_ride_only_when_the_node_carries_them() {
        let zsrc = comp(
            "ZSrc",
            &["out"],
            &[],
            &["out.P = 1", "out.h = 2", "out.mdot = 3", "out.z = 0.4"],
        );
        let zsnk = comp("ZSnk", &["in"], &[], &["q = in.z + in.mdot"]);
        let z = expand_ok(
            &[zsrc, zsnk],
            &[inst("ZSrc", "A", &[], &[]), inst("ZSnk", "C", &[], &[])],
            &[conn(&["A.out", "C.in"])],
            &[],
        );
        assert!(z.eqs().contains(&"a$out$z = c$in$z".to_string()));

        let gsrc = comp(
            "GSrc",
            &["out"],
            &[("domain$", Some("gas"))],
            &[
                "out.P = 1",
                "out.h = 2",
                "out.mdot = 3",
                "out.yO2 = 0.21",
                "out.yN2 = 0.79",
            ],
        );
        let gsnk = comp(
            "GSnk",
            &["in"],
            &[("domain$", Some("gas"))],
            &["q = in.yO2 + in.yN2 + in.mdot"],
        );
        let g = expand_ok(
            &[gsrc, gsnk],
            &[inst("GSrc", "A", &[], &[]), inst("GSnk", "C", &[], &[])],
            &[conn(&["A.out", "C.in"])],
            &[],
        );
        let junctions: Vec<String> = g
            .equations
            .iter()
            .zip(g.eqs())
            .filter(|(e, _)| e.source_text.starts_with("CONNECT"))
            .map(|(_, shown)| shown)
            .collect();
        assert_eq!(
            junctions,
            vec![
                "a$out$p = c$in$p",
                "a$out$h = c$in$h",
                // SPECIES_RIDERS order: y, yo2, yco2, yh2o, yn2, oc.
                "a$out$yo2 = c$in$yo2",
                "a$out$yn2 = c$in$yn2",
                "a$out$mdot = c$in$mdot",
            ]
        );

        // A pure line carries none of them: a plain (P, ṁ, h) bond.
        let plain = expand_ok(
            &[src(), snk()],
            &[inst("Src", "S", &[], &[]), inst("Snk", "K", &[], &[])],
            &[conn(&["S.out", "K.in"])],
            &[],
        );
        assert_eq!(
            plain
                .source_texts()
                .into_iter()
                .filter(|t| t.starts_with("CONNECT"))
                .count(),
            3
        );
    }

    // =====================================================================
    //  9. Fluids: per-port inference, propagation, derived properties
    //     oracle: corpus/c7_fluid.frees, g1_hx_perport, g2_propagate, g3_opaque
    // =====================================================================

    #[test]
    fn a_string_parameter_is_baked_into_the_property_call_and_derived_members_resolve() {
        let pump = comp(
            "Pump",
            &["in", "out"],
            &[("eta", Some("0.7")), ("fluid$", Some("Water"))],
            &[
                "v = Volume(fluid$, P = in.P, h = in.h)",
                "out.mdot = in.mdot",
                "out.h = in.h + v * (out.P - in.P) / eta",
                "W = in.mdot * (out.h - in.h)",
            ],
        );
        let out = expand_ok(
            &[pump],
            &[inst("Pump", "P1", &["s1", "s2"], &[])],
            &[],
            &["s1.T = 300", "s1.P = 101325"],
        );
        // `Volume(fluid$, …)` encodes as `prop$volume$fluid$$p$h`; the instance's
        // `fluid$ = Water` bakes into `prop$volume$water$p$h`.
        assert_eq!(out.eqs()[0], "p1$v = prop$volume$water$p$h(s1$p, s1$h)");
        assert_eq!(out.display_of("p1$v"), Some("p1.v"));
        assert_eq!(out.display_of("p1$w"), Some("p1.w"));
        // A top-level `s1.T` on a stream that HAS a fluid becomes the property
        // call on (P, h) — the solver inverts it for the enthalpy.
        assert_eq!(
            out.tops(),
            vec![
                "prop$temperature$water$p$h(s1$p, s1$h) = 300",
                "s1$p = 101325",
            ]
        );
    }

    #[test]
    fn a_two_fluid_exchanger_maps_each_side_by_parameter_prefix() {
        // Oracle: corpus/g1_hx_perport.frees — `hot$` covers hot_in/hot_out and
        // `cold$` covers the cold side, taken in declaration order.
        let hx = comp(
            "HX",
            &["hot_in", "hot_out", "cold_in", "cold_out"],
            &[
                ("hot$", Some("Water")),
                ("cold$", Some("Air")),
                ("UA", Some("100")),
            ],
            &[
                "hot_out.mdot = hot_in.mdot",
                "cold_out.mdot = cold_in.mdot",
                "hot_out.P = hot_in.P",
                "cold_out.P = cold_in.P",
                "Q = hot_in.mdot * (hot_in.h - hot_out.h)",
            ],
        );
        let out = expand_ok(
            &[hx],
            &[inst("HX", "E", &["h1", "h2", "c1", "c2"], &[])],
            &[],
            &["h1.T = 400", "c1.T = 300"],
        );
        assert_eq!(
            out.tops(),
            vec![
                "prop$temperature$water$p$h(h1$p, h1$h) = 400",
                "prop$temperature$air$p$h(c1$p, c1$h) = 300",
            ]
        );
    }

    #[test]
    fn a_fluid_propagates_along_a_connect_to_a_fluid_less_component() {
        // Oracle: corpus/g2_propagate.frees — the Boiler declares no fluid, so
        // `K.in.T` can only resolve because the fluid crossed two connects.
        let fsrc = comp(
            "FluidSrc",
            &["out"],
            &[("fluid$", Some("Water"))],
            &["out.P = 200000", "out.h = 112745", "out.mdot = 1"],
        );
        let boiler = comp(
            "Boiler",
            &["in", "out"],
            &[("Q", Some("50000"))],
            &[
                "out.mdot = in.mdot",
                "out.P = in.P",
                "out.h = in.h + Q / in.mdot",
            ],
        );
        let sink = comp("Sink", &["in"], &[], &["y = in.mdot + in.P"]);
        let out = expand_ok(
            &[fsrc, boiler, sink],
            &[
                inst("FluidSrc", "F", &[], &[]),
                inst("Boiler", "B", &[], &[]),
                inst("Sink", "K", &[], &[]),
            ],
            &[conn(&["F.out", "B.in"]), conn(&["B.out", "K.in"])],
            &["Tout = K.in.T"],
        );
        assert_eq!(
            out.tops(),
            vec!["tout = prop$temperature$water$p$h(k$in$p, k$in$h)"]
        );
        assert_eq!(out.stream_fluid_of("k$in"), Some("water"));
    }

    #[test]
    fn a_member_on_a_fluid_less_stream_stays_an_opaque_rider() {
        // Oracle: corpus/g3_opaque.frees — `.x` must NOT become thermodynamic
        // quality when the stream carries no fluid.
        let gen = comp(
            "Gen",
            &["out"],
            &[],
            &["out.P = 10", "out.h = 2", "out.mdot = 1", "out.x = 7"],
        );
        let use_ = comp("Use", &["in"], &[], &["y = in.x + in.P"]);
        let out = expand_ok(
            &[gen, use_],
            &[inst("Gen", "G", &[], &[]), inst("Use", "U", &[], &[])],
            &[conn(&["G.out", "U.in"])],
            &["z = U.in.x"],
        );
        assert_eq!(out.tops(), vec!["z = u$in$x"]);
        // The node does not carry `x` either — only (P, h, ṁ) — which is why the
        // oracle reports `u.in.x` as a free quantity rather than solving it.
        let junctions: Vec<String> = out
            .equations
            .iter()
            .zip(out.eqs())
            .filter(|(e, _)| e.source_text.starts_with("CONNECT"))
            .map(|(_, shown)| shown)
            .collect();
        assert_eq!(
            junctions,
            vec![
                "g$out$p = u$in$p",
                "g$out$h = u$in$h",
                "g$out$mdot = u$in$mdot",
            ]
        );
    }

    // =====================================================================
    // 10. Hierarchical subsystems — oracle: corpus/f3_hier.frees, k2, k3
    // =====================================================================

    #[test]
    fn a_subsystem_flattens_into_namespaced_leaf_instances_and_connects() {
        let duo = comp_full(
            "Duo",
            &["a", "b"],
            &[("k", Some("3"))],
            &[],
            vec![],
            vec![
                inst("Res", "R1", &[], &[("k", "k")]),
                inst("Res", "R2", &[], &[("k", "k / 3")]),
            ],
            vec![
                conn(&["a", "R1.in"]),
                conn(&["R1.out", "R2.in"]),
                conn(&["R2.out", "b"]),
            ],
        );
        let out = expand_ok(
            &[res(), duo, src(), snk()],
            &[
                inst("Src", "S", &[], &[]),
                inst("Duo", "D", &[], &[("k", "6")]),
                inst("Snk", "K", &[], &[]),
            ],
            &[conn(&["S.out", "D.a"]), conn(&["D.b", "K.in"])],
            &[],
        );
        // The subsystem's own ports become streams `d$a` / `d$b`; its
        // sub-instances are `d.r1` / `d.r2` with streams `d.r1$in`, …
        assert_eq!(out.display_of("d$a$p"), Some("d.a.p"));
        assert_eq!(out.display_of("d.r1$in$p"), Some("d.r1.in.p"));
        assert_eq!(out.display_of("d.r2$out$p"), Some("d.r2.out.p"));
        // The outer `k = 6` substitutes into both sub-instances' expressions:
        // R1 gets 6, R2 gets 6/3.
        assert!(out
            .eqs()
            .contains(&"d.r1$out$p = (d.r1$in$p - (6 * d.r1$in$mdot))".to_string()));
        assert!(out
            .eqs()
            .contains(&"d.r2$out$p = (d.r2$in$p - ((6 / 3) * d.r2$in$mdot))".to_string()));
        // A top-level connect to the subsystem's boundary port resolves through
        // the alias map onto the same stream its internal connect used.
        assert!(out.eqs().contains(&"s$out$p = d$a$p".to_string()));
        assert!(out.eqs().contains(&"d$a$p = d.r1$in$p".to_string()));
    }

    #[test]
    fn a_subsystem_may_wire_its_children_with_shared_stream_names() {
        // Oracle: corpus/k2_hier_shared.frees — `mid` is internal, so it is
        // namespaced to `d.mid`, while `a`/`b` resolve to the bound streams.
        let duo = comp_full(
            "Duo",
            &["a", "b"],
            &[],
            &[],
            vec![],
            vec![
                inst("Res", "R1", &["a", "mid"], &[]),
                inst("Res", "R2", &["mid", "b"], &[]),
            ],
            vec![],
        );
        let out = expand_ok(
            &[res(), duo],
            &[inst("Duo", "D", &["s1", "s2"], &[])],
            &[],
            &[],
        );
        assert_eq!(
            out.eqs(),
            vec![
                "d.mid$mdot = s1$mdot",
                "d.mid$p = (s1$p - (2 * s1$mdot))",
                "d.mid$h = s1$h",
                "s2$mdot = d.mid$mdot",
                "s2$p = (d.mid$p - (2 * d.mid$mdot))",
                "s2$h = d.mid$h",
            ]
        );
        assert_eq!(out.display_of("d.mid$p"), Some("d.mid.p"));
    }

    #[test]
    fn subsystems_nest_and_carry_parameters_down_two_levels() {
        // Oracle: corpus/k3_nested.frees.
        let duo = comp_full(
            "Duo",
            &["a", "b"],
            &[("k", Some("3"))],
            &[],
            vec![],
            vec![
                inst("Res", "R1", &[], &[("k", "k")]),
                inst("Res", "R2", &[], &[("k", "k")]),
            ],
            vec![
                conn(&["a", "R1.in"]),
                conn(&["R1.out", "R2.in"]),
                conn(&["R2.out", "b"]),
            ],
        );
        let quad = comp_full(
            "Quad",
            &["a", "b"],
            &[("k", Some("1"))],
            &[],
            vec![],
            vec![
                inst("Duo", "D1", &[], &[("k", "k")]),
                inst("Duo", "D2", &[], &[("k", "k * 2")]),
            ],
            vec![
                conn(&["a", "D1.a"]),
                conn(&["D1.b", "D2.a"]),
                conn(&["D2.b", "b"]),
            ],
        );
        let out = expand_ok(
            &[res(), duo, quad],
            &[inst("Quad", "Q", &[], &[("k", "1")])],
            &[],
            &[],
        );
        assert_eq!(out.display_of("q.d1.r1$in$p"), Some("q.d1.r1.in.p"));
        assert_eq!(out.display_of("q.d2.r2$out$p"), Some("q.d2.r2.out.p"));
        assert!(out.eqs().contains(&"q.d1$a$p = q.d1.r1$in$p".to_string()));
        // D2's `k = k * 2` substitutes the outer 1 → `(1 * 2)`.
        assert!(out
            .eqs()
            .contains(&"q.d2.r1$out$p = (q.d2.r1$in$p - ((1 * 2) * q.d2.r1$in$mdot))".to_string()));
    }

    #[test]
    fn a_subsystem_that_instantiates_itself_is_rejected() {
        // Oracle: corpus/k4_selfcycle.frees.
        let looping = comp_full(
            "Loop",
            &["a", "b"],
            &[],
            &[],
            vec![],
            vec![inst("Loop", "L1", &[], &[])],
            vec![conn(&["a", "L1.a"]), conn(&["L1.b", "b"])],
        );
        let msg = expand_err(&[looping], &[inst("Loop", "X", &[], &[])], &[], &[]);
        assert_eq!(
            msg,
            "COMPONENT 'loop' instantiates itself (hierarchical cycle)."
        );
    }

    #[test]
    fn a_subsystem_bound_to_too_few_streams_is_rejected_by_name() {
        // The Java dies here with `IndexOutOfBoundsException: Index 1 out of
        // bounds for length 1` (oracle: corpus/k1_hier_arity.frees). This port
        // raises the arity diagnostic instead — the engine's contract is that a
        // rejection names the component and its instance.
        let duo = comp_full(
            "Duo",
            &["a", "b"],
            &[],
            &[],
            vec![],
            vec![inst("Res", "R1", &[], &[])],
            vec![conn(&["a", "R1.in"]), conn(&["R1.out", "b"])],
        );
        let msg = expand_err(&[res(), duo], &[inst("Duo", "D", &["s1"], &[])], &[], &[]);
        assert_eq!(
            msg,
            "Component 'd' (duo) binds 1 port(s) but COMPONENT duo declares 2 (a, b). \
             Bind every port to a stream, or none and wire them with connect(...)."
        );
    }

    // =====================================================================
    // 11. Top-level references: named outputs, ports, streams
    //     oracle: corpus/f4_output.frees, f11, f12
    // =====================================================================

    #[test]
    fn a_named_output_surfaces_as_inst_dot_output() {
        let pump = comp(
            "Pump",
            &["in", "out"],
            &[("eta", Some("0.5"))],
            &[
                "out.mdot = in.mdot",
                "out.h = in.h + (out.P - in.P) / eta",
                "W = in.mdot * (out.h - in.h)",
            ],
        );
        let out = expand_ok(
            &[pump],
            &[inst("Pump", "P1", &["s1", "s2"], &[])],
            &[],
            &["Wtot = P1.W * 2"],
        );
        assert_eq!(out.tops(), vec!["wtot = (p1$w * 2)"]);
        assert_eq!(out.display_of("p1$w"), Some("p1.w"));
    }

    #[test]
    fn a_free_port_can_be_pinned_from_the_top_level() {
        // Oracle: corpus/f12_freeport_top.frees.
        let out = expand_ok(
            &[res()],
            &[inst("Res", "A", &[], &[])],
            &[],
            &["A.in.P = 100", "q = A.out.P"],
        );
        assert_eq!(out.tops(), vec!["a$in$p = 100", "q = a$out$p"]);
    }

    #[test]
    fn a_port_reference_without_a_member_is_rejected() {
        // Oracle: corpus/f11_topref_noport.frees.
        let msg = expand_err(
            &[pipe()],
            &[inst("Pipe", "A", &["s1", "s2"], &[])],
            &[],
            &["z = A.in"],
        );
        assert_eq!(
            msg,
            "Reference 'a.in' to port 'in' of component 'a' needs a member (e.g. a.in.P)."
        );
    }

    #[test]
    fn a_deep_reference_into_a_component_that_is_not_a_port_is_rejected() {
        let msg = expand_err(
            &[pipe()],
            &[inst("Pipe", "A", &["s1", "s2"], &[])],
            &[],
            &["z = A.foo.bar"],
        );
        assert_eq!(
            msg,
            "Reference 'a.foo.bar' to component 'a' is not a port member or named output."
        );
    }

    #[test]
    fn a_dotted_name_that_names_no_instance_flattens_to_a_plain_variable() {
        let out = expand_ok(
            &[pipe()],
            &[inst("Pipe", "A", &["s1", "s2"], &[])],
            &[],
            &["z = foo.bar.baz"],
        );
        assert_eq!(out.tops(), vec!["z = foo$bar$baz"]);
        assert_eq!(out.display_of("foo$bar$baz"), Some("foo.bar.baz"));
    }

    // =====================================================================
    // 12. Reserved names and string parameters used as table names
    //     oracle: corpus/f8_time.frees, f9_tableparam.frees
    // =====================================================================

    #[test]
    fn time_in_a_component_body_stays_the_global_simulation_time() {
        let ramp = comp(
            "Ramp",
            &["out"],
            &[("slope", Some("2"))],
            &["out.sig = slope * time"],
        );
        let out = expand_ok(&[ramp], &[inst("Ramp", "R", &[], &[])], &[], &[]);
        // NOT `r$time` — a time-driven source block would otherwise be unable to
        // exist as a library component.
        assert_eq!(out.eqs(), vec!["r$out$sig = (2 * time)"]);
    }

    #[test]
    fn a_string_parameter_used_as_a_call_name_resolves_to_that_table() {
        // Oracle: corpus/f9_tableparam.frees.
        let c = comp(
            "Comp",
            &["in", "out"],
            &[("map$", Some("effmap"))],
            &[
                "out.mdot = in.mdot",
                "eff = map$(in.mdot)",
                "out.h = in.h / eff",
            ],
        );
        let out = expand_ok(&[c], &[inst("Comp", "C", &["s1", "s2"], &[])], &[], &[]);
        assert_eq!(
            out.eqs(),
            vec![
                "s2$mdot = s1$mdot",
                "c$eff = effmap(s1$mdot)",
                "s2$h = (s1$h / c$eff)",
            ]
        );
        assert_eq!(out.display_of("c$eff"), Some("c.eff"));
    }

    #[test]
    fn a_non_fluid_string_parameter_substitutes_as_a_literal() {
        let hx = comp(
            "HX",
            &["in", "out"],
            &[("arr$", Some("counterflow"))],
            &["out.mdot = in.mdot", "e = hx_effectiveness(arr$, in.mdot)"],
        );
        let out = expand_ok(&[hx], &[inst("HX", "E", &["s1", "s2"], &[])], &[], &[]);
        assert_eq!(
            out.eqs()[1],
            "e$e = hx_effectiveness('counterflow', s1$mdot)"
        );
    }

    // =====================================================================
    // 13. Storage: der/init lifting and the high-index guard
    //     oracle: corpus/g5_storage_steady.frees, f1_highindex, f2_cc
    // =====================================================================

    #[test]
    fn init_lines_are_lifted_out_and_der_lines_mark_storage() {
        let tank = comp(
            "Tank",
            &["p"],
            &[("C", Some("5")), ("Qin", Some("100"))],
            &["der(T) = (Qin + p.Qdot) / C", "init(T) = 300", "p.T = T"],
        );
        let out = expand_ok(&[tank], &[inst("Tank", "TK", &[], &[])], &[], &[]);
        assert!(out.has_storage);
        assert_eq!(
            out.eqs(),
            vec!["der(tk$t) = ((100 + tk$p$qdot) / 5)", "tk$p$t = tk$t",]
        );
        // `init(T) = 300` is an initial condition, not a solver equation.
        assert_eq!(out.initials.len(), 1);
        assert_eq!(out.initials[0].state, "tk$t");
        assert_eq!(show(&out.initials[0].value), "300");
    }

    #[test]
    fn two_rigidly_coupled_storage_states_are_rejected_as_high_index() {
        // Oracle: corpus/f1_highindex.frees — and the message quotes the DISPLAY
        // names, never the mangled scalars.
        let mass = comp(
            "Mass",
            &["p"],
            &[("C", Some("10"))],
            &["der(p.T) = p.Qdot / C", "init(p.T) = 300"],
        );
        let msg = expand_err(
            &[mass],
            &[inst("Mass", "M1", &[], &[]), inst("Mass", "M2", &[], &[])],
            &[conn(&["M1.p", "M2.p"])],
            &[],
        );
        assert_eq!(
            msg,
            "High-index DAE: storage states 'm1.p.t' and 'm2.p.t' are rigidly coupled \
             (directly equated) — index ≥ 2. Lump them into one storage element, or \
             insert a small resistance/compliance between them."
        );
    }

    #[test]
    fn two_capacitive_volumes_at_one_node_are_rejected_by_instance_name() {
        // Oracle: corpus/f2_cc.frees — the C-R-C rule.
        let vol = comp(
            "Vol",
            &["a", "b"],
            &[("V", Some("1"))],
            &[
                "der(a.P) = (a.mdot + b.mdot) / V",
                "init(a.P) = 100",
                "b.P = a.P",
                "b.h = a.h",
            ],
        );
        let msg = expand_err(
            &[vol],
            &[inst("Vol", "V1", &[], &[]), inst("Vol", "V2", &[], &[])],
            &[conn(&["V1.b", "V2.a"])],
            &[],
        );
        assert_eq!(
            msg,
            "connect(...): capacitive volumes [v1, v2] are connected directly with no \
             resistance between them (C-C). Two pressure-storage volumes at one node \
             make the DAE index-2; interpose a resistive flow element between them \
             (the C-R-C rule)."
        );
    }

    // =====================================================================
    // 14. Instantiation errors — every one names the component AND instance
    //     oracle: corpus/e1..e9, e13, e14, f10
    // =====================================================================

    #[test]
    fn instantiation_rejections_name_the_component_and_its_instance() {
        for (msg, expected) in [
            (
                expand_err(
                    &[pipe()],
                    &[inst("Pumpx", "A", &["s1", "s2"], &[])],
                    &[],
                    &[],
                ),
                "Unknown component type 'pumpx' for instance 'a'. Define it with \
                 COMPONENT pumpx(...).",
            ),
            (
                expand_err(
                    &[pipe()],
                    &[
                        inst("Pipe", "A", &["s1", "s2"], &[]),
                        inst("Pipe", "A", &["s2", "s3"], &[]),
                    ],
                    &[],
                    &[],
                ),
                "Component instance 'a' is declared more than once.",
            ),
            (
                expand_err(&[pipe()], &[inst("Pipe", "A", &["s1"], &[])], &[], &[]),
                "Component 'a' (pipe) binds 1 port(s) but COMPONENT pipe declares 2 \
                 (in, out). Bind every port to a stream, or none and wire them with \
                 connect(...).",
            ),
            (
                expand_err(
                    &[pipe()],
                    &[inst("Pipe", "A", &["s1", "s2"], &[("zz", "3")])],
                    &[],
                    &[],
                ),
                "Component 'a' (pipe): unknown parameter 'zz'.",
            ),
            (
                expand_err(
                    &[comp(
                        "Pipe",
                        &["in", "out"],
                        &[("k", None)],
                        &["out.mdot = in.mdot * k"],
                    )],
                    &[inst("Pipe", "A", &["s1", "s2"], &[])],
                    &[],
                    &[],
                ),
                "Component 'a' (pipe): parameter 'k' has no value (give it a default \
                 or pass k=value).",
            ),
            (
                expand_err(
                    &[comp(
                        "Pipe",
                        &["in", "out"],
                        &[("fluid$", None)],
                        &["out.mdot = in.mdot"],
                    )],
                    &[inst("Pipe", "A", &["s1", "s2"], &[])],
                    &[],
                    &[],
                ),
                "Component 'a' (pipe): string parameter 'fluid$' has no value (give it \
                 a default or pass fluid$=Name).",
            ),
        ] {
            assert_eq!(msg, expected);
        }
    }

    #[test]
    fn variant_rejections_name_the_component_and_list_the_choices() {
        let with_variants = |default: Option<&str>| {
            comp_full(
                "C",
                &["in", "out"],
                &[("model$", default)],
                &["out.mdot = in.mdot"],
                vec![
                    variant("a", &["r"], &["out.P = in.P * r"]),
                    variant("b", &["q"], &["out.P = in.P * q"]),
                ],
                vec![],
                vec![],
            )
        };
        // Oracle: corpus/e7_unknown_variant.frees
        assert_eq!(
            expand_err(
                &[with_variants(Some("zzz"))],
                &[inst("C", "X", &["s1", "s2"], &[("r", "2")])],
                &[],
                &[],
            ),
            "Component 'x' (c): unknown model$ 'zzz'. Variants: a, b."
        );
        // Oracle: corpus/e8_no_selector.frees
        let no_selector = comp_full(
            "C",
            &["in", "out"],
            &[],
            &["out.mdot = in.mdot"],
            vec![variant("a", &["r"], &["out.P = in.P * r"])],
            vec![],
            vec![],
        );
        assert_eq!(
            expand_err(
                &[no_selector],
                &[inst("C", "X", &["s1", "s2"], &[("r", "2")])],
                &[],
                &[],
            ),
            "Component 'x' (c): declares VARIANT blocks but no 'PARAM model$' selector \
             to choose between them."
        );
        // Oracle: corpus/e9_variant_hint.frees — the hint points at the SELECTED
        // variant, so the user knows which model demanded the parameter.
        assert_eq!(
            expand_err(
                &[with_variants(Some("a"))],
                &[inst("C", "X", &["s1", "s2"], &[])],
                &[],
                &[],
            ),
            "Component 'x' (c): parameter 'r' has no value (give it a default or pass \
             r=value). (required by the selected 'a' variant)."
        );
    }

    #[test]
    fn supplied_inactive_variant_params_warn_without_rejecting() {
        let def = comp_full(
            "C",
            &["in", "out"],
            &[("model$", Some("a")), ("r", None), ("q", None)],
            &["out.mdot = in.mdot"],
            vec![
                variant("a", &["r"], &["out.P = in.P * r"]),
                variant("b", &["q"], &["out.P = in.P * q"]),
            ],
            vec![],
            vec![],
        );
        let defs = [def];
        let insts = [inst(
            "C",
            "X",
            &["s1", "s2"],
            &[("model$", "a"), ("r", "2"), ("q", "3")],
        )];
        let mut display = BTreeMap::new();
        let mut ex =
            ComponentExpander::new(&[], &defs, &insts, &[], &mut display).expect("resolve");
        ex.expand().expect("expand");
        let warnings = ex.inactive_warnings();
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("'q'") && w.contains("'a'")),
            "expected an inactive-parameter advisory, got {warnings:?}"
        );
        assert!(
            !warnings.iter().any(|w| w.contains("'r'")),
            "active parameter 'r' must not warn, got {warnings:?}"
        );
    }

    #[test]
    fn two_user_definitions_of_one_name_collide() {
        let msg = expand_err(&[pipe(), pipe()], &[], &[], &[]);
        assert_eq!(msg, "COMPONENT 'pipe' is defined more than once.");
    }

    #[test]
    fn a_user_definition_silently_overrides_a_builtin_of_the_same_name() {
        let builtin = comp("Pipe", &["in", "out"], &[], &["out.mdot = 0"]);
        let user = comp("Pipe", &["in", "out"], &[], &["out.mdot = in.mdot"]);
        let builtins = vec![builtin];
        let user_defs = vec![user];
        let mut display = BTreeMap::new();
        let insts = vec![inst("Pipe", "A", &["s1", "s2"], &[])];
        let mut ex =
            ComponentExpander::new(&builtins, &user_defs, &insts, &[], &mut display).unwrap();
        assert_eq!(
            ex.expand().unwrap().iter().map(show_eq).collect::<Vec<_>>(),
            vec!["s2$mdot = s1$mdot"]
        );
    }

    #[test]
    fn a_body_reference_to_an_unknown_port_lists_the_real_ports() {
        // Oracle: corpus/e14_unknown_port_in_body.frees.
        let msg = expand_err(
            &[comp("Pipe", &["in", "out"], &[], &["out.mdot = zz.mdot"])],
            &[inst("Pipe", "A", &["s1", "s2"], &[])],
            &[],
            &[],
        );
        assert_eq!(
            msg,
            "Component 'pipe': 'zz.mdot' references unknown port 'zz'. Ports: in, out."
        );
    }

    #[test]
    fn a_connect_endpoint_that_is_neither_a_port_nor_a_stream_is_rejected() {
        // Oracle: corpus/e13_bad_endpoint.frees.
        let msg = expand_err(
            &[res()],
            &[inst("Res", "A", &[], &[])],
            &[conn(&["A.zz", "A.out"])],
            &[],
        );
        assert_eq!(
            msg,
            "connect(...): 'a.zz' is not a port (instance.port) or a stream name. \
             connect(A.zz,A.out)"
        );
    }

    #[test]
    fn a_connect_with_one_endpoint_is_rejected() {
        // Oracle: corpus/f10_connect_one.frees.
        let msg = expand_err(
            &[src()],
            &[inst("Src", "S", &[], &[])],
            &[conn(&["S.out"])],
            &[],
        );
        assert_eq!(
            msg,
            "connect(...) needs at least two endpoints: connect(S.out)"
        );
    }

    // =====================================================================
    // 15. Mixed binding styles, and the schematic payload
    // =====================================================================

    #[test]
    fn a_bare_stream_name_may_be_a_connect_endpoint() {
        // Oracle: corpus/h3_freeport_shared.frees.
        let out = expand_ok(
            &[src(), pipe(), snk()],
            &[
                inst("Src", "S", &["s1"], &[]),
                inst("Pipe", "A", &["s1", "s2"], &[]),
                inst("Snk", "K", &[], &[]),
            ],
            &[conn(&["s2", "K.in"])],
            &[],
        );
        assert!(out.eqs().contains(&"s2$p = k$in$p".to_string()));
        assert!(out.eqs().contains(&"s2$mdot = k$in$mdot".to_string()));
        // A shared stream keeps its own name as its display prefix.
        assert_eq!(out.display_of("s2$p"), Some("s2.p"));
    }

    #[test]
    fn the_schematic_payload_reports_both_connection_styles() {
        let out = expand_ok(
            &[src(), pipe(), snk()],
            &[
                inst("Src", "S", &["s1"], &[]),
                inst("Pipe", "A", &["s1", "s2"], &[]),
                inst("Snk", "K", &[], &[]),
            ],
            &[conn(&["s2", "K.in"])],
            &[],
        );
        // An explicit connect keeps its endpoints as written; `K.in` is a free
        // port, so its display prefix is `k.in` and `s2`'s is itself.
        let explicit = &out.connections[0];
        assert_eq!(explicit.domain, Domain::Fluid);
        assert_eq!(explicit.endpoints, vec!["s2", "k.in"]);
        assert_eq!(explicit.connector.as_deref(), Some("fluid"));
        assert_eq!(explicit.streams, vec!["s2", "k.in"]);
        // The shared stream `s1` joins two instance ports.
        let shared = out
            .connections
            .iter()
            .find(|c| c.endpoints.contains(&"s.out".to_string()))
            .expect("a shared-stream junction on s1");
        assert_eq!(shared.endpoints, vec!["s.out", "a.in"]);
        assert_eq!(shared.streams, vec!["s1", "s1"]);
    }

    #[test]
    fn a_heat_node_reports_no_connector_and_no_fluid_even_next_to_a_coolant_hx() {
        // `build_stream_fluid_map` tags EVERY port of a fluid-bearing component,
        // wall ports included. The payload must still say the heat node carries
        // heat, not the coolant.
        let hx = comp(
            "HX",
            &["in", "out", "wall"],
            &[("fluid$", Some("Water"))],
            &[
                "out.mdot = in.mdot",
                "out.P = in.P",
                "out.h = in.h",
                "wall.Qdot = in.mdot * (in.h - out.h)",
            ],
        );
        let mass = comp("Mass", &["p"], &[], &["p.T = 300"]);
        let out = expand_ok(
            &[hx, mass],
            &[inst("HX", "E", &[], &[]), inst("Mass", "M", &[], &[])],
            &[conn(&["E.wall", "M.p"])],
            &[],
        );
        let node = &out.connections[0];
        assert_eq!(node.domain, Domain::Heat);
        assert_eq!(node.connector, None);
        assert_eq!(node.fluid, None);
    }

    // =====================================================================
    // 16. Units, emptiness, and the no-component fast path
    // =====================================================================

    #[test]
    fn canonical_stream_members_carry_their_si_units() {
        let out = expand_ok(
            &[pipe()],
            &[inst("Pipe", "A", &["s1", "s2"], &[])],
            &[],
            &[],
        );
        assert_eq!(out.units.get("s1$p"), Some(&"Pa"));
        assert_eq!(out.units.get("s1$h"), Some(&"J/kg"));
        assert_eq!(out.units.get("s1$mdot"), Some(&"kg/s"));
        // A component-local like `dP` is not a stream member and gets none.
        assert!(!out.units.contains_key("a$dp"));
    }

    #[test]
    fn an_empty_network_still_mangles_dotted_names_like_the_java() {
        // RE-PINNED 2026-08-22 (docs harvest, `docs_components_05`). This test
        // used to assert the opposite — "no components ⇒ no rewrite at all" —
        // via a short-circuit the Java does not have. The oracle's golden for
        // `RET.in.P = 100000 [Pa]` shows `rewriteTopVar` runs on an empty
        // network too: a dotted name becomes its `$`-flattened form and the
        // dotted spelling lands in `displayNames`.
        let builtins: Vec<ComponentDef> = Vec::new();
        let defs: Vec<ComponentDef> = Vec::new();
        let mut display = BTreeMap::new();
        let statements = vec![Statement::Eq(eq("z = a.b"))];
        let mut ex = ComponentExpander::new(&builtins, &defs, &[], &[], &mut display).unwrap();
        assert!(ex.is_empty());
        assert!(ex.expand().unwrap().is_empty());
        let out = ex.rewrite_statements(statements).unwrap();
        let rewritten = match &out[..] {
            [Statement::Eq(equation)] => equation,
            other => panic!("expected one equation, got {other:?}"),
        };
        assert_eq!(rewritten.rhs, Expr::Var("a$b".into()));
        assert!(ex.connections().is_empty());
        assert_eq!(display.get("a$b"), Some(&"a.b".to_string()));
    }

    #[test]
    fn for_blocks_and_dynamic_expressions_get_the_same_rewrite() {
        let builtins: Vec<ComponentDef> = Vec::new();
        let defs = vec![res()];
        let insts = vec![inst("Res", "A", &[], &[])];
        let mut display = BTreeMap::new();
        let mut ex = ComponentExpander::new(&builtins, &defs, &insts, &[], &mut display).unwrap();
        let stmts = vec![Statement::For {
            var_name: "i".into(),
            start: expr("1"),
            end: expr("2"),
            body: vec![Statement::Eq(eq("y = A.in.P"))],
        }];
        let out = ex.rewrite_statements(stmts).unwrap();
        match &out[0] {
            Statement::For { body, .. } => match &body[0] {
                Statement::Eq(e) => assert_eq!(show_eq(e), "y = a$in$p"),
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        }
        // A DYNAMIC body equation takes the identical rewrite, which is what
        // lets a transient model drive an acausal component.
        let rewritten = ex
            .rewrite_top_equation(&eq("A.out.mdot = 2 * time"))
            .unwrap();
        assert_eq!(show_eq(&rewritten), "a$out$mdot = (2 * time)");
        assert_eq!(
            show(&ex.rewrite_top_expr(&expr("A.in.h")).unwrap()),
            "a$in$h"
        );
    }

    // =====================================================================
    // 17. Unit-level: bake_fluid, substitute_params, union-find
    // =====================================================================

    #[test]
    fn bake_fluid_replaces_only_the_placeholder_segment() {
        let params = vec![("fluid$".to_string(), "water".to_string())];
        // `Enthalpy(fluid$, P=.., h=..)` → `prop$enthalpy$fluid$$p$h`.
        assert_eq!(
            bake_fluid("prop$enthalpy$fluid$$p$h", &params),
            "prop$enthalpy$water$p$h"
        );
        // An already-concrete fluid is left alone (segment 3 is not empty).
        assert_eq!(
            bake_fluid("prop$enthalpy$r134a$t$x", &params),
            "prop$enthalpy$r134a$t$x"
        );
        // An unknown placeholder is left for the global string-variable pass.
        assert_eq!(
            bake_fluid("prop$enthalpy$r$$t", &params),
            "prop$enthalpy$r$$t"
        );
        // Non-property calls are never touched.
        assert_eq!(bake_fluid("sqrt", &params), "sqrt");
        assert_eq!(bake_fluid("prop$molarmass", &params), "prop$molarmass");
    }

    #[test]
    fn substitute_params_reaches_only_where_the_java_reaches() {
        let params = vec![("k".to_string(), Expr::num(6.0))];
        assert_eq!(show(&substitute_params(&expr("k"), &params)), "6");
        assert_eq!(show(&substitute_params(&expr("k / 3"), &params)), "(6 / 3)");
        assert_eq!(show(&substitute_params(&expr("-k"), &params)), "-6");
        assert_eq!(show(&substitute_params(&expr("f(k)"), &params)), "f(6)");
        // Not descended into — transcribed behaviour, not an oversight.
        assert_eq!(show(&substitute_params(&expr("a[k]"), &params)), "a[k]");
        assert_eq!(
            show(&substitute_params(&expr("[k, 1]"), &params)),
            "[[k, 1]]"
        );
    }

    #[test]
    fn union_find_matches_the_java_semantics() {
        let mut uf = UnionFind::new();
        // An unseen node is its own root.
        assert_eq!(uf.find("a"), "a");
        assert!(!uf.connected("a", "b"));
        uf.union("a", "b");
        uf.union("b", "c");
        assert!(uf.connected("a", "c"));
        // A long chain still resolves (the Java recurses; this iterates).
        for i in 0..2_000 {
            uf.union(&format!("n{i}"), &format!("n{}", i + 1));
        }
        assert!(uf.connected("n0", "n2000"));
        assert!(!uf.connected("n0", "a"));
    }

    #[test]
    fn is_pressure_capacitive_sees_der_of_a_port_pressure_only() {
        let capacitive = comp("V", &["a", "b"], &[], &["der(a.P) = a.mdot", "b.P = a.P"]);
        assert!(is_pressure_capacitive(&capacitive));
        // A der on a component-local, or on a non-pressure member, is not it.
        let thermal = comp("M", &["p"], &[], &["der(T) = p.Qdot", "p.T = T"]);
        assert!(!is_pressure_capacitive(&thermal));
        let enthalpy = comp("E", &["a"], &[], &["der(a.h) = a.mdot"]);
        assert!(!is_pressure_capacitive(&enthalpy));
        // A der(port.P) inside a VARIANT counts too.
        let variant_only = comp_full(
            "VV",
            &["a"],
            &[("model$", Some("dyn"))],
            &[],
            vec![variant("dyn", &[], &["der(a.P) = a.mdot"])],
            vec![],
            vec![],
        );
        assert!(is_pressure_capacitive(&variant_only));
    }

    // =====================================================================
    // 18. Numeric parity: the expanded system must *solve* like the Java's
    //
    //     The front end cannot parse `COMPONENT` yet, so a full end-to-end
    //     replay is impossible. This is the next best thing and a much stronger
    //     check than shape alone: the expanded equations are rendered back to a
    //     plain scalar document (`$` and `.` folded to `_`, since neither is a
    //     legal identifier character), solved by this crate's own Newton/Tarjan
    //     pipeline, and compared value-for-value against the Java oracle's
    //     result table for the same source document.
    // =====================================================================

    fn flatten_name(n: &str) -> String {
        n.replace(['$', '.'], "_")
    }

    /// [`show`], with every variable name folded into a legal identifier.
    fn render(e: &Expr) -> String {
        match e {
            Expr::Var(n) => flatten_name(n),
            Expr::ArrayAccess { name, indices } => format!(
                "{}[{}]",
                flatten_name(name),
                indices.iter().map(render).collect::<Vec<_>>().join(", ")
            ),
            Expr::Num { value, .. } => num(*value),
            Expr::Str(s) => format!("'{s}'"),
            Expr::Neg(i) => format!("-{}", render(i)),
            Expr::Not(i) => format!("not {}", render(i)),
            Expr::BinOp { op, left, right } => {
                format!("({} {} {})", render(left), op.as_str(), render(right))
            }
            Expr::Compare { op, left, right } => {
                format!("({} {} {})", render(left), op.as_str(), render(right))
            }
            Expr::Logical { op, left, right } => {
                format!("({} {} {})", render(left), op.as_str(), render(right))
            }
            Expr::Range { start, end } => format!("{}:{}", render(start), render(end)),
            Expr::ArrayLiteral(els) => {
                format!(
                    "[{}]",
                    els.iter().map(render).collect::<Vec<_>>().join(", ")
                )
            }
            Expr::Call { function, args } => format!(
                "{function}({})",
                args.iter().map(render).collect::<Vec<_>>().join(", ")
            ),
        }
    }

    /// Solves the expanded system (bodies + rewritten top-level statements) and
    /// asserts each value against the Java oracle's.
    fn assert_solves(out: &Expanded, expected: &[(&str, f64)]) {
        let mut src = String::new();
        for e in out
            .equations
            .iter()
            .chain(out.statements.iter().filter_map(|s| match s {
                Statement::Eq(e) => Some(e),
                _ => None,
            }))
        {
            src.push_str(&render(&e.lhs));
            src.push_str(" = ");
            src.push_str(&render(&e.rhs));
            src.push('\n');
        }
        let solution = crate::engine::solve(&src, &crate::solver::SolverSettings::default())
            .unwrap_or_else(|e| panic!("the expanded system did not solve:\n{src}\n{e:?}"));
        for (flat, want) in expected {
            let key = flatten_name(flat);
            let got = solution.values.get(&key).copied().unwrap_or_else(|| {
                panic!(
                    "no solved value for {flat} (as {key}); document was:\n{src}\nvalues: {:?}",
                    solution.values
                )
            });
            let tolerance = 1e-9 * want.abs().max(1.0);
            assert!(
                (got - want).abs() <= tolerance,
                "{flat}: solved {got}, oracle {want}"
            );
        }
    }

    #[test]
    fn the_chain_solves_to_the_oracles_numbers() {
        // Oracle: corpus/c1_chain.frees
        let out = expand_ok(
            &[pipe()],
            &[
                inst("Pipe", "A", &["s1", "s2"], &[]),
                inst("Pipe", "B", &["s2", "s3"], &[("k", "5")]),
            ],
            &[],
            &["s1.P = 500", "s1.mdot = 3", "s1.h = 100"],
        );
        assert_solves(
            &out,
            &[
                ("a$dp", 6.0),
                ("b$dp", 15.0),
                ("s1$p", 500.0),
                ("s2$p", 494.0),
                ("s3$p", 479.0),
                ("s2$mdot", 3.0),
                ("s3$h", 100.0),
            ],
        );
    }

    #[test]
    fn the_connect_wired_flowsheet_solves_to_the_oracles_numbers() {
        // Oracle: corpus/c2_connect.frees
        let out = expand_ok(
            &[src(), res(), snk()],
            &[
                inst("Src", "S", &[], &[]),
                inst("Res", "L", &[], &[]),
                inst("Snk", "K", &[], &[]),
            ],
            &[conn(&["S.out", "L.in"]), conn(&["L.out", "K.in"])],
            &[],
        );
        assert_solves(
            &out,
            &[
                ("s$out$p", 400.0),
                ("s$out$mdot", 2.0),
                ("l$in$p", 400.0),
                ("l$out$p", 396.0),
                ("k$in$p", 396.0),
                ("k$in$h", 50.0),
                ("k$w", 100.0),
            ],
        );
    }

    #[test]
    fn the_closed_loop_solves_to_the_oracles_numbers() {
        // Oracle: corpus/c5_loop.frees — note `r.out.p = 292` while
        // `p.in.p = 200`: the loop-closing connect emitted nothing, which is the
        // documented behaviour, not a rounding artefact.
        let pump = comp(
            "Pump",
            &["in", "out"],
            &[("dp", Some("100"))],
            &["out.mdot = in.mdot", "out.P = in.P + dp", "out.h = in.h"],
        );
        let out = expand_ok(
            &[pump, res()],
            &[inst("Pump", "P", &[], &[]), inst("Res", "R", &[], &[])],
            &[conn(&["P.out", "R.in"]), conn(&["R.out", "P.in"])],
            &["P.in.P = 200", "P.in.h = 50", "P.in.mdot = 4"],
        );
        assert_solves(
            &out,
            &[
                ("p$in$p", 200.0),
                ("p$out$p", 300.0),
                ("r$in$p", 300.0),
                ("r$out$p", 292.0),
                ("r$out$mdot", 4.0),
                ("r$out$h", 50.0),
            ],
        );
    }

    #[test]
    fn the_variant_selection_solves_to_the_oracles_numbers() {
        // Oracle: corpus/c4_variant.frees
        let c = comp_full(
            "Comp",
            &["in", "out"],
            &[("model$", Some("simple"))],
            &["out.mdot = in.mdot"],
            vec![
                variant(
                    "simple",
                    &["ratio"],
                    &["out.P = in.P * ratio", "out.h = in.h * 1.1"],
                ),
                variant(
                    "detailed",
                    &["ratio", "eta"],
                    &[
                        "out.P = in.P * ratio",
                        "out.h = in.h + (in.h * (ratio - 1)) / eta",
                    ],
                ),
            ],
            vec![],
            vec![],
        );
        let out = expand_ok(
            &[c],
            &[
                inst("Comp", "C1", &["s1", "s2"], &[("ratio", "3")]),
                inst(
                    "Comp",
                    "C2",
                    &["s2", "s3"],
                    &[("model$", "detailed"), ("ratio", "2"), ("eta", "0.8")],
                ),
            ],
            &[],
            &["s1.P = 100", "s1.h = 50", "s1.mdot = 1"],
        );
        assert_solves(
            &out,
            &[
                ("s2$p", 300.0),
                ("s2$h", 55.000_000_000_000_01),
                ("s3$p", 600.0),
                ("s3$h", 123.75),
                ("s3$mdot", 1.0),
            ],
        );
    }

    #[test]
    fn the_heat_and_electrical_networks_solve_to_the_oracles_numbers() {
        // Oracle: corpus/c6_heat_elec.frees
        let tsrc = comp("TSrc", &["p"], &[("tset", Some("400"))], &["p.T = tset"]);
        let cond = comp(
            "Cond",
            &["a", "b"],
            &[("UA", Some("10"))],
            &["a.Qdot = UA * (a.T - b.T)", "b.Qdot = -a.Qdot"],
        );
        let tsink = comp("TSink", &["p"], &[("tset", Some("300"))], &["p.T = tset"]);
        let heat = expand_ok(
            &[tsrc, cond, tsink],
            &[
                inst("TSrc", "HOT", &[], &[]),
                inst("Cond", "W", &[], &[]),
                inst("TSink", "COLD", &[], &[]),
            ],
            &[conn(&["HOT.p", "W.a"]), conn(&["W.b", "COLD.p"])],
            &[],
        );
        assert_solves(
            &heat,
            &[
                ("hot$p$t", 400.0),
                ("hot$p$qdot", -1000.0),
                ("w$a$qdot", 1000.0),
                ("w$b$qdot", -1000.0),
                ("cold$p$qdot", 1000.0),
                ("cold$p$t", 300.0),
            ],
        );

        let vsrc = comp(
            "VSrc",
            &["p", "n"],
            &[("vset", Some("12"))],
            &["p.V - n.V = vset", "p.I + n.I = 0"],
        );
        let rst = comp(
            "Rst",
            &["p", "n"],
            &[("R", Some("4"))],
            &["p.V - n.V = R * p.I", "p.I + n.I = 0"],
        );
        let gnd = comp("Gnd", &["p"], &[], &["p.V = 0"]);
        let elec = expand_ok(
            &[vsrc, rst, gnd],
            &[
                inst("VSrc", "B", &[], &[]),
                inst("Rst", "R1", &[], &[]),
                inst("Gnd", "G", &[], &[]),
            ],
            &[conn(&["B.p", "R1.p"]), conn(&["R1.n", "B.n", "G.p"])],
            &[],
        );
        assert_solves(
            &elec,
            &[
                ("b$p$v", 12.0),
                ("b$p$i", -3.0),
                ("b$n$v", 0.0),
                ("b$n$i", 3.0),
                ("r1$p$i", 3.0),
                ("r1$n$i", -3.0),
                ("g$p$i", 0.0),
                ("g$p$v", 0.0),
            ],
        );
    }

    #[test]
    fn the_hierarchical_subsystem_solves_to_the_oracles_numbers() {
        // Oracle: corpus/f3_hier.frees
        let duo = comp_full(
            "Duo",
            &["a", "b"],
            &[("k", Some("3"))],
            &[],
            vec![],
            vec![
                inst("Res", "R1", &[], &[("k", "k")]),
                inst("Res", "R2", &[], &[("k", "k / 3")]),
            ],
            vec![
                conn(&["a", "R1.in"]),
                conn(&["R1.out", "R2.in"]),
                conn(&["R2.out", "b"]),
            ],
        );
        let out = expand_ok(
            &[res(), duo, src_h5(), snk_f3()],
            &[
                inst("Src", "S", &[], &[]),
                inst("Duo", "D", &[], &[("k", "6")]),
                inst("Snk", "K", &[], &[]),
            ],
            &[conn(&["S.out", "D.a"]), conn(&["D.b", "K.in"])],
            &[],
        );
        assert_solves(
            &out,
            &[
                ("s$out$p", 400.0),
                ("d$a$p", 400.0),
                ("d.r1$out$p", 388.0),
                ("d.r2$in$p", 388.0),
                ("d.r2$out$p", 384.0),
                ("d$b$p", 384.0),
                ("k$in$p", 384.0),
                ("k$w", 394.0),
                ("k$in$mdot", 2.0),
            ],
        );
    }

    #[test]
    fn the_nested_subsystem_solves_to_the_oracles_numbers() {
        // Oracle: corpus/k3_nested.frees — parameters carried down two levels.
        let duo = comp_full(
            "Duo",
            &["a", "b"],
            &[("k", Some("3"))],
            &[],
            vec![],
            vec![
                inst("Res", "R1", &[], &[("k", "k")]),
                inst("Res", "R2", &[], &[("k", "k")]),
            ],
            vec![
                conn(&["a", "R1.in"]),
                conn(&["R1.out", "R2.in"]),
                conn(&["R2.out", "b"]),
            ],
        );
        let quad = comp_full(
            "Quad",
            &["a", "b"],
            &[("k", Some("1"))],
            &[],
            vec![],
            vec![
                inst("Duo", "D1", &[], &[("k", "k")]),
                inst("Duo", "D2", &[], &[("k", "k * 2")]),
            ],
            vec![
                conn(&["a", "D1.a"]),
                conn(&["D1.b", "D2.a"]),
                conn(&["D2.b", "b"]),
            ],
        );
        let out = expand_ok(
            &[res(), duo, quad, src_h5(), snk_sum()],
            &[
                inst("Src", "S", &[], &[]),
                inst("Quad", "Q", &[], &[("k", "1")]),
                inst("Snk", "K", &[], &[]),
            ],
            &[conn(&["S.out", "Q.a"]), conn(&["Q.b", "K.in"])],
            &[],
        );
        assert_solves(
            &out,
            &[
                ("q$a$p", 400.0),
                ("q.d1.r1$out$p", 398.0),
                ("q.d1.r2$out$p", 396.0),
                ("q.d2.r1$out$p", 392.0),
                ("q.d2.r2$out$p", 388.0),
                ("q$b$p", 388.0),
                ("k$in$p", 388.0),
                ("k$w", 395.0),
            ],
        );
    }

    #[test]
    fn the_shared_name_subsystem_solves_to_the_oracles_numbers() {
        // Oracle: corpus/k2_hier_shared.frees
        let duo = comp_full(
            "Duo",
            &["a", "b"],
            &[],
            &[],
            vec![],
            vec![
                inst("Res", "R1", &["a", "mid"], &[]),
                inst("Res", "R2", &["mid", "b"], &[]),
            ],
            vec![],
        );
        let out = expand_ok(
            &[res(), duo, src_h5(), snk_sum()],
            &[
                inst("Src", "S", &["s1"], &[]),
                inst("Duo", "D", &["s1", "s2"], &[]),
                inst("Snk", "K", &["s2"], &[]),
            ],
            &[],
            &[],
        );
        assert_solves(
            &out,
            &[
                ("s1$p", 400.0),
                ("d.mid$p", 396.0),
                ("s2$p", 392.0),
                ("s2$mdot", 2.0),
                ("k$w", 399.0),
            ],
        );
    }

    #[test]
    fn the_signal_chain_solves_to_the_oracles_numbers() {
        // Oracle: corpus/f6_signal.frees — one writer broadcasting to two readers.
        let step = comp("Step", &["out"], &[("v", Some("5"))], &["out.sig = v"]);
        let gain = comp(
            "Gain",
            &["in", "out"],
            &[("g", Some("2"))],
            &["out.sig = g * in.sig"],
        );
        let probe = comp("Probe", &["in"], &[], &["y = in.sig"]);
        let out = expand_ok(
            &[step, gain, probe],
            &[
                inst("Step", "S", &[], &[]),
                inst("Gain", "G", &[], &[]),
                inst("Probe", "P1", &[], &[]),
                inst("Probe", "P2", &[], &[]),
            ],
            &[conn(&["S.out", "G.in"]), conn(&["G.out", "P1.in", "P2.in"])],
            &[],
        );
        assert_solves(
            &out,
            &[
                ("s$out$sig", 5.0),
                ("g$in$sig", 5.0),
                ("g$out$sig", 10.0),
                ("p1$y", 10.0),
                ("p2$y", 10.0),
            ],
        );
    }

    #[test]
    fn the_steady_storage_network_solves_to_the_oracles_numbers() {
        // Oracle: corpus/g5_storage_steady.frees — with no DYNAMIC block the
        // engine solves the equilibrium, so `der(T) = rhs` becomes `rhs = 0`.
        // That routing is the caller's job; here the algebraic half is checked
        // by pinning the derivative to zero the same way.
        let tank = comp(
            "Tank",
            &["p"],
            &[("C", Some("5")), ("Qin", Some("100"))],
            &["der(T) = (Qin + p.Qdot) / C", "init(T) = 300", "p.T = T"],
        );
        let tsink = comp(
            "TSink",
            &["p"],
            &[("UA", Some("4")), ("Tinf", Some("290"))],
            &["p.Qdot = UA * (p.T - Tinf)"],
        );
        let out = expand_ok(
            &[tank, tsink],
            &[inst("Tank", "TK", &[], &[]), inst("TSink", "S", &[], &[])],
            &[conn(&["TK.p", "S.p"])],
            &[],
        );
        assert!(out.has_storage);
        assert_eq!(out.initials.len(), 1);
        // Steady form: replace the single `der(...) = rhs` with `0 = rhs`.
        let steady: Vec<Equation> = out
            .equations
            .iter()
            .map(|e| match &e.lhs {
                Expr::Call { function, .. } if function == "der" => {
                    Equation::new(Expr::num(0.0), e.rhs.clone(), e.source_text.clone())
                }
                _ => e.clone(),
            })
            .collect();
        let steady = Expanded {
            equations: steady,
            statements: vec![],
            display: out.display.clone(),
            units: out.units.clone(),
            connections: vec![],
            initials: vec![],
            has_storage: false,
            fluids: BTreeMap::new(),
        };
        assert_solves(
            &steady,
            &[
                ("tk$t", 315.0),
                ("tk$p$t", 315.0),
                ("tk$p$qdot", -100.0),
                ("s$p$qdot", 100.0),
            ],
        );
    }

    // =====================================================================
    // 19. End-to-end against the Java oracle's own documents
    //
    //     The front end now parses `COMPONENT` / `connect` into
    //     `Document::components`, so these tests take the *verbatim source* of
    //     an oracle corpus document, run it through the real parser and this
    //     expander, solve the result, and compare the table against the JSON
    //     the Java engine produced for the same text — keyed by the user-visible
    //     DISPLAY names (`s2.p`, `a.dp`, `d.r1.in.p`), which is the contract the
    //     result table actually exposes.
    //
    //     Only the wiring between the two (the `EquationParser.parseResult`
    //     equivalent) is still someone else's; everything either side of it is
    //     the shipping code.
    // =====================================================================

    /// Parses `source`, expands its component layer, solves the flattened
    /// system, and returns the table keyed by display name.
    fn parse_expand_solve(source: &str) -> BTreeMap<String, f64> {
        parse_expand_solve_using(source, &[])
    }

    /// The same, but with the **real 295-component built-in library** behind the
    /// document — the shipped `.frees` text, parsed by the ordinary front end.
    fn parse_expand_solve_with_library(source: &str) -> BTreeMap<String, f64> {
        let library = crate::components::library::builtins()
            .unwrap_or_else(|e| panic!("the built-in library failed to load: {e}"));
        parse_expand_solve_using(source, library.defs())
    }

    fn parse_expand_solve_using(source: &str, builtins: &[ComponentDef]) -> BTreeMap<String, f64> {
        let mut doc =
            crate::parser::parse_document(source).unwrap_or_else(|e| panic!("parse failed: {e}"));
        let components = std::mem::take(&mut doc.components);
        let statements = std::mem::take(&mut doc.statements);
        let mut display: BTreeMap<String, String> = std::mem::take(&mut doc.display_names);
        let (equations, statements) = {
            let mut ex = ComponentExpander::new(
                builtins,
                &components.defs,
                &components.instances,
                &components.connects,
                &mut display,
            )
            .unwrap_or_else(|e| panic!("expansion failed: {e}"));
            let equations = ex
                .expand()
                .unwrap_or_else(|e| panic!("expansion failed: {e}"));
            let statements = ex
                .rewrite_statements(statements)
                .unwrap_or_else(|e| panic!("rewrite failed: {e}"));
            (equations, statements)
        };

        let mut flat = String::new();
        for e in equations
            .iter()
            .chain(statements.iter().filter_map(|s| match s {
                Statement::Eq(e) => Some(e),
                _ => None,
            }))
        {
            flat.push_str(&render(&e.lhs));
            flat.push_str(" = ");
            flat.push_str(&render(&e.rhs));
            flat.push('\n');
        }
        let solution = crate::engine::solve(&flat, &crate::solver::SolverSettings::default())
            .unwrap_or_else(|e| panic!("the expanded system did not solve:\n{flat}\n{e:?}"));

        // Re-key by display name, exactly as the Java result table does
        // (`displayNames.getOrDefault(name, name)`).
        let mut out = BTreeMap::new();
        for (flat_name, shown) in &display {
            if let Some(v) = solution.values.get(&flatten_name(flat_name)) {
                out.insert(shown.clone(), *v);
            }
        }
        for (name, v) in &solution.values {
            out.entry(name.clone()).or_insert(*v);
        }
        out
    }

    fn assert_table(source: &str, expected: &[(&str, f64)]) {
        check_table(parse_expand_solve(source), expected);
    }

    fn assert_library_table(source: &str, expected: &[(&str, f64)]) {
        check_table(parse_expand_solve_with_library(source), expected);
    }

    fn check_table(table: BTreeMap<String, f64>, expected: &[(&str, f64)]) {
        for (key, want) in expected {
            let got = table
                .get(*key)
                .copied()
                .unwrap_or_else(|| panic!("no result row '{key}'; table was {table:?}"));
            let tolerance = 1e-9 * want.abs().max(1.0);
            assert!(
                (got - want).abs() <= tolerance,
                "{key}: solved {got}, oracle {want}"
            );
        }
    }

    #[test]
    fn oracle_document_c1_chain() {
        // Verbatim source of fixtures corpus document `c1_chain.frees`, and the
        // Java engine's own result table for it.
        assert_table(
            "COMPONENT Pipe(in, out)\n\
             \x20 PARAM k = 2\n\
             \x20 out.mdot = in.mdot\n\
             \x20 out.P = in.P - k * in.mdot\n\
             \x20 out.h = in.h\n\
             \x20 dP = in.P - out.P\n\
             END\n\
             \n\
             Pipe A(s1, s2)\n\
             Pipe B(s2, s3, k = 5)\n\
             \n\
             s1.P = 500\n\
             s1.mdot = 3\n\
             s1.h = 100\n",
            &[
                ("a.dp", 6.0),
                ("b.dp", 15.0),
                ("s1.h", 100.0),
                ("s1.mdot", 3.0),
                ("s1.p", 500.0),
                ("s2.h", 100.0),
                ("s2.mdot", 3.0),
                ("s2.p", 494.0),
                ("s3.h", 100.0),
                ("s3.mdot", 3.0),
                ("s3.p", 479.0),
            ],
        );
    }

    #[test]
    fn oracle_document_c2_connect() {
        assert_table(
            "COMPONENT Src(out)\n\
             \x20 PARAM p0 = 400, h0 = 50, m0 = 2\n\
             \x20 out.P = p0\n\
             \x20 out.h = h0\n\
             \x20 out.mdot = m0\n\
             END\n\
             COMPONENT Pipe(in, out)\n\
             \x20 PARAM k = 2\n\
             \x20 out.mdot = in.mdot\n\
             \x20 out.P = in.P - k * in.mdot\n\
             \x20 out.h = in.h\n\
             END\n\
             COMPONENT Snk(in)\n\
             \x20 PARAM c = 0\n\
             \x20 W = in.mdot * in.h + c\n\
             END\n\
             Src S()\n\
             Pipe L()\n\
             Snk K()\n\
             connect(S.out, L.in)\n\
             connect(L.out, K.in)\n",
            &[
                ("k.in.h", 50.0),
                ("k.in.mdot", 2.0),
                ("k.in.p", 396.0),
                ("k.w", 100.0),
                ("l.in.p", 400.0),
                ("l.out.p", 396.0),
                ("s.out.h", 50.0),
                ("s.out.mdot", 2.0),
                ("s.out.p", 400.0),
            ],
        );
    }

    #[test]
    fn oracle_document_c4_variant() {
        assert_table(
            "COMPONENT Comp(in, out)\n\
             \x20 PARAM model$ = simple\n\
             \x20 out.mdot = in.mdot\n\
             \x20 VARIANT simple REQUIRE ratio\n\
             \x20   out.P = in.P * ratio\n\
             \x20   out.h = in.h * 1.1\n\
             \x20 END\n\
             \x20 VARIANT detailed REQUIRE ratio, eta\n\
             \x20   out.P = in.P * ratio\n\
             \x20   out.h = in.h + (in.h * (ratio - 1)) / eta\n\
             \x20 END\n\
             END\n\
             Comp C1(s1, s2, ratio = 3)\n\
             Comp C2(s2, s3, model$ = detailed, ratio = 2, eta = 0.8)\n\
             s1.P = 100\n\
             s1.h = 50\n\
             s1.mdot = 1\n",
            &[
                ("s1.h", 50.0),
                ("s1.mdot", 1.0),
                ("s1.p", 100.0),
                ("s2.h", 55.000_000_000_000_01),
                ("s2.mdot", 1.0),
                ("s2.p", 300.0),
                ("s3.h", 123.75),
                ("s3.mdot", 1.0),
                ("s3.p", 600.0),
            ],
        );
    }

    #[test]
    fn oracle_document_c5_loop() {
        assert_table(
            "COMPONENT Pump(in, out)\n\
             \x20 PARAM dp = 100\n\
             \x20 out.mdot = in.mdot\n\
             \x20 out.P = in.P + dp\n\
             \x20 out.h = in.h\n\
             END\n\
             COMPONENT Res(in, out)\n\
             \x20 PARAM k = 2\n\
             \x20 out.mdot = in.mdot\n\
             \x20 out.P = in.P - k * in.mdot\n\
             \x20 out.h = in.h\n\
             END\n\
             Pump P()\n\
             Res R()\n\
             connect(P.out, R.in)\n\
             connect(R.out, P.in)\n\
             P.in.P = 200\n\
             P.in.h = 50\n\
             P.in.mdot = 4\n",
            &[
                ("p.in.h", 50.0),
                ("p.in.mdot", 4.0),
                ("p.in.p", 200.0),
                ("p.out.p", 300.0),
                ("r.in.p", 300.0),
                ("r.out.h", 50.0),
                ("r.out.mdot", 4.0),
                ("r.out.p", 292.0),
            ],
        );
    }

    #[test]
    fn oracle_document_c6_heat_and_electrical() {
        assert_table(
            "COMPONENT TSrc(p)\n\
             \x20 PARAM tset = 400\n\
             \x20 p.T = tset\n\
             END\n\
             COMPONENT Cond(a, b)\n\
             \x20 PARAM UA = 10\n\
             \x20 a.Qdot = UA * (a.T - b.T)\n\
             \x20 b.Qdot = -a.Qdot\n\
             END\n\
             COMPONENT TSink(p)\n\
             \x20 PARAM tset = 300\n\
             \x20 p.T = tset\n\
             END\n\
             COMPONENT VSrc(p, n)\n\
             \x20 PARAM vset = 12\n\
             \x20 p.V - n.V = vset\n\
             \x20 p.I + n.I = 0\n\
             END\n\
             COMPONENT Rst(p, n)\n\
             \x20 PARAM R = 4\n\
             \x20 p.V - n.V = R * p.I\n\
             \x20 p.I + n.I = 0\n\
             END\n\
             COMPONENT Gnd(p)\n\
             \x20 p.V = 0\n\
             END\n\
             TSrc HOT()\n\
             Cond W()\n\
             TSink COLD()\n\
             connect(HOT.p, W.a)\n\
             connect(W.b, COLD.p)\n\
             VSrc B()\n\
             Rst R1()\n\
             Gnd G()\n\
             connect(B.p, R1.p)\n\
             connect(R1.n, B.n, G.p)\n",
            &[
                ("b.n.i", 3.0),
                ("b.n.v", 0.0),
                ("b.p.i", -3.0),
                ("b.p.v", 12.0),
                ("cold.p.qdot", 1000.0),
                ("cold.p.t", 300.0),
                ("g.p.i", 0.0),
                ("g.p.v", 0.0),
                ("hot.p.qdot", -1000.0),
                ("hot.p.t", 400.0),
                ("r1.n.i", -3.0),
                ("r1.n.v", 0.0),
                ("r1.p.i", 3.0),
                ("r1.p.v", 12.0),
                ("w.a.qdot", 1000.0),
                ("w.a.t", 400.0),
                ("w.b.qdot", -1000.0),
                ("w.b.t", 300.0),
            ],
        );
    }

    #[test]
    fn oracle_document_f3_hierarchical_subsystem() {
        assert_table(
            "COMPONENT Res(in, out)\n\
             \x20 PARAM k = 2\n\
             \x20 out.mdot = in.mdot\n\
             \x20 out.P = in.P - k * in.mdot\n\
             \x20 out.h = in.h\n\
             END\n\
             COMPONENT Duo(a, b)\n\
             \x20 PARAM k = 3\n\
             \x20 Res R1(k = k)\n\
             \x20 Res R2(k = k / 3)\n\
             \x20 connect(a, R1.in)\n\
             \x20 connect(R1.out, R2.in)\n\
             \x20 connect(R2.out, b)\n\
             END\n\
             COMPONENT Src(out)\n\
             \x20 out.P = 400\n\
             \x20 out.h = 5\n\
             \x20 out.mdot = 2\n\
             END\n\
             COMPONENT Snk(in)\n\
             \x20 W = in.mdot * in.h + in.P\n\
             END\n\
             Src S()\n\
             Duo D(k = 6)\n\
             Snk K()\n\
             connect(S.out, D.a)\n\
             connect(D.b, K.in)\n",
            &[
                ("d.a.h", 5.0),
                ("d.a.mdot", 2.0),
                ("d.a.p", 400.0),
                ("d.b.p", 384.0),
                ("d.r1.in.p", 400.0),
                ("d.r1.out.p", 388.0),
                ("d.r2.in.p", 388.0),
                ("d.r2.out.p", 384.0),
                ("k.in.p", 384.0),
                ("k.w", 394.0),
                ("s.out.p", 400.0),
            ],
        );
    }

    #[test]
    fn oracle_document_f6_signal_broadcast() {
        assert_table(
            "COMPONENT Step(out)\n\
             \x20 PARAM v = 5\n\
             \x20 out.sig = v\n\
             END\n\
             COMPONENT Gain(in, out)\n\
             \x20 PARAM g = 2\n\
             \x20 out.sig = g * in.sig\n\
             END\n\
             COMPONENT Probe(in)\n\
             \x20 y = in.sig\n\
             END\n\
             Step S()\n\
             Gain G()\n\
             Probe P1()\n\
             Probe P2()\n\
             connect(S.out, G.in)\n\
             connect(G.out, P1.in, P2.in)\n",
            &[
                ("g.in.sig", 5.0),
                ("g.out.sig", 10.0),
                ("p1.in.sig", 10.0),
                ("p1.y", 10.0),
                ("p2.in.sig", 10.0),
                ("p2.y", 10.0),
                ("s.out.sig", 5.0),
            ],
        );
    }

    #[test]
    fn oracle_document_k3_nested_subsystems() {
        assert_table(
            "COMPONENT Res(in, out)\n\
             \x20 PARAM k = 2\n\
             \x20 out.mdot = in.mdot\n\
             \x20 out.P = in.P - k * in.mdot\n\
             \x20 out.h = in.h\n\
             END\n\
             COMPONENT Duo(a, b)\n\
             \x20 PARAM k = 3\n\
             \x20 Res R1(k = k)\n\
             \x20 Res R2(k = k)\n\
             \x20 connect(a, R1.in)\n\
             \x20 connect(R1.out, R2.in)\n\
             \x20 connect(R2.out, b)\n\
             END\n\
             COMPONENT Quad(a, b)\n\
             \x20 PARAM k = 1\n\
             \x20 Duo D1(k = k)\n\
             \x20 Duo D2(k = k * 2)\n\
             \x20 connect(a, D1.a)\n\
             \x20 connect(D1.b, D2.a)\n\
             \x20 connect(D2.b, b)\n\
             END\n\
             COMPONENT Src(out)\n\
             \x20 out.P = 400\n\
             \x20 out.h = 5\n\
             \x20 out.mdot = 2\n\
             END\n\
             COMPONENT Snk(in)\n\
             \x20 W = in.mdot + in.P + in.h\n\
             END\n\
             Src S()\n\
             Quad Q(k = 1)\n\
             Snk K()\n\
             connect(S.out, Q.a)\n\
             connect(Q.b, K.in)\n",
            &[
                ("k.in.p", 388.0),
                ("k.w", 395.0),
                ("q.a.p", 400.0),
                ("q.b.p", 388.0),
                ("q.d1.a.p", 400.0),
                ("q.d1.b.p", 396.0),
                ("q.d1.r1.out.p", 398.0),
                ("q.d1.r2.out.p", 396.0),
                ("q.d2.a.p", 396.0),
                ("q.d2.r1.out.p", 392.0),
                ("q.d2.r2.out.p", 388.0),
                ("q.d2.b.p", 388.0),
                ("s.out.p", 400.0),
            ],
        );
    }

    #[test]
    fn oracle_document_h3_mixed_binding_styles() {
        assert_table(
            "COMPONENT Pipe(in, out)\n\
             \x20 PARAM k = 1\n\
             \x20 out.mdot = in.mdot\n\
             \x20 out.P = in.P - k * in.mdot\n\
             \x20 out.h = in.h\n\
             END\n\
             COMPONENT Src(out)\n\
             \x20 out.P = 100\n\
             \x20 out.h = 5\n\
             \x20 out.mdot = 2\n\
             END\n\
             COMPONENT Snk(in)\n\
             \x20 y = in.mdot + in.P + in.h\n\
             END\n\
             Src S(s1)\n\
             Pipe A(s1, s2)\n\
             Snk K()\n\
             connect(s2, K.in)\n",
            &[
                ("k.in.h", 5.0),
                ("k.in.mdot", 2.0),
                ("k.in.p", 98.0),
                ("k.y", 105.0),
                ("s1.h", 5.0),
                ("s1.mdot", 2.0),
                ("s1.p", 100.0),
                ("s2.h", 5.0),
                ("s2.mdot", 2.0),
                ("s2.p", 98.0),
            ],
        );
    }

    /// The rejections, checked on the oracle's own source text end to end.
    fn parse_expand_err(source: &str) -> String {
        let mut doc =
            crate::parser::parse_document(source).unwrap_or_else(|e| panic!("parse failed: {e}"));
        let components = std::mem::take(&mut doc.components);
        let statements = std::mem::take(&mut doc.statements);
        let mut display: BTreeMap<String, String> = std::mem::take(&mut doc.display_names);
        let builtins: Vec<ComponentDef> = Vec::new();
        let result = (|| -> Result<()> {
            let mut ex = ComponentExpander::new(
                &builtins,
                &components.defs,
                &components.instances,
                &components.connects,
                &mut display,
            )?;
            ex.expand()?;
            ex.rewrite_statements(statements)?;
            Ok(())
        })();
        match result {
            Err(FreesError::Parse { message, .. }) => message,
            Err(other) => panic!("expected a parse error, got {other:?}"),
            Ok(()) => panic!("expected a parse error, but the document expanded"),
        }
    }

    #[test]
    fn oracle_rejections_reproduce_verbatim_end_to_end() {
        // corpus/e10_domain_mix.frees
        assert_eq!(
            parse_expand_err(
                "COMPONENT Src(out)\n  out.P = 100\n  out.h = 5\n  out.mdot = 1\nEND\n\
                 COMPONENT TSrc(p)\n  p.T = 400\n  p.Qdot = 10\nEND\n\
                 Src S()\nTSrc H()\nconnect(S.out, H.p)\n"
            ),
            "connect(s.out, h.p): cannot connect a fluid port (s.out) to a heat port \
             (h.p) — different physical domains. Couple domains through a transducer \
             component (a motor, pump, heating resistor, …), not a direct connect."
        );
        // corpus/e12_shared_connector_type.frees
        assert_eq!(
            parse_expand_err(
                "COMPONENT GSrc(out)\n  PARAM domain$ = gas\n  out.P = 100\n  out.h = 5\n  \
                 out.mdot = 1\nEND\n\
                 COMPONENT OSnk(in)\n  PARAM domain$ = oil\n  W = in.mdot * in.h + in.P\nEND\n\
                 GSrc G(s1)\nOSnk O(s1)\n"
            ),
            "Incompatible fluid connector types on stream 's1': 'gas' and 'oil' bound \
             to the same stream. Pneumatic ('gas'), hydraulic ('oil') and thermofluid \
             ('fluid') lines are different connector types and cannot share a port."
        );
        // corpus/e14_unknown_port_in_body.frees
        assert_eq!(
            parse_expand_err(
                "COMPONENT Pipe(in, out)\n  out.mdot = zz.mdot\nEND\nPipe A(s1, s2)\n"
            ),
            "Component 'pipe': 'zz.mdot' references unknown port 'zz'. Ports: in, out."
        );
        // corpus/f1_highindex.frees
        assert_eq!(
            parse_expand_err(
                "COMPONENT Mass(p)\n  PARAM C = 10\n  der(p.T) = p.Qdot / C\n  \
                 init(p.T) = 300\nEND\nMass M1()\nMass M2()\nconnect(M1.p, M2.p)\n"
            ),
            "High-index DAE: storage states 'm1.p.t' and 'm2.p.t' are rigidly coupled \
             (directly equated) — index ≥ 2. Lump them into one storage element, or \
             insert a small resistance/compliance between them."
        );
        // corpus/f2_cc.frees
        assert_eq!(
            parse_expand_err(
                "COMPONENT Vol(a, b)\n  PARAM V = 1\n  der(a.P) = (a.mdot + b.mdot) / V\n  \
                 init(a.P) = 100\n  b.P = a.P\n  b.h = a.h\nEND\n\
                 COMPONENT Src(out)\n  out.P = 200\n  out.h = 5\n  out.mdot = 1\nEND\n\
                 Vol V1()\nVol V2()\nSrc S()\nconnect(S.out, V1.a)\nconnect(V1.b, V2.a)\n"
            ),
            "connect(...): capacitive volumes [v1, v2] are connected directly with no \
             resistance between them (C-C). Two pressure-storage volumes at one node \
             make the DAE index-2; interpose a resistive flow element between them \
             (the C-R-C rule)."
        );
        // corpus/e15_mass_dir_unknown.frees — the message quotes the declaration
        // text, which the real parser supplies here rather than a test stub.
        let msg = parse_expand_err(
            "COMPONENT Src(a)\n  a.P = 100\n  a.h = 5\n  a.mdot = 1\nEND\n\
             COMPONENT Snk(b)\n  W = b.mdot * b.h + b.P\nEND\n\
             Src S()\nSnk K1()\nSnk K2()\nconnect(S.a, K1.b, K2.b)\n",
        );
        assert!(
            msg.starts_with(
                "connect(...): cannot tell whether 's.a' is an inlet or an outlet for \
                 the mass balance"
            ),
            "{msg}"
        );
    }

    // =====================================================================
    // 20. Against the SHIPPED library — real components, no hand-written defs
    //
    //     `library::builtins()` parses the 295 vendored `.frees` components
    //     with the ordinary front end; the document below names them and
    //     nothing else. These are the tightest checks available short of the
    //     full `EquationParser` wiring: the only thing not shipping code is the
    //     three-line glue in `parse_expand_solve_using`.
    // =====================================================================

    #[test]
    fn a_real_electrical_circuit_from_the_shipped_library_solves() {
        // 12 V across 4 Ω + 8 Ω in series ⇒ 1 A, mid-node at 8 V.
        // Oracle table for exactly this document.
        assert_library_table(
            "VoltageSource B(E = 12)\n\
             Resistor R1(R = 4)\n\
             Resistor R2(R = 8)\n\
             Ground G()\n\
             \n\
             connect(B.p, R1.a)\n\
             connect(R1.b, R2.a)\n\
             connect(R2.b, B.n, G.port)\n",
            &[
                ("b.n.i", 1.0),
                ("b.n.v", 0.0),
                ("b.p.i", -1.0),
                ("b.p.v", 12.0),
                ("g.port.i", 0.0),
                ("g.port.v", 0.0),
                ("r1.a.i", 1.0),
                ("r1.a.v", 12.0),
                ("r1.b.i", -1.0),
                ("r1.b.v", 8.0),
                ("r2.a.i", 1.0),
                ("r2.a.v", 8.0),
                ("r2.b.i", -1.0),
                ("r2.b.v", 0.0),
            ],
        );
    }

    #[test]
    fn a_real_signal_chain_from_the_shipped_library_solves() {
        // 3 → ×5 → 15, summed with 7 ⇒ 22, across a broadcast node.
        assert_library_table(
            "SigConstant K(k = 3)\n\
             SigGain G(k = 5)\n\
             SigConstant K2(k = 7)\n\
             SigSum S()\n\
             \n\
             connect(K.out, G.in)\n\
             connect(G.out, S.in1)\n\
             connect(K2.out, S.in2)\n",
            &[
                ("g.in.sig", 3.0),
                ("g.out.sig", 15.0),
                ("k.out.sig", 3.0),
                ("k2.out.sig", 7.0),
                ("s.in1.sig", 15.0),
                ("s.in2.sig", 7.0),
                ("s.out.sig", 22.0),
            ],
        );
    }

    #[test]
    fn a_real_rotational_network_from_the_shipped_library_solves() {
        // 10 N·m into a 5 N·m·s/rad damper against ground ⇒ 2 rad/s.
        assert_library_table(
            "TorqueSource T(T = 10)\n\
             RotationalDamper D(c = 5)\n\
             MechGround G()\n\
             MechGround G2()\n\
             \n\
             connect(T.a, D.a)\n\
             connect(D.b, G.port)\n\
             connect(T.b, G2.port)\n",
            &[
                ("d.a.tau", 10.0),
                ("d.a.w", 2.0),
                ("d.b.tau", -10.0),
                ("d.b.w", 0.0),
                ("g.port.tau", 10.0),
                ("g.port.w", 0.0),
                ("g2.port.tau", -10.0),
                ("g2.port.w", 0.0),
                ("t.a.tau", -10.0),
                ("t.a.w", 2.0),
                ("t.b.tau", 10.0),
                ("t.b.w", 0.0),
            ],
        );
    }

    #[test]
    fn every_shipped_component_instantiates_and_expands() {
        // A blunt but broad smoke test: instantiate each of the 295 built-ins on
        // its own with free ports, and require that expansion either succeeds or
        // fails with a diagnostic that NAMES the component — never a panic, and
        // never an anonymous message. Most failures here are the deliberate
        // "parameter has no value" rejection, since the library gives physical
        // inputs no defaults on purpose.
        let library = crate::components::library::builtins().expect("the built-in library");
        let mut expanded = 0usize;
        let mut demanded_parameters = 0usize;
        for builtin in library.iter() {
            let name = &builtin.def.name;
            let insts = vec![ComponentInst {
                type_name: name.clone(),
                name: "u1".into(),
                port_args: vec![],
                params: ParamOverrides::new(),
                source_text: format!("{name} U1()"),
            }];
            let mut display = BTreeMap::new();
            let result = (|| -> Result<usize> {
                let mut ex =
                    ComponentExpander::new(library.defs(), &[], &insts, &[], &mut display)?;
                Ok(ex.expand()?.len())
            })();
            match result {
                Ok(n) => {
                    expanded += 1;
                    // A component with a body must produce equations.
                    if !builtin.def.body.is_empty() || !builtin.def.variants.is_empty() {
                        assert!(n > 0, "{name} expanded to nothing");
                    }
                }
                Err(FreesError::Parse { message, .. }) => {
                    assert!(
                        message.contains("'u1'") || message.contains(&format!("'{name}'")),
                        "{name}: rejection names neither the instance nor the component: \
                         {message}"
                    );
                    if message.contains("has no value") {
                        demanded_parameters += 1;
                    }
                }
                Err(other) => panic!("{name}: unexpected error kind {other:?}"),
            }
        }
        assert_eq!(
            expanded + demanded_parameters,
            library.len(),
            "every built-in should either expand or demand a parameter"
        );
        assert!(expanded > 0 && demanded_parameters > 0);
    }
}
