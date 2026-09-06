import { requireValidTable } from '../tableValidation'
// tablesGrid/composeTables.ts
//
// Pure conversions behind the Wave-H composition features (D10 phase 4):
//   1. Sweep → Function      — parametric-table columns → a FunctionTableSpec
//   2. Digitizer → Fit → Fn  — a sampled fitted curve → a FunctionTableSpec
//   3. CSV → Function        — two columns of an imported .csv → a
//                              FunctionTableSpec (D11 moved this route out of
//                              the Data Analyzer and into ImportCsvModal; the
//                              conversion below is unchanged)
//
// Everything here is data in / data out (no React, no store access) so every
// edge — failed-row skipping, family-parameter mapping, decimation, name
// validation, replace-vs-new — is unit-testable.
//
// The D10 merge-direction rule applies to every spec these functions produce:
// on a name collision the DOCUMENT `TABLE` block wins on the solve/check path
// (`EquationSystemSolver.withExtraDefs`: source definitions win). A GUI table
// never overrides a same-named document table, and the dialogs that host
// these conversions must say so rather than silently promise an override
// (`checkFunctionName` flags the collision as `shadowedByCode`).

import { FunctionTableSpec, identifier, newTableId, ParamTableSpec, TableSpec } from '../tables'
import { TABLE_MAX_ROWS } from './tableGridModel'

// ---------------------------------------------------------------------------
// Cell resolution (paramComputedValue semantics)

/** A resolved parametric cell: the numeric value plus the text to store in
 * the produced function table (the typed draft verbatim, or the solved value
 * at full precision — never a lossy re-format of user input). */
export interface CellEntry {
  num: number
  text: string
}

/** Resolves one parametric cell the way the grid paints it (same precedence
 * as `readOnlyCellText`/`paramComputedValue`): a non-blank input draft wins;
 * otherwise the solved value of a successful run. `null` = unusable (blank,
 * non-numeric, or the row failed). */
export function paramCellEntry(
  t: ParamTableSpec,
  rowIndex: number,
  varName: string,
): CellEntry | null {
  const draft = (t.rows[rowIndex]?.values[varName] ?? '').trim()
  if (draft !== '') {
    const n = Number(draft)
    return Number.isFinite(n) ? { num: n, text: draft } : null
  }
  const res = t.results[rowIndex]
  if (res?.success) {
    const v = res.values[varName]
    if (typeof v === 'number' && Number.isFinite(v)) return { num: v, text: String(v) }
  }
  return null
}

// ---------------------------------------------------------------------------
// Decimation

/** `target` indices uniformly spread over `0..n-1`, always keeping the first
 * and last. For `n <= target` it is the identity. Strictly increasing (the
 * stride is > 1 whenever decimation actually happens). */
export function decimationIndices(n: number, target: number): number[] {
  if (n <= target) return Array.from({ length: n }, (_, i) => i)
  if (target <= 1) return n > 0 ? [0] : []
  const step = (n - 1) / (target - 1)
  return Array.from({ length: target }, (_, i) => Math.round(i * step))
}

// ---------------------------------------------------------------------------
// Sweep → Function

export interface SweepFunctionInput {
  table: ParamTableSpec
  /** Column providing the lookup argument. */
  xVar: string
  /** Column providing the function values. */
  yVar: string
  /** Optional family-parameter column: its distinct values become the curve
   * columns of a 2-D function `name(x, param)`. */
  familyVar?: string | null
  /** The function name (validate with `checkFunctionName` first). */
  name: string
}

export interface ComposeResult {
  spec: FunctionTableSpec
  /** Rows that contributed a point (or a curve cell). */
  usedRows: number
  /** Rows dropped: run failed, or an involved cell blank/non-numeric. */
  skippedRows: number
  /** True when the x grid exceeded TABLE_MAX_ROWS and was thinned uniformly. */
  decimated: boolean
}

interface BuiltRows {
  rows: { x: string; ys: string[] }[]
  decimated: boolean
}

/** Sorts row entries by numeric x and enforces the row cap. */
function finishRows(entries: { xNum: number; x: string; ys: string[] }[]): BuiltRows {
  entries.sort((a, b) => a.xNum - b.xNum)
  const idx = decimationIndices(entries.length, TABLE_MAX_ROWS)
  return {
    rows: idx.map((i) => ({ x: entries[i].x, ys: entries[i].ys })),
    decimated: idx.length < entries.length,
  }
}

/**
 * Builds an editable GUI FunctionTableSpec from parametric-table columns.
 * Cell precedence is `paramCellEntry` (input draft, else solved value);
 * rows missing any involved value are skipped and counted. Duplicate x
 * values keep the first-seen row (a function table needs one y per x).
 */
export function functionSpecFromParamColumns(input: SweepFunctionInput): ComposeResult {
  const { table, xVar, yVar, name } = input
  requireValidTable(table, false)
  const familyVar = input.familyVar ?? null
  let used = 0
  let skipped = 0

  if (familyVar === null) {
    const byX = new Map<number, { xNum: number; x: string; ys: string[] }>()
    for (let i = 0; i < table.rows.length; i++) {
      const x = paramCellEntry(table, i, xVar)
      const y = paramCellEntry(table, i, yVar)
      if (x === null || y === null) {
        skipped++
        continue
      }
      if (!byX.has(x.num)) byX.set(x.num, { xNum: x.num, x: x.text, ys: [y.text] })
      used++
    }
    const { rows, decimated } = finishRows([...byX.values()])
    return {
      spec: {
        id: newTableId(),
        kind: 'function',
        name,
        argName: identifier(xVar, 'x'),
        paramName: '',
        xLog: false,
        yLog: false,
        columns: [''],
        rows,
        is1D: true,
        source: 'gui',
      },
      usedRows: used,
      skippedRows: skipped,
      decimated,
    }
  }

  // 2-D family: distinct family values (ascending) become the curve columns.
  const famTexts = new Map<number, string>()
  const byX = new Map<number, { xNum: number; x: string; ys: Map<number, string> }>()
  for (let i = 0; i < table.rows.length; i++) {
    const x = paramCellEntry(table, i, xVar)
    const y = paramCellEntry(table, i, yVar)
    const f = paramCellEntry(table, i, familyVar)
    if (x === null || y === null || f === null) {
      skipped++
      continue
    }
    if (!famTexts.has(f.num)) famTexts.set(f.num, f.text)
    let row = byX.get(x.num)
    if (row === undefined) {
      row = { xNum: x.num, x: x.text, ys: new Map() }
      byX.set(x.num, row)
    }
    if (!row.ys.has(f.num)) row.ys.set(f.num, y.text)
    used++
  }
  const famNums = [...famTexts.keys()].sort((a, b) => a - b)
  const { rows, decimated } = finishRows(
    [...byX.values()].map((r) => ({
      xNum: r.xNum,
      x: r.x,
      ys: famNums.map((fn) => r.ys.get(fn) ?? ''),
    })),
  )
  return {
    spec: {
      id: newTableId(),
      kind: 'function',
      name,
      argName: identifier(xVar, 'x'),
      paramName: identifier(familyVar, 'param'),
      xLog: false,
      yLog: false,
      columns: famNums.map((fn) => famTexts.get(fn) as string),
      rows,
      is1D: false,
      source: 'gui',
    },
    usedRows: used,
    skippedRows: skipped,
    decimated,
  }
}

// ---------------------------------------------------------------------------
// Numeric series → Function (CSV channels, sampled fitted curves)

export interface SeriesFunctionInput {
  name: string
  argName: string
  xs: ArrayLike<number>
  ys: ArrayLike<number>
  xLog?: boolean
  yLog?: boolean
  /** Row cap (default TABLE_MAX_ROWS); longer series decimate uniformly. */
  maxRows?: number
}

/**
 * Builds a 1-D GUI FunctionTableSpec from paired numeric series. Non-finite
 * pairs are skipped, points sort ascending by x, exact-duplicate x values
 * keep the first point, and series past the row cap are decimated uniformly
 * (first and last point always kept). Values are stored at full precision.
 */
export function functionSpecFromXY(input: SeriesFunctionInput): ComposeResult {
  const maxRows = input.maxRows ?? TABLE_MAX_ROWS
  const n = Math.min(input.xs.length, input.ys.length)
  const pairs: { x: number; y: number }[] = []
  let skipped = 0
  for (let i = 0; i < n; i++) {
    const x = Number(input.xs[i])
    const y = Number(input.ys[i])
    if (!Number.isFinite(x) || !Number.isFinite(y)) {
      skipped++
      continue
    }
    pairs.push({ x, y })
  }
  pairs.sort((a, b) => a.x - b.x)
  const dedup: { x: number; y: number }[] = []
  for (const p of pairs) {
    if (dedup.length > 0 && dedup[dedup.length - 1].x === p.x) continue
    dedup.push(p)
  }
  const idx = decimationIndices(dedup.length, maxRows)
  const chosen = idx.map((i) => dedup[i])
  return {
    spec: {
      id: newTableId(),
      kind: 'function',
      name: input.name,
      argName: input.argName,
      paramName: '',
      xLog: input.xLog ?? false,
      yLog: input.yLog ?? false,
      columns: [''],
      rows: chosen.map((p) => ({ x: String(p.x), ys: [String(p.y)] })),
      is1D: true,
      source: 'gui',
    },
    usedRows: chosen.length,
    skippedRows: skipped,
    decimated: chosen.length < dedup.length,
  }
}

// ---------------------------------------------------------------------------
// Name validation & replace-vs-new

export interface NameCheck {
  /** The name is a usable identifier (conflicts are reported, not fatal). */
  ok: boolean
  error: string | null
  /** A same-named GUI function table exists — creating means replacing it
   * (the dialogs must ask, never overwrite silently). */
  replacesGui: boolean
  /** A same-named code TABLE block exists — the DOCUMENT definition takes
   * precedence on the solve path (D10 rule); surface the hint. */
  shadowedByCode: boolean
}

const IDENTIFIER = /^[A-Za-z]\w*$/

/** Validates a function-table name and reports collisions (case-insensitive,
 * matching the engine's case-insensitive name space). */
export function checkFunctionName(tables: readonly TableSpec[], rawName: string): NameCheck {
  const name = rawName.trim()
  if (!IDENTIFIER.test(name)) {
    return {
      ok: false,
      error:
        name === ''
          ? 'A function name is required.'
          : 'Not a valid identifier — use a letter followed by letters, digits or _.',
      replacesGui: false,
      shadowedByCode: false,
    }
  }
  const lower = name.toLowerCase()
  const hit = (source: 'gui' | 'code') =>
    tables.some(
      (t) =>
        t.kind === 'function' &&
        (source === 'code' ? t.source === 'code' : t.source !== 'code') &&
        t.name.trim().toLowerCase() === lower,
    )
  return { ok: true, error: null, replacesGui: hit('gui'), shadowedByCode: hit('code') }
}

/**
 * Adds the produced specs to the table list. A same-named GUI function table
 * is replaced IN PLACE, keeping its id (window identity, active-table id and
 * saved layouts stay stable); otherwise the spec is appended. Code tables are
 * never touched — the document definition simply keeps winning in the solver
 * (D10 merge direction). Returns the applied ids (replaced specs adopt the
 * replaced table's id).
 */
export function applyFunctionSpecs(
  tables: readonly TableSpec[],
  specs: readonly FunctionTableSpec[],
): { tables: TableSpec[]; ids: string[] } {
  const next: TableSpec[] = [...tables]
  const ids: string[] = []
  for (const spec of specs) {
    const lower = spec.name.trim().toLowerCase()
    const at = next.findIndex(
      (t) =>
        t.kind === 'function' && t.source !== 'code' && t.name.trim().toLowerCase() === lower,
    )
    if (at >= 0) {
      const replaced = { ...spec, id: next[at].id }
      next[at] = replaced
      ids.push(replaced.id)
    } else {
      next.push(spec)
      ids.push(spec.id)
    }
  }
  return { tables: next, ids }
}
