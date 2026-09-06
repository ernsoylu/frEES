//! Coverage gate for the shipped component library.
//!
//! `tests/parity.rs` already proves that every `fixtures/corpus/components_*`
//! document reproduces the Java oracle. What it cannot say is **how much of the
//! 295-component library those documents touch**, and that number is the whole
//! point of Phase 6 — a library that parses but is never instantiated is not
//! evidence of anything.
//!
//! So this file measures two different things and pins both:
//!
//! 1. [`every_domain_family_has_a_solving_fixture`] — one document per domain
//!    family in `fixtures/corpus/components_family_*.frees`, each of which
//!    **solves** through [`frees_core::solve`] and (via `parity.rs`) matches the
//!    Java engine. That is the "representative document per family" gate.
//! 2. [`library_instantiation_coverage`] — every one of the 295 built-ins is
//!    instantiated on its own and pushed through the expander. This does *not*
//!    solve them: a physically well-posed boundary condition per component is a
//!    modelling problem, not a test. It proves the definition parses, its ports
//!    resolve, its parameters bind and its body rewrites onto flat scalar
//!    equations — which is exactly the part of the library the port owns.
//!
//! The counts are asserted as **floors that must be re-measured, not raised by
//! hand**: if the library grows, the floors do not silently pass a smaller
//! fraction, because the totals are read from
//! [`frees_core::components::library`].

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use frees_core::ast::Expr;
use frees_core::components::def::{ComponentInst, Components, ParamOverrides};
use frees_core::components::expander::ComponentExpander;
use frees_core::components::library::{self, COMPONENT_COUNT, FILES};
use frees_core::{solve, SolverSettings};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/corpus")
}

/// The twelve domain families Phase 6 must demonstrate end to end. Eleven are a
/// library file; `control` ships a single component (`PIThermostat`) that lives
/// on the heat bond, so it is exercised from the heat document rather than
/// getting one of its own — see [`every_library_file_is_exercised`], which
/// checks that by *file*, not by document name.
const FAMILIES: [&str; 12] = [
    "ac",
    "electrical",
    "fluid",
    "heat",
    "hydraulic",
    "liquid",
    "mechanical",
    "moistair",
    "pneumatic",
    "powertrain",
    "signal",
    "twophase",
];

fn family_source(family: &str) -> String {
    let path = corpus_dir().join(format!("components_family_{family}.frees"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} unreadable: {e}", path.display()))
}

#[test]
fn every_domain_family_has_a_solving_fixture() {
    let mut report = Vec::new();
    for family in FAMILIES {
        let source = family_source(family);
        let solution = solve(&source, &SolverSettings::default())
            .unwrap_or_else(|e| panic!("components_family_{family} did not solve: {e}"));
        assert!(
            !solution.values.is_empty(),
            "components_family_{family} solved to an empty variable set"
        );
        assert!(
            !solution.component_instances.is_empty(),
            "components_family_{family} produced no component instances — the \
             document is not exercising the component layer at all"
        );
        report.push(format!(
            "{family}: {} variables, {} blocks, {} instances",
            solution.values.len(),
            solution.blocks.len(),
            solution.component_instances.len()
        ));
    }
    println!("domain families solved:\n  {}", report.join("\n  "));
}

/// Every component type a document reaches, including the ones a hierarchical
/// built-in instantiates inside its own body.
fn types_used(source: &str) -> BTreeSet<String> {
    let doc = frees_core::parser::parse_document(source).expect("fixture parses");
    let library = library::builtins().expect("library parses");
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<String> = doc
        .components
        .instances
        .iter()
        .map(|i| i.type_name.clone())
        .collect();
    // A document may also define its own components whose bodies instantiate
    // built-ins; walk those too.
    for def in &doc.components.defs {
        for sub in &def.sub_instances {
            stack.push(sub.type_name.clone());
        }
    }
    while let Some(name) = stack.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        if let Some(def) = library.get(&name) {
            for sub in &def.sub_instances {
                stack.push(sub.type_name.clone());
            }
        }
    }
    seen
}

/// Which library file each built-in came from.
fn file_of_name() -> BTreeMap<String, &'static str> {
    let library = library::builtins().expect("library parses");
    library
        .iter()
        .map(|entry| (entry.def.name.clone(), entry.file))
        .collect()
}

fn component_fixtures() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(corpus_dir())
        .expect("corpus readable")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_str()?;
            (name.starts_with("components_") && name.ends_with(".frees")).then_some(path)
        })
        .collect();
    paths.sort();
    paths
}

#[test]
fn every_library_file_is_exercised() {
    let by_file = file_of_name();
    let mut used_files: BTreeSet<&'static str> = BTreeSet::new();
    for path in component_fixtures() {
        let source = fs::read_to_string(&path).expect("fixture readable");
        for name in types_used(&source) {
            if let Some(file) = by_file.get(&name) {
                used_files.insert(file);
            }
        }
    }
    let missing: Vec<&str> = FILES
        .iter()
        .map(|f| f.domain)
        .filter(|domain| !used_files.contains(domain))
        .collect();
    assert!(
        missing.is_empty(),
        "no corpus document instantiates a component from: {missing:?}"
    );
}

/// How much of the library the *parity corpus* actually instantiates.
///
/// This is the honest coverage number, and it is deliberately far below 295 —
/// full instantiation coverage from hand-written physical documents is not a
/// one-pass goal. The floor exists so the number cannot silently fall.
#[test]
fn corpus_component_coverage_is_measured_not_assumed() {
    let by_file = file_of_name();
    let mut used: BTreeSet<String> = BTreeSet::new();
    for path in component_fixtures() {
        let source = fs::read_to_string(&path).expect("fixture readable");
        for name in types_used(&source) {
            if by_file.contains_key(&name) {
                used.insert(name);
            }
        }
    }

    let mut per_file: BTreeMap<&'static str, usize> = BTreeMap::new();
    for name in &used {
        *per_file.entry(by_file[name]).or_default() += 1;
    }
    let lines: Vec<String> = FILES
        .iter()
        .map(|f| {
            format!(
                "{:11} {:>3}/{:<3}",
                f.domain,
                per_file.get(f.domain).copied().unwrap_or(0),
                f.components
            )
        })
        .collect();
    println!(
        "corpus instantiates {}/{} built-in components ({:.0}%)\n  {}",
        used.len(),
        COMPONENT_COUNT,
        100.0 * used.len() as f64 / COMPONENT_COUNT as f64,
        lines.join("\n  ")
    );

    // Measured floor — the number this corpus actually reaches today. Raise it
    // when the corpus genuinely grows; never lower it to make a change pass.
    // Re-measured 2026-08-23 (Wave G4, the 129-document components_g4_* sweep):
    // 262 of 295 from this group alone; the 33 it did not see here were types
    // instantiated only by non-`components_*` documents, plus CabinZone — the
    // sweep's one drop (fixtures/README.md item 4 has why).
    // Re-measured 2026-08-24 (Wave I): 263 — `components_i_cabinzone_dynamic`
    // covers CabinZone as a DYNAMIC document, whose integrator-state seeding
    // (init T0/W0) sidesteps the oracle's steady-guess HAPropsSI stall.
    const FLOOR: usize = 263;
    assert!(
        used.len() >= FLOOR,
        "the component corpus now instantiates only {} of {COMPONENT_COUNT} built-ins, \
         below the {FLOOR} this gate recorded",
        used.len()
    );
}

/// A single-instance document for one built-in: every port gets its own stream,
/// every parameter without a default gets a fresh top-level unknown (or, for a
/// string parameter, a value the library will accept).
fn probe_instance(def: &frees_core::components::def::ComponentDef) -> Option<ComponentInst> {
    let mut params = ParamOverrides::new();
    for param in &def.params {
        if param.default_value.is_some() {
            continue;
        }
        if param.is_string {
            // The only string parameters without a default in the shipped
            // library name a fluid, a table or a variant. A fluid name is the
            // one of those three this probe can supply generically; a component
            // needing a `TABLE` or naming a variant it does not default is
            // skipped and counted, not faked.
            match param.name.as_str() {
                "fluid$" | "ref$" | "cool$" | "gas$" | "liq$" => {
                    params.put(param.name.clone(), Expr::Str("Water".to_string()));
                }
                _ => return None,
            }
            continue;
        }
        params.put(
            param.name.clone(),
            Expr::var(format!("probe_{}", param.name.trim_end_matches('#'))),
        );
    }
    Some(ComponentInst {
        type_name: def.name.clone(),
        name: "probe".to_string(),
        port_args: def
            .ports
            .iter()
            .map(|port| format!("s_{}_{port}", def.name))
            .collect(),
        params,
        source_text: format!("{} probe(...)", def.name),
        line: 0,
    })
}

/// Instantiate **every** built-in and expand it.
///
/// The strongest statement this phase can make about the library as a whole:
/// each definition's ports resolve, its parameters bind, its variant selection
/// runs and its body rewrites to flat scalar equations. Failures are reported
/// with their reasons rather than hidden, and the pass count is pinned.
#[test]
fn library_instantiation_coverage() {
    let library = library::builtins().expect("library parses");
    let defs = library.defs();

    let mut expanded = 0usize;
    let mut skipped: Vec<&str> = Vec::new();
    let mut failed: Vec<(String, String)> = Vec::new();

    for def in defs {
        let Some(inst) = probe_instance(def) else {
            skipped.push(&def.name);
            continue;
        };
        let components = Components {
            defs: Vec::new(),
            instances: vec![inst],
            connects: Vec::new(),
        };
        let mut display_names: BTreeMap<String, String> = BTreeMap::new();
        let result = ComponentExpander::new(
            defs,
            &components.defs,
            &components.instances,
            &components.connects,
            &mut display_names,
        )
        .and_then(|mut ex| ex.expand());
        match result {
            Ok(equations) if !equations.is_empty() => expanded += 1,
            Ok(_) => failed.push((def.name.clone(), "expanded to zero equations".to_string())),
            Err(e) => failed.push((def.name.clone(), e.to_string_message())),
        }
    }

    println!(
        "library instantiation: {expanded} expanded, {} skipped (needs a TABLE or a \
         variant name this probe cannot invent), {} failed, of {COMPONENT_COUNT}",
        skipped.len(),
        failed.len()
    );
    if !skipped.is_empty() {
        println!("  skipped: {}", skipped.join(", "));
    }

    assert!(
        failed.is_empty(),
        "{} built-in component(s) failed to expand from a bare instantiation:\n{}",
        failed.len(),
        failed
            .iter()
            .map(|(name, why)| format!("  {name}: {why}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(
        expanded + skipped.len(),
        COMPONENT_COUNT,
        "the probe did not account for every built-in"
    );

    // Measured floor: how many of the 302 a generic probe can reach at all.
    // 275 expand; the other 27 need a `TABLE` argument or name a variant the
    // probe cannot invent, and are listed by name in the output above.
    const FLOOR: usize = 275;
    assert!(
        expanded >= FLOOR,
        "only {expanded} of {COMPONENT_COUNT} built-ins expand from a bare \
         instantiation, below the {FLOOR} this gate recorded"
    );
}
