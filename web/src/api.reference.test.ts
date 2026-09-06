// getReference() over the wasm boundary: the Help page's Units & Constants
// tables and the built-in constants table read this, both with a bare
// `.then()` and no `.catch()`. Two contracts under test:
//
//  1. the LanguageReference arrays arrive with the field names and types
//     HelpPage dereferences (UnitInfo.symbol/dimension/siFactor,
//     ConstantInfo.name/value/unit/description), and
//  2. an engine-infrastructure failure RESOLVES to the empty reference — a
//     rejection would land as an unhandled promise in those effects.
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
  wasmReference: vi.fn(),
}))

import { getReference } from './api'
import { wasmReference } from './wasm/engineClient'

const referenceMock = vi.mocked(wasmReference)

// Verbatim boundary output (crates/frees `reference()`), trimmed to one
// row per array.
const REFERENCE =
  '{"constants":[{"description":"Universal (molar) gas constant","name":"R#","unit":"J/mol-K","value":8.314462618},' +
  '{"description":"Ratio of a circle\'s circumference to its diameter","name":"pi#","unit":"-","value":3.141592653589793}],' +
  '"functions":[{"category":"Built-in","description":"","name":"atan2","signature":"atan2(a, b)"},' +
  '{"category":"Built-in","description":"","name":"round","signature":"round(a[, b])"}],' +
  '"units":[{"dimension":"Pa","siFactor":1000.0,"symbol":"kpa"},{"dimension":"m","siFactor":1.0,"symbol":"m"}]}'

beforeEach(() => {
  referenceMock.mockReset()
})

describe('getReference() over the wasm boundary payload', () => {
  it('maps every array with the field names HelpPage dereferences', async () => {
    referenceMock.mockResolvedValueOnce(JSON.parse(REFERENCE) as never)
    const ref = await getReference()

    // UnitInfo: symbol/dimension are strings, siFactor a number. HelpPage
    // groups on `dimension` and prints `symbol` + formatSiFactor(siFactor).
    expect(ref.units).toEqual([
      { symbol: 'kpa', dimension: 'Pa', siFactor: 1000 },
      { symbol: 'm', dimension: 'm', siFactor: 1 },
    ])

    // ConstantInfo: a dimensionless constant carries "-", never null — the
    // Help table prints the cell verbatim.
    expect(ref.constants[0]).toEqual({
      name: 'R#',
      value: 8.314462618,
      unit: 'J/mol-K',
      description: 'Universal (molar) gas constant',
    })
    expect(ref.constants[1].unit).toBe('-')
    expect(ref.constants.every((c) => c.name.endsWith('#'))).toBe(true)

    // FunctionInfo: the engine's authoritative "does this exist?" list.
    expect(ref.functions[0]).toEqual({
      name: 'atan2',
      signature: 'atan2(a, b)',
      description: '',
      category: 'Built-in',
    })
  })

  it('calls the engine with no arguments', async () => {
    referenceMock.mockResolvedValueOnce(JSON.parse(REFERENCE) as never)
    await getReference()
    expect(referenceMock).toHaveBeenCalledTimes(1)
    expect(referenceMock.mock.calls[0]).toEqual([])
  })

  it('resolves (never rejects) when the engine infrastructure dies', async () => {
    referenceMock.mockRejectedValueOnce(new Error('The engine worker failed'))
    const ref = await getReference()
    expect(ref).toEqual({ units: [], constants: [], functions: [] })
  })

  it('defaults a missing array rather than handing undefined to .map()', async () => {
    referenceMock.mockResolvedValueOnce({ units: [{ symbol: 'm', dimension: 'm', siFactor: 1 }] } as never)
    const ref = await getReference()
    expect(ref.units).toHaveLength(1)
    expect(ref.constants).toEqual([])
    expect(ref.functions).toEqual([])
  })
})
