//! The built-in standard component library — 312 components, embedded as
//! `.frees` text and parsed by the ordinary front end.
//!
//! Port of
//! `../frEES/backend/core/src/main/java/com/frees/backend/parser/ComponentLibrary.java`
//! (86 LOC).
//!
//! # The library is data, not code
//!
//! The Java keeps the standard library in frEES source — 13 per-domain
//! `resources/components/*.frees` files — rather than as hand-built ASTs,
//! because that "keeps the physics transparent and editable, and exercises the
//! same parse path as user-authored components". This port keeps that property
//! exactly: the same 13 files are vendored verbatim under `library-data/`,
//! embedded with `include_str!` ([`FILES`]), and handed to the ordinary
//! [`parse_document`] front end. There is no second grammar here and not one
//! hand-translated component. A component that fails to parse is a grammar bug,
//! not a licence to rewrite the component.
//!
//! # Parse order and assembly
//!
//! The Java concatenates the files in a fixed order (`DOMAIN_FILES`), separating
//! each with `"\n\n"`, and parses the single resulting string once. [`source`]
//! reproduces that string byte for byte. [`Library::load`] instead parses **one
//! file at a time**, so every definition can carry the domain file it came from
//! — the source file the inventory owes the metadata/UI layer. The two are
//! equivalent (a `COMPONENT … END` block never spans a file boundary) and
//! `tests::per_file_parsing_matches_the_javas_single_concatenated_parse` pins
//! that they stay so.
//!
//! # Lookup semantics (checked against the Java, not assumed)
//!
//! * **Names are stored lowercase.** `AstBuilder.buildComponentDef` does
//!   `ctx.IDENT().getText().toLowerCase()`, and `buildComponentInst` lowercases
//!   the instantiated type the same way, so lookup is case-insensitive. This
//!   port lowercases with `to_ascii_lowercase`, the convention already used for
//!   identifiers throughout the crate.
//! * **A user definition shadows a built-in of the same name, silently.**
//!   `ComponentExpander`'s constructor loads `builtinDefs` into a
//!   `LinkedHashMap` first and then `put`s every user definition over the top —
//!   "built-in standard-library components are curated; a user definition of the
//!   same name overrides the built-in".
//! * **Two *user* definitions of one name are a hard error** —
//!   `"COMPONENT '<name>' is defined more than once."` Two *built-ins* of one
//!   name are not checked by the Java at all (the later one silently wins); the
//!   shipped library contains no duplicate, which the tests pin rather than
//!   trust.
//!
//! [`Library::resolve`] is that rule, in one place, for the expander to consume.
//!
//! # No defaults, by design
//!
//! Every library parameter is required at instantiation — the Java's class
//! comment is explicit that "a silent default for a physical input (a pipe
//! length, a fluid, an efficiency) is a footgun: a model of an R134a system that
//! forgets `fluid$` should error, not quietly run as water". The one exception
//! the text itself carries is `model$` (and the reserved connector-type selector
//! `domain$`), whose default selects a component's default physics variant.
//! Nothing here enforces that; it is a property of the data, and
//! `tests::library_parameters_carry_no_defaults_except_model` keeps it true.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::components::def::ComponentDef;
use crate::diag::{FreesError, Result};
use crate::parser::parse_document;

/// One embedded domain file of the standard library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LibraryFile {
    /// The domain name — the Java's resource stem (`/components/<domain>.frees`)
    /// and the file an inventory entry reports.
    pub domain: &'static str,
    /// The file's verbatim text, embedded with `include_str!`.
    pub text: &'static str,
    /// How many `COMPONENT` blocks the file declares.
    ///
    /// A **pinned expectation**, not a computed value: a file that silently
    /// loses a component (a bad re-vendor, a truncated copy, a grammar
    /// regression that swallows a block) fails the build instead of quietly
    /// shrinking the library.
    pub components: usize,
}

/// The 13 domain files, in the Java's `ComponentLibrary.DOMAIN_FILES` order.
///
/// The order is load-bearing twice over: it fixes the byte layout of
/// [`source`], and it fixes which definition would win if two files ever
/// declared the same component name (the later one — see the module docs).
pub static FILES: [LibraryFile; 13] = [
    LibraryFile {
        domain: "fluid",
        text: include_str!("library-data/fluid.frees"),
        components: 31,
    },
    LibraryFile {
        domain: "liquid",
        text: include_str!("library-data/liquid.frees"),
        components: 21,
    },
    LibraryFile {
        domain: "twophase",
        text: include_str!("library-data/twophase.frees"),
        components: 47,
    },
    LibraryFile {
        domain: "ac",
        text: include_str!("library-data/ac.frees"),
        components: 7,
    },
    LibraryFile {
        domain: "heat",
        text: include_str!("library-data/heat.frees"),
        components: 17,
    },
    LibraryFile {
        domain: "electrical",
        text: include_str!("library-data/electrical.frees"),
        components: 31,
    },
    LibraryFile {
        domain: "mechanical",
        text: include_str!("library-data/mechanical.frees"),
        components: 27,
    },
    LibraryFile {
        domain: "powertrain",
        text: include_str!("library-data/powertrain.frees"),
        components: 19,
    },
    LibraryFile {
        domain: "control",
        text: include_str!("library-data/control.frees"),
        components: 1,
    },
    LibraryFile {
        domain: "moistair",
        text: include_str!("library-data/moistair.frees"),
        components: 36,
    },
    LibraryFile {
        domain: "pneumatic",
        text: include_str!("library-data/pneumatic.frees"),
        components: 18,
    },
    LibraryFile {
        domain: "hydraulic",
        text: include_str!("library-data/hydraulic.frees"),
        components: 23,
    },
    LibraryFile {
        domain: "signal",
        text: include_str!("library-data/signal.frees"),
        components: 34,
    },
];

/// How many components the standard library exposes.
///
/// Was 295, matching the Java's own assertion — `LiquidDomainTest`:
/// `assertEquals(295, names.size(), "built-in component count after Program II
/// Wave 10")`. The seventeen added since are moist-air components with no Java
/// counterpart, so the port now leads the reference here rather than trailing
/// it: sensible air-to-air recovery, total-energy exchange, solid and liquid
/// desiccant, indirect and two-stage evaporative cooling, steam humidification,
/// the apparatus-dew-point and face-and-bypass coils, the heat-pipe wrap-around,
/// the airside economizer, four terminal units and a DOAS block.
pub const COMPONENT_COUNT: usize = 312;

/// The separator the Java appends after **every** file, including the last
/// (`ComponentLibrary.loadSource`).
const FILE_SEPARATOR: &str = "\n\n";

/// The concatenated library source — the Java's `ComponentLibrary.SOURCE`,
/// assembled once and shared.
///
/// Provided for parity checks and for anything that wants the library as one
/// document; [`Library::load`] parses per file instead, so it can attribute each
/// definition to its domain file.
pub fn source() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE.get_or_init(|| {
        let capacity = FILES
            .iter()
            .map(|f| f.text.len() + FILE_SEPARATOR.len())
            .sum();
        let mut out = String::with_capacity(capacity);
        for file in &FILES {
            out.push_str(file.text);
            out.push_str(FILE_SEPARATOR);
        }
        out
    })
}

/// The built-in component registry — parsed **once**, on first use, and shared
/// immutably from then on.
///
/// The Java parses it in a static initializer (`private static final
/// List<ComponentDef> BUILTINS = parse(SOURCE)`) and would abort class loading
/// on failure. A wasm build has no such ceremony and no way to report a panic
/// usefully, so the failure surfaces as an ordinary parse error naming the file
/// that broke — that is a grammar bug to fix, never something to work around by
/// editing a component.
pub fn builtins() -> Result<&'static Library> {
    static BUILTINS: OnceLock<Result<Library>> = OnceLock::new();
    BUILTINS
        .get_or_init(Library::load)
        .as_ref()
        .map_err(Clone::clone)
}

/// A built-in definition together with the domain file it came from — a
/// borrowed view into a [`Library`], not an owning record.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Builtin<'a> {
    /// The [`LibraryFile::domain`] this definition was parsed from.
    pub file: &'static str,
    /// The parsed definition.
    pub def: &'a ComponentDef,
}

/// The built-in component registry: every definition in declaration order, plus
/// a name index.
///
/// Cheap to consult and immutable once built — the Java parses the library once
/// into a shared, immutable list and hands the same instance to every request.
///
/// The definitions are held as one contiguous `Vec<ComponentDef>` so
/// [`Library::defs`] can hand `ComponentExpander::new` the `&[ComponentDef]` it
/// wants with no copying; the domain file each came from rides alongside in a
/// parallel vector.
#[derive(Debug, Clone)]
pub struct Library {
    defs: Vec<ComponentDef>,
    /// `files[i]` is the domain file `defs[i]` was parsed from.
    files: Vec<&'static str>,
    /// Lowercase name → index into `defs`. A later declaration wins, mirroring
    /// the `LinkedHashMap.put` loop in `ComponentExpander`'s constructor.
    by_name: HashMap<String, usize>,
}

impl Library {
    /// Parses all 13 embedded files through the ordinary front end.
    ///
    /// Prefer [`builtins`], which does this once and caches it; this is public
    /// for the parity harness and for tests that want a fresh copy.
    pub fn load() -> Result<Library> {
        Library::load_with(|text| parse_document(text).map(|doc| doc.components.defs))
    }

    /// [`Library::load`] with the front end injected — the seam the tests use to
    /// exercise the failure path without a broken component file.
    fn load_with(
        mut parse: impl FnMut(&'static str) -> Result<Vec<ComponentDef>>,
    ) -> Result<Library> {
        let mut entries = Vec::with_capacity(COMPONENT_COUNT);
        for file in &FILES {
            let defs = parse(file.text).map_err(|err| {
                FreesError::parse(format!(
                    "the built-in component library failed to parse ({}.frees): {err}",
                    file.domain
                ))
            })?;
            entries.extend(defs.into_iter().map(|def| (file.domain, def)));
        }
        Ok(Library::from_entries(entries))
    }

    /// Builds the registry over already-parsed `(domain file, definition)`
    /// pairs, in declaration order.
    pub fn from_entries(entries: Vec<(&'static str, ComponentDef)>) -> Library {
        let mut defs = Vec::with_capacity(entries.len());
        let mut files = Vec::with_capacity(entries.len());
        let mut by_name = HashMap::with_capacity(entries.len());
        for (index, (file, def)) in entries.into_iter().enumerate() {
            by_name.insert(def.name.clone(), index);
            defs.push(def);
            files.push(file);
        }
        Library {
            defs,
            files,
            by_name,
        }
    }

    /// The definition of `name`, or `None`.
    ///
    /// Case-insensitive: the language is, and both the definition and the
    /// instantiated type are lowercased by the AST builder.
    pub fn get(&self, name: &str) -> Option<&ComponentDef> {
        self.index_of(name).map(|index| &self.defs[index])
    }

    /// The definition of `name` together with its domain file, or `None`.
    pub fn entry(&self, name: &str) -> Option<Builtin<'_>> {
        self.index_of(name).map(|index| Builtin {
            file: self.files[index],
            def: &self.defs[index],
        })
    }

    fn index_of(&self, name: &str) -> Option<usize> {
        let key = name.to_ascii_lowercase();
        self.by_name.get(&key).copied()
    }

    /// The domain file `name` is defined in, or `None`.
    pub fn file_of(&self, name: &str) -> Option<&'static str> {
        self.index_of(name).map(|index| self.files[index])
    }

    /// Whether the library defines `name`.
    pub fn contains(&self, name: &str) -> bool {
        self.index_of(name).is_some()
    }

    /// How many components the library exposes.
    pub fn len(&self) -> usize {
        self.defs.len()
    }

    /// Whether the library is empty (never true for the shipped one).
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    /// Every entry, in declaration order (file order, then order within file).
    pub fn iter(&self) -> impl Iterator<Item = Builtin<'_>> {
        self.defs
            .iter()
            .zip(self.files.iter())
            .map(|(def, &file)| Builtin { file, def })
    }

    /// Every definition, in declaration order — the Java's
    /// `ComponentLibrary.builtins()`, which `EquationParser` hands straight to
    /// the `ComponentExpander` constructor.
    pub fn defs(&self) -> &[ComponentDef] {
        &self.defs
    }

    /// Every component name, in declaration order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.defs.iter().map(|def| def.name.as_str())
    }

    /// How many components came from the given domain file.
    pub fn count_in(&self, domain: &str) -> usize {
        self.files.iter().filter(|&&file| file == domain).count()
    }

    /// The library inventory the metadata/UI layer consumes: every component's
    /// name, source file, port names and parameter names, in declaration order.
    pub fn inventory(&self) -> Vec<ComponentInfo> {
        self.iter()
            .map(|entry| ComponentInfo {
                name: entry.def.name.clone(),
                file: entry.file,
                ports: entry.def.ports.clone(),
                params: entry
                    .def
                    .params
                    .iter()
                    .map(|param| param.name.clone())
                    .collect(),
            })
            .collect()
    }

    /// The lookup table a document actually sees: the built-ins, with the
    /// document's own `COMPONENT` definitions layered over the top.
    ///
    /// Mirrors the `ComponentExpander` constructor — a user definition shadows a
    /// built-in of the same name silently (the built-ins are curated, so
    /// overriding one is a deliberate act), while two user definitions of one
    /// name are a hard parse error carrying the Java's own message.
    ///
    /// It does not *mirror* it any more: it **is** it. This delegates to
    /// [`crate::components::expander::resolve_defs`], the copy the solve path
    /// runs, so the shadowing rule and its error text exist once.
    pub fn resolve<'a>(&'a self, user_defs: &'a [ComponentDef]) -> Result<DefTable<'a>> {
        Ok(DefTable {
            by_name: crate::components::expander::resolve_defs(&self.defs, user_defs)?,
        })
    }
}

/// A component's public description, for the metadata/UI layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInfo {
    /// Lowercase component name.
    pub name: String,
    /// The domain file it is defined in (`"twophase"`, `"signal"`, …).
    pub file: &'static str,
    /// Port names, in declaration order.
    pub ports: Vec<String>,
    /// Parameter names, in declaration order — declared `PARAM`s first, then the
    /// variant-scoped names promoted out of `VARIANT … REQUIRE …` clauses, as
    /// [`ComponentDef::new`] assembles them.
    pub params: Vec<String>,
}

/// The whole inventory of the built-in library: 295 entries in declaration
/// order. Fails only if the library does not parse.
pub fn inventory() -> Result<Vec<ComponentInfo>> {
    Ok(builtins()?.inventory())
}

/// Built-ins plus a document's own definitions, resolved into one lookup table.
///
/// See [`Library::resolve`].
#[derive(Debug, Clone)]
pub struct DefTable<'a> {
    by_name: HashMap<&'a str, &'a ComponentDef>,
}

impl<'a> DefTable<'a> {
    /// The definition bound to `name`, or `None`. Case-insensitive on the
    /// caller's side: a stored name is always lowercase, so a query that is not
    /// gets lowercased first.
    pub fn get(&self, name: &str) -> Option<&'a ComponentDef> {
        if name.bytes().any(|byte| byte.is_ascii_uppercase()) {
            let key = name.to_ascii_lowercase();
            self.by_name.get(key.as_str()).copied()
        } else {
            self.by_name.get(name).copied()
        }
    }

    /// Whether `name` resolves to a definition.
    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// How many distinct component names are visible.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// Whether no definition is visible.
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::token::TokenKind;
    use std::collections::HashSet;

    /// Parses component definitions out of a DSL snippet — the tests build their
    /// fixtures through the real front end rather than hand-assembling ASTs, for
    /// the same reason the library itself is text.
    fn defs_of(text: &str) -> Vec<ComponentDef> {
        parse_document(text)
            .unwrap_or_else(|error| panic!("test fixture failed to parse: {error}"))
            .components
            .defs
    }

    /// Counts `COMPONENT` block headers with the engine's **own lexer** — no
    /// second front end, and the three comment forms (`//`, `{…}`, `"…"`) are
    /// skipped for free, so the word "component" in prose never counts. This is
    /// what makes the count assertion independent of the parser: if the grammar
    /// ever silently swallows a block, the lexer count and the parsed count
    /// disagree and the build fails.
    fn count_component_blocks(text: &str) -> usize {
        tokenize(text)
            .expect("a built-in component file must lex")
            .iter()
            .filter(|token| token.kind == TokenKind::Component)
            .count()
    }

    // ── the count assertion: 295 total, and the per-file breakdown ───────────

    /// The assertion this phase owes, through the **parsed** library: both the
    /// total and the per-file breakdown, so a silently dropped component fails
    /// the build.
    #[test]
    fn the_library_exposes_exactly_295_components() {
        let library = builtins().expect("the built-in library must parse");
        assert_eq!(
            library.len(),
            COMPONENT_COUNT,
            "the standard library must expose exactly {COMPONENT_COUNT} components"
        );
        for file in &FILES {
            assert_eq!(
                library.count_in(file.domain),
                file.components,
                "{}.frees parsed to {} components, expected {}",
                file.domain,
                library.count_in(file.domain),
                file.components
            );
        }
    }

    /// The same two numbers taken off the **text** with the lexer alone. Belt to
    /// the parser's braces: together they mean a component can go missing
    /// neither from the vendored data nor from the grammar.
    #[test]
    fn every_domain_file_declares_the_pinned_number_of_components() {
        let mut total = 0;
        for file in &FILES {
            let found = count_component_blocks(file.text);
            assert_eq!(
                found, file.components,
                "{}.frees declares {found} components, expected {}",
                file.domain, file.components
            );
            total += found;
        }
        assert_eq!(total, COMPONENT_COUNT);
    }

    #[test]
    fn the_pinned_per_file_counts_sum_to_the_pinned_total() {
        let sum: usize = FILES.iter().map(|f| f.components).sum();
        assert_eq!(sum, COMPONENT_COUNT);
    }

    #[test]
    fn the_thirteen_domain_files_are_in_the_java_order() {
        let domains: Vec<&str> = FILES.iter().map(|f| f.domain).collect();
        // ComponentLibrary.DOMAIN_FILES, verbatim.
        assert_eq!(
            domains,
            vec![
                "fluid",
                "liquid",
                "twophase",
                "ac",
                "heat",
                "electrical",
                "mechanical",
                "powertrain",
                "control",
                "moistair",
                "pneumatic",
                "hydraulic",
                "signal",
            ]
        );
    }

    // ── source assembly (ComponentLibrary.loadSource) ───────────────────────

    #[test]
    fn source_concatenates_every_file_with_a_blank_line_after_each() {
        let assembled = source();
        let expected_len: usize = FILES.iter().map(|f| f.text.len() + 2).sum();
        assert_eq!(assembled.len(), expected_len);
        assert!(
            assembled.ends_with("\n\n"),
            "the Java appends the separator after every file"
        );
        let mut cursor = 0usize;
        for file in &FILES {
            let at = assembled[cursor..]
                .find(file.text)
                .unwrap_or_else(|| panic!("{}.frees is missing or out of order", file.domain));
            cursor += at + file.text.len();
        }
    }

    #[test]
    fn source_is_built_once_and_shared() {
        assert!(std::ptr::eq(source(), source()));
    }

    /// The Java parses the 13 files as one concatenated string; this port parses
    /// them one at a time so each definition knows its file. That is only safe
    /// while the two agree exactly — assert it rather than assume it.
    #[test]
    fn per_file_parsing_matches_the_javas_single_concatenated_parse() {
        let as_one = parse_document(source())
            .expect("the concatenated library source must parse")
            .components;
        let library = builtins().expect("the built-in library must parse");
        assert_eq!(as_one.defs.len(), library.len());
        for (whole, split) in as_one.defs.iter().zip(library.defs()) {
            assert_eq!(whole, split);
        }
        assert!(
            as_one.instances.is_empty() && as_one.connects.is_empty(),
            "the library declares templates only — no instantiations, no connects"
        );
    }

    #[test]
    fn the_library_declares_no_top_level_statements() {
        for file in &FILES {
            let doc = parse_document(file.text).expect("a built-in file must parse");
            assert!(
                doc.statements.is_empty(),
                "{}.frees has top-level statements outside its COMPONENT blocks",
                file.domain
            );
        }
    }

    // ── the parsed library ──────────────────────────────────────────────────

    #[test]
    fn builtins_is_parsed_once_and_shared() {
        let first = builtins().expect("must parse");
        let second = builtins().expect("must parse");
        assert!(std::ptr::eq(first, second));
    }

    #[test]
    fn every_name_is_lowercase_and_unique() {
        let library = builtins().unwrap();
        let mut seen = HashSet::new();
        for name in library.names() {
            assert_eq!(
                name,
                name.to_ascii_lowercase(),
                "component names are stored lowercase"
            );
            assert!(seen.insert(name), "the library declares '{name}' twice");
        }
        assert_eq!(seen.len(), COMPONENT_COUNT);
    }

    /// A sample of the names the Java's own `LiquidDomainTest` asserts are
    /// present, across seven domain files.
    #[test]
    fn the_named_components_the_java_test_checks_are_present() {
        let library = builtins().unwrap();
        for name in [
            "liquidsource",
            "liquidsink",
            "liquidpump",
            "liquidpipe",
            "liquidcoldplate",
            "liquidvolume",
            "liquidorifice",
            "pump",
            "heatexchanger",
            "gaspipe",
            "hydraulicpump",
            "coolingcoil",
            "fuelcellstack",
            "twophasepipe",
        ] {
            assert!(library.contains(name), "missing built-in '{name}'");
        }
    }

    #[test]
    fn lookup_is_case_insensitive() {
        let library = builtins().unwrap();
        assert!(library.contains("pump"));
        assert!(library.contains("Pump"));
        assert!(library.contains("PUMP"));
        assert_eq!(library.get("PuMp").map(|d| d.name.as_str()), Some("pump"));
        assert!(library.get("nosuchcomponent").is_none());
    }

    #[test]
    fn entries_carry_their_source_file() {
        let library = builtins().unwrap();
        assert_eq!(library.file_of("Pump"), Some("fluid"));
        assert_eq!(library.file_of("LiquidPump"), Some("liquid"));
        assert_eq!(library.file_of("PIThermostat"), Some("control"));
        assert_eq!(library.file_of("Chiller"), Some("ac"));
        assert_eq!(library.file_of("nosuchcomponent"), None);
    }

    #[test]
    fn definitions_keep_declaration_order_within_a_file() {
        let library = builtins().unwrap();
        let fluid: Vec<&str> = library
            .iter()
            .filter(|entry| entry.file == "fluid")
            .map(|entry| entry.def.name.as_str())
            .collect();
        // The first three COMPONENT blocks of fluid.frees, in order.
        assert_eq!(&fluid[..3], &["pump", "turbine", "compressor"]);
    }

    /// The Java's class comment: **no defaults — every parameter is required**,
    /// with the variant selector `model$` as the deliberate exception (it names
    /// the default physics model).
    #[test]
    fn library_parameters_carry_no_defaults_except_model() {
        let library = builtins().unwrap();
        for entry in library.iter() {
            for param in &entry.def.params {
                if param.default_value.is_some() {
                    assert!(
                        param.name == "model$" || param.name == "domain$",
                        "{} ({}.frees) gives '{}' a default",
                        entry.def.name,
                        entry.file,
                        param.name
                    );
                }
            }
        }
    }

    // ── the inventory ───────────────────────────────────────────────────────

    #[test]
    fn inventory_reports_name_file_ports_and_params_for_every_component() {
        let items = inventory().expect("the built-in library must parse");
        assert_eq!(items.len(), COMPONENT_COUNT);

        let pump = items.iter().find(|i| i.name == "pump").expect("Pump");
        assert_eq!(pump.file, "fluid");
        assert_eq!(pump.ports, vec!["in".to_string(), "out".to_string()]);
        assert_eq!(pump.params, vec!["eta".to_string(), "fluid$".to_string()]);

        // Compressor's parameters include the variant-scoped names promoted out
        // of its `VARIANT … REQUIRE …` clauses (all of which are explicit PARAMs
        // here) — and its ladder is what makes it worth pinning.
        let compressor = items
            .iter()
            .find(|i| i.name == "compressor")
            .expect("Compressor");
        assert_eq!(compressor.file, "fluid");
        assert_eq!(
            compressor.params,
            vec![
                "eta".to_string(),
                "fluid$".to_string(),
                "eta_v".to_string(),
                "disp".to_string(),
                "rpm".to_string(),
                "map_eta$".to_string(),
                "model$".to_string(),
            ]
        );
    }

    #[test]
    fn inventory_is_in_declaration_order_and_covers_every_file() {
        let items = inventory().unwrap();
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for item in &items {
            *counts.entry(item.file).or_default() += 1;
        }
        for file in &FILES {
            assert_eq!(counts.get(file.domain).copied(), Some(file.components));
        }
        // Files appear in one contiguous run each, in FILES order.
        let order: Vec<&str> = items.iter().map(|i| i.file).collect();
        let mut expected: Vec<&str> = Vec::with_capacity(COMPONENT_COUNT);
        for file in &FILES {
            expected.extend(std::iter::repeat_n(file.domain, file.components));
        }
        assert_eq!(order, expected);
    }

    #[test]
    fn every_inventory_entry_names_at_least_one_port() {
        // A component with no ports could not be connected to anything; a zero
        // here would mean the port list was dropped in parsing.
        for item in inventory().unwrap() {
            assert!(
                !item.ports.is_empty(),
                "{} ({}) has no ports",
                item.name,
                item.file
            );
        }
    }

    // ── shadowing (the ComponentExpander constructor rule) ──────────────────

    #[test]
    fn a_user_definition_shadows_a_builtin_of_the_same_name() {
        let library = builtins().unwrap();
        let user = defs_of("COMPONENT Pump(a, b)\n  PARAM k\n  a.mdot = k * b.mdot\nEND");
        let table = library.resolve(&user).expect("shadowing is not an error");
        assert_eq!(table.len(), COMPONENT_COUNT, "shadowing replaces, not adds");
        assert_eq!(
            table.get("Pump").map(|d| d.ports.as_slice()),
            Some(&["a".to_string(), "b".to_string()][..]),
            "the user's Pump wins"
        );
        assert_eq!(
            table.get("Turbine").map(|d| d.name.as_str()),
            Some("turbine"),
            "unshadowed built-ins are still visible"
        );
    }

    #[test]
    fn a_user_definition_of_a_new_name_is_added() {
        let library = builtins().unwrap();
        let user =
            defs_of("COMPONENT MyValve(in, out)\n  PARAM cv\n  out.mdot = cv * in.mdot\nEND");
        let table = library.resolve(&user).unwrap();
        assert_eq!(table.len(), COMPONENT_COUNT + 1);
        assert!(table.contains("MyValve"));
        assert!(table.contains("myvalve"));
        assert!(!table.is_empty());
    }

    #[test]
    fn two_user_definitions_of_one_name_are_a_hard_error() {
        let library = builtins().unwrap();
        let user = defs_of(
            "COMPONENT MyValve(in, out)\n  PARAM cv\n  out.mdot = cv * in.mdot\nEND\n\
             COMPONENT MyValve(a, b)\n  a.mdot = b.mdot\nEND",
        );
        assert_eq!(user.len(), 2, "both definitions reach the registry");
        let error = library.resolve(&user).unwrap_err();
        // The Java's message, verbatim.
        assert_eq!(
            error.to_string(),
            FreesError::parse("COMPONENT 'myvalve' is defined more than once.").to_string()
        );
    }

    #[test]
    fn resolving_with_no_user_definitions_is_just_the_builtins() {
        let library = builtins().unwrap();
        let table = library.resolve(&[]).unwrap();
        assert_eq!(table.len(), COMPONENT_COUNT);
        assert!(table.contains("HeatExchanger"));
        assert!(!table.contains("nosuchcomponent"));
    }

    // ── failure reporting ───────────────────────────────────────────────────

    #[test]
    fn load_reports_the_file_that_failed_to_parse() {
        let error = Library::load_with(|text| {
            if std::ptr::eq(text, FILES[2].text) {
                Err(FreesError::parse("synthetic grammar failure"))
            } else {
                Ok(Vec::new())
            }
        })
        .unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("twophase.frees") && message.contains("synthetic grammar failure"),
            "{message}"
        );
    }

    #[test]
    fn a_later_definition_of_a_name_wins_as_in_the_java_map() {
        // ComponentExpander loads the built-ins with LinkedHashMap.put, so a
        // second definition of one name replaces the first. The shipped library
        // has none (see `every_name_is_lowercase_and_unique`), but the registry
        // must not diverge from the Java if one ever appears.
        let def = |text: &str| defs_of(text).remove(0);
        let library = Library::from_entries(vec![
            (
                "fluid",
                def("COMPONENT Widget(in, out)\n  out.mdot = in.mdot\nEND"),
            ),
            (
                "liquid",
                def("COMPONENT Widget(a, b)\n  a.mdot = b.mdot\nEND"),
            ),
        ]);
        assert_eq!(library.file_of("widget"), Some("liquid"));
        assert_eq!(library.len(), 2, "both entries stay in the inventory");
        assert_eq!(library.defs().len(), 2);
    }
}
