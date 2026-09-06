//! Per-instance component metadata — the Variable Explorer's datasheet view.
//!
//! Port of `../frEES/backend/core/src/main/java/com/frees/backend/api/ComponentMetadata.java`
//! (147 lines).
//!
//! # What this layer is for
//!
//! A solved component network already ships its *outputs*: the port members
//! (`chlr.out.h`, `s2.p`, …) flow through as ordinary variables and the
//! frontend groups them by instance prefix. This supplies the other half — the
//! **inputs**, which exist only in the AST: the instance's type, and every
//! parameter it was built with (`UA=UA_chl_r`, `SH=5`, `fluid$=R1234yf`).
//!
//! Each parameter's bound expression is resolved against the solved variables,
//! so a binding like `UA=UA_chl_r` shows both the symbol and its value. Shared
//! bindings (one `UA_rad` used by several instances) resolve to the same value
//! under each — by design, so the sharing is visible.
//!
//! # Wire shape
//!
//! [`ComponentMeta`] / [`ComponentParamMeta`] are `SolveDtos.ComponentDto` /
//! `SolveDtos.ComponentParamDto`, which `web/src/api.ts` already types as
//! `ComponentResult` / `ComponentParamResult`:
//!
//! ```json
//! { "name": "CHLR", "type": "TwoPhaseEvaporatorUA",
//!   "params": [ { "name": "ua", "ref": "UA_chl_r", "value": 575.46, "units": "W/K" } ] }
//! ```
//!
//! Two field names are Rust keywords, so the boundary renames them on the way
//! out: [`ComponentMeta::type_name`] → `type` and
//! [`ComponentParamMeta::reference`] → `ref`. Nothing else is renamed, and
//! parameter names stay **lowercase** exactly as the AST stores them (`ua`,
//! `fluid$`) — the Java DTO does the same.
//!
//! # Seam with the expander
//!
//! The Java entry point re-parses the document text and reads
//! `ProgramResult.componentInsts()`; parsing `COMPONENT`/`connect` belongs to
//! [`crate::components::expander`] here, so [`build_from_instances`] takes the
//! already-parsed [`ComponentInst`]s instead. The Java "metadata never blocks a
//! solve" rule (a parse failure yields an empty list) becomes: a caller with
//! nothing to report passes an empty slice.
//!
//! Values come in as the boundary already renders them ([`VariableRow`] is
//! `SolveDtos.VariableDto`), i.e. **in the requested display unit system** —
//! `SolveController` passes its `variableDtos`, not the SI map.

use std::collections::BTreeMap;

use crate::ast::{BinOp, Expr};
use crate::components::def::ComponentInst;

/// One solved variable as the boundary renders it — the Java
/// `SolveDtos.VariableDto(name, value, units, uncertainty)`, minus the
/// uncertainty this layer never reads.
///
/// `name` carries the **display spelling** (`UA_chl_r`), because that is what a
/// resolved binding shows; lookups lowercase it.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableRow {
    pub name: String,
    pub value: f64,
    /// Display unit, `""` when the variable has none (the Java DTO's `units`
    /// uses the same empty-string-not-null convention).
    pub units: String,
}

impl VariableRow {
    pub fn new(name: impl Into<String>, value: f64, units: impl Into<String>) -> VariableRow {
        VariableRow {
            name: name.into(),
            value,
            units: units.into(),
        }
    }
}

/// One parsed `COMPONENT` instantiation, as much of `ast/ComponentInst.java` as
/// the datasheet needs: `Pump P1(s3, s4, eta=0.8, fluid$=Water)`.
///
/// Field conventions are the AST's, not the source's:
///
/// * `type_name` and `name` are **lowercase** (`AstBuilder.buildComponentInst`
///   lowercases both for case-insensitive registry lookup). Their original
///   spelling is recovered from `source_text` — see [`display_identity`].
/// * `params` are the `name=value` bindings in **declaration order** with
///   lowercase keys, i.e. the Java `LinkedHashMap<String, Expr>` flattened. A
///   repeated key must already have been collapsed the way that map does
///   (later value, earlier position); this layer reports what it is given.
/// * `source_text` is the instance's own text. The Java stores ANTLR's
///   `ctx.getText()`, which concatenates the child tokens **with the
///   whitespace dropped**; a verbatim source slice works here too (see
///   [`take_token`]).
///
/// Port arguments (`portArgs`) are deliberately absent: they are connectivity,
/// which the schematic payload reports, not datasheet inputs.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentInstMeta {
    pub type_name: String,
    pub name: String,
    pub source_text: String,
    pub params: Vec<(String, Expr)>,
    pub line: usize,
}

impl From<&ComponentInst> for ComponentInstMeta {
    /// The adapter from the expander's own AST node — the whole seam between
    /// this layer and [`crate::components::def`], kept on the consumer side so
    /// the AST owes the datasheet nothing.
    ///
    /// [`ParamOverrides::iter`](crate::components::def::ParamOverrides::iter) is
    /// insertion-ordered and already collapses a repeated key in place, which is
    /// exactly the `LinkedHashMap` contract [`ComponentInstMeta::params`]
    /// documents.
    fn from(inst: &ComponentInst) -> ComponentInstMeta {
        ComponentInstMeta {
            type_name: inst.type_name.clone(),
            name: inst.name.clone(),
            source_text: inst.source_text.clone(),
            params: inst
                .params
                .iter()
                .map(|(name, value)| (name.to_string(), value.clone()))
                .collect(),
            line: inst.line,
        }
    }
}

/// [`build`] straight off the expander's instance list — the call the wasm
/// boundary makes.
pub fn build_from_instances(insts: &[ComponentInst], vars: &[VariableRow]) -> Vec<ComponentMeta> {
    let metas: Vec<ComponentInstMeta> = insts.iter().map(ComponentInstMeta::from).collect();
    build(&metas, vars)
}

/// One parameter binding on a component — the Java
/// `SolveDtos.ComponentParamDto(name, ref, value, units)`.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentParamMeta {
    /// Parameter name, lowercase as the AST stores it (`ua`, `sh`, `fluid$`).
    pub name: String,
    /// The bound expression as written: a variable name, a literal, or a
    /// source-like rendering of a compound expression. Serialized as `ref`.
    pub reference: String,
    /// Resolved value when the binding is a variable with a solved value or a
    /// numeric literal; `None` for strings and unevaluated expressions.
    pub value: Option<f64>,
    /// Unit of [`ComponentParamMeta::value`], when one is known.
    pub units: Option<String>,
}

/// A solved component instance: its identity and the parameters it was built
/// with — the Java `SolveDtos.ComponentDto(name, type, params)`.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentMeta {
    /// Instance name in its original source spelling (`CHLR`).
    pub name: String,
    /// Component type in its original source spelling (`TwoPhaseEvaporatorUA`).
    /// Serialized as `type`.
    pub type_name: String,
    pub params: Vec<ComponentParamMeta>,
}

/// One [`ComponentMeta`] per component instance, in declaration order.
///
/// Port of `ComponentMetadata.build`. `vars` are the solved variables the
/// bindings resolve against; the first row of any repeated (case-insensitive)
/// name wins, mirroring the Java `putIfAbsent`.
pub fn build(insts: &[ComponentInstMeta], vars: &[VariableRow]) -> Vec<ComponentMeta> {
    if insts.is_empty() {
        return Vec::new();
    }
    // Index solved variables by lowercase display name for binding lookups
    // (frees names are case-insensitive; `Expr::Var` names are already
    // lowercase).
    let mut by_name: BTreeMap<String, &VariableRow> = BTreeMap::new();
    for v in vars {
        by_name.entry(v.name.to_lowercase()).or_insert(v);
    }

    let mut out = Vec::with_capacity(insts.len());
    for inst in insts {
        let params = inst
            .params
            .iter()
            .map(|(name, value)| resolve_param(name, value, &by_name))
            .collect();
        let (type_name, name) = display_identity(inst);
        out.push(ComponentMeta {
            name,
            type_name,
            params,
        });
    }
    out
}

/// Recovers the original camelCase spelling of the type (`TwoPhaseEvaporatorUA`)
/// and of the instance name (`CHLR`), which the AST lowercases for
/// case-insensitive registry lookup.
///
/// Port of `ComponentMetadata.displayType` + `displayName`: the type is the
/// leading run of `type.length()` characters of the instance source and the
/// name the run right after it, each trusted only when it matches the
/// lowercased token — a guard against an unexpected source shape (e.g.
/// hierarchical sub-instances) that falls back to the lowercased token.
pub(crate) fn display_identity(inst: &ComponentInstMeta) -> (String, String) {
    let src = inst.source_text.as_str();
    let (type_name, after_type) = match take_token(src, 0, inst.type_name.len()) {
        Some((slice, end)) if slice.eq_ignore_ascii_case(&inst.type_name) => {
            (slice.to_string(), end)
        }
        // Java resumes at `type.length()` whether or not the type slice was
        // trusted; so does this.
        _ => (inst.type_name.clone(), inst.type_name.len()),
    };
    let name = match take_token(src, after_type, inst.name.len()) {
        Some((slice, _)) if slice.eq_ignore_ascii_case(&inst.name) => slice.to_string(),
        _ => inst.name.clone(),
    };
    (type_name, name)
}

/// The `len`-character token starting at or after byte `from`, plus the byte
/// index just past it.
///
/// This is `ComponentMetadata.originalCase`'s `source.substring(from, to)`
/// generalised by **one** rule: leading whitespace is skipped. On ANTLR's
/// `ctx.getText()` — which drops inter-token whitespace, so there is none to
/// skip — the two are character-for-character identical. The generalisation
/// exists because this port's parser may hand over a verbatim source slice
/// (`Pump P1(…)`, with the space), where fixed offsets would slice `" P1"`,
/// fail the case-insensitive guard, and silently lose the instance's spelling.
fn take_token(src: &str, from: usize, len: usize) -> Option<(&str, usize)> {
    if len == 0 || from > src.len() || !src.is_char_boundary(from) {
        return None;
    }
    let offset = src[from..].find(|c: char| !c.is_whitespace())?;
    let start = from + offset;
    let end = start + len;
    if end > src.len() || !src.is_char_boundary(end) {
        return None;
    }
    Some((&src[start..end], end))
}

/// Port of `ComponentMetadata.resolveParam`.
fn resolve_param(
    name: &str,
    value: &Expr,
    by_name: &BTreeMap<String, &VariableRow>,
) -> ComponentParamMeta {
    match value {
        Expr::Var(var) => {
            let found = by_name.get(var.as_str());
            ComponentParamMeta {
                name: name.to_string(),
                // Show the symbol as the user typed it (original case) when known.
                reference: found.map_or_else(|| var.clone(), |v| v.name.clone()),
                value: found.map(|v| v.value),
                units: found.map(|v| v.units.clone()),
            }
        }
        Expr::Num { value: n, unit, .. } => ComponentParamMeta {
            name: name.to_string(),
            reference: format_num(*n, unit.as_deref()),
            value: Some(*n),
            units: unit.clone(),
        },
        Expr::Str(s) => ComponentParamMeta {
            name: name.to_string(),
            reference: s.clone(),
            value: None,
            units: None,
        },
        // Compound expression (e.g. UA = 2*A): show its source-ish text,
        // leave the value unresolved (we don't re-evaluate here).
        other => ComponentParamMeta {
            name: name.to_string(),
            reference: expr_text(other),
            value: None,
            units: None,
        },
    }
}

/// Port of `ComponentMetadata.formatNum`.
///
/// `n == Math.rint(n)` is the Java integrality test; `round_ties_even` is
/// `rint` under the default rounding mode. Infinity satisfies it and is
/// excluded explicitly, exactly as the Java does, so it renders as
/// `"Infinity"` rather than a saturated `Long`.
fn format_num(n: f64, unit: Option<&str>) -> String {
    let s = if n == n.round_ties_even() && !n.is_infinite() {
        // Java's `(long) n` saturates at Long.MIN/MAX; so does Rust's `as i64`.
        (n as i64).to_string()
    } else {
        java_double_to_string(n)
    };
    match unit {
        Some(u) if !u.trim().is_empty() => format!("{s} [{u}]"),
        _ => s,
    }
}

/// Compact, source-like rendering of an expression for display only.
/// Port of `ComponentMetadata.exprText`.
fn expr_text(e: &Expr) -> String {
    match e {
        Expr::Num { value, unit, .. } => format_num(*value, unit.as_deref()),
        Expr::Str(s) => s.clone(),
        Expr::Var(name) => name.clone(),
        Expr::Neg(operand) => format!("-{}", expr_text(operand)),
        Expr::BinOp { op, left, right } => format!(
            "{} {} {}",
            expr_text(left),
            java_bin_op_char(*op),
            expr_text(right)
        ),
        Expr::Call { function, args } => {
            let rendered: Vec<String> = args.iter().map(expr_text).collect();
            format!("{function}({})", rendered.join(", "))
        }
        // The Java `switch` has no arm for the remaining six variants and falls
        // through to `default -> e.toString()`, i.e. the *record* toString. It
        // is not pretty, but it is what the datasheet shows, so the port
        // renders the same string rather than inventing a nicer one.
        other => java_record_to_string(other),
    }
}

/// The `char` the Java AST stores for a binary operator.
///
/// The four element-wise forms are private-use sentinels
/// (`EquationParser.ELEMENT_MUL` = `⊙`, `ELEMENT_DIV` = `⊘`,
/// `ELEMENT_LDIV` = `∖`, `ELEMENT_POW` = `↑`), transcribed as such: the Java
/// `exprText` interpolates the raw `char`, so `a .* b` renders as `a ⊙ b`.
fn java_bin_op_char(op: BinOp) -> char {
    match op {
        BinOp::Add => '+',
        BinOp::Sub => '-',
        BinOp::Mul => '*',
        BinOp::Div => '/',
        BinOp::LeftDiv => '\\',
        BinOp::Pow => '^',
        BinOp::ElemMul => '⊙',
        BinOp::ElemDiv => '⊘',
        BinOp::ElemLeftDiv => '∖',
        BinOp::ElemPow => '↑',
    }
}

/// The generated `toString()` of the Java `Expr` records:
/// `Name[component=value, …]`, with nested expressions rendered the same way
/// and a `List` as `[a, b]`.
fn java_record_to_string(e: &Expr) -> String {
    fn list(items: &[Expr]) -> String {
        let rendered: Vec<String> = items.iter().map(java_record_to_string).collect();
        format!("[{}]", rendered.join(", "))
    }
    match e {
        Expr::Num {
            value,
            unit,
            is_imaginary,
        } => format!(
            "Num[value={}, unit={}, isImaginary={}]",
            java_double_to_string(*value),
            unit.as_deref().unwrap_or("null"),
            is_imaginary
        ),
        Expr::Str(s) => format!("Str[value={s}]"),
        Expr::Var(name) => format!("Var[name={name}]"),
        Expr::BinOp { op, left, right } => format!(
            "BinOp[op={}, left={}, right={}]",
            java_bin_op_char(*op),
            java_record_to_string(left),
            java_record_to_string(right)
        ),
        Expr::Neg(operand) => format!("Neg[operand={}]", java_record_to_string(operand)),
        Expr::Call { function, args } => {
            format!("Call[function={function}, args={}]", list(args))
        }
        Expr::ArrayAccess { name, indices } => {
            format!("ArrayAccess[name={name}, indices={}]", list(indices))
        }
        Expr::Range { start, end } => format!(
            "Range[start={}, end={}]",
            java_record_to_string(start),
            java_record_to_string(end)
        ),
        Expr::ArrayLiteral(elements) => format!("ArrayLiteral[elements={}]", list(elements)),
        Expr::Compare { op, left, right } => format!(
            "Compare[op={}, left={}, right={}]",
            op.as_str(),
            java_record_to_string(left),
            java_record_to_string(right)
        ),
        Expr::Logical { op, left, right } => format!(
            "Logical[op={}, left={}, right={}]",
            op.as_str(),
            java_record_to_string(left),
            java_record_to_string(right)
        ),
        Expr::Not(operand) => format!("Not[operand={}]", java_record_to_string(operand)),
    }
}

/// `Double.toString(d)` — Java's spelling of a `double`, which is *not*
/// Rust's.
///
/// Both pick the shortest decimal digit string that round-trips (Java has,
/// since JDK 19, the same shortest form Rust's `{:e}` produces), so the digits
/// agree; only the notation differs. Java switches to scientific form outside
/// `[10^-3, 10^7)` and always keeps one fractional digit: `6.0E-7`, `1.0E23`,
/// `0.001`, `575.46`. Rust would write `0.0000006` and
/// `100000000000000000000000`.
///
/// This matters because the rendered string is the `ref` a datasheet shows for
/// a literal binding (`grad=6e-7`).
fn java_double_to_string(d: f64) -> String {
    if d.is_nan() {
        return "NaN".to_string();
    }
    if d.is_infinite() {
        return if d > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if d == 0.0 {
        return if d.is_sign_negative() { "-0.0" } else { "0.0" }.to_string();
    }
    let sign = if d < 0.0 { "-" } else { "" };
    // `{:e}` is shortest-round-trip scientific: "6e-7", "5.7546e2".
    let sci = format!("{:e}", d.abs());
    let (mantissa, exponent) = sci.split_once('e').expect("{:e} always emits an exponent");
    let exp: i32 = exponent.parse().expect("{:e} emits an integer exponent");
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();

    // 10^-3 <= m < 10^7 is Java's plain decimal form; everything else is its
    // "computerized scientific notation".
    if (-3..=6).contains(&exp) {
        let int_len = (exp + 1) as usize;
        let mut out = String::with_capacity(digits.len() + 8);
        out.push_str(sign);
        if exp < 0 {
            out.push_str("0.");
            for _ in 0..(-exp - 1) {
                out.push('0');
            }
            out.push_str(&digits);
        } else if digits.len() <= int_len {
            out.push_str(&digits);
            for _ in 0..(int_len - digits.len()) {
                out.push('0');
            }
            out.push_str(".0");
        } else {
            out.push_str(&digits[..int_len]);
            out.push('.');
            out.push_str(&digits[int_len..]);
        }
        return out;
    }
    let fraction = if digits.len() > 1 { &digits[1..] } else { "0" };
    format!("{sign}{}.{fraction}E{exp}", &digits[..1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{CmpOp, LogicOp};

    fn num(value: f64) -> Expr {
        Expr::num(value)
    }

    fn inst(
        source_text: &str,
        type_name: &str,
        name: &str,
        params: &[(&str, Expr)],
    ) -> ComponentInstMeta {
        ComponentInstMeta {
            type_name: type_name.to_string(),
            name: name.to_string(),
            source_text: source_text.to_string(),
            params: params
                .iter()
                .map(|(k, v)| ((*k).to_string(), v.clone()))
                .collect(),
            line: 0,
        }
    }

    // ── build ────────────────────────────────────────────────────────────────

    /// Port of `ComponentMetadataTest.resolvesVariableLiteralAndStringParamBindings`.
    #[test]
    fn resolves_variable_literal_and_string_param_bindings() {
        // TwoPhaseEvaporatorUA CHLR(fluid$=R1234yf, UA=UA_chl_r, SH=5)
        let insts = [inst(
            "TwoPhaseEvaporatorUACHLR(fluid$=R1234yf,UA=UA_chl_r,SH=5)",
            "twophaseevaporatorua",
            "chlr",
            &[
                ("fluid$", Expr::var("R1234yf")),
                ("ua", Expr::var("UA_chl_r")),
                ("sh", num(5.0)),
            ],
        )];
        let vars = [VariableRow::new("UA_chl_r", 575.46, "W/K")];

        let comps = build(&insts, &vars);
        assert_eq!(comps.len(), 1);
        let chlr = &comps[0];
        // Display name/type recover their original spelling from the source
        // even though the AST lowercases both.
        assert_eq!(chlr.name, "CHLR");
        assert_eq!(chlr.type_name, "TwoPhaseEvaporatorUA");

        let by_name: BTreeMap<&str, &ComponentParamMeta> =
            chlr.params.iter().map(|p| (p.name.as_str(), p)).collect();

        // Variable binding resolves to its solved value + units, shown by symbol.
        let ua = by_name["ua"];
        assert_eq!(ua.reference, "UA_chl_r");
        assert_eq!(ua.value, Some(575.46));
        assert_eq!(ua.units.as_deref(), Some("W/K"));

        // Numeric literal carries its own value, no backing variable.
        assert_eq!(by_name["sh"].value, Some(5.0));
        assert_eq!(by_name["sh"].reference, "5");

        // A bare-identifier string parameter (a fluid name) has no backing
        // variable, so it shows the identifier (lowercased) with no value.
        let fluid = by_name["fluid$"];
        assert_eq!(fluid.reference, "r1234yf");
        assert_eq!(fluid.value, None);
        assert_eq!(fluid.units, None);
    }

    /// Port of `ComponentMetadataTest.sharedVariableResolvesToSameValueUnderEachInstance`.
    #[test]
    fn shared_variable_resolves_to_same_value_under_each_instance() {
        let insts = [
            inst(
                "LiquidWallHXRAD1(fluid$=EG50,UA=UA_rad)",
                "liquidwallhx",
                "rad1",
                &[("fluid$", Expr::var("EG50")), ("ua", Expr::var("UA_rad"))],
            ),
            inst(
                "LiquidWallHXRAD2(fluid$=EG50,UA=UA_rad)",
                "liquidwallhx",
                "rad2",
                &[("fluid$", Expr::var("EG50")), ("ua", Expr::var("UA_rad"))],
            ),
        ];
        let vars = [VariableRow::new("UA_rad", 361.5, "W/K")];

        let comps = build(&insts, &vars);
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].name, "RAD1");
        assert_eq!(comps[1].name, "RAD2");
        for c in &comps {
            let ua = c.params.iter().find(|p| p.name == "ua").expect("ua param");
            assert_eq!(ua.reference, "UA_rad");
            assert_eq!(ua.value, Some(361.5));
        }
    }

    /// Port of `ComponentMetadataTest.returnsEmptyWhenNoComponents`.
    #[test]
    fn returns_empty_when_no_components() {
        assert!(build(&[], &[VariableRow::new("x", 1.0, "")]).is_empty());
        assert!(build_from_instances(&[], &[]).is_empty());
    }

    /// The seam itself: an expander `ComponentInst` — whose `source_text` is the
    /// user's verbatim slice, spaces and all — reports the same datasheet the
    /// Java builds from ANTLR's whitespace-free `getText()`.
    #[test]
    fn an_expander_instance_converts_and_reports() {
        let mut params = crate::components::def::ParamOverrides::new();
        params.put("fluid$".into(), Expr::var("R1234yf"));
        params.put("ua".into(), Expr::var("UA_chl_r"));
        params.put("sh".into(), num(5.0));
        let inst = ComponentInst {
            type_name: "twophaseevaporatorua".into(),
            name: "chlr".into(),
            port_args: vec!["s1".into(), "s2".into()],
            params,
            source_text: "TwoPhaseEvaporatorUA CHLR(s1, s2, fluid$=R1234yf, UA=UA_chl_r, SH=5)"
                .into(),
            line: 0,
        };
        let comps = build_from_instances(&[inst], &[VariableRow::new("UA_chl_r", 575.46, "W/K")]);
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "CHLR");
        assert_eq!(comps[0].type_name, "TwoPhaseEvaporatorUA");
        let names: Vec<&str> = comps[0].params.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["fluid$", "ua", "sh"]);
        assert_eq!(comps[0].params[1].value, Some(575.46));
        // Port arguments are connectivity, not datasheet inputs.
        assert_eq!(comps[0].params.len(), 3);
    }

    #[test]
    fn parameter_order_is_declaration_order() {
        let insts = [inst(
            "PumpP1(eta=0.8,fluid$=Water,n=2)",
            "pump",
            "p1",
            &[
                ("eta", num(0.8)),
                ("fluid$", Expr::var("Water")),
                ("n", num(2.0)),
            ],
        )];
        let comps = build(&insts, &[]);
        let names: Vec<&str> = comps[0].params.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["eta", "fluid$", "n"]);
    }

    #[test]
    fn first_row_wins_for_a_repeated_variable_name() {
        let insts = [inst(
            "PumpP1(ua=UA)",
            "pump",
            "p1",
            &[("ua", Expr::var("UA"))],
        )];
        let vars = [
            VariableRow::new("UA", 1.0, "W/K"),
            VariableRow::new("ua", 2.0, "kW/K"),
        ];
        let p = &build(&insts, &vars)[0].params[0];
        assert_eq!(p.value, Some(1.0));
        assert_eq!(p.units.as_deref(), Some("W/K"));
    }

    #[test]
    fn an_unsolved_variable_binding_shows_the_symbol_without_a_value() {
        let insts = [inst(
            "PumpP1(ua=UA_missing)",
            "pump",
            "p1",
            &[("ua", Expr::var("UA_missing"))],
        )];
        let p = &build(&insts, &[])[0].params[0];
        assert_eq!(p.reference, "ua_missing");
        assert_eq!(p.value, None);
        assert_eq!(p.units, None);
    }

    #[test]
    fn a_quoted_string_parameter_shows_its_contents() {
        let insts = [inst(
            "PumpP1(fluid$='Water')",
            "pump",
            "p1",
            &[("fluid$", Expr::Str("Water".into()))],
        )];
        let p = &build(&insts, &[])[0].params[0];
        assert_eq!(p.reference, "Water");
        assert_eq!(p.value, None);
    }

    #[test]
    fn a_unit_annotated_literal_keeps_its_si_unit() {
        // `P=140 [kPa]` is already converted to SI by the parser, and the
        // literal carries the SI display name.
        let insts = [inst(
            "SourceS1(p=140[kPa])",
            "source",
            "s1",
            &[(
                "p",
                Expr::Num {
                    value: 140_000.0,
                    unit: Some("Pa".into()),
                    is_imaginary: false,
                },
            )],
        )];
        let p = &build(&insts, &[])[0].params[0];
        assert_eq!(p.reference, "140000 [Pa]");
        assert_eq!(p.value, Some(140_000.0));
        assert_eq!(p.units.as_deref(), Some("Pa"));
    }

    /// The `ProportionalReliefValve` line of `fixtures/corpus-pending/corpus/
    /// pressure-cooker.frees`, with the DTOs the Java `ComponentMetadata`
    /// actually produced for it.
    #[test]
    fn a_corpus_instance_matches_the_java_dtos() {
        // ProportionalReliefValve PRV(fluid$=Water, Pcrack=202600, grad=6e-7, eps=2000)
        let insts = [inst(
            "ProportionalReliefValvePRV(fluid$=Water,Pcrack=202600,grad=6e-7,eps=2000)",
            "proportionalreliefvalve",
            "prv",
            &[
                ("fluid$", Expr::var("Water")),
                ("pcrack", num(202_600.0)),
                ("grad", num(6e-7)),
                ("eps", num(2000.0)),
            ],
        )];
        let comps = build(&insts, &[]);
        assert_eq!(comps[0].name, "PRV");
        assert_eq!(comps[0].type_name, "ProportionalReliefValve");
        let rendered: Vec<(&str, &str, Option<f64>)> = comps[0]
            .params
            .iter()
            .map(|p| (p.name.as_str(), p.reference.as_str(), p.value))
            .collect();
        assert_eq!(
            rendered,
            vec![
                ("fluid$", "water", None),
                ("pcrack", "202600", Some(202_600.0)),
                // Java spells small magnitudes in scientific notation.
                ("grad", "6.0E-7", Some(6e-7)),
                ("eps", "2000", Some(2000.0)),
            ]
        );
    }

    #[test]
    fn a_compound_binding_shows_source_like_text_and_no_value() {
        // UA = 2 * A
        let insts = [inst(
            "HXH1(ua=2*A)",
            "hx",
            "h1",
            &[("ua", Expr::bin(BinOp::Mul, num(2.0), Expr::var("A")))],
        )];
        let p = &build(&insts, &[])[0].params[0];
        assert_eq!(p.reference, "2 * a");
        assert_eq!(p.value, None);
    }

    // ── display identity ─────────────────────────────────────────────────────

    #[test]
    fn a_verbatim_source_slice_recovers_the_same_spelling_as_antlr_text() {
        // ANTLR's getText() drops the whitespace; a verbatim slice keeps it.
        // Both must recover `Pump` / `P1`.
        for src in ["PumpP1(eta=0.8)", "Pump P1(eta = 0.8)", "Pump   P1 (x)"] {
            let i = inst(src, "pump", "p1", &[]);
            assert_eq!(display_identity(&i), ("Pump".into(), "P1".into()), "{src}");
        }
    }

    #[test]
    fn an_unexpected_source_shape_falls_back_to_the_lowercased_tokens() {
        // The Java guard: a slice that does not match the token case-insensitively
        // is not trusted (hierarchical sub-instances reach this).
        let i = inst("something else entirely", "pump", "p1", &[]);
        assert_eq!(display_identity(&i), ("pump".into(), "p1".into()));
        // Truncated source: no slice at all.
        let short = inst("Pu", "pump", "p1", &[]);
        assert_eq!(display_identity(&short), ("pump".into(), "p1".into()));
        // Empty source.
        let empty = inst("", "pump", "p1", &[]);
        assert_eq!(display_identity(&empty), ("pump".into(), "p1".into()));
    }

    #[test]
    fn a_trusted_type_with_an_untrusted_name_keeps_the_recovered_type() {
        let i = inst("Pump??", "pump", "p1", &[]);
        assert_eq!(display_identity(&i), ("Pump".into(), "p1".into()));
    }

    // ── formatting ───────────────────────────────────────────────────────────

    #[test]
    fn integral_values_render_without_a_decimal_point() {
        assert_eq!(format_num(5.0, None), "5");
        assert_eq!(format_num(-3.0, None), "-3");
        assert_eq!(format_num(0.0, None), "0");
        assert_eq!(format_num(-0.0, None), "0");
        assert_eq!(format_num(140_000.0, Some("Pa")), "140000 [Pa]");
        // Java's `(long)` narrowing saturates, and so does Rust's `as i64`.
        assert_eq!(format_num(1e21, None), "9223372036854775807");
        assert_eq!(format_num(-1e21, None), "-9223372036854775808");
    }

    #[test]
    fn non_integral_values_use_javas_double_spelling() {
        assert_eq!(format_num(575.46, None), "575.46");
        assert_eq!(format_num(6e-7, None), "6.0E-7");
        assert_eq!(format_num(0.5, Some("")), "0.5");
        assert_eq!(format_num(0.5, Some("  ")), "0.5");
        assert_eq!(format_num(f64::INFINITY, None), "Infinity");
        assert_eq!(format_num(f64::NEG_INFINITY, None), "-Infinity");
        assert_eq!(format_num(f64::NAN, None), "NaN");
    }

    /// Every case here is `Double.toString`'s documented contract.
    #[test]
    fn java_double_spelling_matches_the_jvm() {
        let cases = [
            (0.0, "0.0"),
            (-0.0, "-0.0"),
            (1.0, "1.0"),
            (-1.0, "-1.0"),
            (0.1, "0.1"),
            (575.46, "575.46"),
            (1e-3, "0.001"),
            (9.999e-4, "9.999E-4"),
            (1e-4, "1.0E-4"),
            (6e-7, "6.0E-7"),
            (9_999_999.0, "9999999.0"),
            (1e7, "1.0E7"),
            (1.5e7, "1.5E7"),
            (1e23, "1.0E23"),
            (1.0 / 3.0, "0.3333333333333333"),
            (-2.5e-8, "-2.5E-8"),
            (100.0, "100.0"),
            (123.0, "123.0"),
            (f64::MAX, "1.7976931348623157E308"),
            (f64::MIN_POSITIVE, "2.2250738585072014E-308"),
        ];
        for (value, expected) in cases {
            assert_eq!(java_double_to_string(value), expected, "for {value:?}");
        }
    }

    #[test]
    fn element_wise_operators_keep_the_java_sentinel_chars() {
        let e = Expr::bin(BinOp::ElemMul, Expr::var("a"), Expr::var("b"));
        assert_eq!(expr_text(&e), "a ⊙ b");
    }

    #[test]
    fn a_call_binding_renders_its_arguments() {
        let e = Expr::call(
            "Enthalpy",
            vec![Expr::var("Water"), Expr::var("p"), num(0.0)],
        );
        assert_eq!(expr_text(&e), "enthalpy(water, p, 0)");
    }

    #[test]
    fn a_negated_binding_keeps_the_leading_minus() {
        assert_eq!(expr_text(&Expr::Neg(Box::new(Expr::var("q")))), "-q");
    }

    /// The six variants the Java `switch` does not name fall through to the
    /// record `toString()`.
    #[test]
    fn unhandled_variants_render_as_java_record_text() {
        let access = Expr::ArrayAccess {
            name: "a".into(),
            indices: vec![num(2.0)],
        };
        assert_eq!(
            expr_text(&access),
            "ArrayAccess[name=a, indices=[Num[value=2.0, unit=null, isImaginary=false]]]"
        );
        let literal = Expr::ArrayLiteral(vec![num(1.0), Expr::var("b")]);
        assert_eq!(
            expr_text(&literal),
            "ArrayLiteral[elements=[Num[value=1.0, unit=null, isImaginary=false], Var[name=b]]]"
        );
        let cmp = Expr::Compare {
            op: CmpOp::Le,
            left: Box::new(Expr::var("a")),
            right: Box::new(num(1.0)),
        };
        assert_eq!(
            expr_text(&cmp),
            "Compare[op=<=, left=Var[name=a], right=Num[value=1.0, unit=null, isImaginary=false]]"
        );
        let logical = Expr::Logical {
            op: LogicOp::And,
            left: Box::new(Expr::var("a")),
            right: Box::new(Expr::var("b")),
        };
        assert_eq!(
            expr_text(&logical),
            "Logical[op=and, left=Var[name=a], right=Var[name=b]]"
        );
        assert_eq!(
            expr_text(&Expr::Not(Box::new(Expr::var("a")))),
            "Not[operand=Var[name=a]]"
        );
        let range = Expr::Range {
            start: Box::new(num(1.0)),
            end: Box::new(Expr::var("n")),
        };
        assert_eq!(
            expr_text(&range),
            "Range[start=Num[value=1.0, unit=null, isImaginary=false], end=Var[name=n]]"
        );
    }

    #[test]
    fn a_unit_annotated_literal_inside_a_record_rendering_keeps_its_unit() {
        let e = Expr::ArrayAccess {
            name: "a".into(),
            indices: vec![Expr::Num {
                value: 140_000.0,
                unit: Some("Pa".into()),
                is_imaginary: false,
            }],
        };
        assert_eq!(
            expr_text(&e),
            "ArrayAccess[name=a, indices=[Num[value=140000.0, unit=Pa, isImaginary=false]]]"
        );
    }
}
