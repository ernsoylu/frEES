// The REPL seam: `replEvaluate` / `replClear` in api.ts against the wasm
// engine boundary (crates/frees/src/repl.rs). Two contracts:
//
//  1. the ReplResponse fields ReplTerminal.tsx dereferences (`success`,
//     `text`, `value`, `error`, `name`, `assignedVariables`) survive the shim
//     with the right casing, and
//  2. NOTHING here may reject — ReplTerminal awaits `replEvaluate` with no
//     catch, and App.tsx calls `replClear` with `void`, so a rejection would
//     be an unhandled one.
import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('./wasm/engineClient', () => ({
  wasmSolve: vi.fn(),
  wasmSolveTable: vi.fn(),
  wasmMonteCarlo: vi.fn(),
  wasmOptimize: vi.fn(),
  wasmOptimizeMulti: vi.fn(),
  wasmCurveFit: vi.fn(),
  wasmParameterFit: vi.fn(),
  wasmPidTune: vi.fn(),
  wasmExtractPlant: vi.fn(),
  wasmCheck: vi.fn(),
  wasmReplEvaluate: vi.fn(),
  wasmReplClear: vi.fn(),
}))

import { replClear, replEvaluate } from './api'
import { wasmReplClear, wasmReplEvaluate } from './wasm/engineClient'

const evaluateMock = vi.mocked(wasmReplEvaluate)
const clearMock = vi.mocked(wasmReplClear)

// Captured verbatim from the Rust boundary — `repl_evaluate` on a workspace
// seeded with `T_out = 300 [K]`.
const ECHO =
  '{"assignedVariables":[],"error":null,"name":"ans","success":true,"text":"300 [K]","uncertainty":null,"units":"K","value":300.0}'

const ASSIGN =
  '{"assignedVariables":[{"name":"y","units":"","uncertainty":null,"value":6.0}],"error":null,"name":"y","success":true,"text":"y = 6","uncertainty":null,"units":"","value":6.0}'

const CAS =
  '{"assignedVariables":[],"error":null,"name":null,"success":true,"text":"(-1+x)*(1+x)","uncertainty":null,"units":"","value":0.0}'

const NO_CONTEXT =
  '{"assignedVariables":[],"error":"No solved context for this session yet — solve the document first.","name":null,"success":false,"text":null,"uncertainty":null,"units":null,"value":null}'

function engineReturns(payload: string) {
  evaluateMock.mockResolvedValueOnce(JSON.parse(payload) as never)
}

describe('replEvaluate', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('echoes a workspace variable with its display unit', async () => {
    engineReturns(ECHO)
    const r = await replEvaluate('session-1', 'T_out', 'SI')
    expect(r.success).toBe(true)
    expect(r.text).toBe('300 [K]')
    expect(r.value).toBe(300)
    expect(r.units).toBe('K')
    // ReplTerminal prints `text` and reflects `name` into the workspace.
    expect(r.name).toBe('ans')
  })

  it('reports an assignment so the workspace can reflect it', async () => {
    engineReturns(ASSIGN)
    const r = await replEvaluate('session-1', 'y = 3 * x')
    expect(r.text).toBe('y = 6')
    expect(r.name).toBe('y')
    expect(r.assignedVariables).toEqual([
      { name: 'y', value: 6, units: '', uncertainty: null },
    ])
  })

  it('returns a CAS transformation as text', async () => {
    engineReturns(CAS)
    const r = await replEvaluate('session-1', 'Factor(x^2 - 1)')
    expect(r.success).toBe(true)
    expect(r.text).toBe('(-1+x)*(1+x)')
  })

  it('passes the expression and unit system through unchanged', async () => {
    engineReturns(ECHO)
    await replEvaluate('session-1', 'T_out', 'ENGLISH')
    expect(evaluateMock).toHaveBeenCalledWith('T_out', 'ENGLISH')
  })

  it('defaults the unit system to SI when the caller omits it', async () => {
    engineReturns(ECHO)
    await replEvaluate('session-1', 'T_out')
    expect(evaluateMock).toHaveBeenCalledWith('T_out', 'SI')
  })

  it('surfaces a missing workspace as data, not a rejection', async () => {
    engineReturns(NO_CONTEXT)
    const r = await replEvaluate('session-1', '2 + 2')
    expect(r.success).toBe(false)
    expect(r.error).toMatch(/solve the document first/)
  })

  it('resolves (never rejects) when the engine infrastructure dies', async () => {
    evaluateMock.mockRejectedValueOnce(new Error('wasm failed to load'))
    const r = await replEvaluate('session-1', '2 + 2')
    expect(r.success).toBe(false)
    expect(r.error).toBe('Browser engine error: wasm failed to load')
    expect(r.value).toBeNull()
    expect(r.text).toBeNull()
  })
})

describe('replClear', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('clears every overlay when no name is given', async () => {
    clearMock.mockResolvedValueOnce(undefined as never)
    await replClear('session-1')
    expect(clearMock).toHaveBeenCalledWith(undefined)
  })

  it('clears one overlay by name', async () => {
    clearMock.mockResolvedValueOnce(undefined as never)
    await replClear('session-1', 'eta')
    expect(clearMock).toHaveBeenCalledWith('eta')
  })

  it('swallows an engine failure — App.tsx calls it with `void`', async () => {
    clearMock.mockRejectedValueOnce(new Error('worker died'))
    await expect(replClear('session-1')).resolves.toBeUndefined()
  })
})
