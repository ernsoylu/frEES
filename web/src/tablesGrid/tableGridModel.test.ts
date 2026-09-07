// tablesGrid/tableGridModel.test.ts
//
// The behavioural contract of the Tables grid, retargeted from
// spreadsheet/tableBinding.test.ts (D10: the Univer workbook is replaced by a
// native glide grid; the spec is the only document). Cases that were
// Univer-materialization-specific are retired and named in the D10 work log:
// f-cell re-materialization (formulas no longer evaluate — replaced by the
// read-only surfacing tests below) and Run-column drift (the Run column is
// simply not editable in the native grid; the writable-through-protection
// hole it guarded no longer exists — replaced by the paste-into-Run test).

import { describe, expect, it } from 'vitest'
import { FunctionTableSpec, ParamTableSpec, toFunctionTableDtos } from '../tables'
import {
  appendRow,
  applyCellEdit,
  applyCellEdits,
  restoreUserEdit,
  applyColumnFill,
  applyPaste,
  boundColumnCount,
  cellViewAt,
  clearCells,
  csvValuesFor,
  formulaAt,
  gridRowCount,
  headerTitles,
  isErrorValue,
  isHostedTable,
  paramComputedValue,
  removeLastRow,
  storedFormulaList,
  TABLE_MAX_ROWS,
  insertRowAt,
  deleteRowsAt,
  duplicateRowsAt,
} from './tableGridModel'

function spec1D(over: Partial<FunctionTableSpec> = {}): FunctionTableSpec {
  return {
    id: 't1',
    kind: 'function',
    name: 'eff',
    argName: 'Re',
    paramName: '',
    xLog: false,
    yLog: false,
    columns: [''],
    rows: [
      { x: '1', ys: ['10'] },
      { x: '2', ys: ['20'] },
    ],
    is1D: true,
    ...over,
  }
}

function spec2D(over: Partial<FunctionTableSpec> = {}): FunctionTableSpec {
  return {
    id: 't2',
    kind: 'function',
    name: 'fanCurve',
    argName: 'V',
    paramName: 'rpm',
    xLog: false,
    yLog: false,
    columns: ['1000', '2000'],
    rows: [
      { x: '0', ys: ['100', '200'] },
      { x: '1', ys: ['80', '160'] },
    ],
    is1D: false,
    ...over,
  }
}

function paramSpec(over: Partial<ParamTableSpec> = {}): ParamTableSpec {
  return {
    id: 'p1',
    kind: 'parametric',
    name: 'Sweep',
    vars: ['T', 'P'],
    rows: [
      { id: 'r1', values: { T: '300', P: '' } },
      { id: 'r2', values: { T: '350', P: '' } },
    ],
    results: [],
    stats: null,
    checkResult: null,
    checkMessage: '',
    ...over,
  }
}

function solvedParam(): ParamTableSpec {
  return paramSpec({
    results: [
      { success: true, values: { T: 300, P: 101.3 }, error: null },
      { success: false, error: 'diverged', values: {} },
    ],
  })
}

// ---------------------------------------------------------------------------
// Layout (retargeted from specToSheetData)

describe('function-table grid layout', () => {
  it('lays out the header row + data rows anchored at (0,0)', () => {
    const spec = spec2D()
    expect(gridRowCount(spec)).toBe(3) // header + 2 data rows
    expect(cellViewAt(spec, 0, 0)).toMatchObject({ text: 'V', kind: 'header', editable: false })
    expect(cellViewAt(spec, 0, 1)).toMatchObject({ text: '1000', kind: 'header', editable: true })
    expect(cellViewAt(spec, 0, 2).text).toBe('2000')
    expect(cellViewAt(spec, 1, 0).text).toBe('0')
    expect(cellViewAt(spec, 2, 2).text).toBe('160')
  })

  it('uses a fixed, non-editable y label for 1-D tables', () => {
    expect(cellViewAt(spec1D(), 0, 1)).toMatchObject({ text: 'y', kind: 'header', editable: false })
  })

  it('falls back to x for a blank argument name', () => {
    expect(cellViewAt(spec1D({ argName: '' }), 0, 0).text).toBe('x')
  })
})

describe('parametric grid layout', () => {
  it('puts Run + variable names (with units) in the glide headers', () => {
    expect(headerTitles(paramSpec())).toEqual(['Run', 'T', 'P'])
    expect(headerTitles(paramSpec({ columnUnits: { T: 'K' } }))).toEqual(['Run', 'T [K]', 'P'])
  })

  it('shows argument and output units on function-table headers', () => {
    expect(headerTitles(spec1D({ argName: 'P', argUnit: 'kPa', outputUnit: 'm' }))).toEqual([
      'P [kPa]',
      'y [m]',
    ])
  })

  it('renders run labels, inputs and blank cells as data rows', () => {
    const spec = paramSpec()
    expect(gridRowCount(spec)).toBe(2) // no in-grid header row
    expect(cellViewAt(spec, 0, 0)).toMatchObject({ text: '1', kind: 'run', editable: false })
    expect(cellViewAt(spec, 0, 1)).toMatchObject({ text: '300', kind: 'input', editable: true })
    expect(cellViewAt(spec, 0, 2).text).toBe('')
  })

  it('shows computed cells (blank input + successful run) and the failed-run marker', () => {
    const spec = solvedParam()
    expect(cellViewAt(spec, 0, 2)).toMatchObject({ text: '101.3', kind: 'computed' })
    expect(cellViewAt(spec, 1, 0)).toMatchObject({ text: '2 ✗', kind: 'run', failed: true })
    // T has an input draft: never shown as computed even though solved.
    expect(cellViewAt(spec, 0, 1)).toMatchObject({ text: '300', kind: 'input' })
  })
})

// ---------------------------------------------------------------------------
// Editability rules (retargeted from the protection map — the native grid
// has no range protection; these ARE the guarantee now)

describe('editability', () => {
  it('protects only schema header cells on GUI function tables', () => {
    // 1-D: argName + the y label are toolbar-owned.
    expect(cellViewAt(spec1D(), 0, 0).editable).toBe(false)
    expect(cellViewAt(spec1D(), 0, 1).editable).toBe(false)
    expect(cellViewAt(spec1D(), 1, 0).editable).toBe(true)
    // 2-D: curve-parameter VALUE headers stay editable (they map to columns).
    expect(cellViewAt(spec2D(), 0, 0).editable).toBe(false)
    expect(cellViewAt(spec2D(), 0, 1).editable).toBe(true)
  })

  it('protects the Run column on parametric tables', () => {
    expect(cellViewAt(paramSpec(), 0, 0).editable).toBe(false)
    expect(cellViewAt(paramSpec(), 1, 1).editable).toBe(true)
  })

  it('protects everything on code tables', () => {
    const code1 = spec2D({ source: 'code' })
    expect(cellViewAt(code1, 0, 1).editable).toBe(false)
    expect(cellViewAt(code1, 1, 0).editable).toBe(false)
    const code2 = paramSpec({ source: 'code' })
    expect(cellViewAt(code2, 0, 1).editable).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// Cell edits (retargeted from sheetEditsToSpec)

describe('function-table cell edits', () => {
  it('an identical commit is a no-op (round-trip identity)', () => {
    const spec = spec2D({ formulas: { B2: '=C2/2' } })
    const back = applyCellEdit(spec, 1, 0, '0')
    expect(back.changed).toBe(false)
    expect(back.spec).toBe(spec)
  })

  it('maps 2-D header edits back to spec.columns but never changes the column count', () => {
    const spec = spec2D()
    const back = applyCellEdit(spec, 0, 2, '3000')
    const f = back.spec as FunctionTableSpec
    expect(f.columns).toEqual(['1000', '3000'])
    expect(f.columns.length).toBe(spec.columns.length)
  })

  it('blocks conversion of invalid numeric drafts', () => {
    const back = applyCellEdit(spec1D(), 2, 1, 'garbage')
    expect(() => toFunctionTableDtos([back.spec])).toThrow(/invalid number/)
  })

  it('sanitizes formula-error literals to blank and reports the cell', () => {
    const back = applyCellEdit(spec1D(), 1, 1, '#REF!')
    const f = back.spec as FunctionTableSpec
    expect(back.errorCells).toEqual(['B2'])
    expect(f.rows[0].ys[0]).toBe('')
    const dtos = toFunctionTableDtos([back.spec])
    expect(dtos[0].curves[0].points).toEqual([[2, 20]])
  })

  it('drops the stored-formula entry when the cell is edited to a literal', () => {
    const spec = spec1D({ formulas: { B2: '=A2*10' } })
    const back = applyCellEdit(spec, 1, 1, '42')
    const f = back.spec as FunctionTableSpec
    expect(f.formulas).toBeUndefined()
    expect(f.rows[0].ys[0]).toBe('42')
  })

  it('never mutates code-sourced specs (mapper defense in depth)', () => {
    const spec = spec1D({ source: 'code' })
    expect(applyCellEdit(spec, 1, 1, '999999').spec).toBe(spec)
    expect(applyPaste(spec, 1, 0, [['9', '9']]).spec).toBe(spec)
    expect(clearCells(spec, [{ gridRow: 1, col: 1 }]).spec).toBe(spec)
  })
})

describe('parametric cell edits', () => {
  it('committing the displayed computed text unchanged stays computed (no invalidation)', () => {
    const spec = solvedParam()
    const back = applyCellEdit(spec, 0, 2, '101.3')
    expect(back.changed).toBe(false)
    expect(back.spec).toBe(spec)
    // Same for an untouched input draft.
    expect(applyCellEdit(spec, 0, 1, '300').changed).toBe(false)
  })

  it('an input edit invalidates results/check and preserves row identities', () => {
    const spec = solvedParam()
    const p = applyCellEdit(spec, 0, 1, '310').spec as ParamTableSpec
    expect(p.rows[0].values.T).toBe('310')
    expect(p.rows.map((r) => r.id)).toEqual(['r1', 'r2'])
    expect(p.results).toEqual([])
    expect(p.checkResult).toBeNull()
  })

  it('typing over a computed cell turns it into an input override and invalidates', () => {
    const spec = solvedParam()
    const p = applyCellEdit(spec, 0, 2, '999').spec as ParamTableSpec
    expect(p.rows[0].values.P).toBe('999')
    expect(p.results).toEqual([])
  })

  it('a cell edit never changes the row count', () => {
    const spec = paramSpec()
    const p = applyCellEdit(spec, 1, 1, '360').spec as ParamTableSpec
    expect(p.rows.length).toBe(2)
  })
})

// ---------------------------------------------------------------------------
// Clearing cells (retargeted run-deletion rules)

describe('clearCells', () => {
  it('drops trailing runs the user deleted by clearing their cells', () => {
    const spec = paramSpec({
      rows: [
        { id: 'r1', values: { T: '300', P: '' } },
        { id: 'r2', values: { T: '350', P: '' } },
        { id: 'r3', values: { T: '400', P: '' } },
        { id: 'r4', values: { T: '450', P: '' } },
      ],
    })
    const p = clearCells(spec, [
      { gridRow: 2, col: 1 },
      { gridRow: 3, col: 1 },
    ]).spec as ParamTableSpec
    expect(p.rows.length).toBe(2)
    expect(p.rows.map((r) => r.id)).toEqual(['r1', 'r2'])
    expect(p.results).toEqual([])
  })

  it('keeps rows that were already blank (Add Row survives unrelated edits)', () => {
    const spec = paramSpec({
      rows: [
        { id: 'r1', values: { T: '300', P: '' } },
        { id: 'r2', values: { T: '350', P: '' } },
        { id: 'r3', values: {} },
      ],
    })
    const p = applyCellEdit(spec, 0, 1, '310').spec as ParamTableSpec
    expect(p.rows.length).toBe(3)
    expect(p.rows[0].values.T).toBe('310')
  })

  it('clearing a middle row keeps it as a blank (free-solve) run', () => {
    const spec = paramSpec({
      rows: [
        { id: 'r1', values: { T: '300', P: '' } },
        { id: 'r2', values: { T: '350', P: '' } },
        { id: 'r3', values: { T: '400', P: '' } },
      ],
    })
    const p = clearCells(spec, [{ gridRow: 1, col: 1 }]).spec as ParamTableSpec
    expect(p.rows.length).toBe(3)
    expect(p.rows[1].values.T).toBe('')
  })

  it('clears 2-D curve-parameter header cells but ignores schema labels', () => {
    const spec = spec2D()
    const f = clearCells(spec, [
      { gridRow: 0, col: 0 }, // argName label: schema, ignored
      { gridRow: 0, col: 2 },
    ]).spec as FunctionTableSpec
    expect(f.columns).toEqual(['1000', ''])
    expect(f.argName).toBe('V')
  })

  it('trims trailing function rows that became blank, keeping starter blanks', () => {
    const spec = spec1D({
      rows: [
        { x: '1', ys: ['10'] },
        { x: '2', ys: ['20'] },
      ],
    })
    const f = clearCells(spec, [
      { gridRow: 2, col: 0 },
      { gridRow: 2, col: 1 },
    ]).spec as FunctionTableSpec
    expect(f.rows).toEqual([{ x: '1', ys: ['10'] }])
  })
})

// ---------------------------------------------------------------------------
// Paste (multi-cell TSV; region rules retargeted from the out-of-region /
// row-cap contract)

describe('applyPaste', () => {
  it('applies a multi-cell matrix and invalidates parametric results', () => {
    const spec = solvedParam()
    const back = applyPaste(spec, 0, 1, [
      ['301', '5'],
      ['351', '6'],
    ])
    const p = back.spec as ParamTableSpec
    expect(p.rows[0].values).toEqual({ T: '301', P: '5' })
    expect(p.rows[1].values).toEqual({ T: '351', P: '6' })
    expect(p.results).toEqual([])
    expect(back.outOfRegion).toBe(false)
  })

  it('grows rows downward (region growth by pasting below the data)', () => {
    const spec = paramSpec()
    const back = applyPaste(spec, 2, 1, [['400'], ['450']])
    const p = back.spec as ParamTableSpec
    expect(p.rows.length).toBe(4)
    expect(p.rows[3].values.T).toBe('450')
  })

  it('clips content beyond the bound columns and flags it (columns are schema)', () => {
    const spec = spec1D()
    const back = applyPaste(spec, 1, 0, [['5', '50', 'stray']])
    const f = back.spec as FunctionTableSpec
    expect(back.outOfRegion).toBe(true)
    expect(f.rows[0]).toEqual({ x: '5', ys: ['50'] })
  })

  it('truncates at the row cap', () => {
    const spec = spec1D()
    const matrix = Array.from({ length: TABLE_MAX_ROWS + 100 }, (_, i) => [String(i), String(i * 2)])
    const back = applyPaste(spec, 1, 0, matrix)
    expect(back.truncated).toBe(true)
    expect((back.spec as FunctionTableSpec).rows.length).toBe(TABLE_MAX_ROWS)
  })

  it('never writes the Run column and flags the attempt', () => {
    const spec = paramSpec()
    const back = applyPaste(spec, 0, 0, [['99', '301']])
    const p = back.spec as ParamTableSpec
    expect(back.intoReadOnly).toBe(true)
    expect(cellViewAt(p, 0, 0).text).toBe('1') // run label managed by the table
    expect(p.rows[0].values.T).toBe('301') // the rest of the paste still lands
  })

  it('updates 2-D curve headers from a paste but skips 1-D schema labels', () => {
    const two = applyPaste(spec2D(), 0, 1, [['1500', '2500']])
    expect((two.spec as FunctionTableSpec).columns).toEqual(['1500', '2500'])
    const one = applyPaste(spec1D(), 0, 1, [['zzz']])
    expect(one.intoReadOnly).toBe(true)
    expect((one.spec as FunctionTableSpec).columns).toEqual([''])
  })

  it('sanitizes pasted error literals to blank and reports them', () => {
    const back = applyPaste(spec1D(), 1, 1, [['#DIV/0!']])
    expect(back.errorCells).toEqual(['B2'])
    expect((back.spec as FunctionTableSpec).rows[0].ys[0]).toBe('')
  })
})

// ---------------------------------------------------------------------------
// Toolbar / dialog applications

describe('row operations', () => {
  it('appendRow adds a blank row (and invalidates parametric runs)', () => {
    const p = appendRow(solvedParam()) as ParamTableSpec
    expect(p.rows.length).toBe(3)
    expect(p.results).toEqual([])
    const f = appendRow(spec2D()) as FunctionTableSpec
    expect(f.rows.length).toBe(3)
    expect(f.rows[2]).toEqual({ x: '', ys: ['', ''] })
  })

  it('removeLastRow drops one row, never below one, never on code tables', () => {
    const p = removeLastRow(paramSpec()) as ParamTableSpec
    expect(p.rows.length).toBe(1)
    expect(removeLastRow(p)).toBe(p)
    const code = paramSpec({ source: 'code' })
    expect(removeLastRow(code)).toBe(code)
  })

  it('applyColumnFill writes one value per row and invalidates (Fill Column dialog)', () => {
    const p = applyColumnFill(solvedParam(), 'T', [100, 200])
    expect(p.rows.map((r) => r.values.T)).toEqual(['100', '200'])
    expect(p.rows.map((r) => r.id)).toEqual(['r1', 'r2'])
    expect(p.results).toEqual([])
    expect(p.checkResult).toBeNull()
  })
})

// ---------------------------------------------------------------------------
// Read-only formula surfacing (D10: shown, never evaluated, never silently
// dropped)

describe('stored formula surfacing', () => {
  it('maps overlay refs onto grid cells in both layouts', () => {
    // Function: B2 = first data row, first y column (grid row 1, col 1).
    const f = spec1D({ formulas: { B2: '=A2*10' } })
    expect(formulaAt(f, 1, 1)).toBe('=A2*10')
    expect(cellViewAt(f, 1, 1).formula).toBe('=A2*10')
    // Parametric: B2 is the SHEET ref (header row + data), so it lands on
    // grid row 0 (the first data row) in the header-less parametric grid.
    const p = paramSpec({ formulas: { B2: '=C2/2' } })
    expect(formulaAt(p, 0, 1)).toBe('=C2/2')
    expect(cellViewAt(p, 0, 1).formula).toBe('=C2/2')
  })

  it('lists every stored formula for the hint line', () => {
    const f = spec1D({ formulas: { B2: '=A2*10', B3: '=A3*10' } })
    expect(storedFormulaList(f)).toEqual([
      { ref: 'B2', formula: '=A2*10' },
      { ref: 'B3', formula: '=A3*10' },
    ])
    expect(storedFormulaList(spec1D())).toEqual([])
  })

  it('keeps untouched formulas across unrelated edits', () => {
    const spec = spec1D({ formulas: { B2: '=A2*10' } })
    const back = applyCellEdit(spec, 2, 0, '3')
    expect((back.spec as FunctionTableSpec).formulas).toEqual({ B2: '=A2*10' })
  })

  it('surfaces the stored formula even when the cell value is blank', () => {
    // Ported from tableBinding.test.ts ("emits overlay cells even when the
    // cached value is blank"): a legacy formula whose cached result never
    // materialized must still be visible, not silently invisible.
    const spec = spec1D({ rows: [{ x: '1', ys: [''] }], formulas: { B2: '=1/0' } })
    expect(cellViewAt(spec, 1, 1)).toMatchObject({ text: '', formula: '=1/0' })
    expect(storedFormulaList(spec)).toEqual([{ ref: 'B2', formula: '=1/0' }])
  })
})

// ---------------------------------------------------------------------------
// Shared predicates (kept verbatim from tableBinding)

describe('error literal detection', () => {
  it.each(['#REF!', '#DIV/0!', '#VALUE!', '#NAME?', '#N/A', '#NUM!'])('flags %s', (s) => {
    expect(isErrorValue(s)).toBe(true)
  })
  it('does not flag ordinary strings or numbers', () => {
    expect(isErrorValue('#hashtag')).toBe(false)
    expect(isErrorValue('REF')).toBe(false)
    expect(isErrorValue(42)).toBe(false)
  })
})

describe('boundColumnCount', () => {
  it('is x plus one per curve / Run plus one per variable', () => {
    expect(boundColumnCount(spec1D())).toBe(2)
    expect(boundColumnCount(spec2D())).toBe(3)
    expect(boundColumnCount(paramSpec())).toBe(3)
  })
})

describe('hosting and computed detection', () => {
  it('code parametric tables are not hosted; code function tables are', () => {
    expect(isHostedTable(paramSpec({ source: 'code' }))).toBe(false)
    expect(isHostedTable(paramSpec())).toBe(true)
    expect(isHostedTable(spec1D({ source: 'code' }))).toBe(true)
  })

  it('paramComputedValue only fires for blank input + successful run', () => {
    const spec = solvedParam()
    expect(paramComputedValue(spec, 0, 'P')).toBe(101.3)
    expect(paramComputedValue(spec, 0, 'T')).toBeUndefined() // has input draft
    expect(paramComputedValue(spec, 1, 'P')).toBeUndefined() // failed run
  })
})

// ---------------------------------------------------------------------------
// CSV export (what the grid shows: headers + merged computed values)

describe('csvValuesFor', () => {
  it('exports parametric headers, run labels and merged computed values', () => {
    expect(csvValuesFor(solvedParam())).toEqual([
      ['Run', 'T', 'P'],
      ['1', '300', '101.3'],
      ['2 ✗', '350', ''],
    ])
  })

  it('exports function headers and raw rows', () => {
    expect(csvValuesFor(spec2D())).toEqual([
      ['V', '1000', '2000'],
      ['0', '100', '200'],
      ['1', '80', '160'],
    ])
    expect(csvValuesFor(spec1D())[0]).toEqual(['Re', 'y'])
  })

  it('formats a report export without changing the exact-data grid values', () => {
    const spec = spec1D({ rows: [{ x: '1.000001', ys: ['1.23456789'] }] })
    expect(csvValuesFor(spec, 'exact')[1]).toEqual(['1.000001', '1.23456789'])
    expect(csvValuesFor(spec, 'display')[1][1]).toBe(Number.parseFloat((1.23456789).toPrecision(6)).toString())
    expect(spec.rows[0].ys[0]).toBe('1.23456789')
  })
})

describe('atomic edits and scoped history', () => {
  it('classifies computed echoes against the original snapshot regardless of batch order', () => {
    const before = solvedParam()
    const edits = [{ gridRow: 0, col: 1, text: '301' }, { gridRow: 0, col: 2, text: '101.3' }]
    const result = applyCellEdits(before, edits).spec as ParamTableSpec
    expect(result).toEqual(applyCellEdits(before, [...edits].reverse()).spec)
    expect(result.rows[0].values).toEqual({ T: '301', P: '' })
    const restored = restoreUserEdit({ ...result, name: 'Renamed' }, result, before) as ParamTableSpec
    expect(restored.name).toBe('Renamed')
    expect(restored.rows).toEqual(before.rows)
    expect(restored.results).toEqual([])
  })
})

it('detaches legacy formulas before structural changes so removed cells never reattach', () => {
  const table = spec1D({ formulas: { A3: '=old_x', B3: '=old_y' } })
  const edited = appendRow(removeLastRow(table))
  expect(formulaAt(edited, 2, 1)).toBeUndefined()
  expect(storedFormulaList(edited)).toEqual([{ ref: 'Detached 1: A3', formula: '=old_x' }, { ref: 'Detached 1: B3', formula: '=old_y' }])
})

describe('selected-row structural operations (Phase 10D)', () => {
  it('inserts an empty row at the selected index in a parametric table', () => {
    const table = paramSpec()
    const inserted = insertRowAt(table, 1) as ParamTableSpec
    expect(inserted.rows.length).toBe(3)
    expect(inserted.rows[0].values.T).toBe('300')
    expect(inserted.rows[1].values).toEqual({})
    expect(inserted.rows[2].values.T).toBe('350')
    expect(inserted.results).toEqual([])
  })

  it('inserts an empty row at the selected index in a function table', () => {
    const table = spec2D()
    const inserted = insertRowAt(table, 0) as FunctionTableSpec
    expect(inserted.rows.length).toBe(3)
    expect(inserted.rows[0]).toEqual({ x: '', ys: ['', ''] })
    expect(inserted.rows[1].x).toBe('0')
    expect(inserted.rows[2].x).toBe('1')
  })

  it('duplicates selected rows at their position in a parametric table', () => {
    const table = paramSpec()
    const duped = duplicateRowsAt(table, [0]) as ParamTableSpec
    expect(duped.rows.length).toBe(3)
    expect(duped.rows[0].values.T).toBe('300')
    expect(duped.rows[1].values.T).toBe('300')
    expect(duped.rows[1].id).not.toBe(duped.rows[0].id)
    expect(duped.rows[2].values.T).toBe('350')
    expect(duped.results).toEqual([])
  })

  it('duplicates selected rows in a function table', () => {
    const table = spec1D()
    const duped = duplicateRowsAt(table, [1]) as FunctionTableSpec
    expect(duped.rows.length).toBe(3)
    expect(duped.rows[1]).toEqual({ x: '2', ys: ['20'] })
    expect(duped.rows[2]).toEqual({ x: '2', ys: ['20'] })
  })

  it('deletes selected rows in a parametric table, ensuring at least one row remains', () => {
    const table = paramSpec()
    const deleted = deleteRowsAt(table, [0]) as ParamTableSpec
    expect(deleted.rows.length).toBe(1)
    expect(deleted.rows[0].values.T).toBe('350')
    expect(deleted.results).toEqual([])

    // Deleting all rows leaves a fresh empty row
    const deletedAll = deleteRowsAt(deleted, [0]) as ParamTableSpec
    expect(deletedAll.rows.length).toBe(1)
    expect(deletedAll.rows[0].values).toEqual({})
  })

  it('deletes selected rows in a function table, ensuring at least one row remains', () => {
    const table = spec2D()
    const deleted = deleteRowsAt(table, [0, 1]) as FunctionTableSpec
    expect(deleted.rows.length).toBe(1)
    expect(deleted.rows[0]).toEqual({ x: '', ys: ['', ''] })
  })

  it('enforces TABLE_MAX_ROWS ceiling during insert and duplicate', () => {
    const fullRows = Array.from({ length: TABLE_MAX_ROWS }, (_, i) => ({
      id: `r${i}`,
      values: { T: String(i) },
    }))
    const fullTable = paramSpec({ rows: fullRows })
    expect(insertRowAt(fullTable, 0)).toBe(fullTable)
    expect(duplicateRowsAt(fullTable, [0])).toBe(fullTable)
  })
})

