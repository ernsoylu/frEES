// tablesGrid/tableGridModel.ts
//
// Pure projections between a TableSpec and the native Tables grid (decision
// D10: the Univer workbook is replaced by glide-data-grid; the spec is the
// only document — there is no materialized sheet layer any more). These are
// the tableBinding.ts rules that survive the swap, retargeted: computed-value
// visibility, the row cap, error-literal sanitization, header/Run protection
// (now simple editability rules), region-bounded paste, and the read-only
// surfacing of stored spec.formulas. No React, no Univer, no glide imports —
// every rule is unit-testable as data in / data out.
//
// Grid layout:
//   parametric — glide column headers carry `Run` + one `name [unit]` per
//                variable; grid rows are exactly spec.rows (no in-grid header
//                row). Column 0 is the display-only Run column.
//   function   — the grid renders its own bold header row at grid row 0
//                (glide's header is hidden): A = argument name (toolbar-
//                edited), B.. = the curve-parameter values (editable for 2-D,
//                they map back to spec.columns) or the fixed 'y' label (1-D).
//                Data rows follow at grid rows 1..N.

import { newParamRow, ParamRow, ParamTableSpec, TableSpec } from '../tables'

/** Hard cap on data rows (contract a of the old binding layer, kept): a
 * runaway 50k-row paste truncates here instead of bloating the .frees file
 * and swamping the solver. Matches the engine-side cap. */
export const TABLE_MAX_ROWS = 5000

export { isHostedTable } from './tablesWorkbookBridge'

/** Formula error literals (#REF! after a referenced sheet was deleted,
 * #DIV/0!, …). They can no longer be produced in the grid (formulas are not
 * evaluated since D10) but still arrive in pastes from Excel/Sheets and in
 * legacy stored data: they map to omitted spec values — never a crash-risk
 * string in a solver DTO. */
const ERROR_VALUE = /^#(?:REF!|DIV\/0!|VALUE!|NAME\?|N\/A|NUM!|NULL!|CALC!|SPILL!|ERROR!|CYCLE!)$/i

export function isErrorValue(v: unknown): boolean {
  return typeof v === 'string' && ERROR_VALUE.test(v.trim())
}

/** Bound columns: function tables = x plus one per curve; parametric tables
 * = the Run column plus one per table variable. */
export function boundColumnCount(spec: TableSpec): number {
  return spec.kind === 'function' ? 1 + spec.columns.length : 1 + spec.vars.length
}

/** The computed (solver-written) value shown in a parametric cell: only when
 * the row solved successfully AND the user left the input draft blank. */
export function paramComputedValue(
  spec: ParamTableSpec,
  rowIndex: number,
  varName: string,
): number | undefined {
  const res = spec.results[rowIndex]
  if (!res?.success) return undefined
  if ((spec.rows[rowIndex]?.values[varName] ?? '').trim() !== '') return undefined
  return res.values[varName]
}

/** Spreadsheet column name for a 0-based index (A, B, …, Z, AA, …). Copied
 * from univerAdapter.ts — this module must not import the Univer surface. */
export function colName(c: number): string {
  let s = ''
  let t = c
  while (t >= 0) {
    s = String.fromCodePoint(65 + (t % 26)) + s
    t = Math.floor(t / 26) - 1
  }
  return s
}

/** A1 ref for 0-based sheet coordinates. */
export function a1(r: number, c: number): string {
  return `${colName(c)}${r + 1}`
}

// ---------------------------------------------------------------------------
// Layout / cell views

/** Grid rows for a spec: function tables carry their own header row at grid
 * row 0; parametric tables are data-only (glide headers carry the schema). */
export function gridRowCount(spec: TableSpec): number {
  return spec.kind === 'function' ? spec.rows.length + 1 : spec.rows.length
}

/** Glide column-header titles. Only parametric grids show glide's header
 * (function grids render their own header row); still defined for both so a
 * column always has an identity. */
export function headerTitles(spec: TableSpec): string[] {
  if (spec.kind === 'parametric') {
    return [
      'Run',
      ...spec.vars.map((name) => {
        const unit = spec.columnUnits?.[name]
        return unit ? `${name} [${unit}]` : name
      }),
    ]
  }
  return [
    spec.argName || 'x',
    ...spec.columns.map((param, j) => (spec.is1D ? 'y' : param || `curve ${j + 1}`)),
  ]
}

export type CellKind = 'header' | 'run' | 'input' | 'computed'

export interface CellView {
  text: string
  kind: CellKind
  editable: boolean
  /** Run cell of a failed run (rendered red, `N ✗`). */
  failed?: boolean
  /** Stored legacy formula for this cell (surfaced read-only per D10 — shown,
   * never evaluated, never silently dropped). */
  formula?: string
}

/** The A1 ref of a grid cell in the OLD bound-sheet layout, which is the key
 * space of spec.formulas: both kinds had a header row at sheet row 1, so a
 * parametric data row i lives at sheet row i+2 while the function grid rows
 * (which include the header at grid row 0) map 1:1. */
export function formulaRefFor(spec: TableSpec, gridRow: number, col: number): string {
  return spec.kind === 'function' ? a1(gridRow, col) : a1(gridRow + 1, col)
}

export function formulaAt(spec: TableSpec, gridRow: number, col: number): string | undefined {
  if (!spec.formulas) return undefined
  if (spec.kind === 'function' && gridRow === 0) return undefined
  return spec.formulas[formulaRefFor(spec, gridRow, col)]
}

/** Every stored legacy formula, for the read-only hint line. */
export function storedFormulaList(spec: TableSpec): { ref: string; formula: string }[] {
  return Object.entries(spec.formulas ?? {}).map(([ref, formula]) => ({ ref, formula }))
}

export function cellViewAt(spec: TableSpec, gridRow: number, col: number): CellView {
  const readOnly = spec.source === 'code'
  if (spec.kind === 'function') {
    if (gridRow === 0) {
      if (col === 0) {
        return { text: spec.argName || 'x', kind: 'header', editable: false }
      }
      const j = col - 1
      return {
        text: spec.is1D ? 'y' : spec.columns[j] ?? '',
        kind: 'header',
        // 2-D curve-parameter VALUE headers are editable — they map back to
        // spec.columns (the old sheet's one editable header range).
        editable: !readOnly && !spec.is1D,
      }
    }
    const row = spec.rows[gridRow - 1]
    const text = col === 0 ? row?.x ?? '' : row?.ys[col - 1] ?? ''
    return {
      text,
      kind: 'input',
      editable: !readOnly,
      formula: formulaAt(spec, gridRow, col),
    }
  }

  // Parametric: col 0 is the display-only Run column.
  const i = gridRow
  if (col === 0) {
    const res = spec.results[i]
    const failed = res ? !res.success : false
    return { text: failed ? `${i + 1} ✗` : String(i + 1), kind: 'run', editable: false, failed }
  }
  const name = spec.vars[col - 1]
  const computed = paramComputedValue(spec, i, name)
  if (computed !== undefined) {
    // Full-precision echo (String, not fmt6) so committing the displayed text
    // unchanged still compares equal and stays a computed cell.
    return { text: String(computed), kind: 'computed', editable: !readOnly }
  }
  return {
    text: spec.rows[i]?.values[name] ?? '',
    kind: 'input',
    editable: !readOnly,
    formula: formulaAt(spec, gridRow, col),
  }
}

// ---------------------------------------------------------------------------
// Edits (grid -> spec; the spec is the only document)

export interface EditResult {
  spec: TableSpec
  /** False when the edit was a no-op (identical text, read-only cell, code
   * table): the caller must not push an undo entry or re-render. */
  changed: boolean
  /** A1 refs whose value was a formula-error literal, stored as blank. */
  errorCells: string[]
}

function unchanged(spec: TableSpec): EditResult {
  return { spec, changed: false, errorCells: [] }
}

const invalidated = (t: ParamTableSpec): ParamTableSpec => ({
  ...t,
  results: [],
  stats: null,
  checkResult: null,
  checkMessage: '',
})

/** Drops the formula overlay entry for an edited cell (an edit replaces the
 * legacy formula with the typed literal — explicit, never silent: the grid
 * marks formula cells and the hint line says editing does this). */
function withoutFormula(
  formulas: Record<string, string> | undefined,
  ref: string,
): Record<string, string> | undefined {
  if (!formulas || !(ref in formulas)) return formulas
  const next = { ...formulas }
  delete next[ref]
  return Object.keys(next).length > 0 ? next : undefined
}

/** One committed cell edit. Enforces every editability rule again at the
 * mapper level (defense in depth, exactly like the old sheetEditsToSpec):
 * code tables and non-editable cells return the spec unchanged. */
export function applyCellEdit(
  spec: TableSpec,
  gridRow: number,
  col: number,
  rawIn: string,
  snapshot: TableSpec = spec,
  invalidate = true,
): EditResult {
  if (spec.source === 'code') return unchanged(spec)
  const view = cellViewAt(spec, gridRow, col)
  if (!view.editable) return unchanged(spec)

  const errorCells: string[] = []
  let raw = rawIn
  if (isErrorValue(raw)) {
    errorCells.push(formulaRefFor(spec, gridRow, col))
    raw = ''
  }

  if (spec.kind === 'function') {
    if (gridRow === 0) {
      const j = col - 1
      if ((spec.columns[j] ?? '') === raw) return unchanged(spec)
      const columns = spec.columns.map((c, idx) => (idx === j ? raw : c))
      return { spec: { ...spec, columns }, changed: true, errorCells }
    }
    const i = gridRow - 1
    if (i < 0 || i >= spec.rows.length) return unchanged(spec)
    const ref = formulaRefFor(spec, gridRow, col)
    const current = col === 0 ? spec.rows[i].x : spec.rows[i].ys[col - 1] ?? ''
    if (current === raw && errorCells.length === 0) return unchanged(spec)
    const rows = spec.rows.map((r, idx) => {
      if (idx !== i) return r
      if (col === 0) return { ...r, x: raw }
      const ys = [...r.ys]
      ys[col - 1] = raw
      return { ...r, ys }
    })
    return {
      spec: { ...spec, rows, formulas: withoutFormula(spec.formulas, ref) },
      changed: true,
      errorCells,
    }
  }

  // Parametric input cell.
  const i = gridRow
  if (i < 0 || i >= spec.rows.length) return unchanged(spec)
  const name = spec.vars[col - 1]
  if (name === undefined) return unchanged(spec)
  const computed = snapshot.kind === 'parametric' ? paramComputedValue(snapshot, i, name) : undefined
  // A cell still showing the solver's value is not an input (committing the
  // displayed computed text unchanged must not become an override).
  const isUntouchedComputed =
    computed !== undefined && raw.trim() !== '' && Number(raw) === computed
  const draft = isUntouchedComputed ? '' : raw
  const prevDraft = spec.rows[i].values[name] ?? ''
  const ref = formulaRefFor(spec, gridRow, col)
  if (draft === prevDraft && errorCells.length === 0) return unchanged(spec)
  const rows = spec.rows.map((r, idx) =>
    idx === i ? { ...r, values: { ...r.values, [name]: draft } } : r,
  )
  return {
    spec: { ...spec, rows, formulas: withoutFormula(spec.formulas, ref), ...(invalidate ? { results: [], stats: null, checkResult: null, checkMessage: '' } : {}) },
    changed: true,
    errorCells,
  }
}

/** Fill gestures preserve untouched computed cells; explicit paste freezes supplied values. */
export function applyCellEdits(spec: TableSpec, edits: { gridRow: number; col: number; text: string }[]): EditResult {
  let next = spec
  const errorCells: string[] = []
  for (const edit of edits) {
    const result = applyCellEdit(next, edit.gridRow, edit.col, edit.text, spec, false)
    next = result.spec
    errorCells.push(...result.errorCells)
  }
  return { spec: next !== spec && next.kind === 'parametric' ? invalidated(next) : next, changed: next !== spec, errorCells }
}

/** Restore only fields changed by the gesture; never restore solver output. */
export function restoreUserEdit(current: TableSpec, from: TableSpec, to: TableSpec): TableSpec {
  const restored = { ...current }
  for (const key of Object.keys(to) as (keyof TableSpec)[]) {
    if (['results', 'stats', 'checkResult', 'checkMessage'].includes(key)) continue
    if (from[key] !== to[key]) Object.assign(restored, { [key]: to[key] })
  }
  return restored.kind === 'parametric' ? invalidated(restored) : restored
}

/** Blanks a set of data cells (Delete over a selection). Trailing parametric
 * rows whose inputs just transitioned from filled to fully blank are dropped
 * — the user deleted those runs by hand. Rows that were ALREADY blank
 * (starter rows, Add Row) are kept: blank runs are legitimate, and Add Row
 * must survive unrelated edits. The same trailing rule applies to function
 * tables. Header/Run/out-of-range cells are ignored. */
export function clearCells(
  spec: TableSpec,
  cells: readonly { gridRow: number; col: number }[],
): EditResult {
  if (spec.source === 'code') return unchanged(spec)

  let changed = false
  if (spec.kind === 'function') {
    const rows = spec.rows.map((r) => ({ ...r, ys: [...r.ys] }))
    let columns = [...spec.columns]
    let formulas = spec.formulas
    for (const { gridRow, col } of cells) {
      if (gridRow === 0) {
        // 2-D curve-parameter header cells are editable and clearable; the
        // argName cell and the 1-D 'y' label are schema (toolbar-owned).
        if (!spec.is1D && col >= 1 && col < boundColumnCount(spec) && (columns[col - 1] ?? '') !== '') {
          columns = columns.map((c, idx) => (idx === col - 1 ? '' : c))
          changed = true
        }
        continue
      }
      const i = gridRow - 1
      if (i < 0 || i >= rows.length || col < 0 || col >= boundColumnCount(spec)) continue
      const current = col === 0 ? rows[i].x : rows[i].ys[col - 1] ?? ''
      if (current === '') continue
      if (col === 0) rows[i].x = ''
      else rows[i].ys[col - 1] = ''
      formulas = withoutFormula(formulas, formulaRefFor(spec, gridRow, col))
      changed = true
    }
    if (!changed) return unchanged(spec)
    const isBlank = (r: { x: string; ys: string[] }) =>
      r.x.trim() === '' && r.ys.every((y) => (y ?? '').trim() === '')
    while (rows.length > 0) {
      const i = rows.length - 1
      const prevRow = spec.rows[i]
      if (!prevRow || isBlank(prevRow) || !isBlank(rows[i])) break
      rows.pop()
    }
    return { spec: { ...spec, rows, columns, formulas }, changed: true, errorCells: [] }
  }

  const rows: ParamRow[] = spec.rows.map((r) => ({ ...r, values: { ...r.values } }))
  let formulas = spec.formulas
  for (const { gridRow, col } of cells) {
    const i = gridRow
    if (i < 0 || i >= rows.length || col < 1 || col >= boundColumnCount(spec)) continue
    const name = spec.vars[col - 1]
    if ((rows[i].values[name] ?? '') === '') continue
    rows[i].values[name] = ''
    formulas = withoutFormula(formulas, formulaRefFor(spec, gridRow, col))
    changed = true
  }
  if (!changed) return unchanged(spec)
  const isBlank = (values: Record<string, string>) =>
    spec.vars.every((name) => (values[name] ?? '').trim() === '')
  while (rows.length > 0) {
    const i = rows.length - 1
    const prevRow = spec.rows[i]
    if (!prevRow || isBlank(prevRow.values) || !isBlank(rows[i].values)) break
    rows.pop()
  }
  return { spec: invalidated({ ...spec, rows, formulas }), changed: true, errorCells: [] }
}

// ---------------------------------------------------------------------------
// Paste (multi-cell, Excel-style TSV — glide hands us the parsed matrix)

export interface PasteResult {
  spec: TableSpec
  changed: boolean
  /** Rows past TABLE_MAX_ROWS were dropped (toast the user). */
  truncated: boolean
  /** Content fell beyond the bound columns and was not applied. */
  outOfRegion: boolean
  /** Content targeted the Run column / read-only header cells (skipped). */
  intoReadOnly: boolean
  /** A1 refs whose pasted value was a formula-error literal (stored blank). */
  errorCells: string[]
}

/** Applies a pasted matrix anchored at (startGridRow, startCol). The region
 * grows downward (new rows) up to the cap; content beyond the bound columns
 * is clipped and flagged — columns are schema and never grow from a paste
 * (contract a, kept). Read-only cells (Run column, function header labels)
 * are skipped and flagged. */
export function applyPaste(
  spec: TableSpec,
  startGridRow: number,
  startCol: number,
  matrix: readonly (readonly string[])[],
): PasteResult {
  const none: PasteResult = {
    spec,
    changed: false,
    truncated: false,
    outOfRegion: false,
    intoReadOnly: false,
    errorCells: [],
  }
  if (spec.source === 'code' || matrix.length === 0) return none

  const cols = boundColumnCount(spec)
  const errorCells: string[] = []
  let outOfRegion = false
  let truncated = false
  let intoReadOnly = false
  let changed = false

  const sanitize = (raw: string, gridRow: number, col: number): string => {
    if (isErrorValue(raw)) {
      errorCells.push(formulaRefFor(spec, gridRow, col))
      return ''
    }
    return raw
  }

  if (spec.kind === 'function') {
    const rows = spec.rows.map((r) => ({ ...r, ys: [...r.ys] }))
    let columns = [...spec.columns]
    let formulas = spec.formulas
    // Maximum grid row a paste may reach: data rows are capped.
    const maxGridRow = TABLE_MAX_ROWS // data row TABLE_MAX_ROWS lives at grid row TABLE_MAX_ROWS
    for (let dr = 0; dr < matrix.length; dr++) {
      const gridRow = startGridRow + dr
      if (gridRow > maxGridRow) {
        truncated = true
        break
      }
      for (let dc = 0; dc < matrix[dr].length; dc++) {
        const col = startCol + dc
        if (col >= cols) {
          outOfRegion = true
          continue
        }
        const raw = sanitize(String(matrix[dr][dc] ?? ''), gridRow, col)
        if (gridRow === 0) {
          if (col === 0 || spec.is1D) {
            intoReadOnly = true
            continue
          }
          if ((columns[col - 1] ?? '') !== raw) {
            columns = columns.map((c, idx) => (idx === col - 1 ? raw : c))
            changed = true
          }
          continue
        }
        const i = gridRow - 1
        while (rows.length <= i) {
          rows.push({ x: '', ys: columns.map(() => '') })
          changed = true
        }
        const current = col === 0 ? rows[i].x : rows[i].ys[col - 1] ?? ''
        if (current !== raw) {
          if (col === 0) rows[i].x = raw
          else rows[i].ys[col - 1] = raw
          formulas = withoutFormula(formulas, formulaRefFor(spec, gridRow, col))
          changed = true
        }
      }
    }
    if (!changed) return { ...none, truncated, outOfRegion, intoReadOnly, errorCells }
    return {
      spec: { ...spec, rows, columns, formulas },
      changed,
      truncated,
      outOfRegion,
      intoReadOnly,
      errorCells,
    }
  }

  // Parametric.
  const rows: ParamRow[] = spec.rows.map((r) => ({ ...r, values: { ...r.values } }))
  let formulas = spec.formulas
  for (let dr = 0; dr < matrix.length; dr++) {
    const gridRow = startGridRow + dr
    if (gridRow >= TABLE_MAX_ROWS) {
      truncated = true
      break
    }
    for (let dc = 0; dc < matrix[dr].length; dc++) {
      const col = startCol + dc
      if (col === 0) {
        intoReadOnly = true
        continue
      }
      if (col >= cols) {
        outOfRegion = true
        continue
      }
      const name = spec.vars[col - 1]
      const raw = sanitize(String(matrix[dr][dc] ?? ''), gridRow, col)
      while (rows.length <= gridRow) {
        rows.push(newParamRow())
        changed = true
      }
      if ((rows[gridRow].values[name] ?? '') !== raw) {
        rows[gridRow].values[name] = raw
        formulas = withoutFormula(formulas, formulaRefFor(spec, gridRow, col))
        changed = true
      }
    }
  }
  if (!changed) return { ...none, truncated, outOfRegion, intoReadOnly, errorCells }
  return {
    spec: invalidated({ ...spec, rows, formulas }),
    changed,
    truncated,
    outOfRegion,
    intoReadOnly,
    errorCells,
  }
}

// ---------------------------------------------------------------------------
// Toolbar / dialog applications

/** Appends one blank row (the grid's trailing "add" row and the Add Row
 * button). Parametric row counts are part of the run set: results invalidate. */
export function appendRow(spec: TableSpec): TableSpec {
  if (spec.source === 'code' || spec.rows.length >= TABLE_MAX_ROWS) return spec
  if (spec.kind === 'function') {
    return { ...spec, rows: [...spec.rows, { x: '', ys: spec.columns.map(() => '') }] }
  }
  return invalidated({ ...spec, rows: [...spec.rows, newParamRow()] })
}

/** Drops the last row (the Remove Row button; disabled at 1 row by the UI).
 * Parametric row counts are part of the run set: results invalidate. */
export function removeLastRow(spec: TableSpec): TableSpec {
  if (spec.source === 'code' || spec.rows.length <= 1) return spec
  if (spec.kind === 'function') return { ...spec, rows: spec.rows.slice(0, -1) }
  return invalidated({ ...spec, rows: spec.rows.slice(0, -1) })
}

/** Fill Column (the AlterValuesModal application): writes one value per row
 * into the named column and invalidates the runs. */
export function applyColumnFill(
  spec: ParamTableSpec,
  varName: string,
  values: number[],
): ParamTableSpec {
  return invalidated({
    ...spec,
    rows: spec.rows.map((row, i) => ({
      ...row,
      values: { ...row.values, [varName]: String(values[i] ?? '') },
    })),
  })
}

// ---------------------------------------------------------------------------
// CSV export (what the grid shows: headers + merged computed values)

export function csvValuesFor(spec: TableSpec): string[][] {
  const out: string[][] = [headerTitles(spec)]
  if (spec.kind === 'function') {
    for (const row of spec.rows) out.push([row.x, ...spec.columns.map((_, j) => row.ys[j] ?? '')])
    return out
  }
  spec.rows.forEach((_, i) => {
    out.push(
      Array.from({ length: boundColumnCount(spec) }, (_, c) => cellViewAt(spec, i, c).text),
    )
  })
  return out
}
