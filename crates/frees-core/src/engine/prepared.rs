use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::*;
use crate::analysis::parametric::ParametricAccessors;
use crate::ast::{Equation, Expr};
use crate::diag::Diagnostic;
use crate::engine::{
    block_equation_texts, block_system, builtin_constants, check_bounds, collect_unit_warnings,
    complete_display_names, declared_units, dense_export, dense_plan, initial_guess,
    merge_advisory_warnings, override_uncertainties, pinned_block_cache, relaxed_ode_settings,
    residuals_at, run_blocks, seed_property_argument_guesses, solve_dynamic_systems,
    solve_equation_list, surfaced_count, unknowns, variable_specs, BlockLoopFailure, DensePlan,
    DenseRun, Missing, PartialDiagnostics, PinnedBlockCache, Solution, SolveFailure, SolveStats,
    VarSpec, VariableOverride,
};
use crate::eval::{EvalContext, Scope};
use crate::ode::accessors::OdeTableAccessors;
use crate::parser::defs::FunctionTableDef;
use crate::parser::Document;
use crate::solver::SolverSettings;

/// Cached structural compilation and block structures for a specific set of pinned names.
struct DocumentPrep {
    pinned_names: Vec<String>,
    subsystem: Vec<Equation>,
    report: BlockingReport,
    specs: BTreeMap<String, VarSpec>,
    unknowns: Vec<String>,
    block_cache: PinnedBlockCache,
    work_scope: Scope,
    dense: Option<DensePlan>,
    dense_values: Vec<f64>,
    display_names: BTreeMap<String, String>,
    declared_units: BTreeMap<String, String>,
    unit_report: crate::units::checker::UnitReport,
    diagnostics: Vec<Diagnostic>,
}

/// A prepared document for repeated solving with different numeric pins.
///
/// Hoists the 13-stage document compilation pipeline (parsing, component
/// expansion, CALL flattening, string variable resolution, integral hoisting,
/// and complex/ODE expansion) and reuses block structures, analytic derivatives,
/// variable specs, and scratch scopes across repeated solves.
pub struct PreparedDocument {
    pub source: String,
    pub settings: SolverSettings,
    pub overrides: Vec<VariableOverride>,
    pub extra_tables: Vec<FunctionTableDef>,
    pub doc: Document,
    pub diagnostics: Vec<Diagnostic>,
    pub component_instances: Vec<crate::components::metadata::ComponentInstMeta>,
    pub component_connections: Vec<crate::components::expander::Connection>,
    pub component_member_units: BTreeMap<String, String>,
    pub ordinary_equations: Vec<Equation>,
    pub uncertainty_exprs: BTreeMap<String, Expr>,
    pub stepping_iterations: usize,
    pub ode_accessors: bool,
    pub unaugmented: Option<Vec<Equation>>,
    prep: Option<DocumentPrep>,
    prep_count: usize,
    solve_count: usize,
}

impl PreparedDocument {
    /// Pins that participate in lowering must enter before that lowering runs.
    fn with_source_pins(
        &self,
        pinned: &[(String, f64)],
    ) -> std::result::Result<Option<Self>, SolveFailure> {
        if pinned.is_empty()
            || !(self.settings.complex_mode
                || self.stepping_iterations > 0
                || !self.doc.linearizes.is_empty()
                || pinned.iter().any(|(name, _)| {
                    name.is_empty()
                        || name.as_bytes()[0].is_ascii_digit()
                        || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
                }))
        {
            return Ok(None);
        }
        // ponytail: uncommon lowered pins recompile; cache before lowering if
        // profiling shows complex/array/component sweeps need the fast path.
        Self::new(
            &crate::analysis::parametric::row_source(&self.source, pinned),
            &self.settings,
            &self.overrides,
            &self.extra_tables,
        )
        .map(Some)
    }

    /// Prepare a base document: parses and runs stages 1 through 3e.
    pub fn new(
        source: &str,
        settings: &SolverSettings,
        overrides: &[VariableOverride],
        extra_tables: &[FunctionTableDef],
    ) -> std::result::Result<Self, SolveFailure> {
        crate::props::tables::install_builtin_once();
        let mut doc = parse_document(source)?;
        reject_unsupported(&doc)?;

        // Pipeline stage 1b — the acausal component layer
        let mut diagnostics = doc.diagnostics.clone();
        let mut components = expand_component_layer(&mut doc, &mut diagnostics)?;

        // Pipeline stages 2–4
        let statements = std::mem::take(&mut doc.statements);
        let mut parsed_names = std::mem::take(&mut doc.display_names);
        let (flattened, module_count) =
            flatten_calls_counted(statements, &doc.defs, &mut parsed_names)?;
        doc.statements = flattened;
        let mut equations = std::mem::take(&mut components.equations);
        equations.extend(crate::parser::expand::expand_document_with(
            &doc,
            module_count,
            &mut parsed_names,
        )?);
        doc.display_names = parsed_names;
        let equations = crate::parser::string_variables::resolve(equations, &doc.display_names)?;
        let extra_tables_vec = extra_tables.to_vec();
        doc.defs.merge_extra_tables(&extra_tables_vec);

        let equations = crate::integral::hoist_nested(equations);
        let ext = crate::analysis::uncertainty::extract_uncertainty_equations(&equations);
        let equations = ext.active_equations;
        let uncertainty_exprs = ext.uncertainty_exprs;

        let base_ctx = EvalContext::with_defs(&doc.defs);
        let integrals = find_integrals(&equations, &doc.defs, settings.complex_mode)?;
        let mut stepping_iterations = 0usize;
        let equations = if integrals.is_empty() {
            if settings.complex_mode {
                let mut complex_names: HashMap<String, String> = doc
                    .display_names
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                let expanded = crate::parser::complex::expand_with_display_names(
                    &equations,
                    &mut complex_names,
                )?;
                doc.display_names = complex_names.into_iter().collect();
                expanded
            } else {
                crate::parser::complex::expand_complex(equations, false)?
            }
        } else {
            let (_, hoisted_knowns) = builtin_constants(&equations);
            let mut dropped = Vec::new();
            let pre_specs =
                variable_specs(&equations, &hoisted_knowns, &doc, overrides, &mut dropped)?;
            let (lowered, driven) =
                lower_integrals(&equations, &integrals, settings, &pre_specs, base_ctx)?;
            stepping_iterations = driven;
            lowered
        };

        let equations = if doc.linearizes.is_empty() {
            equations
        } else {
            let (_, pre_knowns) = builtin_constants(&equations);
            let mut dropped = Vec::new();
            let pre_specs = variable_specs(&equations, &pre_knowns, &doc, overrides, &mut dropped)?;
            let inputs = LinearizeInputs {
                dynamics: &doc.dynamics,
                linearizes: &doc.linearizes,
                defs: &doc.defs,
            };
            let mut names = std::mem::take(&mut doc.display_names);
            let injected = inject_linearizations(
                inputs, &mut names, equations, settings, &pre_specs, base_ctx,
            );
            doc.display_names = names;
            injected?
        };

        let ode_accessors =
            !doc.dynamics.is_empty() && crate::ode::accessors::contains_accessor(&equations);
        let unaugmented = if ode_accessors {
            Some(equations.clone())
        } else {
            None
        };
        let equations = if ode_accessors {
            augment_accessor_dependencies(&doc, equations)
        } else {
            equations
        };

        Ok(PreparedDocument {
            source: source.to_string(),
            settings: *settings,
            overrides: overrides.to_vec(),
            extra_tables: extra_tables_vec,
            doc,
            diagnostics,
            component_instances: components.instances,
            component_connections: components.connections,
            component_member_units: components.member_units,
            ordinary_equations: equations,
            uncertainty_exprs,
            stepping_iterations,
            ode_accessors,
            unaugmented,
            prep: None,
            prep_count: 0,
            solve_count: 0,
        })
    }

    /// Ensure structural preparation exists and pins are updated with current numeric values.
    pub fn ensure_prep(
        &mut self,
        pinned: &[(String, f64)],
    ) -> std::result::Result<(), SolveFailure> {
        let matches = self.prep.as_ref().is_some_and(|p| {
            p.pinned_names.len() == pinned.len()
                && p.pinned_names
                    .iter()
                    .zip(pinned)
                    .all(|(cached, (name, _))| cached == name)
        });

        if !matches {
            self.prep_count += 1;
            let mut subsystem: Vec<Equation> = self.ordinary_equations.to_vec();
            for (name, value) in pinned {
                subsystem.push(Equation::new(
                    Expr::var(name),
                    Expr::num(*value),
                    format!("{name} = {value}"),
                ));
            }
            let (constants, knowns) = builtin_constants(&subsystem);
            let report = block_system(&subsystem, &knowns)?;
            let mut diagnostics = self.diagnostics.clone();
            collect_unit_warnings(&subsystem, &mut diagnostics);
            let mut specs = variable_specs(
                &subsystem,
                &knowns,
                &self.doc,
                &self.overrides,
                &mut diagnostics,
            )?;
            seed_property_argument_guesses(&subsystem, &mut specs, Missing::Skip);
            let unknown_names = unknowns(&subsystem, &knowns);
            let mut block_cache = pinned_block_cache(
                &report.blocks,
                &subsystem,
                self.ordinary_equations.len(),
                &specs,
            );
            let mut work_scope: Scope =
                Scope::with_capacity_and_hasher(specs.len() + constants.len(), Default::default());
            work_scope.extend(constants.iter().map(|(k, v)| (k.clone(), *v)));
            for name in &unknown_names {
                let initial = specs.get(name).map(|s| s.initial()).unwrap_or(0.0);
                work_scope.insert(name.clone(), initial);
            }
            let (dense, dense_values) = match dense_plan(
                &report.blocks,
                &mut block_cache,
                &work_scope,
                &unknown_names,
            ) {
                Some((plan, values)) => (Some(plan), values),
                None => (None, Vec::new()),
            };
            let mut names = self.doc.display_names.clone();
            for (name, _) in pinned {
                names
                    .entry(name.to_ascii_lowercase())
                    .or_insert_with(|| name.clone());
            }
            let display_names = complete_display_names(&names, &subsystem);
            let unit_equations = self.unaugmented.as_deref().unwrap_or(&subsystem);
            let declared = declared_units(
                unit_equations,
                &self.overrides,
                &self.component_member_units,
            );
            let unit_report = crate::units::checker::check_units(unit_equations, &declared);

            self.prep = Some(DocumentPrep {
                pinned_names: pinned.iter().map(|(n, _)| n.clone()).collect(),
                subsystem,
                report,
                specs,
                unknowns: unknown_names,
                block_cache,
                work_scope,
                dense,
                dense_values,
                display_names,
                declared_units: declared,
                unit_report,
                diagnostics,
            });
        }

        let prep = self.prep.as_mut().expect("prep just ensured");
        let base = self.ordinary_equations.len();
        for (k, (name, value)) in pinned.iter().enumerate() {
            let pin = &mut prep.subsystem[base + k];
            if let Expr::Num {
                value: old,
                unit: None,
                is_imaginary: false,
            } = &pin.rhs
            {
                if old.to_bits() == value.to_bits() {
                    continue;
                }
            }
            pin.rhs = Expr::num(*value);
            pin.source_text.clear();
            let _ = write!(pin.source_text, "{name} = {value}");
        }

        Ok(())
    }

    /// Solve with a given set of pinned variable values and optional parametric accessors.
    pub fn solve_with_pins(
        &mut self,
        pinned: &[(String, f64)],
        parametric: Option<&ParametricAccessors>,
    ) -> std::result::Result<Solution, SolveFailure> {
        self.solve_with_pins_and_warm(pinned, parametric, None)
    }

    /// Solve with a given set of pinned variable values, optional parametric accessors, and optional warm-start scope.
    pub fn solve_with_pins_and_warm(
        &mut self,
        pinned: &[(String, f64)],
        parametric: Option<&ParametricAccessors>,
        warm_start: Option<&Scope>,
    ) -> std::result::Result<Solution, SolveFailure> {
        if let Some(mut document) = self.with_source_pins(pinned)? {
            let result = document.solve_with_pins_and_warm(&[], parametric, warm_start);
            self.prep_count += document.prep_count;
            self.solve_count += document.solve_count;
            return result;
        }
        // ODE-only shortcut
        if pinned.is_empty() && self.ordinary_equations.is_empty() && !self.doc.dynamics.is_empty()
        {
            let base_ctx = {
                let mut ctx = EvalContext::with_defs(&self.doc.defs);
                ctx.parametric = parametric;
                ctx
            };
            let ode_tables = solve_dynamic_systems(
                &self.doc,
                &Scope::default(),
                &self.settings,
                &BTreeMap::new(),
                base_ctx,
                None,
            )?;
            return Ok(Solution {
                values: BTreeMap::new(),
                display_names: complete_display_names(
                    &self.doc.display_names,
                    &self.ordinary_equations,
                ),
                blocks: Vec::new(),
                block_equations: Vec::new(),
                residuals: Vec::new(),
                stats: SolveStats {
                    iterations: 0,
                    max_residual: 0.0,
                    elapsed_ms: None,
                },
                inferred_units: BTreeMap::new(),
                unit_warnings: Vec::new(),
                diagnostics: self.diagnostics.clone(),
                iterations: 0,
                component_instances: self.component_instances.clone(),
                component_connections: self.component_connections.clone(),
                ode_tables,
                uncertainties: BTreeMap::new(),
                uncertainty_contributions: BTreeMap::new(),
                plots: self.doc.blocks.plots.clone(),
            });
        }

        self.ensure_prep(pinned)?;

        let base_ctx = {
            let mut ctx = EvalContext::with_defs(&self.doc.defs);
            ctx.parametric = parametric;
            ctx
        };

        let prep = self.prep.as_mut().expect("prep just ensured");

        match &prep.dense {
            Some(plan) => {
                for (name, &slot) in prep.unknowns.iter().zip(&plan.unknown_idx) {
                    let guess = warm_start
                        .and_then(|w| w.get(name).copied())
                        .unwrap_or_else(|| initial_guess(name, &prep.specs));
                    prep.dense_values[slot as usize] = guess;
                }
            }
            None => {
                for name in &prep.unknowns {
                    let guess = warm_start
                        .and_then(|w| w.get(name).copied())
                        .unwrap_or_else(|| initial_guess(name, &prep.specs));
                    match prep.work_scope.get_mut(name) {
                        Some(slot) => *slot = guess,
                        None => {
                            prep.work_scope.insert(name.clone(), guess);
                        }
                    }
                }
            }
        }

        if let Some(warm) = warm_start {
            crate::analysis::uncertainty::carry_uncertainty_entries(warm, &mut prep.work_scope);
        }

        self.solve_count += 1;
        let inner_settings = relaxed_ode_settings(&self.settings, 1e-7);
        let bridge = self
            .ode_accessors
            .then(|| accessor_bridge(&self.doc, &inner_settings, &prep.specs, base_ctx));
        let ctx = EvalContext {
            ode: bridge.as_ref().map(|b| b as &dyn OdeTableAccessors),
            ..base_ctx
        };
        let relaxed;
        let solve_settings = if self.ode_accessors {
            relaxed = relaxed_ode_settings(&self.settings, 1e-4);
            &relaxed
        } else {
            &self.settings
        };

        if prep
            .dense
            .as_ref()
            .is_some_and(|plan| plan.names.len() != prep.work_scope.len())
        {
            prep.dense = None;
        }
        let mut dense_run = match &prep.dense {
            Some(plan) => Some(DenseRun {
                plan,
                values: &mut prep.dense_values,
                live: true,
            }),
            None => None,
        };

        let outcome = run_blocks(
            &prep.report.blocks,
            &prep.subsystem,
            &mut prep.work_scope,
            solve_settings,
            &prep.specs,
            ctx,
            Some(&prep.block_cache),
            dense_run.as_mut(),
        );

        let dense_live = dense_run.is_some_and(|r| r.live);
        if dense_live {
            let plan = prep.dense.as_ref().expect("dense was live");
            dense_export(plan, &prep.dense_values, &mut prep.work_scope);
        }

        let block_iterations = match outcome {
            Ok(iters) => iters,
            Err(BlockLoopFailure {
                error,
                failed_block_index,
                iterations,
            }) => {
                let (residuals, max_residual) = residuals_at(
                    &prep.subsystem,
                    &prep.report.blocks,
                    &prep.work_scope,
                    base_ctx,
                );
                let block_equations = block_equation_texts(&prep.report.blocks, &prep.subsystem);
                let display_names = prep.display_names.clone();
                return Err(SolveFailure {
                    error,
                    failed_block_index: Some(failed_block_index),
                    partial: Some(Box::new(PartialDiagnostics {
                        blocks: prep.report.blocks.clone(),
                        block_equations,
                        display_names,
                        unknown_count: surfaced_count(prep.specs.keys().map(String::as_str)),
                        residuals,
                        stats: SolveStats {
                            iterations: self.stepping_iterations + iterations,
                            max_residual,
                            elapsed_ms: None,
                        },
                    })),
                });
            }
        };
        let iterations = self.stepping_iterations + block_iterations;
        Self::assemble_solution(
            &self.doc,
            &self.settings,
            &self.overrides,
            &self.uncertainty_exprs,
            &self.component_instances,
            &self.component_connections,
            prep,
            &prep.work_scope,
            iterations,
            base_ctx,
            bridge.as_ref(),
            solve_settings,
        )
    }

    /// Assemble a Solution struct from a solved scope.
    #[allow(clippy::too_many_arguments)]
    fn assemble_solution(
        doc: &Document,
        settings: &SolverSettings,
        overrides: &[VariableOverride],
        uncertainty_exprs: &BTreeMap<String, Expr>,
        component_instances: &[crate::components::metadata::ComponentInstMeta],
        component_connections: &[crate::components::expander::Connection],
        prep: &DocumentPrep,
        scope: &Scope,
        iterations: usize,
        base_ctx: EvalContext<'_>,
        bridge: Option<&crate::ode::accessors::DynamicAccessorContext<'_>>,
        solve_settings: &SolverSettings,
    ) -> std::result::Result<Solution, SolveFailure> {
        let mut diagnostics = prep.diagnostics.clone();
        check_bounds(&prep.specs, scope, &mut diagnostics);

        let stated = override_uncertainties(overrides);
        let mut unc_specs: BTreeMap<String, crate::analysis::uncertainty::UncertaintySpec> = prep
            .specs
            .iter()
            .map(|(name, spec)| {
                (
                    name.clone(),
                    crate::analysis::uncertainty::UncertaintySpec {
                        guess: spec.guess,
                        lower: spec.lower,
                        upper: spec.upper,
                        uncertainty: stated.get(name).copied().unwrap_or(0.0),
                    },
                )
            })
            .collect();
        let mut scope_mut = scope.clone();
        let ctx = EvalContext {
            ode: bridge.map(|b| b as &dyn OdeTableAccessors),
            ..base_ctx
        };
        let propagation = crate::analysis::uncertainty::analyze(
            &prep.subsystem,
            &mut scope_mut,
            &mut unc_specs,
            uncertainty_exprs,
            ctx,
            |eqs, warm| {
                solve_equation_list(eqs, solve_settings, &prep.specs, ctx, Some(warm))
                    .map(|inner| inner.values)
            },
        )?;

        let mut solved: BTreeMap<String, f64> = prep
            .specs
            .keys()
            .filter(|name| !crate::parser::toplevel::is_ignored_sink(name))
            .map(|name| {
                let value = scope_mut.get(name).copied().unwrap_or(f64::NAN);
                (name.clone(), value)
            })
            .collect();
        for (name, value) in &scope_mut {
            if name.starts_with(crate::analysis::uncertainty::UNCERTAINTY_OF_FN) {
                solved.insert(name.clone(), *value);
            }
        }

        let (residuals, max_residual) =
            residuals_at(&prep.subsystem, &prep.report.blocks, &scope_mut, base_ctx);
        let block_equations = block_equation_texts(&prep.report.blocks, &prep.subsystem);

        let mut inferred_units = prep.unit_report.inferred.clone();
        inferred_units.extend(prep.declared_units.clone());

        let ode_tables =
            solve_dynamic_systems(doc, &scope_mut, settings, &prep.specs, base_ctx, bridge)?;

        Ok(Solution {
            values: solved,
            display_names: prep.display_names.clone(),
            blocks: prep.report.blocks.clone(),
            block_equations,
            residuals,
            stats: SolveStats {
                iterations,
                max_residual,
                elapsed_ms: None,
            },
            inferred_units,
            unit_warnings: merge_advisory_warnings(prep.unit_report.warnings.clone(), &diagnostics),
            diagnostics,
            iterations,
            component_instances: component_instances.to_vec(),
            component_connections: component_connections.to_vec(),
            ode_tables,
            uncertainties: propagation.uncertainties,
            uncertainty_contributions: propagation.contributions,
            plots: doc.blocks.plots.clone(),
        })
    }

    /// Solve finding all roots bounded up to MAX_SOLUTIONS (32) for the document.
    pub fn solve_all(&mut self) -> std::result::Result<Vec<Solution>, SolveFailure> {
        self.solve_all_with_pins(&[], None)
    }

    /// Solve finding all roots bounded up to MAX_SOLUTIONS (32) for a given set of pinned variable values and optional parametric accessors.
    pub fn solve_all_with_pins(
        &mut self,
        pinned: &[(String, f64)],
        parametric: Option<&ParametricAccessors>,
    ) -> std::result::Result<Vec<Solution>, SolveFailure> {
        if let Some(mut document) = self.with_source_pins(pinned)? {
            let result = document.solve_all_with_pins(&[], parametric);
            self.prep_count += document.prep_count;
            self.solve_count += document.solve_count;
            return result;
        }
        if self.ordinary_equations.is_empty() {
            let sol = self.solve_with_pins(pinned, parametric)?;
            return Ok(vec![sol]);
        }

        self.ensure_prep(pinned)?;

        let base_ctx = {
            let mut ctx = EvalContext::with_defs(&self.doc.defs);
            ctx.parametric = parametric;
            ctx
        };

        let prep = self.prep.as_ref().expect("prep just ensured");

        let inner_settings = relaxed_ode_settings(&self.settings, 1e-7);
        let bridge = self
            .ode_accessors
            .then(|| accessor_bridge(&self.doc, &inner_settings, &prep.specs, base_ctx));
        let relaxed;
        let solve_settings = if self.ode_accessors {
            relaxed = relaxed_ode_settings(&self.settings, 1e-4);
            &relaxed
        } else {
            &self.settings
        };

        let root_specs: BTreeMap<String, crate::analysis::allroots::RootSpec> = prep
            .specs
            .iter()
            .map(|(k, s)| {
                (
                    k.clone(),
                    crate::analysis::allroots::RootSpec {
                        guess: s.guess,
                        lower: s.lower,
                        upper: s.upper,
                    },
                )
            })
            .collect();

        let mut solver = crate::analysis::allroots::AllRootsSolver::new(
            *solve_settings,
            &root_specs,
            &self.doc.defs,
            &prep.subsystem,
        );

        let scopes = solver
            .find_all(&prep.report.blocks, &prep.work_scope)
            .map_err(|err| SolveFailure {
                error: err,
                failed_block_index: None,
                partial: None,
            })?;

        self.solve_count += scopes.len();
        let iterations = self.stepping_iterations + solver.total_iterations();

        let mut solutions = Vec::with_capacity(scopes.len());
        for scope in &scopes {
            let sol = Self::assemble_solution(
                &self.doc,
                &self.settings,
                &self.overrides,
                &self.uncertainty_exprs,
                &self.component_instances,
                &self.component_connections,
                prep,
                scope,
                iterations,
                base_ctx,
                bridge.as_ref(),
                solve_settings,
            )?;
            solutions.push(sol);
        }

        Ok(solutions)
    }

    /// How many times full block preparation ran (cache misses).
    pub fn prep_count(&self) -> usize {
        self.prep_count
    }

    /// How many times solve_with_pins was executed.
    pub fn solve_count(&self) -> usize {
        self.solve_count
    }
}
