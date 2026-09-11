//! Analysis and design: optimization, fitting, sweeps and uncertainty.
//!
//! Port of `core/{Optimizer,MultiObjectiveOptimizer,AllRootsSolver,CurveFitter}`,
//! `api/{MonteCarlo,ParameterFit}`, `ast/ParametricTable` +
//! `core/ParametricAccessorContext` — 2,321 LOC.
//!
//! Everything here drives the EXISTING solver repeatedly; none of it may
//! reimplement Newton. Sweeps are embarrassingly parallel and are the reason
//! decision D3 chose a worker pool over shared-memory threads.
pub mod allroots;
pub mod curvefit;
pub mod distributions;
pub mod montecarlo;
pub mod optimizer;
pub mod parametric;
pub mod paramfit;
pub mod pareto;
pub mod sampling;
pub mod sensitivity;
pub mod uncertainty;
