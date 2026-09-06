//! Multi-objective optimisation by NSGA-II — Calculate ▸ Min/Max ▸ Pareto.
//!
//! Port of
//! `../frEES/backend/core/src/main/java/com/frees/backend/core/MultiObjectiveOptimizer.java`
//! (477 LOC): a genetic algorithm that returns the Pareto-optimal *front*
//! rather than a single optimum (Deb et al., 2002). Each candidate is a vector
//! of decision-variable values, scored by solving the equation system with
//! those decisions fixed and reading the named objective variables. Objectives
//! flagged `maximize` are internally negated so the algorithm always minimises.
//!
//! Per generation: binary tournament selection on `(rank, crowding distance)`,
//! simulated-binary crossover (SBX, η = 15), polynomial mutation (η = 20), then
//! elitist (μ+λ) replacement by fast non-dominated sorting and crowding
//! distance. The returned front is the first non-dominated rank of the final
//! population.
//!
//! # This is deterministic, and stays deterministic
//!
//! The Java seeds `new Random(p.seed())` — `java.util.Random`, a specified
//! 48-bit LCG — and `OptimizeController.computeOptimizeMulti` passes the
//! literal `42L`. So an NSGA-II run in the Java engine is **reproducible**, and
//! a faithful port has to reproduce the *same* pseudo-random stream, not merely
//! a Pareto front with the same shape. [`JavaRandom`] is a bit-exact port of
//! `java.util.Random.{next, nextDouble, nextInt}`; every draw site below is in
//! the Java's order, including the short-circuit `||`/`&&` operands that decide
//! whether a second draw happens at all.
//!
//! # Ordering is part of the algorithm
//!
//! `assignCrowding` sorts each front *in place*, once per objective, and
//! `selectNextGeneration` then copies the fronts into the next population in
//! whatever order that last sort left. Since `tournament` picks by index, the
//! population's order feeds straight back into the RNG stream. Both sorts are
//! `List.sort` (a stable merge sort) over `Comparator.comparingDouble`
//! (`Double.compare`), which is [`java_compare`] here and `slice::sort_by`
//! (also stable) there.

use crate::analysis::optimizer::{java_compare, jmax, jmin, plain_string};
use crate::diag::{FreesError, Result};
use crate::engine::VariableOverride;
use crate::solver::SolverSettings;

/// The value a failed solve or a non-finite objective reports.
const PENALTY: f64 = 1e12;
/// Distribution index of the simulated-binary crossover.
const SBX_ETA: f64 = 15.0;
/// Distribution index of the polynomial mutation.
const MUT_ETA: f64 = 20.0;
/// Solved-variable name prefix for serialised constraint left-hand sides.
const CON_VAR_PREFIX: &str = "zz_mo_con_";
/// Feasibility threshold of Deb's constraint-domination rule.
const FEASIBLE_EPS: f64 = 1e-9;
/// Crossover is skipped entirely with probability 0.1 (`nextDouble() > 0.9`).
const SBX_PROBABILITY_GATE: f64 = 0.9;
/// Per-gene crossover gate (`nextDouble() > 0.5` ⇒ skip this gene).
const SBX_GENE_GATE: f64 = 0.5;
/// Two parents whose gene differs by less than this are not crossed.
const SBX_GENE_EPS: f64 = 1e-14;
/// Two Pareto points closer than this (relative) are treated as one.
const DEDUPE_EPS: f64 = 1e-9;
/// Population ceiling — `OptimizeController.clampPositive(populationSize, 40, 200)`.
/// See [`optimize_multi`] for why the port enforces it here rather than upstream.
const MAX_POPULATION: usize = 200;

// ---------------------------------------------------------------------------
// Problem / result records
// ---------------------------------------------------------------------------

/// One multi-objective request — the Java `MultiObjectiveOptimizer.Problem`.
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    /// The document, verbatim. Decision assignments are appended to it.
    pub text: String,
    pub settings: SolverSettings,
    /// Per-variable guesses/bounds — the Java `specs` map.
    pub overrides: Vec<VariableOverride>,
    /// The objective variables (at least two, per the controller's validation).
    pub objectives: Vec<String>,
    /// Aligned with [`Problem::objectives`]: `true` ⇒ that objective is
    /// maximised (negated internally).
    pub maximize: Vec<bool>,
    pub decisions: Vec<String>,
    pub lowers: Vec<f64>,
    pub uppers: Vec<f64>,
    /// Floored at 8 (`Math.max(8, populationSize)`); the controller clamps the
    /// request to `[1, 200]` with a default of 40 before it gets here.
    pub population_size: usize,
    pub generations: usize,
    /// `OptimizeController` passes `42`.
    pub seed: i64,
    /// `expr <= value` / `expr >= value` / `expr = value`.
    pub constraints: Vec<String>,
    /// Externally supplied Function Table definitions.
    pub extra_tables: Vec<crate::parser::defs::FunctionTableDef>,
}

/// One Pareto point: the decision vector and the raw (user-facing) objectives.
#[derive(Debug, Clone, PartialEq)]
pub struct ParetoPoint {
    pub decisions: Vec<f64>,
    pub objectives: Vec<f64>,
}

/// The Java `MultiObjectiveOptimizer.Result`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParetoResult {
    /// The first non-dominated rank of the final population, sorted by the
    /// first objective and deduplicated.
    pub front: Vec<ParetoPoint>,
    /// Full system solves spent.
    pub evaluations: usize,
}

/// A parsed inequality/equality constraint `lhsExpr <op> rhs`.
///
/// Note this is **not** the same parser as [`crate::analysis::optimizer`]'s:
/// the Java `MultiObjectiveOptimizer.parseConstraint` scans with `indexOf`
/// rather than a regex, so `a<=b>=c` splits differently in the two classes.
/// Both are transcribed as written.
#[derive(Debug, Clone, PartialEq)]
struct Constraint {
    lhs_expr: String,
    operator: Op,
    rhs: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Le,
    Ge,
    Eq,
}

/// One member of the population.
#[derive(Debug, Clone)]
struct Individual {
    /// Decisions.
    x: Vec<f64>,
    /// Objectives as the user reads them.
    obj_raw: Vec<f64>,
    /// Objectives in minimisation form.
    obj_min: Vec<f64>,
    /// Total constraint violation (0 ⇒ feasible).
    violation: f64,
    rank: usize,
    crowding: f64,
}

impl Individual {
    fn new(x: Vec<f64>) -> Individual {
        Individual {
            x,
            obj_raw: Vec::new(),
            obj_min: Vec::new(),
            violation: 0.0,
            rank: 0,
            crowding: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Run NSGA-II — the Java `MultiObjectiveOptimizer.optimize`.
///
/// # Errors
///
/// [`FreesError::Solver`] when a constraint string is malformed. Failed
/// *solves* are not errors: they score as [`PENALTY`] on every objective and a
/// full violation, so an infeasible corner of the box simply loses.
pub fn optimize_multi(p: &Problem) -> Result<ParetoResult> {
    let n = p.decisions.len();
    // `Math.max(8, populationSize)` in the Java, plus the ceiling its caller
    // applies. `OptimizeController` clamps the request to `[1, 200]` before
    // constructing the problem; this port has no controller and `optimize_multi`
    // is a public entry point, so a `population_size` of `usize::MAX` would
    // reserve `MAX * size_of::<Individual>()` and abort the process — the same
    // defect `montecarlo::run` had with its sample count, and unrecoverable for
    // the same reason (`panic = "abort"` on wasm32). The ceiling is the
    // controller's own, so no in-range request behaves differently.
    let pop_size = p.population_size.clamp(8, MAX_POPULATION);
    let mut rng = JavaRandom::new(p.seed);
    let mut evaluations = 0usize;
    let constraints = parse_constraints(&p.constraints)?;

    let mut population: Vec<Individual> = Vec::with_capacity(pop_size);
    for _ in 0..pop_size {
        let individual = Individual::new(random_decisions(p, n, &mut rng));
        population.push(evaluate(individual, p, &constraints, &mut evaluations));
    }

    for _gen in 0..p.generations {
        assign_ranks_and_crowding(&mut population);
        let mut offspring: Vec<Individual> = Vec::with_capacity(pop_size);
        while offspring.len() < pop_size {
            let parent_a = tournament(&population, &mut rng);
            let parent_b = tournament(&population, &mut rng);
            let (child0, child1) = sbx_crossover(
                &population[parent_a].x.clone(),
                &population[parent_b].x.clone(),
                p,
                &mut rng,
            );
            let mutated0 = mutate(&child0, p, &mut rng);
            offspring.push(evaluate(
                Individual::new(mutated0),
                p,
                &constraints,
                &mut evaluations,
            ));
            if offspring.len() < pop_size {
                let mutated1 = mutate(&child1, p, &mut rng);
                offspring.push(evaluate(
                    Individual::new(mutated1),
                    p,
                    &constraints,
                    &mut evaluations,
                ));
            }
        }
        let mut combined = population;
        combined.extend(offspring);
        population = select_next_generation(combined, pop_size);
    }

    assign_ranks_and_crowding(&mut population);
    let mut front: Vec<ParetoPoint> = population
        .iter()
        .filter(|ind| ind.rank == 0)
        .map(|ind| ParetoPoint {
            decisions: ind.x.clone(),
            objectives: ind.obj_raw.clone(),
        })
        .collect();
    front.sort_by(|a, b| java_compare(a.objectives[0], b.objectives[0]));
    Ok(ParetoResult {
        front: dedupe(front),
        evaluations,
    })
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

/// The Java `evaluate`: one full system solve reads every objective *and* every
/// constraint LHS at once.
fn evaluate(
    mut ind: Individual,
    p: &Problem,
    constraints: &[Constraint],
    evaluations: &mut usize,
) -> Individual {
    *evaluations += 1;
    let m = p.objectives.len();
    ind.obj_raw = vec![0.0; m];
    ind.obj_min = vec![0.0; m];
    let solved = solve_with_decisions(p, &ind.x, constraints);
    for j in 0..m {
        let raw = solved
            .get(&p.objectives[j].to_ascii_lowercase())
            .copied()
            .filter(|v| v.is_finite())
            .unwrap_or(f64::NAN);
        ind.obj_raw[j] = raw;
        ind.obj_min[j] = minimisation_value(raw, p.maximize.get(j).copied().unwrap_or(false));
    }
    ind.violation = total_violation(constraints, &solved);
    ind
}

fn minimisation_value(raw: f64, maximize: bool) -> f64 {
    if raw.is_nan() {
        return PENALTY;
    }
    if maximize {
        -raw
    } else {
        raw
    }
}

/// The Java `solveWithDecisions`: pin the decisions, add one
/// `zz_mo_con_<i> = <lhs>` equation per constraint, solve once. A parse or
/// solver failure answers an empty map (`Map.of()`).
fn solve_with_decisions(
    p: &Problem,
    x: &[f64],
    constraints: &[Constraint],
) -> std::collections::BTreeMap<String, f64> {
    let mut text = String::with_capacity(p.text.len() + 32 * (x.len() + constraints.len()));
    text.push_str(&p.text);
    for (name, value) in p.decisions.iter().zip(x) {
        text.push('\n');
        text.push_str(name);
        text.push_str(" = ");
        text.push_str(&plain_string(*value));
    }
    for (i, c) in constraints.iter().enumerate() {
        text.push('\n');
        text.push_str(CON_VAR_PREFIX);
        text.push_str(&i.to_string());
        text.push_str(" = ");
        text.push_str(&c.lhs_expr);
    }
    match crate::engine::solve_with_tables(&text, &p.settings, &p.overrides, &p.extra_tables) {
        Ok(solution) => solution.values,
        Err(_) => std::collections::BTreeMap::new(),
    }
}

/// The Java `totalViolation`: sum of normalised constraint violations, 0 ⇒
/// feasible. A constraint whose LHS did not solve costs a full [`PENALTY`].
fn total_violation(
    constraints: &[Constraint],
    solved: &std::collections::BTreeMap<String, f64>,
) -> f64 {
    let mut total = 0.0f64;
    for (i, c) in constraints.iter().enumerate() {
        let key = format!("{CON_VAR_PREFIX}{i}");
        let Some(&lhs) = solved.get(&key).filter(|v| v.is_finite()) else {
            total += PENALTY;
            continue;
        };
        let g = match c.operator {
            Op::Le => jmax(0.0, lhs - c.rhs),
            Op::Ge => jmax(0.0, c.rhs - lhs),
            Op::Eq => (lhs - c.rhs).abs(),
        };
        total += g / (1.0 + c.rhs.abs());
    }
    total
}

/// The Java `parseConstraints` / `parseConstraint`: `<=` wins wherever it
/// appears, then `>=`, then the first bare `=`.
fn parse_constraints(raw: &[String]) -> Result<Vec<Constraint>> {
    let mut out = Vec::new();
    for line in raw {
        let s = line.trim();
        if s.is_empty() {
            continue;
        }
        out.push(parse_constraint(line, s)?);
    }
    Ok(out)
}

fn parse_constraint(original: &str, s: &str) -> Result<Constraint> {
    let (operator, token) = if s.contains("<=") {
        (Op::Le, "<=")
    } else if s.contains(">=") {
        (Op::Ge, ">=")
    } else if s.contains('=') {
        (Op::Eq, "=")
    } else {
        return Err(FreesError::solver(format!(
            "Constraint '{original}' needs <=, >= or =."
        )));
    };
    let idx = s.find(token).expect("the token was just found");
    let lhs = s[..idx].trim().to_string();
    let rhs_text = s[idx + token.len()..].trim();
    let rhs = rhs_text.parse::<f64>().map_err(|_| {
        FreesError::solver(format!("Constraint '{original}' must end with a number."))
    })?;
    Ok(Constraint {
        lhs_expr: lhs,
        operator,
        rhs,
    })
}

// ---------------------------------------------------------------------------
// NSGA-II core
// ---------------------------------------------------------------------------

/// Deb's constraint-domination: feasible beats infeasible; among infeasible,
/// lower violation wins; among feasible, standard Pareto dominance applies.
fn dominates(a: &Individual, b: &Individual) -> bool {
    let a_feasible = a.violation <= FEASIBLE_EPS;
    let b_feasible = b.violation <= FEASIBLE_EPS;
    if a_feasible != b_feasible {
        return a_feasible;
    }
    if !a_feasible {
        return a.violation < b.violation;
    }
    let mut strictly_better = false;
    for i in 0..a.obj_min.len() {
        if a.obj_min[i] > b.obj_min[i] {
            return false;
        }
        if a.obj_min[i] < b.obj_min[i] {
            strictly_better = true;
        }
    }
    strictly_better
}

/// Fast non-dominated sort. Sets `rank` on every individual and returns the
/// fronts as index lists into `pop`, in rank order.
fn non_dominated_sort(pop: &mut [Individual]) -> Vec<Vec<usize>> {
    let n = pop.len();
    let mut dominated: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut domination_count = vec![0usize; n];
    compute_domination(pop, &mut dominated, &mut domination_count);
    build_fronts(pop, &dominated, &mut domination_count)
}

/// The Java `computeDomination`: for each individual, the indices it dominates
/// and how many dominate it. Only the upper triangle is walked, as in the Java.
fn compute_domination(
    pop: &[Individual],
    dominated: &mut [Vec<usize>],
    domination_count: &mut [usize],
) {
    let n = pop.len();
    for i in 0..n {
        for j in i + 1..n {
            if dominates(&pop[i], &pop[j]) {
                dominated[i].push(j);
                domination_count[j] += 1;
            } else if dominates(&pop[j], &pop[i]) {
                dominated[j].push(i);
                domination_count[i] += 1;
            }
        }
    }
}

/// The Java `buildFronts`.
fn build_fronts(
    pop: &mut [Individual],
    dominated: &[Vec<usize>],
    domination_count: &mut [usize],
) -> Vec<Vec<usize>> {
    let mut fronts: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    for i in 0..pop.len() {
        if domination_count[i] == 0 {
            pop[i].rank = 0;
            current.push(i);
        }
    }
    let mut rank = 0usize;
    while !current.is_empty() {
        let mut front_inds: Vec<usize> = Vec::with_capacity(current.len());
        let mut next: Vec<usize> = Vec::new();
        for &idx in &current {
            front_inds.push(idx);
            for &dj in &dominated[idx] {
                domination_count[dj] -= 1;
                if domination_count[dj] == 0 {
                    pop[dj].rank = rank + 1;
                    next.push(dj);
                }
            }
        }
        fronts.push(front_inds);
        current = next;
        rank += 1;
    }
    fronts
}

fn assign_ranks_and_crowding(pop: &mut [Individual]) {
    let mut fronts = non_dominated_sort(pop);
    for front in &mut fronts {
        assign_crowding(pop, front);
    }
}

/// The Java `assignCrowding`.
///
/// The front is re-sorted **once per objective** and the loop leaves it sorted
/// by the last one; that residual order is what
/// [`select_next_generation`] copies into the next population, so the sort is
/// load-bearing, not incidental.
fn assign_crowding(pop: &mut [Individual], front: &mut [usize]) {
    for &i in front.iter() {
        pop[i].crowding = 0.0;
    }
    if front.is_empty() {
        return;
    }
    let m = pop[front[0]].obj_min.len();
    for o in 0..m {
        front.sort_by(|&a, &b| java_compare(pop[a].obj_min[o], pop[b].obj_min[o]));
        let last = front.len() - 1;
        pop[front[0]].crowding = f64::INFINITY;
        pop[front[last]].crowding = f64::INFINITY;
        let range = pop[front[last]].obj_min[o] - pop[front[0]].obj_min[o];
        if range <= 0.0 {
            continue;
        }
        for i in 1..last {
            let delta = pop[front[i + 1]].obj_min[o] - pop[front[i - 1]].obj_min[o];
            pop[front[i]].crowding += delta / range;
        }
    }
}

/// The Java `selectNextGeneration`: elitist (μ+λ) replacement, filling whole
/// fronts while they fit and then the least-crowded members of the front that
/// straddles the boundary.
fn select_next_generation(mut combined: Vec<Individual>, pop_size: usize) -> Vec<Individual> {
    let mut fronts = non_dominated_sort(&mut combined);
    let mut next: Vec<usize> = Vec::with_capacity(pop_size);
    for front in &mut fronts {
        assign_crowding(&mut combined, front);
        if next.len() + front.len() <= pop_size {
            next.extend(front.iter().copied());
        } else {
            // `Comparator.comparingDouble(crowding).reversed()` — a stable
            // descending sort, so equal-crowding members keep the order the
            // last objective sort left them in.
            front.sort_by(|&a, &b| java_compare(combined[b].crowding, combined[a].crowding));
            for &idx in front.iter() {
                if next.len() >= pop_size {
                    break;
                }
                next.push(idx);
            }
            break;
        }
    }
    // Materialise in selection order without disturbing it.
    let mut taken: Vec<Option<Individual>> = combined.into_iter().map(Some).collect();
    next.into_iter()
        .map(|i| taken[i].take().expect("each index is selected once"))
        .collect()
}

/// The Java `tournament`: binary tournament on `(rank, crowding)`, ties going
/// to `a`. Returns an index so the caller can clone the parent's genes.
fn tournament(pop: &[Individual], rng: &mut JavaRandom) -> usize {
    let a = rng.next_int(pop.len() as i32) as usize;
    let b = rng.next_int(pop.len() as i32) as usize;
    if pop[a].rank != pop[b].rank {
        return if pop[a].rank < pop[b].rank { a } else { b };
    }
    if pop[a].crowding >= pop[b].crowding {
        a
    } else {
        b
    }
}

// ---------------------------------------------------------------------------
// Variation operators
// ---------------------------------------------------------------------------

/// The Java `randomDecisions`: uniform inside the box, `n` draws.
fn random_decisions(p: &Problem, n: usize, rng: &mut JavaRandom) -> Vec<f64> {
    let mut x = Vec::with_capacity(n);
    for i in 0..n {
        let lo = p.lowers[i];
        let hi = p.uppers[i];
        x.push(lo + rng.next_double() * (hi - lo));
    }
    x
}

/// The Java `sbxCrossover` — simulated-binary crossover, η = [`SBX_ETA`].
///
/// The draw pattern is exact: one gate draw for the whole operator, then **one
/// draw per gene** for the per-gene gate (the `||` short-circuits *after* it,
/// so the draw always happens), and a third draw only for genes that actually
/// cross.
fn sbx_crossover(a: &[f64], b: &[f64], p: &Problem, rng: &mut JavaRandom) -> (Vec<f64>, Vec<f64>) {
    let n = a.len();
    let mut c1 = a.to_vec();
    let mut c2 = b.to_vec();
    if rng.next_double() > SBX_PROBABILITY_GATE {
        return (c1, c2);
    }
    for i in 0..n {
        if rng.next_double() > SBX_GENE_GATE || (a[i] - b[i]).abs() < SBX_GENE_EPS {
            continue;
        }
        let lo = p.lowers[i];
        let hi = p.uppers[i];
        let u = rng.next_double();
        let beta = if u <= 0.5 {
            (2.0 * u).powf(1.0 / (SBX_ETA + 1.0))
        } else {
            (1.0 / (2.0 * (1.0 - u))).powf(1.0 / (SBX_ETA + 1.0))
        };
        let x1 = 0.5 * ((1.0 + beta) * a[i] + (1.0 - beta) * b[i]);
        let x2 = 0.5 * ((1.0 - beta) * a[i] + (1.0 + beta) * b[i]);
        c1[i] = clamp(x1, lo, hi);
        c2[i] = clamp(x2, lo, hi);
    }
    (c1, c2)
}

/// The Java `mutate` — polynomial mutation, η = [`MUT_ETA`], rate `1/n`.
///
/// One draw per gene for the rate test (the `&&` short-circuits after it), plus
/// one more for the genes that actually mutate.
fn mutate(x: &[f64], p: &Problem, rng: &mut JavaRandom) -> Vec<f64> {
    let n = x.len();
    let mut out = x.to_vec();
    let rate = 1.0 / n as f64;
    for i in 0..n {
        let lo = p.lowers[i];
        let hi = p.uppers[i];
        let range = hi - lo;
        if rng.next_double() <= rate && range > 0.0 {
            let u = rng.next_double();
            let delta = if u < 0.5 {
                (2.0 * u).powf(1.0 / (MUT_ETA + 1.0)) - 1.0
            } else {
                1.0 - (2.0 * (1.0 - u)).powf(1.0 / (MUT_ETA + 1.0))
            };
            out[i] = clamp(x[i] + delta * range, lo, hi);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The Java `dedupe`: keep the first of every near-equal objective vector.
fn dedupe(front: Vec<ParetoPoint>) -> Vec<ParetoPoint> {
    let mut out: Vec<ParetoPoint> = Vec::with_capacity(front.len());
    for pt in front {
        if !out
            .iter()
            .any(|kept| close(&kept.objectives, &pt.objectives))
        {
            out.push(pt);
        }
    }
    out
}

fn close(a: &[f64], b: &[f64]) -> bool {
    for i in 0..a.len() {
        if (a[i] - b[i]).abs() > DEDUPE_EPS * (1.0 + b[i].abs()) {
            return false;
        }
    }
    true
}

/// `Math.max(lo, Math.min(hi, v))` — NaN-propagating, unlike `f64::clamp`
/// (which also panics when `lo > hi`).
fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    jmax(lo, jmin(hi, v))
}

// ---------------------------------------------------------------------------
// java.util.Random
// ---------------------------------------------------------------------------

/// A bit-exact port of `java.util.Random` (the 48-bit LCG specified in its
/// Javadoc), so a seeded NSGA-II run reproduces the Java engine's stream draw
/// for draw.
///
/// Only the three methods `MultiObjectiveOptimizer` uses are provided.
#[derive(Debug, Clone)]
pub struct JavaRandom {
    seed: i64,
}

impl JavaRandom {
    const MULTIPLIER: i64 = 0x5_DEEC_E66D;
    const ADDEND: i64 = 0xB;
    const MASK: i64 = (1 << 48) - 1;

    /// `new Random(seed)` — the constructor scrambles the seed.
    pub fn new(seed: i64) -> JavaRandom {
        JavaRandom {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    /// `protected int next(int bits)`.
    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        // `seed` is masked to 48 non-negative bits, so `>>` is Java's `>>>`.
        (self.seed >> (48 - bits)) as i32
    }

    /// `public double nextDouble()`.
    pub fn next_double(&mut self) -> f64 {
        let hi = i64::from(self.next(26)) << 27;
        let lo = i64::from(self.next(27));
        (hi + lo) as f64 * f64::from_bits(0x3CA0_0000_0000_0000) // 0x1.0p-53
    }

    /// `public int nextInt(int bound)` — power-of-two fast path plus the
    /// rejection loop, including its reliance on 32-bit overflow.
    ///
    /// # Panics
    ///
    /// When `bound <= 0`, mirroring the Java `IllegalArgumentException`. The
    /// only caller passes a population size floored at 8.
    pub fn next_int(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        let mut r = self.next(31);
        let m = bound - 1;
        if (bound & m) == 0 {
            // bound is a power of two
            r = ((i64::from(bound) * i64::from(r)) >> 31) as i32;
        } else {
            let mut u = r;
            loop {
                r = u % bound;
                if u.wrapping_sub(r).wrapping_add(m) >= 0 {
                    break;
                }
                u = self.next(31);
            }
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── java.util.Random parity ───────────────────────────────────────────

    #[test]
    fn java_random_matches_the_reference_stream() {
        // Ground truth: `new Random(42)` on this machine's JDK, first five
        // `nextDouble()`s, printed at full `Double.toString` precision.
        let mut rng = JavaRandom::new(42);
        let expected = [
            0.727_563_680_032_868_1,
            0.683_223_471_759_845_4,
            0.308_719_455_332_659_76,
            0.277_078_490_074_136_65,
            0.665_548_951_794_573_6,
        ];
        for want in expected {
            let got = rng.next_double();
            assert_eq!(got, want, "nextDouble mismatch");
        }
    }

    #[test]
    fn java_random_next_int_matches_the_reference_stream() {
        // Ground truth: `new Random(42)`, first eight `nextInt(40)`s — the
        // rejection-loop path, since 40 is not a power of two.
        let mut rng = JavaRandom::new(42);
        let expected = [10, 3, 8, 4, 10, 5, 25, 38];
        for want in expected {
            assert_eq!(rng.next_int(40), want);
        }
    }

    #[test]
    fn java_random_power_of_two_bound_takes_the_fast_path() {
        // Ground truth: `new Random(42)`, first eight `nextInt(8)`s.
        let mut rng = JavaRandom::new(42);
        let expected = [5, 0, 5, 0, 2, 7, 2, 5];
        for want in expected {
            assert_eq!(rng.next_int(8), want);
        }
    }

    #[test]
    fn the_same_seed_reproduces_the_same_stream() {
        let mut a = JavaRandom::new(7);
        let mut b = JavaRandom::new(7);
        for _ in 0..64 {
            assert_eq!(a.next_double(), b.next_double());
        }
    }

    // ── constraint parsing ────────────────────────────────────────────────

    #[test]
    fn constraint_parsing_prefers_le_then_ge_then_eq() {
        let raw = vec![
            "x <= 5".to_string(),
            "y >= -1".to_string(),
            "z = 2.5".to_string(),
            "  ".to_string(), // blank lines are skipped
        ];
        let parsed = parse_constraints(&raw).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].operator, Op::Le);
        assert_eq!(parsed[0].lhs_expr, "x");
        assert_eq!(parsed[1].operator, Op::Ge);
        assert_eq!(parsed[1].rhs, -1.0);
        assert_eq!(parsed[2].operator, Op::Eq);
        assert_eq!(parsed[2].rhs, 2.5);
    }

    #[test]
    fn constraint_parsing_rejects_missing_operator_and_non_numeric_rhs() {
        assert!(parse_constraints(&["x + y".to_string()]).is_err());
        assert!(parse_constraints(&["x <= y".to_string()]).is_err());
    }

    #[test]
    fn violation_is_zero_when_feasible_and_normalised_otherwise() {
        let constraints = vec![Constraint {
            lhs_expr: "a".into(),
            operator: Op::Le,
            rhs: 10.0,
        }];
        let mut solved = std::collections::BTreeMap::new();
        solved.insert("zz_mo_con_0".to_string(), 4.0);
        assert_eq!(total_violation(&constraints, &solved), 0.0);
        solved.insert("zz_mo_con_0".to_string(), 21.0);
        // (21 - 10) / (1 + 10)
        assert!((total_violation(&constraints, &solved) - 1.0).abs() < 1e-12);
        // A missing LHS costs a full penalty.
        solved.clear();
        assert_eq!(total_violation(&constraints, &solved), PENALTY);
    }

    // ── NSGA-II primitives ────────────────────────────────────────────────

    fn ind(obj_min: &[f64], violation: f64) -> Individual {
        Individual {
            x: vec![0.0],
            obj_raw: obj_min.to_vec(),
            obj_min: obj_min.to_vec(),
            violation,
            rank: 0,
            crowding: 0.0,
        }
    }

    #[test]
    fn dominance_is_debs_constraint_domination() {
        let feasible = ind(&[1.0, 1.0], 0.0);
        let infeasible = ind(&[0.0, 0.0], 1.0);
        assert!(
            dominates(&feasible, &infeasible),
            "feasible beats infeasible"
        );
        assert!(!dominates(&infeasible, &feasible));

        let less_violating = ind(&[9.0, 9.0], 0.5);
        assert!(dominates(&less_violating, &infeasible));

        let better = ind(&[0.0, 1.0], 0.0);
        assert!(
            dominates(&better, &feasible),
            "weakly better in both, strictly in one"
        );
        assert!(!dominates(&feasible, &better));

        let a = ind(&[0.0, 5.0], 0.0);
        let b = ind(&[5.0, 0.0], 0.0);
        assert!(!dominates(&a, &b), "a trade-off dominates nothing");
        assert!(!dominates(&b, &a));

        let same = ind(&[1.0, 1.0], 0.0);
        assert!(!dominates(&same, &feasible), "equal never dominates");
    }

    #[test]
    fn non_dominated_sort_ranks_a_known_population() {
        let mut pop = vec![
            ind(&[0.0, 3.0], 0.0), // rank 0
            ind(&[1.0, 1.0], 0.0), // rank 0
            ind(&[3.0, 0.0], 0.0), // rank 0
            ind(&[2.0, 2.0], 0.0), // dominated by [1,1] → rank 1
            ind(&[5.0, 5.0], 0.0), // rank 2
        ];
        let fronts = non_dominated_sort(&mut pop);
        assert_eq!(fronts.len(), 3);
        assert_eq!(fronts[0], vec![0, 1, 2]);
        assert_eq!(fronts[1], vec![3]);
        assert_eq!(fronts[2], vec![4]);
        assert_eq!(pop[0].rank, 0);
        assert_eq!(pop[3].rank, 1);
        assert_eq!(pop[4].rank, 2);
    }

    #[test]
    fn crowding_gives_the_extremes_infinity_and_scales_the_middle() {
        let mut pop = vec![
            ind(&[0.0, 4.0], 0.0),
            ind(&[1.0, 2.0], 0.0),
            ind(&[4.0, 0.0], 0.0),
        ];
        let mut front = vec![0, 1, 2];
        assign_crowding(&mut pop, &mut front);
        assert!(pop[0].crowding.is_infinite());
        assert!(pop[2].crowding.is_infinite());
        // Interior: (4-0)/4 on objective 0 plus (4-0)/4 on objective 1.
        assert!((pop[1].crowding - 2.0).abs() < 1e-12, "{}", pop[1].crowding);
        // The front is left sorted by the LAST objective, ascending.
        assert_eq!(front, vec![2, 1, 0]);
    }

    #[test]
    fn crowding_leaves_a_degenerate_objective_alone() {
        // Every point shares objective 1, so its range is 0 and it contributes
        // nothing — but the extremes are still infinite.
        let mut pop = vec![
            ind(&[0.0, 1.0], 0.0),
            ind(&[1.0, 1.0], 0.0),
            ind(&[2.0, 1.0], 0.0),
        ];
        let mut front = vec![0, 1, 2];
        assign_crowding(&mut pop, &mut front);
        assert!(pop[0].crowding.is_infinite());
        assert!(pop[2].crowding.is_infinite());
        assert!((pop[1].crowding - 1.0).abs() < 1e-12, "{}", pop[1].crowding);
    }

    #[test]
    fn select_next_generation_is_elitist_and_truncates_by_crowding() {
        let combined = vec![
            ind(&[0.0, 3.0], 0.0), // front 0
            ind(&[1.0, 1.0], 0.0), // front 0
            ind(&[3.0, 0.0], 0.0), // front 0
            ind(&[9.0, 9.0], 0.0), // front 1
        ];
        let next = select_next_generation(combined, 3);
        assert_eq!(next.len(), 3);
        // The whole first front fits, so the dominated point is dropped.
        assert!(next.iter().all(|i| i.obj_min != vec![9.0, 9.0]));
    }

    #[test]
    fn sbx_and_mutation_stay_inside_the_box() {
        let p = Problem {
            text: String::new(),
            settings: SolverSettings::default(),
            overrides: Vec::new(),
            objectives: vec!["f".into(), "g".into()],
            maximize: vec![false, false],
            decisions: vec!["a".into(), "b".into()],
            lowers: vec![-1.0, -1.0],
            uppers: vec![1.0, 1.0],
            population_size: 8,
            generations: 1,
            seed: 42,
            constraints: Vec::new(),
            extra_tables: Vec::new(),
        };
        let mut rng = JavaRandom::new(1);
        for _ in 0..200 {
            let a = random_decisions(&p, 2, &mut rng);
            let b = random_decisions(&p, 2, &mut rng);
            let (c1, c2) = sbx_crossover(&a, &b, &p, &mut rng);
            for child in [&c1, &c2] {
                let m = mutate(child, &p, &mut rng);
                for v in m {
                    assert!((-1.0..=1.0).contains(&v), "{v} escaped the box");
                }
            }
        }
    }

    #[test]
    fn dedupe_keeps_the_first_of_near_equal_points() {
        let front = vec![
            ParetoPoint {
                decisions: vec![1.0],
                objectives: vec![1.0, 2.0],
            },
            ParetoPoint {
                decisions: vec![1.000_000_000_1],
                objectives: vec![1.0, 2.0],
            },
            ParetoPoint {
                decisions: vec![2.0],
                objectives: vec![3.0, 4.0],
            },
        ];
        let out = dedupe(front);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].decisions, vec![1.0]);
    }

    // ── end to end ────────────────────────────────────────────────────────

    fn two_objective_problem(generations: usize, population: usize) -> Problem {
        // A classic convex trade-off: f = x², g = (x - 2)². Any x in [0, 2] is
        // Pareto-optimal; nothing outside is.
        Problem {
            text: "f = x^2\ng = (x - 2)^2".to_string(),
            settings: SolverSettings::default(),
            overrides: Vec::new(),
            objectives: vec!["f".into(), "g".into()],
            maximize: vec![false, false],
            decisions: vec!["x".into()],
            lowers: vec![-2.0],
            uppers: vec![4.0],
            population_size: population,
            generations,
            seed: 42,
            constraints: Vec::new(),
            extra_tables: Vec::new(),
        }
    }

    #[test]
    fn the_front_is_mutually_non_dominated() {
        let result = optimize_multi(&two_objective_problem(6, 12)).unwrap();
        assert!(!result.front.is_empty());
        assert!(result.evaluations > 0);
        for a in &result.front {
            for b in &result.front {
                if std::ptr::eq(a, b) {
                    continue;
                }
                let a_ind = ind(&a.objectives, 0.0);
                let b_ind = ind(&b.objectives, 0.0);
                assert!(
                    !dominates(&a_ind, &b_ind),
                    "{:?} dominates {:?}",
                    a.objectives,
                    b.objectives
                );
            }
        }
    }

    #[test]
    fn the_front_lands_on_the_known_pareto_set() {
        let result = optimize_multi(&two_objective_problem(12, 16)).unwrap();
        for pt in &result.front {
            let x = pt.decisions[0];
            assert!(
                (-0.15..=2.15).contains(&x),
                "x = {x} is outside the Pareto set [0, 2]"
            );
        }
    }

    #[test]
    fn the_front_is_sorted_by_the_first_objective() {
        let result = optimize_multi(&two_objective_problem(8, 12)).unwrap();
        for pair in result.front.windows(2) {
            assert!(
                pair[0].objectives[0] <= pair[1].objectives[0],
                "front is not sorted by objective 0"
            );
        }
    }

    #[test]
    fn the_same_seed_reproduces_the_same_front() {
        let a = optimize_multi(&two_objective_problem(5, 10)).unwrap();
        let b = optimize_multi(&two_objective_problem(5, 10)).unwrap();
        assert_eq!(a.evaluations, b.evaluations);
        assert_eq!(a.front, b.front);
    }

    #[test]
    fn a_different_seed_gives_a_different_run() {
        let mut p = two_objective_problem(5, 10);
        let a = optimize_multi(&p).unwrap();
        p.seed = 4242;
        let b = optimize_multi(&p).unwrap();
        assert_ne!(a.front, b.front, "the RNG seed must actually matter");
    }

    #[test]
    fn population_size_is_floored_at_eight() {
        let mut p = two_objective_problem(1, 1);
        p.population_size = 1;
        let result = optimize_multi(&p).unwrap();
        // 8 initial + 8 offspring for one generation.
        assert_eq!(result.evaluations, 16);
    }

    #[test]
    fn maximised_objectives_are_negated_internally() {
        // Maximise f = -x² (peak at 0) against minimising g = (x - 2)².
        let mut p = two_objective_problem(8, 12);
        p.text = "f = -(x^2)\ng = (x - 2)^2".to_string();
        p.maximize = vec![true, false];
        let result = optimize_multi(&p).unwrap();
        for pt in &result.front {
            let x = pt.decisions[0];
            assert!((-0.15..=2.15).contains(&x), "x = {x}");
        }
    }

    #[test]
    fn a_constraint_pushes_the_front_into_the_feasible_region() {
        let mut p = two_objective_problem(12, 16);
        p.constraints = vec!["x >= 1".to_string()];
        let result = optimize_multi(&p).unwrap();
        assert!(!result.front.is_empty());
        for pt in &result.front {
            assert!(
                pt.decisions[0] >= 0.9,
                "x = {} violates x >= 1",
                pt.decisions[0]
            );
        }
    }

    // ── parity with the Java oracle ───────────────────────────────────────
    //
    // Because `java.util.Random` is specified and the seed is fixed, NSGA-II is
    // reproducible, so these are *exact* expectations lifted from a run of the
    // real `com.frees.backend.core.MultiObjectiveOptimizer`. They pin the RNG
    // stream, the draw order of every operator, the crowding sorts and the
    // dedupe rule all at once: change any one and the front moves.

    #[test]
    fn oracle_one_generation_front_is_reproduced_point_for_point() {
        let mut p = two_objective_problem(1, 8);
        p.seed = 42;
        let result = optimize_multi(&p).unwrap();

        // Java: evals=16 front=7
        assert_eq!(result.evaluations, 16);
        let expected: [(f64, f64, f64); 7] = [
            (
                0.009_240_271_441_575_354,
                8.538_261_631_399_308e-5,
                3.963_124_296_850_012_7,
            ),
            (
                0.036_978_285_103_285_796,
                0.001_367_393_569_179_888_3,
                3.853_454_253_156_036_7,
            ),
            (
                0.212_505_967_349_312_49,
                0.045_158_786_159_067_064,
                3.195_134_916_761_817_5,
            ),
            (
                0.212_697_480_467_833_88,
                0.045_240_218_197_364_575,
                3.194_450_296_326_029_3,
            ),
            (
                0.407_403_087_725_107_6,
                0.165_977_275_887_951_7,
                2.536_364_924_987_521_5,
            ),
            (
                1.949_798_996_071_057_8,
                3.801_716_125_079_704_7,
                0.002_520_140_795_473_672_3,
            ),
            (
                1.993_293_710_767_441_8,
                3.973_219_817_385_037_7,
                4.497_431_527_072_641e-5,
            ),
        ];
        assert_eq!(result.front.len(), expected.len(), "{:#?}", result.front);
        for (got, (x, f, g)) in result.front.iter().zip(expected) {
            assert!(
                (got.decisions[0] - x).abs() <= 1e-14 * x.abs().max(1.0),
                "decision {} vs oracle {x}",
                got.decisions[0]
            );
            assert!(
                (got.objectives[0] - f).abs() <= 1e-12 * f.abs().max(1e-9),
                "objective f {} vs oracle {f}",
                got.objectives[0]
            );
            assert!(
                (got.objectives[1] - g).abs() <= 1e-12 * g.abs().max(1e-9),
                "objective g {} vs oracle {g}",
                got.objectives[1]
            );
        }
    }

    #[test]
    fn oracle_six_generation_run_keeps_the_same_budget_and_front_size() {
        // Java: pop 12, gens 6, seed 42 → evals=84 front=12, first decision
        // -0.00604009204096756 and last 1.9932937107674418.
        let mut p = two_objective_problem(6, 12);
        p.seed = 42;
        let result = optimize_multi(&p).unwrap();
        assert_eq!(result.evaluations, 84);
        assert_eq!(result.front.len(), 12, "{:#?}", result.front);
        assert!(
            (result.front[0].decisions[0] + 0.006_040_092_040_967_56).abs() < 1e-12,
            "{}",
            result.front[0].decisions[0]
        );
        assert!(
            (result.front[11].decisions[0] - 1.993_293_710_767_441_8).abs() < 1e-12,
            "{}",
            result.front[11].decisions[0]
        );
    }

    #[test]
    fn oracle_constrained_run_matches() {
        // Java: pop 16, gens 12, seed 42, `x >= 1` → evals=208 front=16,
        // first decision 1.0351302176786314.
        let mut p = two_objective_problem(12, 16);
        p.seed = 42;
        p.constraints = vec!["x >= 1".to_string()];
        let result = optimize_multi(&p).unwrap();
        assert_eq!(result.evaluations, 208);
        assert_eq!(result.front.len(), 16, "{:#?}", result.front);
        assert!(
            (result.front[0].decisions[0] - 1.035_130_217_678_631_4).abs() < 1e-12,
            "{}",
            result.front[0].decisions[0]
        );
    }

    #[test]
    fn a_malformed_constraint_is_an_error_not_a_silent_skip() {
        let mut p = two_objective_problem(1, 8);
        p.constraints = vec!["x is small".to_string()];
        assert!(optimize_multi(&p).is_err());
    }
}
