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

import {
  FunctionConversion,
  FunctionTableSpec,
  identifier,
  newTableId,
  ParamTableSpec,
  TableSpec,
} from '../tables'
import { TABLE_MAX_ROWS } from './tableGridModel'

/** How to stay inside the 5,000-row function-table cap. */
export type ReductionChoice = 'decimate' | 'trim'

/** Which parametric cells become lookup points. */
export type ParamValueSource = 'mixed' | 'raw' | 'solved'

// ---------------------------------------------------------------------------
// Cell resolution (paramComputedValue semantics)

/** A resolved parametric cell: the numeric value plus the text to store in
 * the produced function table (the typed draft verbatim, or the solved value
 * at full precision — never a lossy re-format of user input). */
export interface CellEntry {
  num: number
  text: string
}

/** Resolves one parametric cell. `mixed` is grid paint order (typed draft,
 * else a successful solve). `raw` uses only typed drafts; `solved` uses only
 * successful row results. `null` = unusable for that policy. */
export function paramCellEntry(
  t: ParamTableSpec,
  rowIndex: number,
  varName: string,
  source: ParamValueSource = 'mixed',
): CellEntry | null {
  const draft = (t.rows[rowIndex]?.values[varName] ?? '').trim()
  if (source !== 'solved' && draft !== '') {
    const n = Number(draft)
    return Number.isFinite(n) ? { num: n, text: draft } : null
  }
  if (source === 'raw') return null
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
  valueSource?: ParamValueSource
  reduction?: ReductionChoice
  xMin?: number
  xMax?: number
  maxRows?: number
  argUnit?: string
  outputUnit?: string
  paramUnit?: string
}

export interface ComposeCounts {
  /** Pairs or source rows examined. */
  sourceCount: number
  /** Non-finite / failed / incomplete pairs. */
  invalidCount: number
  /** Exact-duplicate X (or X+family) pairs dropped after the first. */
  duplicateCount: number
  /** Distinct X values after invalid/duplicate removal, before trim/thin. */
  uniqueCount: number
  /** Distinct X dropped by an explicit range trim. */
  trimmedCount: number
  /** Distinct X dropped by uniform thinning. */
  reducedCount: number
  /** Distinct X kept in `spec.rows`. */
  retainedCount: number
  /** Unique X still exceed the cap and no accepted reduction fits. */
  needsReduction: boolean
  /** Largest-|y| point uniform thinning would drop; null when nothing is dropped. */
  droppedPeak: { x: number; y: number } | null
}

export interface ComposeResult extends ComposeCounts {
  spec: FunctionTableSpec
  /** Final retained points (`spec.rows.length`). */
  usedRows: number
  /** Alias of `invalidCount` (failed or non-numeric source pairs). */
  skippedRows: number
  /** True when uniform thinning produced the retained rows. */
  decimated: boolean
}

function conversionOf(
  counts: ComposeCounts,
  extra: Pick<FunctionConversion, 'valueSource' | 'reduction' | 'xMin' | 'xMax'> = {},
): FunctionConversion {
  return {
    ...extra,
    sourceCount: counts.sourceCount,
    invalidCount: counts.invalidCount,
    duplicateCount: counts.duplicateCount,
    uniqueCount: counts.uniqueCount,
    trimmedCount: counts.trimmedCount,
    reducedCount: counts.reducedCount,
    retainedCount: counts.retainedCount,
  }
}

function droppedPeakOf<T>(
  rows: T[],
  keptIdx: number[],
  xOf: (row: T) => number,
  yOf: (row: T) => number,
): { x: number; y: number } | null {
  const kept = new Set(keptIdx)
  let peak: { x: number; y: number } | null = null
  for (let i = 0; i < rows.length; i++) {
    if (kept.has(i)) continue
    const y = yOf(rows[i])
    if (!Number.isFinite(y)) continue
    if (peak === null || Math.abs(y) > Math.abs(peak.y)) peak = { x: xOf(rows[i]), y }
  }
  return peak
}

function reduceUnique<T>(
  unique: T[],
  opts: {
    maxRows: number
    reduction?: ReductionChoice
    xMin?: number
    xMax?: number
    xOf: (row: T) => number
    yOf: (row: T) => number
  },
): {
  kept: T[]
  trimmedCount: number
  reducedCount: number
  needsReduction: boolean
  droppedPeak: { x: number; y: number } | null
} {
  const inRange = unique.filter((row) => {
    const x = opts.xOf(row)
    if (opts.xMin !== undefined && x < opts.xMin) return false
    if (opts.xMax !== undefined && x > opts.xMax) return false
    return true
  })
  const trimmedCount = unique.length - inRange.length
  const decimateIdx = decimationIndices(inRange.length, opts.maxRows)
  const droppedPeak = droppedPeakOf(inRange, decimateIdx, opts.xOf, opts.yOf)
  if (inRange.length <= opts.maxRows) {
    return { kept: inRange, trimmedCount, reducedCount: 0, needsReduction: false, droppedPeak: null }
  }
  if (opts.reduction === 'decimate') {
    return {
      kept: decimateIdx.map((i) => inRange[i]),
      trimmedCount,
      reducedCount: inRange.length - decimateIdx.length,
      needsReduction: false,
      droppedPeak,
    }
  }
  return { kept: [], trimmedCount, reducedCount: 0, needsReduction: true, droppedPeak }
}

function composeResult(
  spec: FunctionTableSpec,
  counts: ComposeCounts,
  extra: Pick<FunctionConversion, 'valueSource' | 'reduction' | 'xMin' | 'xMax'> = {},
): ComposeResult {
  spec.conversion = conversionOf(counts, extra)
  return {
    spec,
    ...counts,
    usedRows: counts.retainedCount,
    skippedRows: counts.invalidCount,
    decimated: extra.reduction === 'decimate' && counts.reducedCount > 0,
  }
}

/**
 * Builds an editable GUI FunctionTableSpec from parametric-table columns.
 * Cell policy is `valueSource` (default `mixed`). Duplicate x values keep the
 * first-seen row. Crossing the row cap does not thin until `reduction` is set.
 */
export function functionSpecFromParamColumns(input: SweepFunctionInput): ComposeResult {
  const { table, xVar, yVar, name } = input
  requireValidTable(table, false)
  const familyVar = input.familyVar ?? null
  const valueSource = input.valueSource ?? 'mixed'
  const maxRows = input.maxRows ?? TABLE_MAX_ROWS
  const cell = (i: number, v: string) => paramCellEntry(table, i, v, valueSource)
  const extra = {
    valueSource,
    reduction: input.reduction,
    xMin: input.xMin,
    xMax: input.xMax,
  }

  if (familyVar === null) {
    const byX = new Map<number, { xNum: number; x: string; ys: string[] }>()
    let duplicateCount = 0
    let invalidCount = 0
    for (let i = 0; i < table.rows.length; i++) {
      const x = cell(i, xVar)
      const y = cell(i, yVar)
      if (x === null || y === null) {
        invalidCount++
        continue
      }
      if (byX.has(x.num)) {
        duplicateCount++
        continue
      }
      byX.set(x.num, { xNum: x.num, x: x.text, ys: [y.text] })
    }
    const unique = [...byX.values()].sort((a, b) => a.xNum - b.xNum)
    const reduced = reduceUnique(unique, {
      maxRows,
      reduction: input.reduction,
      xMin: input.xMin,
      xMax: input.xMax,
      xOf: (r) => r.xNum,
      yOf: (r) => Number(r.ys[0]),
    })
    const counts: ComposeCounts = {
      sourceCount: table.rows.length,
      invalidCount,
      duplicateCount,
      uniqueCount: unique.length,
      trimmedCount: reduced.trimmedCount,
      reducedCount: reduced.reducedCount,
      retainedCount: reduced.kept.length,
      needsReduction: reduced.needsReduction,
      droppedPeak: reduced.droppedPeak,
    }
    return composeResult(
      {
        id: newTableId(),
        kind: 'function',
        name,
        argName: identifier(xVar, 'x'),
        paramName: '',
        xLog: false,
        yLog: false,
        columns: [''],
        rows: reduced.kept.map((r) => ({ x: r.x, ys: r.ys })),
        is1D: true,
        source: 'gui',
        argUnit: input.argUnit ?? table.columnUnits?.[xVar],
        outputUnit: input.outputUnit ?? table.columnUnits?.[yVar],
      },
      counts,
      extra,
    )
  }

  const famTexts = new Map<number, string>()
  const byX = new Map<number, { xNum: number; x: string; ys: Map<number, string> }>()
  let duplicateCount = 0
  let invalidCount = 0
  for (let i = 0; i < table.rows.length; i++) {
    const x = cell(i, xVar)
    const y = cell(i, yVar)
    const f = cell(i, familyVar)
    if (x === null || y === null || f === null) {
      invalidCount++
      continue
    }
    if (!famTexts.has(f.num)) famTexts.set(f.num, f.text)
    let row = byX.get(x.num)
    if (row === undefined) {
      row = { xNum: x.num, x: x.text, ys: new Map() }
      byX.set(x.num, row)
    }
    if (row.ys.has(f.num)) {
      duplicateCount++
      continue
    }
    row.ys.set(f.num, y.text)
  }
  const famNums = [...famTexts.keys()].sort((a, b) => a - b)
  const unique = [...byX.values()]
    .sort((a, b) => a.xNum - b.xNum)
    .map((r) => ({
      xNum: r.xNum,
      x: r.x,
      ys: famNums.map((fn) => r.ys.get(fn) ?? ''),
    }))
  const reduced = reduceUnique(unique, {
    maxRows,
    reduction: input.reduction,
    xMin: input.xMin,
    xMax: input.xMax,
    xOf: (r) => r.xNum,
    yOf: (r) => Number(r.ys.find((y) => y !== '') ?? Number.NaN),
  })
  const counts: ComposeCounts = {
    sourceCount: table.rows.length,
    invalidCount,
    duplicateCount,
    uniqueCount: unique.length,
    trimmedCount: reduced.trimmedCount,
    reducedCount: reduced.reducedCount,
    retainedCount: reduced.kept.length,
    needsReduction: reduced.needsReduction,
    droppedPeak: reduced.droppedPeak,
  }
  return composeResult(
    {
      id: newTableId(),
      kind: 'function',
      name,
      argName: identifier(xVar, 'x'),
      paramName: identifier(familyVar, 'param'),
      xLog: false,
      yLog: false,
      columns: famNums.map((fn) => famTexts.get(fn) as string),
      rows: reduced.kept.map((r) => ({ x: r.x, ys: r.ys })),
      is1D: false,
      source: 'gui',
      argUnit: input.argUnit ?? table.columnUnits?.[xVar],
      outputUnit: input.outputUnit ?? table.columnUnits?.[yVar],
      paramUnit: input.paramUnit ?? table.columnUnits?.[familyVar],
    },
    counts,
    extra,
  )
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
  maxRows?: number
  reduction?: ReductionChoice
  xMin?: number
  xMax?: number
  argUnit?: string
  outputUnit?: string
}

/**
 * Builds a 1-D GUI FunctionTableSpec from paired numeric series. Non-finite
 * pairs are skipped, points sort ascending by x, and exact-duplicate x values
 * keep the first point. Series past the row cap are not thinned until
 * `reduction` is `decimate` or a trim range fits. Values stay at full precision.
 */
export function functionSpecFromXY(input: SeriesFunctionInput): ComposeResult {
  const maxRows = input.maxRows ?? TABLE_MAX_ROWS
  const n = Math.min(input.xs.length, input.ys.length)
  const pairs: { x: number; y: number }[] = []
  let invalidCount = 0
  for (let i = 0; i < n; i++) {
    const x = Number(input.xs[i])
    const y = Number(input.ys[i])
    if (!Number.isFinite(x) || !Number.isFinite(y)) {
      invalidCount++
      continue
    }
    pairs.push({ x, y })
  }
  pairs.sort((a, b) => a.x - b.x)
  const unique: { x: number; y: number }[] = []
  let duplicateCount = 0
  for (const p of pairs) {
    if (unique.length > 0 && unique[unique.length - 1].x === p.x) {
      duplicateCount++
      continue
    }
    unique.push(p)
  }
  const reduced = reduceUnique(unique, {
    maxRows,
    reduction: input.reduction,
    xMin: input.xMin,
    xMax: input.xMax,
    xOf: (p) => p.x,
    yOf: (p) => p.y,
  })
  const counts: ComposeCounts = {
    sourceCount: n,
    invalidCount,
    duplicateCount,
    uniqueCount: unique.length,
    trimmedCount: reduced.trimmedCount,
    reducedCount: reduced.reducedCount,
    retainedCount: reduced.kept.length,
    needsReduction: reduced.needsReduction,
    droppedPeak: reduced.droppedPeak,
  }
  return composeResult(
    {
      id: newTableId(),
      kind: 'function',
      name: input.name,
      argName: input.argName,
      paramName: '',
      xLog: input.xLog ?? false,
      yLog: input.yLog ?? false,
      columns: [''],
      rows: reduced.kept.map((p) => ({ x: String(p.x), ys: [String(p.y)] })),
      is1D: true,
      source: 'gui',
      argUnit: input.argUnit,
      outputUnit: input.outputUnit,
    },
    counts,
    { reduction: input.reduction, xMin: input.xMin, xMax: input.xMax },
  )
}

export function formatComposeCounts(result: ComposeCounts): string {
  const parts = [
    `${result.sourceCount.toLocaleString()} source`,
    `${result.invalidCount.toLocaleString()} invalid`,
    `${result.duplicateCount.toLocaleString()} duplicate x`,
    `${result.uniqueCount.toLocaleString()} unique`,
  ]
  if (result.trimmedCount > 0) parts.push(`${result.trimmedCount.toLocaleString()} trimmed`)
  if (result.reducedCount > 0) parts.push(`${result.reducedCount.toLocaleString()} reduced`)
  if (!result.needsReduction) parts.push(`${result.retainedCount.toLocaleString()} retained`)
  return parts.join(' · ')
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
