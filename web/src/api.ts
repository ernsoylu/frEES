// All engine operations (solve, check, REPL, optimization, curve fitting,
// parametric tables, Monte Carlo, control, properties, diagrams) run fully in
// the browser inside a Web Worker (src/wasm/), emitting the same typed data
// structures this module has always exposed to the UI. Nothing in this module
// hits the network.
import {
  wasmCheck,
  wasmFluids,
  wasmPropertyDiagram,
  wasmPsychrometricChart,
  wasmReference,
  wasmReplClear,
  wasmReplEvaluate,
  wasmSolve,
  wasmStop,
  type ProgressListener,
  wasmCurveFit,
  wasmMonteCarlo,
  wasmOptimize,
  wasmOptimizeMulti,
  wasmParameterFit,
  wasmPidTune,
  wasmExtractPlant,
  wasmSolveTable,
} from './wasm/engineClient'
export type { ProgressListener } from './wasm/engineClient'

/** Cancels the in-flight solve or analysis operation by stopping the engine worker. */
export function stopSolve(): void {
  wasmStop()
}

export interface VariableResult {
  name: string
  value: number
  units: string
  uncertainty?: number | null
}

export interface BlockResult {
  index: number
  equations: string[]
  variables: string[]
}

export interface ResidualResult {
  equation: string
  value: number
}

export interface SolveStats {
  equations: number
  unknowns: number
  blocks: number
  iterations: number
  elapsedMillis: number
  maxResidual: number
}

export interface StopCriteria {
  maxIterations: number
  relativeResiduals: number
  changeInVariables: number
  elapsedTimeSeconds: number
  complexMode?: boolean
}

export const DEFAULT_STOP_CRITERIA: StopCriteria = {
  maxIterations: 250,
  relativeResiduals: 1e-12,
  changeInVariables: 1e-15,
  elapsedTimeSeconds: 3600,
}

export interface TableRowResult {
  success: boolean
  values: Record<string, number>
  error: string | null
}

export type UnitSystem = 'SI' | 'ENG_SI' | 'ENGLISH'

export const UNIT_SYSTEM_OPTIONS: { value: UnitSystem; label: string }[] = [
  { value: 'SI', label: 'SI base (Pa, J, W, K)' },
  { value: 'ENG_SI', label: 'Engineering SI (kPa, kJ, kW, °C)' },
  { value: 'ENGLISH', label: 'US / English (psi, Btu, hp, °F)' },
]

export interface SolutionResult {
  variables: VariableResult[]
  maxResidual: number
}

/** One uncertainty source's signed contribution to a variable (tornado bar). */
export interface UncertaintyContributionResult {
  source: string
  value: number
}

/** Tornado breakdown for one variable: propagated sigma + ranked sources. */
export interface VariableUncertaintyResult {
  variable: string
  total: number
  sources: UncertaintyContributionResult[]
}

export interface SolveResponse {
  success: boolean
  variables: VariableResult[]
  blocks: BlockResult[]
  residuals: ResidualResult[]
  stats: SolveStats | null
  solutions: SolutionResult[]
  unitWarnings: string[]
  error: string | null
  /** 1-based editor line a syntax error points at, or null for whole-system errors. */
  errorLine?: number | null
  cyclePath?: Record<string, number>[]
  /** Function tables parsed from TABLE ... END blocks in the editor text. */
  codeTables?: FunctionTableDto[]
  /** Parametric run-tables parsed from PARAMETRIC ... END blocks. */
  parametricTables?: ParametricTableDto[]
  /** Plots declared in the editor text with PLOT 'name' ... END blocks. */
  definedPlots?: PlotDefDto[]
  /** Fluid state tables declared with STATE TABLE ... END blocks. */
  stateTableDefs?: StateTableDto[]
  /** ODE Tables produced by solved DYNAMIC ... END blocks. */
  odeTables?: OdeTableDto[]
  /** Per-instance component metadata (type + parameter bindings) for the datasheet view. */
  components?: ComponentResult[]
  /** Index of the Tarjan block whose solve gave up (failure diagnostics), or null. */
  failedBlockIndex?: number | null
  /** Tornado breakdown: per uncertain variable, the ranked per-source contributions. */
  uncertaintyBreakdown?: VariableUncertaintyResult[]
}

/** One parameter binding on a component instance (`UA=UA_chl_r`, `SH=5`, `fluid$=R1234yf`). */
export interface ComponentParamResult {
  name: string
  /** Bound expression as written: a variable name, literal, or expression. */
  ref: string
  /** Resolved numeric value when the binding is a variable or number; null for strings/expressions. */
  value?: number | null
  units?: string | null
}

/** A solved component instance: its identity and the parameters it was built with. */
export interface ComponentResult {
  name: string
  type: string
  params: ComponentParamResult[]
}

export interface CheckResponse {
  solvable: boolean
  equations: number
  unknowns: number
  variables: string[]
  unitWarnings: string[]
  inferredUnits: Record<string, string>
  message: string
  /** 1-based editor line a syntax error points at, or null for whole-system errors. */
  errorLine?: number | null
  /** Every syntax error the parse collected (line/column 1-based), so the
   *  editor can mark them all — errorLine keeps pointing at the first. */
  errors?: { line: number; column: number; message: string }[]
  /** Function tables parsed from TABLE ... END blocks in the editor text. */
  codeTables?: FunctionTableDto[]
  /** Parametric run-tables parsed from PARAMETRIC ... END blocks. */
  parametricTables?: ParametricTableDto[]
  /** Plots declared in the editor text with PLOT 'name' ... END blocks. */
  definedPlots?: PlotDefDto[]
  /** Fluid state tables declared with STATE TABLE ... END blocks. */
  stateTableDefs?: StateTableDto[]
  /** Connection topology of the component network (domain + instance.port
   *  endpoints per node) — the rendered schematic's data layer. */
  connections?: ConnectionDto[]
  /** Component instances with source lines, for schematic navigation. */
  instances?: InstanceIdentityDto[]
  /** User COMPONENT definitions in the document. */
  definitions?: DefinitionIdentityDto[]
}

export interface InstanceIdentityDto {
  name: string
  label: string
  type: string
  line: number
  localType: boolean
}

export interface DefinitionIdentityDto {
  name: string
  line: number
}

/** One connection-topology node of the component network. */
export interface ConnectionDto {
  domain: string
  endpoints: string[]
  /** Fluid connector type (`liquid`, `twophase`, `gas`, `oil`, `moistair`,
   *  `fluid`); null outside the fluid domain. Distinguishes circuits the
   *  bond-graph domain lumps together — a coolant loop and a refrigerant loop
   *  are both `domain: 'fluid'`. */
  connector?: string | null
  /** The working fluid this node carries, when the model named one. */
  fluid?: string | null
  /** Per endpoint (aligned by index), the display prefix its member variables
   *  use — `chlr.in` for a connect-wired free port, `s2` for a shared-name
   *  stream. Lets the schematic show an endpoint's solved state. */
  streams?: string[]
}

/** A fluid state table parsed from a STATE TABLE name(...) ... END block: the
 * declared state-point variables and the fluid every state in the block uses
 * (null when no FLUID = ... line was given). */
export interface StateTableDto {
  name: string
  variables: string[]
  fluid: string | null
}

/** A parametric run-table parsed from a PARAMETRIC ... END block: the declared
 * variables and a row-major value grid (null cells where a column is short). */
export interface ParametricTableDto {
  name: string
  vars: string[]
  rows: (number | null)[][]
}

/** A plot parsed from a PLOT 'name' ... END block: the plot name and a raw
 * attribute map (lowercased keys → string values) the frontend maps onto a
 * PlotSpec via plotDefToSpec. */
export interface PlotDefDto {
  name: string
  attributes: Record<string, string[]>
}

/** An ODE Table produced by a solved DYNAMIC ... END block: columns are
 * [time, states…, auxiliaries…] and rows are the sampled trajectory. Shaped
 * like a parametric table so it renders in the Tables window and feeds the
 * Plots window through the same path. */
export interface OdeTableDto {
  name: string
  vars: string[]
  /** Per-column SI unit, aligned to `vars` (the ODE rows are SI). */
  units: string[]
  rows: (number | null)[][]
  events: { name: string; time: number }[]
  method: string
  stopped: boolean
  endTime: number
}

export interface VariableInfo {
  name: string
  guess: number | null
  lower: number | null
  upper: number | null
  units: string | null
  uncertainty: number | null
}

/** A Function Table in solver wire format (Epic 8): the table name is the
 * function name callable from equations; argNames lists the column names
 * (lookup argument first, then the family parameter, if any). */
export interface FunctionTableDto {
  name: string
  argNames: string[]
  xLog: boolean
  yLog: boolean
  curves: { param: number | null; points: number[][] }[]
}

export async function check(
  text: string,
  variableInfo: VariableInfo[],
  complexMode: boolean,
  functionTables: FunctionTableDto[] = [],
  overrides: string[] = [],
): Promise<CheckResponse> {
  try {
    // The wasm boundary consumes the same request body POST /api/check did
    // (unknown fields are ignored until their machinery ports) and returns a
    // full CheckResponse for every document problem — syntax errors arrive as
    // data with errorLine/errors[], never as a thrown exception.
    const data = await wasmCheck(
      text,
      JSON.stringify({ variableInfo, stopCriteria: { complexMode }, functionTables, overrides }),
    )
    return {
      solvable: data.solvable ?? false,
      equations: data.equations ?? 0,
      unknowns: data.unknowns ?? 0,
      variables: data.variables ?? [],
      unitWarnings: data.unitWarnings ?? [],
      inferredUnits: data.inferredUnits ?? {},
      message: data.message ?? '',
      errorLine: data.errorLine ?? null,
      errors: data.errors ?? [],
      // PLOT blocks reach the boundary (Phase 9), so the Plots tab populates
      // from a Check as well as from a Solve. The rest are not produced by the
      // wasm boundary yet; the App.tsx call sites treat empty collections as
      // "the document declares none".
      definedPlots: data.definedPlots ?? [],
      codeTables: [],
      parametricTables: [],
      stateTableDefs: [],
      connections: data.connections ?? [],
      instances: data.instances ?? [],
      definitions: data.definitions ?? [],
    }
  } catch (e) {
    // Only infrastructure can land here (worker died, wasm failed to load) —
    // document problems are data above.
    return {
      solvable: false,
      equations: 0,
      unknowns: 0,
      variables: [],
      unitWarnings: [],
      inferredUnits: {},
      message: `The in-browser solver failed (${e instanceof Error ? e.message : String(e)}). Use Stop to reset the worker, then Check again.`,
    }
  }
}

/** The empty solve response returned on any infrastructure failure. */
const SOLVE_FAILURE: Omit<SolveResponse, 'error'> = {
  success: false,
  variables: [],
  blocks: [],
  residuals: [],
  stats: null,
  solutions: [],
  unitWarnings: [],
}

/** Maps a solve result DTO from the engine worker to the typed SolveResponse. */
function mapSolveData(data: any): SolveResponse {
  return {
    success: data.success ?? false,
    variables: data.variables ?? [],
    blocks: data.blocks ?? [],
    residuals: data.residuals ?? [],
    stats: data.stats ?? null,
    solutions: data.solutions ?? [],
    unitWarnings: data.unitWarnings ?? [],
    error: data.error ?? null,
    cyclePath: data.cyclePath ?? [],
    codeTables: data.codeTables ?? [],
    parametricTables: data.parametricTables ?? [],
    definedPlots: data.definedPlots ?? [],
    stateTableDefs: data.stateTableDefs ?? [],
    odeTables: data.odeTables ?? [],
    components: data.components ?? [],
    errorLine: data.errorLine ?? null,
    failedBlockIndex: data.failedBlockIndex ?? null,
    uncertaintyBreakdown: data.uncertaintyBreakdown ?? [],
  }
}

export async function solve(
  text: string,
  stopCriteria: StopCriteria,
  variableInfo: VariableInfo[],
  findAllSolutions: boolean,
  displayUnitSystem: UnitSystem,
  fillMissing: boolean,
  functionTables: FunctionTableDto[] = [],
  // The session id tagged server-side result caching for the REPL; the
  // browser engine has no result cache yet, so it is accepted and unused.
  _sessionId?: string,
  overrides: string[] = [],
  // Called with the engine's overall completion (0…1) while the solve runs, so
  // the caller can paint a bar. Optional and last, so no existing call site
  // changes; the engine only reports from `solve`/`solveTable`.
  onProgress?: ProgressListener,
): Promise<SolveResponse> {
  // The full former POST body (minus `text`, which travels as the source
  // argument). The wasm boundary honours variableInfo + stopCriteria today
  // and ignores the rest until those features port, so the signature and the
  // request stay stable as the engine grows.
  const request = JSON.stringify({
    stopCriteria,
    variableInfo,
    findAllSolutions,
    displayUnitSystem,
    fillMissing,
    functionTables,
    // REPL overrides ("eta = 0.75") take priority over the editor's value.
    overrides,
  })

  try {
    // Success, syntax errors (errorLine) and solver failures (error +
    // failedBlockIndex) all arrive as one SolveResponse envelope — the wasm
    // boundary folds the Java 200/400/422 split into data.
    return mapSolveData(await wasmSolve(text, request, onProgress))
  } catch (e) {
    // Only infrastructure can land here (worker died, wasm failed to load).
    return {
      ...SOLVE_FAILURE,
      error: `Browser engine error: ${e instanceof Error ? e.message : String(e)}`,
    }
  }
}

/** Result of evaluating one REPL line against the cached solved workspace. */
export interface ReplResponse {
  success: boolean
  /** Numeric result (SI for compound expressions, display value for a bare variable). */
  value: number | null
  /** Print-ready rendering, e.g. "600" or "300 ± 0.5 [K]". */
  text: string | null
  units: string | null
  uncertainty: number | null
  error: string | null
  /** Set (display spelling) when the line defined a variable, so the UI can reflect it. */
  name: string | null
  assignedVariables?: VariableResult[]
}

/** Evaluates a single REPL expression against the workspace the last successful
 *  solve left in the engine worker. The browser has exactly one session, so
 *  `sessionId` is accepted and unused — the workspace lives in the wasm module,
 *  which is the browser's `SolveContextCache`.
 *
 *  Never rejects: the terminal's call site has no catch, so worker/infrastructure
 *  failures are folded into a failed ReplResponse like every document problem. */
export async function replEvaluate(
  _sessionId: string,
  expression: string,
  unitSystem: UnitSystem = 'SI',
): Promise<ReplResponse> {
  try {
    return await wasmReplEvaluate(expression, unitSystem)
  } catch (e) {
    return {
      success: false,
      value: null,
      text: null,
      units: null,
      uncertainty: null,
      error: `Browser engine error: ${e instanceof Error ? e.message : String(e)}`,
      name: null,
    }
  }
}

/** Clears all (or a specific) REPL-defined/overridden variables for the session.
 *  Fire-and-forget, same as the old POST (App.tsx calls it with `void`), so a
 *  worker failure is swallowed rather than surfaced as an unhandled rejection. */
export async function replClear(_sessionId: string, variableName?: string): Promise<void> {
  try {
    await wasmReplClear(variableName)
  } catch {
    /* the overlay is gone from the UI either way */
  }
}

export type OptimizeMethod = 'brent' | 'nelder-mead' | 'bobyqa'

export interface OptimizeParams {
  objective: string
  decisions: string[]
  lowers: number[]
  uppers: number[]
  method: OptimizeMethod
  maximize: boolean
  constraints?: string[]
}

export interface OptimizeResponse {
  success: boolean
  error: string | null
  warning: string | null
  objective: VariableResult | null
  decision: VariableResult | null
  decisions: VariableResult[]
  evaluations: number
  variables: VariableResult[]
}

/** The empty optimize response returned on any failure. */
const OPTIMIZE_FAILURE: Omit<OptimizeResponse, 'error'> = {
  success: false,
  warning: null,
  objective: null,
  decision: null,
  decisions: [],
  evaluations: 0,
  variables: [],
}

/** `POST /api/optimize` — served by the wasm `optimize` export (Wave B3).
 *  Never rejects: a refused request and an infrastructure failure both
 *  resolve to a failed OptimizeResponse whose `error` the Min/Max modal
 *  displays inline. */
export async function optimize(
  text: string,
  stopCriteria: StopCriteria,
  variableInfo: VariableInfo[],
  displayUnitSystem: UnitSystem,
  params: OptimizeParams,
  functionTables: FunctionTableDto[] = [],
): Promise<OptimizeResponse> {
  const request = JSON.stringify({ stopCriteria, variableInfo, displayUnitSystem, functionTables, ...params })
  try {
    return JSON.parse(await wasmOptimize(text, request)) as OptimizeResponse
  } catch (e) {
    return {
      ...OPTIMIZE_FAILURE,
      error: `Browser engine error: ${e instanceof Error ? e.message : String(e)}`,
    }
  }
}

export interface MultiObjectiveParams {
  objectives: string[]
  maximize: boolean[]
  decisions: string[]
  lowers: number[]
  uppers: number[]
  populationSize?: number
  generations?: number
  constraints?: string[]
}

export interface ParetoPoint {
  decisions: number[]
  objectives: number[]
}

export interface ParetoResponse {
  success: boolean
  error: string | null
  decisionNames: string[]
  objectiveNames: string[]
  front: ParetoPoint[]
  evaluations: number
}

/** The empty Pareto response returned on any failure. */
const PARETO_FAILURE: Omit<ParetoResponse, 'error'> = {
  success: false,
  decisionNames: [],
  objectiveNames: [],
  front: [],
  evaluations: 0,
}

/** `POST /api/optimize/multi` — served by the wasm `optimize_multi` export
 *  (Wave B3). Never rejects; raw SI numbers, exactly as the Java endpoint. */
export async function optimizeMulti(
  text: string,
  stopCriteria: StopCriteria,
  variableInfo: VariableInfo[],
  params: MultiObjectiveParams,
  functionTables: FunctionTableDto[] = [],
): Promise<ParetoResponse> {
  const request = JSON.stringify({ stopCriteria, variableInfo, functionTables, ...params })
  try {
    return JSON.parse(await wasmOptimizeMulti(text, request)) as ParetoResponse
  } catch (e) {
    return {
      ...PARETO_FAILURE,
      error: `Browser engine error: ${e instanceof Error ? e.message : String(e)}`,
    }
  }
}

export interface CurveFitParams {
  model: string
  yVariable: string
  xVariable: string
  parameters: string[]
  xData: number[]
  yData: number[]
  initialGuess?: number[]
}

export interface CurveFitResponse {
  success: boolean
  error: string | null
  fittedParameters: number[]
  parameterNames: string[]
  rSquared: number
  rmse: number
  iterations: number
  residuals: number[]
  fittedValues: number[]
}

const CURVE_FIT_FAILURE: Omit<CurveFitResponse, 'error'> = {
  success: false,
  fittedParameters: [],
  parameterNames: [],
  rSquared: 0,
  rmse: 0,
  iterations: 0,
  residuals: [],
  fittedValues: [],
}

/** `POST /api/curve-fit` — served by the wasm `curve_fit` export (Wave B3).
 *  Never rejects; the modal displays `error` inline. */
export async function curveFit(params: CurveFitParams): Promise<CurveFitResponse> {
  try {
    return JSON.parse(await wasmCurveFit(JSON.stringify(params))) as CurveFitResponse
  } catch (e) {
    return {
      ...CURVE_FIT_FAILURE,
      error: `Browser engine error: ${e instanceof Error ? e.message : String(e)}`,
    }
  }
}

// ---------------------------------------------------------------------------
// Parameter estimation — POST /api/measurements/parameter-fit
// ---------------------------------------------------------------------------

export interface ParameterFitParams {
  text: string
  stopCriteria: StopCriteria
  variableInfo: VariableInfo[]
  functionTables: FunctionTableDto[]
  parameters: string[]
  initial: number[]
  lower: number[]
  upper: number[]
  odeBlock: string
  column: string
  measuredT: number[]
  measuredV: number[]
  maxEvaluations?: number
}

export interface ParameterFitResult {
  success: boolean
  error: string | null
  parameterNames: string[]
  fittedValues: number[]
  rmse: number
  initialRmse: number
  evaluations: number
  truncated: boolean
  fittedT: number[]
  fittedV: number[]
}

const PARAMETER_FIT_FAILURE: ParameterFitResult = {
  success: false,
  error: null,
  parameterNames: [],
  fittedValues: [],
  rmse: 0,
  initialRmse: 0,
  evaluations: 0,
  truncated: false,
  fittedT: [],
  fittedV: [],
}

/** `POST /api/measurements/parameter-fit` — served by the wasm
 *  `parameter_fit` export (Wave B3), which drives the engine's DYNAMIC path
 *  per candidate. Never rejects; the modal shows `error` via setError. */
export async function parameterFit(params: ParameterFitParams): Promise<ParameterFitResult> {
  try {
    return JSON.parse(await wasmParameterFit(JSON.stringify(params))) as ParameterFitResult
  } catch (e) {
    return {
      ...PARAMETER_FIT_FAILURE,
      error: `Browser engine error: ${e instanceof Error ? e.message : String(e)}`,
    }
  }
}

export interface DiagramCurve {
  family: string
  label: string
  x: (number | null)[]
  y: (number | null)[]
}

export interface DiagramMarker {
  label: string
  x: number
  y: number
}

export interface DiagramResponse {
  fluid: string
  kind: string
  xProperty: string
  yProperty: string
  xLog: boolean
  yLog: boolean
  dome: DiagramCurve[]
  isolines: DiagramCurve[]
  markers: DiagramMarker[]
}

export interface PsychartResponse {
  pressure: number
  tMin: number
  tMax: number
  curves: DiagramCurve[]
}

/** `GET /api/plot/fluids`, read from the engine's own fluid table.
 *
 *  The names follow CoolProp's convention, as `PropertyFunctions.plotFluids()`
 *  publishes them — but, exactly like the Java controller, the list is **empty
 *  unless the engine has a real-fluid property backend installed** (in this
 *  build that is rustprop, so it never is), because offering a
 *  fluid whose properties cannot be evaluated is worse than offering none. The
 *  UI already renders the empty state ("no fluids available").
 *
 *  Never rejects: the three call sites (App.tsx, PlotTab, HelpPage) consume it
 *  at boot with bare `.then()`, so an engine-infrastructure failure resolves to
 *  the empty list rather than surfacing as an unhandled rejection. */
export async function getFluids(): Promise<string[]> {
  try {
    const response = await wasmFluids()
    return response.available ? (response.fluids ?? []) : []
  } catch {
    return []
  }
}

export interface UnitInfo {
  symbol: string
  dimension: string
  siFactor: number
}

export interface ConstantInfo {
  name: string
  value: number
  unit: string
  description: string
}

/** One row of the engine's intrinsic dispatch table (the Java
 *  `FunctionRegistry.FunctionInfo`). `signature` is generated from the arity,
 *  so the argument count is authoritative but the names are placeholders; the
 *  Rust registry carries no per-function prose, so `description` is empty and
 *  `category` is the single label `"Built-in"`. Prefer `functionCatalog.ts`
 *  for wording — use this list to answer "does the engine actually have it?". */
export interface FunctionInfo {
  name: string
  signature: string
  description: string
  category: string
}

export interface LanguageReference {
  units: UnitInfo[]
  constants: ConstantInfo[]
  functions: FunctionInfo[]
}

/** Every unit the checker accepts, the `#`-suffixed built-in constants, and the
 *  intrinsic dispatch table — read live from the wasm engine's own registries,
 *  so the Help page can never drift from what the solver accepts.
 *
 *  Never rejects: the Help page's two call sites use bare `.then()`, so a
 *  rejection would surface as an unhandled promise. An engine-infrastructure
 *  failure resolves to the empty reference instead — the old unreachable-
 *  backend fallback, which renders as empty tables plus the page's own
 *  "live list unavailable" notice. */
export async function getReference(): Promise<LanguageReference> {
  try {
    const reference = await wasmReference()
    return {
      units: reference.units ?? [],
      constants: reference.constants ?? [],
      functions: reference.functions ?? [],
    }
  } catch {
    return { units: [], constants: [], functions: [] }
  }
}

/** `POST /api/plot/propplot` — the saturation dome, quality lines and isolines
 *  for one fluid, generated in-engine by `props::diagrams`.
 *
 *  Rejects with the engine's own message when the diagram cannot be built —
 *  most often "no real-fluid property backend is installed", which names the
 *  fluid it could not reach. The PlotCard catch shows it in the card's error
 *  state, so the failure is visible and specific rather than a blank chart.
 *
 *  Points the backend declines arrive as `null` in `x`/`y`, which Plotly draws
 *  as a line break. A partial backend therefore produces a visibly incomplete
 *  curve, never an interpolated one. */
export async function getPropertyDiagram(
  fluid: string,
  type: string,
): Promise<DiagramResponse> {
  return wasmPropertyDiagram(fluid, type)
}

/** `POST /api/plot/psychart` — the psychrometric chart, generated in-engine by
 *  `props::psychro`.
 *
 *  Same gap contract as `getPropertyDiagram`: every point HAPropsSI cannot
 *  serve is `null`. Rejects only for a bad window (pressure <= 1 kPa,
 *  tMax <= tMin), which is the Java's own guard. */
export async function getPsychrometricChart(
  pressure: number,
  tMin: number,
  tMax: number,
): Promise<PsychartResponse> {
  return wasmPsychrometricChart(pressure, tMin, tMax)
}

export interface TableStats {
  runs: number
  solved: number
  failed: number
  equations: number
  unknowns: number
  iterations: number
  elapsedMillis: number
  maxResidual: number
}

export interface SolveTableResponse {
  results: TableRowResult[]
  stats: TableStats | null
  variables: VariableResult[]
}

/** `POST /api/solve/table` — the Tables workbook Solve, now served by the
 *  wasm `solve_table` export (Wave B). Never rejects: a cap breach, a syntax
 *  error, or an engine-infrastructure failure all resolve to a response whose
 *  every row carries the message — the same shape the App.tsx catch used to
 *  build, so the call site needs no change and always has rows to render. */
export async function solveTable(
  text: string,
  stopCriteria: StopCriteria,
  variableInfo: VariableInfo[],
  displayUnitSystem: UnitSystem,
  variables: string[],
  rows: Record<string, number>[],
  functionTables: FunctionTableDto[] = [],
  onProgress?: ProgressListener,
): Promise<SolveTableResponse> {
  const request = JSON.stringify({
    stopCriteria,
    variableInfo,
    displayUnitSystem,
    table: { variables, rows },
    functionTables,
  })
  const everyRowFailed = (error: string): SolveTableResponse => ({
    results: rows.map(() => ({ success: false, values: {}, error })),
    stats: null,
    variables: [],
  })
  try {
    const parsed = JSON.parse(await wasmSolveTable(text, request, onProgress)) as SolveTableResponse & {
      error?: string
    }
    if (parsed.error) {
      return everyRowFailed(parsed.error)
    }
    return parsed
  } catch (e) {
    // Only infrastructure can land here (worker died, wasm failed to load).
    return everyRowFailed(
      `Browser engine error: ${e instanceof Error ? e.message : String(e)}`,
    )
  }
}

// ---------------------------------------------------------------------------
// Monte Carlo uncertainty — POST /api/solve/montecarlo
// ---------------------------------------------------------------------------

/** Aggregate statistics for one variable across the Monte Carlo samples. */
export interface McVariableStat {
  variable: string
  mean: number
  sigma: number
  p5: number
  p50: number
  p95: number
  firstOrderSigma: number
}

export interface MonteCarloResult {
  stats: McVariableStat[]
  samples: { success: boolean; values: Record<string, number>; error: string | null }[]
  sources: string[]
  requestedSamples: number
  failedSamples: number
  truncated: boolean
}

/** `POST /api/solve/montecarlo` — served by the wasm `monte_carlo` export
 *  (Wave B2). Rejects on a refused request or an infrastructure failure —
 *  the MonteCarloModal's catch shows the message via setError, which is why
 *  this keeps the throwing contract (unlike solveTable, whose caller renders
 *  rows either way). */
export async function runMonteCarlo(
  text: string,
  stopCriteria: StopCriteria,
  variableInfo: VariableInfo[],
  displayUnitSystem: UnitSystem,
  functionTables: FunctionTableDto[],
  samples: number,
  seed: number,
): Promise<MonteCarloResult> {
  const request = JSON.stringify({
    stopCriteria,
    variableInfo,
    displayUnitSystem,
    functionTables,
    samples,
    seed,
  })
  const parsed = JSON.parse(await wasmMonteCarlo(text, request)) as MonteCarloResult & {
    error?: string
  }
  if (parsed.error) {
    throw new Error(parsed.error)
  }
  return parsed
}

// ---------------------------------------------------------------------------
// PID Tuner (control design) — POST /api/control/pidtune
// ---------------------------------------------------------------------------

export interface PidTuneRequest {
  /** Plant transfer function, descending powers. */
  num: number[]
  den: number[]
  type: 'p' | 'pi' | 'pid'
  /** Target open-loop gain crossover (rad/s) — the "response time" knob. */
  wc?: number
  /** Target phase margin (deg) — the "transient behaviour"/robustness knob. */
  pm?: number
  horizon?: number
  points?: number
}

export interface PidTuneResponse {
  kp: number
  ki: number
  kd: number
  wc: number
  pm: number
  t: number[]
  y: number[]
  riseTime: number
  peakTime: number
  settlingTime: number
  overshoot: number
  gainMargin: number
  phaseMargin: number
}

/** PID tuning (loop shaping + step metrics) — served by wasm `pid_tune`. */
export async function pidTune(request: PidTuneRequest): Promise<PidTuneResponse> {
  // Served by the wasm `pid_tune` export (Wave B4). Rejects on a refused
  // request or infrastructure failure — the PidTunerModal's catch handles it.
  const parsed = JSON.parse(await wasmPidTune(JSON.stringify(request))) as PidTuneResponse & {
    error?: string
  }
  if (parsed.error) {
    throw new Error(parsed.error)
  }
  return parsed
}

// ---------------------------------------------------------------------------
// Plant extraction for the SigPID "Tune…" path — POST /api/control/plant
// ---------------------------------------------------------------------------

export interface PlantRequest {
  text: string
  dynamic: string
  /** Reference SigConstant instance whose constant is perturbed (e.g. "SP"). */
  reference: string
  /** Measured plant-output variable (e.g. "bp.t"). */
  output: string
  /** True when the reference drives the PID's sp input (the common wiring). */
  referenceOnSp: boolean
  type: 'p' | 'pi' | 'pid'
  kp: number
  ki: number
  kd: number
}

/** Plant linearization for SigPID — served by wasm `extract_plant`. */
export async function extractPlant(
  request: PlantRequest,
): Promise<{ num: number[]; den: number[] }> {
  // Served by the wasm `extract_plant` export (Wave B4). Rejects on failure —
  // App.tsx's catch falls back to manual plant entry.
  const parsed = JSON.parse(await wasmExtractPlant(JSON.stringify(request))) as {
    num: number[]
    den: number[]
    error?: string
  }
  if (parsed.error) {
    throw new Error(parsed.error)
  }
  return parsed
}
