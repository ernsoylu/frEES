// tablesGrid/composeTables.test.ts
//
// Edge cases of the Wave-H composition conversions: failed-row skipping,
// family-parameter mapping, decimation, name validation, replace-vs-new.

import { describe, expect, it } from 'vitest'
import type { FunctionTableSpec, ParamTableSpec, TableSpec } from '../tables'
import {
  applyFunctionSpecs,
  checkFunctionName,
  decimationIndices,
  functionSpecFromParamColumns,
  functionSpecFromXY,
  paramCellEntry,
} from './composeTables'
import { TABLE_MAX_ROWS } from './tableGridModel'

let rowId = 0

function paramSpec(over: Partial<ParamTableSpec> = {}): ParamTableSpec {
  return {
    id: 'p1',
    kind: 'parametric',
    name: 'Parametric 1',
    vars: ['T', 'eta', 'P'],
    rows: [],
    results: [],
    stats: null,
    checkResult: null,
    checkMessage: '',
    ...over,
  }
}

function row(values: Record<string, string>): ParamTableSpec['rows'][number] {
  return { id: `r${rowId++}`, values }
}

function fnSpec(over: Partial<FunctionTableSpec> = {}): FunctionTableSpec {
  return {
    id: 'f1',
    kind: 'function',
    name: 'eff',
    argName: 'x',
    paramName: '',
    xLog: false,
    yLog: false,
    columns: [''],
    rows: [{ x: '1', ys: ['2'] }],
    is1D: true,
    ...over,
  }
}

// ---------------------------------------------------------------------------
// paramCellEntry (paramComputedValue semantics)

describe('paramCellEntry', () => {
  const t = paramSpec({
    rows: [row({ T: '300', eta: '' }), row({ T: '', eta: '0.5' }), row({ T: 'abc' })],
    results: [
      { success: true, values: { T: 999, eta: 0.42 }, error: null },
      { success: false, values: {}, error: 'diverged' },
      { success: true, values: { T: 1, eta: 2 }, error: null },
    ],
  })

  it('prefers the typed draft over the solved value', () => {
    expect(paramCellEntry(t, 0, 'T')).toEqual({ num: 300, text: '300' })
  })

  it('falls back to the solved value only when the draft is blank', () => {
    expect(paramCellEntry(t, 0, 'eta')).toEqual({ num: 0.42, text: '0.42' })
  })

  it('returns null for a blank cell of a failed run', () => {
    expect(paramCellEntry(t, 1, 'T')).toBeNull()
  })

  it('still uses the draft of a failed run (an input is an input)', () => {
    expect(paramCellEntry(t, 1, 'eta')).toEqual({ num: 0.5, text: '0.5' })
  })

  it('returns null for a non-numeric draft', () => {
    expect(paramCellEntry(t, 2, 'T')).toBeNull()
  })
})

// ---------------------------------------------------------------------------
// Sweep → Function, 1-D

describe('functionSpecFromParamColumns (1-D)', () => {
  it('merges drafts and solved values, skips failed/incomplete rows, counts both', () => {
    const t = paramSpec({
      vars: ['T', 'eta'],
      rows: [
        row({ T: '300', eta: '' }), // solved eta
        row({ T: '400', eta: '' }), // failed run, blank eta -> skipped
        row({ T: '500', eta: '0.7' }), // full draft row (run failed; drafts win)
        row({ T: '', eta: '' }), // nothing -> skipped
      ],
      results: [
        { success: true, values: { eta: 0.42 }, error: null },
        { success: false, values: {}, error: 'diverged' },
        { success: false, values: {}, error: 'diverged' },
      ],
    })
    const out = functionSpecFromParamColumns({ table: t, xVar: 'T', yVar: 'eta', name: 'eta_fn' })
    expect(out.usedRows).toBe(2)
    expect(out.skippedRows).toBe(2)
    expect(out.decimated).toBe(false)
    expect(out.spec.rows).toEqual([
      { x: '300', ys: ['0.42'] },
      { x: '500', ys: ['0.7'] },
    ])
    expect(out.spec.is1D).toBe(true)
    expect(out.spec.columns).toEqual([''])
    expect(out.spec.name).toBe('eta_fn')
    expect(out.spec.argName).toBe('T')
    expect(out.spec.source).toBe('gui')
  })

  it('sorts by numeric x and keeps the first row of a duplicate x', () => {
    const t = paramSpec({
      vars: ['T', 'eta'],
      rows: [
        row({ T: '10', eta: '3' }),
        row({ T: '2', eta: '1' }),
        row({ T: '10', eta: '99' }), // duplicate x -> first wins
      ],
    })
    const out = functionSpecFromParamColumns({ table: t, xVar: 'T', yVar: 'eta', name: 'f' })
    expect(out.spec.rows).toEqual([
      { x: '2', ys: ['1'] },
      { x: '10', ys: ['3'] },
    ])
    expect(out.usedRows).toBe(2)
    expect(out.duplicateCount).toBe(1)
    expect(out.invalidCount + out.duplicateCount + out.uniqueCount).toBe(out.sourceCount)
  })

  it('works on an ODE-origin table (values only, no results)', () => {
    const t = paramSpec({
      vars: ['Time', 'h'],
      origin: 'ode',
      source: 'code',
      rows: [row({ Time: '0', h: '1' }), row({ Time: '1', h: '0.5' })],
    })
    const out = functionSpecFromParamColumns({ table: t, xVar: 'Time', yVar: 'h', name: 'h_of_t' })
    expect(out.usedRows).toBe(2)
    expect(out.spec.rows).toEqual([
      { x: '0', ys: ['1'] },
      { x: '1', ys: ['0.5'] },
    ])
    // The produced table is always an editable GUI table, whatever the source.
    expect(out.spec.source).toBe('gui')
  })

  it('sanitizes a component-scoped x column into an identifier argName', () => {
    const t = paramSpec({
      vars: ['ch.ev$hg', 'eta'],
      rows: [row({ 'ch.ev$hg': '1', eta: '2' })],
    })
    const out = functionSpecFromParamColumns({
      table: t,
      xVar: 'ch.ev$hg',
      yVar: 'eta',
      name: 'f',
    })
    expect(out.spec.argName).toBe('ch_ev_hg')
  })

  it('refuses to thin past the row cap until reduction is chosen', () => {
    const n = TABLE_MAX_ROWS + 500
    const t = paramSpec({
      vars: ['T', 'eta'],
      rows: Array.from({ length: n }, (_, i) => row({ T: String(i), eta: String(2 * i) })),
    })
    const blocked = functionSpecFromParamColumns({ table: t, xVar: 'T', yVar: 'eta', name: 'f' })
    expect(blocked.needsReduction).toBe(true)
    expect(blocked.spec.rows).toHaveLength(0)
    expect(blocked.uniqueCount).toBe(n)
    const out = functionSpecFromParamColumns({
      table: t,
      xVar: 'T',
      yVar: 'eta',
      name: 'f',
      reduction: 'decimate',
    })
    expect(out.decimated).toBe(true)
    expect(out.needsReduction).toBe(false)
    expect(out.spec.rows).toHaveLength(TABLE_MAX_ROWS)
    expect(out.spec.rows[0]).toEqual({ x: '0', ys: ['0'] })
    expect(out.spec.rows.at(-1)).toEqual({ x: String(n - 1), ys: [String(2 * (n - 1))] })
    expect(out.retainedCount).toBe(TABLE_MAX_ROWS)
    expect(out.reducedCount).toBe(n - TABLE_MAX_ROWS)
    expect(out.invalidCount + out.duplicateCount + out.uniqueCount).toBe(out.sourceCount)
  })
})

// ---------------------------------------------------------------------------
// Sweep → Function, 2-D family

describe('functionSpecFromParamColumns (family)', () => {
  it('maps distinct family values to ascending curve columns with blank gaps', () => {
    const t = paramSpec({
      vars: ['Re', 'f', 'T'],
      rows: [
        row({ Re: '100', f: '0.1', T: '200' }),
        row({ Re: '100', f: '0.2', T: '100' }),
        row({ Re: '300', f: '0.3', T: '100' }),
        row({ Re: '200', f: '0.4', T: '200' }),
        row({ Re: '400', f: '0.5', T: '' }), // family missing -> skipped
      ],
    })
    const out = functionSpecFromParamColumns({
      table: t,
      xVar: 'Re',
      yVar: 'f',
      familyVar: 'T',
      name: 'fric',
    })
    expect(out.usedRows).toBe(3)
    expect(out.skippedRows).toBe(1)
    expect(out.uniqueCount).toBe(3)
    expect(out.spec.is1D).toBe(false)
    expect(out.spec.paramName).toBe('T')
    expect(out.spec.columns).toEqual(['100', '200'])
    expect(out.spec.rows).toEqual([
      { x: '100', ys: ['0.2', '0.1'] },
      { x: '200', ys: ['', '0.4'] },
      { x: '300', ys: ['0.3', ''] },
    ])
  })

  it('keeps the first y for a duplicated (x, family) pair', () => {
    const t = paramSpec({
      vars: ['Re', 'f', 'T'],
      rows: [
        row({ Re: '100', f: '0.1', T: '200' }),
        row({ Re: '100', f: '0.9', T: '200' }),
      ],
    })
    const out = functionSpecFromParamColumns({
      table: t,
      xVar: 'Re',
      yVar: 'f',
      familyVar: 'T',
      name: 'fric',
    })
    expect(out.spec.rows).toEqual([{ x: '100', ys: ['0.1'] }])
  })
})

// ---------------------------------------------------------------------------
// Numeric series → Function

describe('functionSpecFromXY', () => {
  it('skips non-finite pairs, sorts ascending and drops exact-duplicate x', () => {
    const out = functionSpecFromXY({
      name: 'sig',
      argName: 'time',
      xs: [3, Number.NaN, 1, 2, 3, 4],
      ys: [30, 5, 10, Number.POSITIVE_INFINITY, 99, 40],
    })
    // (NaN, 5) and (2, Inf) skipped; duplicate x=3 keeps the first pair.
    expect(out.skippedRows).toBe(2)
    expect(out.spec.rows).toEqual([
      { x: '1', ys: ['10'] },
      { x: '3', ys: ['30'] },
      { x: '4', ys: ['40'] },
    ])
    expect(out.usedRows).toBe(3)
    expect(out.decimated).toBe(false)
    expect(out.duplicateCount).toBe(1)
    expect(out.invalidCount).toBe(2)
    expect(out.invalidCount + out.duplicateCount + out.uniqueCount).toBe(out.sourceCount)
    expect(out.spec.is1D).toBe(true)
    expect(out.spec.source).toBe('gui')
  })

  it('stores full precision (no display rounding)', () => {
    const out = functionSpecFromXY({
      name: 'sig',
      argName: 't',
      xs: [1000.0000201, 1000.0000202],
      ys: [0.1, 0.2],
    })
    expect(out.spec.rows).toHaveLength(2)
    expect(out.spec.rows[0].x).toBe('1000.0000201')
  })

  it('decimates uniformly to the row cap, keeping first and last', () => {
    const n = 12_000
    const xs = Float64Array.from({ length: n }, (_, i) => i * 0.001)
    const ys = Float64Array.from({ length: n }, (_, i) => Math.sin(i))
    const blocked = functionSpecFromXY({ name: 'sig', argName: 'time', xs, ys })
    expect(blocked.needsReduction).toBe(true)
    expect(blocked.spec.rows).toHaveLength(0)
    const out = functionSpecFromXY({ name: 'sig', argName: 'time', xs, ys, reduction: 'decimate' })
    expect(out.decimated).toBe(true)
    expect(out.spec.rows).toHaveLength(TABLE_MAX_ROWS)
    expect(out.spec.rows[0].x).toBe('0')
    expect(out.spec.rows.at(-1)?.x).toBe(String((n - 1) * 0.001))
    expect(out.spec.conversion?.reducedCount).toBe(n - TABLE_MAX_ROWS)
  })

  it('honours a custom cap and the log flags', () => {
    const out = functionSpecFromXY({
      name: 'sig',
      argName: 'x',
      xs: [1, 2, 3, 4, 5],
      ys: [1, 2, 3, 4, 5],
      xLog: true,
      yLog: true,
      maxRows: 3,
      reduction: 'decimate',
    })
    expect(out.spec.rows.map((r) => r.x)).toEqual(['1', '3', '5'])
    expect(out.decimated).toBe(true)
    expect(out.spec.xLog).toBe(true)
    expect(out.spec.yLog).toBe(true)
  })

  it('does not drop a narrow pulse without an explicit thinning choice', () => {
    const n = TABLE_MAX_ROWS + 1
    const xs = Array.from({ length: n }, (_, i) => i)
    const ys = Array.from({ length: n }, () => 0)
    const idx = decimationIndices(n, TABLE_MAX_ROWS)
    const kept = new Set(idx)
    const omitted = xs.findIndex((_, i) => !kept.has(i))
    ys[omitted] = 1
    const blocked = functionSpecFromXY({ name: 'pulse', argName: 'x', xs, ys })
    expect(blocked.needsReduction).toBe(true)
    expect(blocked.droppedPeak).toEqual({ x: omitted, y: 1 })
    const trimmed = functionSpecFromXY({
      name: 'pulse',
      argName: 'x',
      xs,
      ys,
      reduction: 'trim',
      xMin: 0,
      xMax: TABLE_MAX_ROWS - 1,
    })
    expect(trimmed.needsReduction).toBe(false)
    expect(trimmed.retainedCount).toBe(TABLE_MAX_ROWS)
    expect(trimmed.spec.rows.some((r) => r.ys[0] === '1')).toBe(omitted <= TABLE_MAX_ROWS - 1)
    const thinned = functionSpecFromXY({ name: 'pulse', argName: 'x', xs, ys, reduction: 'decimate' })
    expect(thinned.spec.rows.some((r) => r.ys[0] === '1')).toBe(false)
    expect(thinned.droppedPeak).toEqual({ x: omitted, y: 1 })
  })
})

describe('paramCellEntry value source', () => {
  const t = paramSpec({
    vars: ['T', 'eta'],
    rows: [row({ T: '300', eta: '' }), row({ T: '400', eta: '0.9' })],
    results: [
      { success: true, values: { T: 301, eta: 0.42 }, error: null },
      { success: true, values: { T: 401, eta: 0.5 }, error: null },
    ],
  })

  it('raw uses only typed drafts', () => {
    expect(paramCellEntry(t, 0, 'eta', 'raw')).toBeNull()
    expect(paramCellEntry(t, 1, 'eta', 'raw')).toEqual({ num: 0.9, text: '0.9' })
  })

  it('solved uses only successful row results', () => {
    expect(paramCellEntry(t, 0, 'eta', 'solved')).toEqual({ num: 0.42, text: '0.42' })
    expect(paramCellEntry(t, 1, 'eta', 'solved')).toEqual({ num: 0.5, text: '0.5' })
  })
})

describe('decimationIndices', () => {
  it('is the identity below the target', () => {
    expect(decimationIndices(3, 5)).toEqual([0, 1, 2])
    expect(decimationIndices(5, 5)).toEqual([0, 1, 2, 3, 4])
  })

  it('spreads uniformly, strictly increasing, first and last kept', () => {
    const idx = decimationIndices(10_001, 5000)
    expect(idx).toHaveLength(5000)
    expect(idx[0]).toBe(0)
    expect(idx.at(-1)).toBe(10_000)
    for (let i = 1; i < idx.length; i++) expect(idx[i]).toBeGreaterThan(idx[i - 1])
  })

  it('degenerates safely', () => {
    expect(decimationIndices(0, 5)).toEqual([])
    expect(decimationIndices(7, 1)).toEqual([0])
  })
})

// ---------------------------------------------------------------------------
// Name validation

describe('checkFunctionName', () => {
  const tables: TableSpec[] = [
    fnSpec({ id: 'g1', name: 'eff', source: 'gui' }),
    fnSpec({ id: 'c1', name: 'Mu_Table', source: 'code' }),
    paramSpec({ id: 'p9', name: 'eff2' }), // parametric names are a different surface
  ]

  it('rejects non-identifiers', () => {
    for (const bad of ['', '  ', '2x', 'a b', 'x-y', 'f(x)', '_lead']) {
      const check = checkFunctionName(tables, bad)
      expect(check.ok).toBe(false)
      expect(check.error).toBeTruthy()
    }
  })

  it('accepts identifiers (with surrounding whitespace)', () => {
    for (const good of ['Re_frac', 'f2', ' eta_fit ', 'A']) {
      expect(checkFunctionName(tables, good).ok).toBe(true)
    }
  })

  it('flags a same-named GUI function table case-insensitively', () => {
    const check = checkFunctionName(tables, 'EFF')
    expect(check.ok).toBe(true)
    expect(check.replacesGui).toBe(true)
    expect(check.shadowedByCode).toBe(false)
  })

  it('flags a same-named code TABLE block (the document wins on solve — D10)', () => {
    const check = checkFunctionName(tables, 'mu_table')
    expect(check.ok).toBe(true)
    expect(check.replacesGui).toBe(false)
    expect(check.shadowedByCode).toBe(true)
  })

  it('does not treat a same-named parametric table as a collision', () => {
    const check = checkFunctionName(tables, 'eff2')
    expect(check.replacesGui).toBe(false)
    expect(check.shadowedByCode).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// Replace-vs-new

describe('applyFunctionSpecs', () => {
  it('replaces a same-named GUI table in place, keeping its id and position', () => {
    const existing: TableSpec[] = [
      fnSpec({ id: 'keep-me', name: 'eff' }),
      paramSpec({ id: 'p1' }),
    ]
    const incoming = fnSpec({ id: 'fresh', name: 'EFF', rows: [{ x: '9', ys: ['9'] }] })
    const { tables, ids } = applyFunctionSpecs(existing, [incoming])
    expect(tables).toHaveLength(2)
    expect(tables[0].id).toBe('keep-me')
    expect((tables[0] as FunctionTableSpec).rows).toEqual([{ x: '9', ys: ['9'] }])
    expect(ids).toEqual(['keep-me'])
    // Input list untouched (pure).
    expect((existing[0] as FunctionTableSpec).rows).toEqual([{ x: '1', ys: ['2'] }])
  })

  it('appends when no GUI table matches — a code table of the same name is never replaced', () => {
    const existing: TableSpec[] = [fnSpec({ id: 'c1', name: 'eff', source: 'code' })]
    const incoming = fnSpec({ id: 'fresh', name: 'eff' })
    const { tables, ids } = applyFunctionSpecs(existing, [incoming])
    expect(tables).toHaveLength(2)
    expect(tables[0].id).toBe('c1')
    expect(tables[0].source).toBe('code')
    expect(tables[1].id).toBe('fresh')
    expect(ids).toEqual(['fresh'])
  })

  it('applies several specs, mixing replace and append', () => {
    const existing: TableSpec[] = [fnSpec({ id: 'g1', name: 'a' })]
    const { tables, ids } = applyFunctionSpecs(existing, [
      fnSpec({ id: 'n1', name: 'a', rows: [] }),
      fnSpec({ id: 'n2', name: 'b' }),
    ])
    expect(tables.map((t) => t.id)).toEqual(['g1', 'n2'])
    expect(ids).toEqual(['g1', 'n2'])
  })
})
