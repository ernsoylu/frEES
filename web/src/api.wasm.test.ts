// DTO-parity assertions for the wasm-engine seam: what the Rust boundary
// actually emits (fixtures below are captured verbatim from
// crates/frees — see tests/dto_parity.rs on that side) versus what
// solve()/check() promise their call sites. The two contracts under test:
//
//  1. every SolveResponse/CheckResponse field App.tsx dereferences is either
//     supplied by the wasm payload with the right type and casing, or
//     defaulted by the shim (mapSolveData / check()'s literal), and
//  2. an engine-infrastructure failure RESOLVES to a failure envelope —
//     solve()/check() call sites have no catch, so a rejection would take
//     down the handler.
import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('./wasm/engineClient', () => ({
  wasmSolve: vi.fn(),
  wasmCheck: vi.fn(),
  wasmSolveTable: vi.fn(),
  wasmMonteCarlo: vi.fn(),
  wasmOptimize: vi.fn(),
  wasmOptimizeMulti: vi.fn(),
  wasmCurveFit: vi.fn(),
  wasmParameterFit: vi.fn(),
  wasmPidTune: vi.fn(),
  wasmExtractPlant: vi.fn(),
  // api.ts imports these too (the REPL seam); unused here but a mocked module
  // must declare every export its importer names.
  wasmReplEvaluate: vi.fn(),
  wasmReplClear: vi.fn(),
  wasmStop: vi.fn(),
}))

import { check, curveFit, optimize, runMonteCarlo, solve, solveTable, DEFAULT_STOP_CRITERIA } from './api'
import { mergeCodeTables } from './tables'
import { wasmCheck, wasmCurveFit, wasmMonteCarlo, wasmOptimize, wasmSolve, wasmSolveTable } from './wasm/engineClient'

const solveMock = vi.mocked(wasmSolve)
const checkMock = vi.mocked(wasmCheck)

// ── Fixtures: verbatim boundary output (crates/frees) ────────────────

const SOLVE_OK =
  '{"blocks":[{"equations":["P = 140 [kPa]"],"index":0,"variables":["P"]},{"equations":["Q = P * 2"],"index":1,"variables":["Q"]}],"error":null,"errorLine":null,"failedBlockIndex":null,"residuals":[{"equation":"P = 140 [kPa]","value":0.0},{"equation":"Q = P * 2","value":0.0}],"solutions":[{"maxResidual":0.0,"variables":[{"name":"P","units":"Pa","value":140000.0},{"name":"Q","units":"Pa","value":280000.0}]}],"stats":{"blocks":2,"elapsedMillis":1,"equations":2,"iterations":4,"maxResidual":0.0,"unknowns":2},"success":true,"unitWarnings":[],"variables":[{"name":"P","units":"Pa","value":140000.0},{"name":"Q","units":"Pa","value":280000.0}]}'

// A document with a DYNAMIC block: the trajectory rides on `odeTables`, and
// `variables` holds only the analytic parameters. Regenerate with
//   cargo test -p frees --release --test dto_parity \
//     print_the_ode_table_payload -- --ignored --nocapture
// (elapsedMillis pinned to 1 — it is wall-clock).
const SOLVE_WITH_ODE_TABLE =
  '{"blocks":[{"equations":["k = 0.05"],"index":0,"variables":["k"]},{"equations":["Tinf = 20"],"index":1,"variables":["Tinf"]}],"components":[],"cyclePath":[],"error":null,"errorLine":null,"failedBlockIndex":null,"odeTables":[{"endTime":60.0,"events":[],"method":"ode45","name":"cooling","rows":[[0.0,95.0],[20.0,47.59095803046333],[40.0,30.15014623853744],[60.0,23.734030127668667]],"stopped":false,"units":["",""],"vars":["time","temp"]}],"residuals":[{"equation":"k = 0.05","value":0.0},{"equation":"Tinf = 20","value":0.0}],"solutions":[{"maxResidual":0.0,"variables":[{"name":"k","units":"-","value":0.05},{"name":"Tinf","units":"-","value":20.0}]}],"stats":{"blocks":2,"elapsedMillis":1,"equations":2,"iterations":3,"maxResidual":0.0,"unknowns":2},"success":true,"unitWarnings":[],"variables":[{"name":"k","units":"-","value":0.05},{"name":"Tinf","units":"-","value":20.0}]}'

const SOLVE_PARSE_ERROR =
  '{"blocks":[],"error":"Syntax error: expected an expression, found `=`","errorLine":2,"failedBlockIndex":null,"residuals":[],"solutions":[],"stats":null,"success":false,"unitWarnings":[],"variables":[]}'

const SOLVE_FAILED_BLOCK =
  '{"blocks":[],"error":"Block 2 (1 equation) failed: Newton iteration stalled.","errorLine":null,"failedBlockIndex":1,"residuals":[],"solutions":[],"stats":null,"success":false,"unitWarnings":[],"variables":[]}'

const CHECK_OK =
  '{"equations":2,"errorLine":null,"errors":[],"inferredUnits":{"T_out":"K","Tin":"K"},"message":"No syntax errors were detected. There are 2 equations and 2 variables.","solvable":true,"unitWarnings":[],"unknowns":2,"variables":["T_out","Tin"]}'

const CHECK_PARSE_ERROR =
  '{"equations":0,"errorLine":3,"errors":[{"column":5,"line":3,"message":"expected an expression, found `=`"}],"inferredUnits":{},"message":"Syntax error: expected an expression, found `=`","solvable":false,"unitWarnings":[],"unknowns":0,"variables":[]}'

/** Resolves like the real engineClient: JSON.parse of the worker's string. */
function engineReturns(
  mock: { mockResolvedValueOnce(value: never): unknown },
  payload: string,
) {
  mock.mockResolvedValueOnce(JSON.parse(payload) as never)
}

function runSolve(text: string) {
  return solve(text, DEFAULT_STOP_CRITERIA, [], false, 'SI', false)
}

beforeEach(() => {
  solveMock.mockReset()
  checkMock.mockReset()
})

describe('solve() over the wasm boundary payloads', () => {
  it('maps a success payload and defaults every not-yet-ported field', async () => {
    engineReturns(solveMock, SOLVE_OK)
    const r = await runSolve('P = 140 [kPa]\nQ = P * 2\n')

    expect(r.success).toBe(true)
    expect(r.error).toBeNull()

    // Supplied by the payload, exact casing and types.
    expect(r.variables).toEqual([
      { name: 'P', units: 'Pa', value: 140000 },
      { name: 'Q', units: 'Pa', value: 280000 },
    ])
    expect(r.blocks[0]).toEqual({ index: 0, equations: ['P = 140 [kPa]'], variables: ['P'] })
    expect(r.residuals[1]).toEqual({ equation: 'Q = P * 2', value: 0 })
    expect(r.stats).toEqual({
      equations: 2, unknowns: 2, blocks: 2, iterations: 4, elapsedMillis: 1, maxResidual: 0,
    })
    expect(r.solutions).toHaveLength(1)
    expect(r.solutions[0].maxResidual).toBe(0)
    // App.tsx prefers solutions[0].variables over variables — they must agree.
    expect(r.solutions[0].variables).toEqual(r.variables)

    // Defaulted by mapSolveData — App.tsx iterates these without guards.
    expect(r.cyclePath).toEqual([])
    expect(r.codeTables).toEqual([])
    expect(r.parametricTables).toEqual([])
    expect(r.definedPlots).toEqual([])
    expect(r.stateTableDefs).toEqual([])
    expect(r.odeTables).toEqual([])
    expect(r.components).toEqual([])
    expect(r.uncertaintyBreakdown).toEqual([])
    expect(r.errorLine).toBeNull()
    expect(r.failedBlockIndex).toBeNull()
  })

  // A solved DYNAMIC block arrives as `odeTables` and NOWHERE ELSE: `variables`
  // carries only the analytic parameters, so a Tables/Plots window fed from
  // `variables` would render an empty transient. Payload captured verbatim from
  // the Rust boundary (crates/frees/tests/dto_parity.rs,
  // `a_solved_dynamic_block_reaches_the_frontend_as_an_ode_table`), whose row
  // values are the Java oracle's from fixtures/golden/dyn_plain_ode.json.
  it('carries a solved DYNAMIC block through as an OdeTableDto', async () => {
    engineReturns(solveMock, SOLVE_WITH_ODE_TABLE)
    const r = await runSolve(
      'k = 0.05\nTinf = 20\nDYNAMIC cooling (method = ode45, time = 0 .. 60, points = 4)\n' +
        '  der(Temp) = -k*(Temp - Tinf)\n  Temp(0) = 95\nEND\n',
    )

    expect(r.success).toBe(true)
    // The states are NOT result variables.
    expect(r.variables.map((v) => v.name)).toEqual(['k', 'Tinf'])
    expect(r.odeTables).toHaveLength(1)

    const table = r.odeTables![0]
    expect(table.name).toBe('cooling')
    expect(table.method).toBe('ode45')
    expect(table.stopped).toBe(false)
    expect(table.endTime).toBe(60)
    expect(table.vars).toEqual(['time', 'temp'])
    // The engine infers no unit for either column here (the ODE table's columns
    // are keyed by flat name, and neither is in `inferredUnits`); the DTO still
    // carries a slot per column so the grid can label them when it can.
    expect(table.units).toEqual(['', ''])
    expect(table.events).toEqual([])
    expect(table.rows).toEqual([
      [0, 95],
      [20, 47.59095803046333],
      [40, 30.15014623853744],
      [60, 23.734030127668667],
    ])

    // …and the Tables window renders it: mergeCodeTables turns the DTO into a
    // parametric-shaped TableSpec tagged `origin: 'ode'`, which is the exact
    // call App.tsx makes on a solve.
    const merged = mergeCodeTables([], r.codeTables, r.parametricTables, r.odeTables)
    expect(merged).toHaveLength(1)
    expect(merged[0].name).toBe('cooling')
    expect(merged[0].source).toBe('code')
    expect(merged[0].kind).toBe('parametric')
    const spec = merged[0] as { vars: string[]; rows: { values: Record<string, string> }[] }
    expect(spec.vars).toEqual(['time', 'temp'])
    expect(spec.rows).toHaveLength(4)
    expect(spec.rows[0].values.temp).toBe('95')
  })

  it('surfaces a parse failure with its 1-based errorLine', async () => {
    engineReturns(solveMock, SOLVE_PARSE_ERROR)
    const r = await runSolve('a = 1\nb = = 2\n')
    expect(r.success).toBe(false)
    expect(r.error).toMatch(/^Syntax error: /)
    expect(r.errorLine).toBe(2)
    expect(r.stats).toBeNull()
    expect(r.variables).toEqual([])
  })

  it('surfaces a solver failure with its 0-based failedBlockIndex', async () => {
    engineReturns(solveMock, SOLVE_FAILED_BLOCK)
    const r = await runSolve('k = 1\nexp(x) = -k\n')
    expect(r.success).toBe(false)
    expect(r.failedBlockIndex).toBe(1)
    expect(r.errorLine).toBeNull()
  })

  it('resolves (never rejects) when the engine infrastructure dies', async () => {
    solveMock.mockRejectedValueOnce(new Error('The engine worker failed'))
    const r = await runSolve('x = 1\n')
    expect(r.success).toBe(false)
    expect(r.error).toBe('Browser engine error: The engine worker failed')
    expect(r.variables).toEqual([])
    expect(r.stats).toBeNull()
  })

  it('sends the full former POST body so the boundary can grow into it', async () => {
    engineReturns(solveMock, SOLVE_OK)
    await runSolve('x = 1\n')
    const [source, requestJson] = solveMock.mock.calls[0]
    expect(source).toBe('x = 1\n')
    const request = JSON.parse(requestJson)
    expect(request.stopCriteria.maxIterations).toBe(250)
    expect(request.variableInfo).toEqual([])
    expect(request.displayUnitSystem).toBe('SI')
    expect(request.overrides).toEqual([])
  })
})

describe('check() over the wasm boundary payloads', () => {
  it('maps a success payload; inferredUnits are keyed by the display spellings in variables', async () => {
    engineReturns(checkMock, CHECK_OK)
    const r = await check('Tin = 300 [K]\nT_out = Tin * 2\n', [], false)

    expect(r.solvable).toBe(true)
    expect(r.equations).toBe(2)
    expect(r.unknowns).toBe(2)
    // Display spellings, not lowercase canonical names …
    expect(r.variables).toEqual(['T_out', 'Tin'])
    // … and the unit lookup App.tsx does with those names must connect.
    for (const name of r.variables) {
      expect(r.inferredUnits[name]).toBe('K')
    }
    expect(r.errorLine).toBeNull()
    expect(r.errors).toEqual([])
    // Defaulted by the shim — App.tsx merges these without guards.
    expect(r.codeTables).toEqual([])
    expect(r.parametricTables).toEqual([])
    expect(r.definedPlots).toEqual([])
    expect(r.stateTableDefs).toEqual([])
    expect(r.connections).toEqual([])
  })

  it('surfaces syntax errors as data with 1-based line/column for the editor marks', async () => {
    engineReturns(checkMock, CHECK_PARSE_ERROR)
    const r = await check('a = 1\nb = 2\nc = = 3\n', [], false)
    expect(r.solvable).toBe(false)
    expect(r.message).toMatch(/^Syntax error: /)
    expect(r.errorLine).toBe(3)
    // EquationEditor filters `line >= 1` and squiggles from `column - 1`.
    expect(r.errors).toEqual([{ line: 3, column: 5, message: 'expected an expression, found `=`' }])
  })

  it('resolves (never rejects) when the engine infrastructure dies', async () => {
    checkMock.mockRejectedValueOnce(new Error('wasm failed to load'))
    const r = await check('x = 1\n', [], false)
    expect(r.solvable).toBe(false)
    expect(r.message).toBe(
      'The in-browser solver failed (wasm failed to load). Use Stop to reset the worker, then Check again.',
    )
    expect(r.variables).toEqual([])
    expect(r.inferredUnits).toEqual({})
  })

  it('forwards variableInfo and complexMode in the request body', async () => {
    engineReturns(checkMock, CHECK_OK)
    const row = { name: 'x', guess: 2, lower: null, upper: null, units: 'm', uncertainty: null }
    await check('x = 1\n', [row], true)
    const [, requestJson] = checkMock.mock.calls[0]
    const request = JSON.parse(requestJson)
    expect(request.variableInfo).toEqual([row])
    expect(request.stopCriteria.complexMode).toBe(true)
  })
})

// ── solveTable (Wave B: the Tables workbook GUI Solve) ────────────────────

// Verbatim boundary output. Regenerate with a one-off example calling
// frees::solve_table("y = 2 * x\n",
//   '{"table": {"variables": ["x", "y"], "rows": [{"x": 1}, {"x": 2}]}}').
const SOLVE_TABLE_OK =
  '{"results":[{"error":null,"success":true,"values":{"x":1.0,"y":2.0}},{"error":null,"success":true,"values":{"x":2.0,"y":4.0}}],"stats":{"elapsedMillis":1,"equations":2,"failed":0,"iterations":3,"maxResidual":0.0,"runs":2,"solved":2,"unknowns":2},"variables":[{"name":"x","uncertainty":0.0,"units":"-","value":2.0},{"name":"y","uncertainty":0.0,"units":"-","value":4.0}]}'

const SOLVE_TABLE_CAP =
  '{"results":[],"stats":null,"variables":[],"error":"The parametric table has too many rows (5001; limit 5000). Reduce the run count."}'

describe('solveTable (wasm engine)', () => {
  const tableMock = vi.mocked(wasmSolveTable)

  it('round-trips the SolveTableResponse the App writes onto the table spec', async () => {
    tableMock.mockResolvedValueOnce(SOLVE_TABLE_OK)
    const r = await solveTable(
      'y = 2 * x\n',
      DEFAULT_STOP_CRITERIA,
      [],
      'SI',
      ['x', 'y'],
      [{ x: 1 }, { x: 2 }],
    )
    expect(r.results).toHaveLength(2)
    expect(r.results[0]).toEqual({ error: null, success: true, values: { x: 1, y: 2 } })
    expect(r.stats?.runs).toBe(2)
    expect(r.stats?.solved).toBe(2)
    expect(r.variables.map((v) => v.name)).toEqual(['x', 'y'])
    // The request carries the table under the Java field names.
    const [source, requestJson] = tableMock.mock.calls[0]
    expect(source).toBe('y = 2 * x\n')
    const request = JSON.parse(requestJson)
    expect(request.table.variables).toEqual(['x', 'y'])
    expect(request.table.rows).toEqual([{ x: 1 }, { x: 2 }])
  })

  it('maps a top-level boundary error onto every row', async () => {
    tableMock.mockResolvedValueOnce(SOLVE_TABLE_CAP)
    const r = await solveTable('y = 2 * x\n', DEFAULT_STOP_CRITERIA, [], 'SI', ['x'], [
      { x: 1 },
      { x: 2 },
    ])
    expect(r.results).toHaveLength(2)
    for (const row of r.results) {
      expect(row.success).toBe(false)
      expect(row.error).toMatch(/too many rows/)
    }
    expect(r.stats).toBeNull()
  })

  it('resolves (never rejects) when the engine infrastructure dies', async () => {
    tableMock.mockRejectedValueOnce(new Error('worker terminated'))
    const r = await solveTable('y = 2 * x\n', DEFAULT_STOP_CRITERIA, [], 'SI', ['x'], [{ x: 1 }])
    expect(r.results).toHaveLength(1)
    expect(r.results[0].success).toBe(false)
    expect(r.results[0].error).toBe('Browser engine error: worker terminated')
  })
})

// ── runMonteCarlo (Wave B2) ────────────────────────────────────────────────

// Verbatim boundary output shape (abridged sample list). Regenerate via
// frees::monte_carlo("x = 2\ny = 3 * x\n",
//   '{"samples": 2, "seed": 42, "variableInfo": [{"name": "x", "guess": 2, "uncertainty": 0.1}]}').
const MONTE_CARLO_OK =
  '{"stats":[{"variable":"y","mean":6.31,"sigma":0.29,"p5":6.1,"p50":6.3,"p95":6.5,"firstOrderSigma":0.3},{"variable":"x","mean":2.1,"sigma":0.097,"p5":2.03,"p50":2.1,"p95":2.17,"firstOrderSigma":0.1}],"samples":[{"error":null,"success":true,"values":{"x":2.11,"y":6.34}},{"error":null,"success":true,"values":{"x":2.09,"y":6.27}}],"sources":["x"],"requestedSamples":2,"failedSamples":0,"truncated":false}'

const MONTE_CARLO_CAP =
  '{"stats":[],"samples":[],"sources":[],"requestedSamples":0,"failedSamples":0,"truncated":false,"error":"Monte Carlo sample count must be between 2 and 1000 (got 1001). Adjust the sample count."}'

describe('runMonteCarlo (wasm engine)', () => {
  const mcMock = vi.mocked(wasmMonteCarlo)

  it('round-trips the MonteCarloResult the modal renders', async () => {
    mcMock.mockResolvedValueOnce(MONTE_CARLO_OK)
    const r = await runMonteCarlo(
      'x = 2\ny = 3 * x\n',
      DEFAULT_STOP_CRITERIA,
      [{ name: 'x', guess: 2, lower: null, upper: null, units: null, uncertainty: 0.1 }],
      'SI',
      [],
      2,
      42,
    )
    expect(r.stats.map((s) => s.variable)).toEqual(['y', 'x'])
    expect(r.sources).toEqual(['x'])
    expect(r.requestedSamples).toBe(2)
    expect(r.truncated).toBe(false)
    const [source, requestJson] = mcMock.mock.calls[0]
    expect(source).toBe('x = 2\ny = 3 * x\n')
    const request = JSON.parse(requestJson)
    expect(request.samples).toBe(2)
    expect(request.seed).toBe(42)
  })

  it('rejects with the boundary error so the modal catch shows it', async () => {
    mcMock.mockResolvedValueOnce(MONTE_CARLO_CAP)
    await expect(
      runMonteCarlo('x = 2\n', DEFAULT_STOP_CRITERIA, [], 'SI', [], 1001, 42),
    ).rejects.toThrow(/between 2 and 1000/)
  })
})

// ── Wave B3 contracts ──────────────────────────────────────────────────────

describe('optimize / curveFit (wasm engine)', () => {
  it('optimize round-trips and spreads params into the request body', async () => {
    vi.mocked(wasmOptimize).mockResolvedValueOnce(
      '{"success":true,"error":null,"warning":null,"objective":{"name":"f","value":1.0,"units":"-","uncertainty":0.0},"decision":{"name":"x","value":3.0,"units":"-","uncertainty":0.0},"decisions":[{"name":"x","value":3.0,"units":"-","uncertainty":0.0}],"evaluations":19,"variables":[]}',
    )
    const r = await optimize('f = (x-3)^2 + 1\n', DEFAULT_STOP_CRITERIA, [], 'SI', {
      objective: 'f',
      decisions: ['x'],
      lowers: [0],
      uppers: [10],
      method: 'brent',
      maximize: false,
    })
    expect(r.success).toBe(true)
    expect(r.decision?.value).toBe(3)
    const [, requestJson] = vi.mocked(wasmOptimize).mock.calls[0]
    const request = JSON.parse(requestJson)
    expect(request.objective).toBe('f')
    expect(request.decisions).toEqual(['x'])
  })

  it('curveFit resolves the boundary error inline, never rejects', async () => {
    vi.mocked(wasmCurveFit).mockResolvedValueOnce(
      '{"success":false,"error":"Model equation is required.","fittedParameters":[],"parameterNames":[],"rSquared":0,"rmse":0,"iterations":0,"residuals":[],"fittedValues":[]}',
    )
    const r = await curveFit({
      model: '', yVariable: 'y', xVariable: 'x', parameters: [], xData: [], yData: [],
    })
    expect(r.success).toBe(false)
    expect(r.error).toBe('Model equation is required.')
  })
})
