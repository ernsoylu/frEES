import { CheckResponse, FunctionTableDto, OdeTableDto, ParametricTableDto, TableRowResult, TableStats } from './api'

/** One parametric-table run: a stable identity plus the cell drafts.
 * (Lived in ParametricTableTab until the Mantine grid was retired in favour
 * of the Univer Tables workbook — Phase 4 of the unification plan.) */
export interface ParamRow {
  id: string
  values: Record<string, string>
}

export function newParamRow(): ParamRow {
  return { id: crypto.randomUUID(), values: {} }
}

// ---------------------------------------------------------------------------
// Tables (Epic 8): the Tables window manages any number of
// Parametric Tables (run tables) and Function Tables (tabulated functions from
// the Graph Digitizer or entered manually). A Function Table's name is the
// function name callable in equations; its column names are the arguments —
// the first column is the lookup argument. If it has a curve family parameter,
// the rest are curves with the family parameter value in the header
// (e.g. T = 100, 200, ...).
// ---------------------------------------------------------------------------

export interface ParamTableSpec {
  id: string
  kind: 'parametric'
  name: string
  vars: string[]
  rows: ParamRow[]
  results: TableRowResult[]
  stats: TableStats | null
  checkResult: CheckResponse | null
  checkMessage: string
  source?: 'code' | 'gui'
  /** Set when this "parametric" table was produced from a DYNAMIC/ODE block
   * rather than a PARAMETRIC block, so the UI can describe it correctly. */
  origin?: 'ode'
  /** Sparse formula overlay (contract f) for input cells edited as a bound
   * sheet in the Tables workbook — same mechanics as FunctionTableSpec. */
  formulas?: Record<string, string>
  /** Per-column SI units (column name → unit), for read-only code/ODE tables
   * whose columns are not solved scalars: used for grid headers and plot axes. */
  columnUnits?: Record<string, string>
}

interface CurveRow {
  x: string
  ys: string[]
}

export interface FunctionTableSpec {
  id: string
  kind: 'function'
  name: string // function name callable in equations
  argName: string // first column: the lookup argument, e.g. Re
  paramName: string // family parameter name, e.g. T ('' for a lone curve)
  xLog: boolean
  yLog: boolean
  columns: string[] // family parameter value per curve column
  rows: CurveRow[]
  is1D?: boolean // if true, it represents a function without a curve family/parameter
  // 'code' tables are parsed from TABLE ... END blocks in the editor text and
  // are shown read-only (the editor text is their source of truth). Undefined
  // / 'gui' tables are created and edited in the Tables window.
  source?: 'code' | 'gui'
  /** Sparse formula overlay (contract f of the table-spreadsheet unification
   * plan): A1 cell ref → Univer formula string for data cells the user
   * entered as formulas in the bound sheet. Row cells keep the last computed
   * value (the solver wire path reads only those); this overlay is re-applied
   * as `f`-only cells at materialization so formulas survive reloads and
   * recompute. Optional — absent on tables never edited as sheets. */
  formulas?: Record<string, string>
}

export type TableSpec = ParamTableSpec | FunctionTableSpec

let tableCounter = 1

export function newTableId(): string {
  return `table-${Date.now()}-${tableCounter++}`
}

export function newParamTable(existing: TableSpec[]): ParamTableSpec {
  const count = existing.filter((t) => t.kind === 'parametric').length
  return {
    id: newTableId(),
    kind: 'parametric',
    name: `Parametric ${count + 1}`,
    vars: [],
    rows: [newParamRow(), newParamRow(), newParamRow()],
    results: [],
    stats: null,
    checkResult: null,
    checkMessage: '',
  }
}

export function newFunctionTable(existing: TableSpec[], is1D: boolean): FunctionTableSpec {
  const count = existing.filter((t) => t.kind === 'function').length
  return {
    id: newTableId(),
    kind: 'function',
    name: `func${count + 1}`,
    argName: 'x',
    paramName: '',
    xLog: false,
    yLog: false,
    columns: is1D ? [''] : ['100', '200'],
    rows: Array.from({ length: 5 }, () => ({ x: '', ys: is1D ? [''] : ['', ''] })),
    is1D,
  }
}

/** Function tables in solver wire format; blank cells are simply omitted.
 * Code-sourced tables are skipped — the backend parses them from the editor
 * text itself, so re-sending them would be redundant. */
export function toFunctionTableDtos(tables: TableSpec[]): FunctionTableDto[] {
  const dtos: FunctionTableDto[] = []
  for (const table of tables) {
    if (table.kind !== 'function' || table.name.trim() === '') continue
    if (table.source === 'code') continue
    const curves = table.columns.map((paramRaw, j) => {
      const points: number[][] = []
      for (const row of table.rows) {
        const x = Number(row.x)
        const y = Number(row.ys[j])
        if (row.x.trim() !== '' && (row.ys[j] ?? '').trim() !== '' && Number.isFinite(x) && Number.isFinite(y)) {
          points.push([x, y])
        }
      }
      const param = Number(paramRaw)
      return {
        param: paramRaw.trim() !== '' && Number.isFinite(param) ? param : null,
        points,
      }
    })
    if (curves.some((c) => c.points.length > 0)) {
      dtos.push({
        name: table.name.trim(),
        argNames: [table.argName, table.paramName].filter((a) => a.trim() !== ''),
        xLog: table.xLog,
        yLog: table.yLog,
        curves,
      })
    }
  }
  return dtos
}

function scaleVal(v: number, log: boolean): number {
  return log && v > 0 ? Math.log10(v) : v
}

function unscaleVal(v: number, log: boolean): number {
  return log ? Math.pow(10, v) : v
}

function fmtCell(v: number): string {
  return String(v)
}

/**
 * Fill missing: every blank cell whose row has an x inside a curve's
 * tabulated span is interpolated from that curve's known points (linear,
 * in log space for log axes). Cells outside the span stay blank — the
 * chart carries no information there.
 */
export function fillMissingCells(table: FunctionTableSpec): FunctionTableSpec {
  const rows = table.rows.map((r) => ({ x: r.x, ys: [...r.ys] }))
  for (let j = 0; j < table.columns.length; j++) {
    const known = collectKnownPoints(rows, j)
    if (known.length < 2) continue
    interpolateColumn(rows, j, known, table)
  }
  return { ...table, rows }
}

type FillRow = { x: string; ys: string[] }

/** The finite (x, y) samples present in column {@code j}, sorted ascending by x. */
function collectKnownPoints(rows: FillRow[], j: number): { x: number; y: number }[] {
  const known: { x: number; y: number }[] = []
  for (const row of rows) {
    const x = Number(row.x)
    const y = Number(row.ys[j])
    if (row.x.trim() !== '' && (row.ys[j] ?? '').trim() !== '' && Number.isFinite(x) && Number.isFinite(y)) {
      known.push({ x, y })
    }
  }
  known.sort((p, q) => p.x - q.x)
  return known
}

/** Fills blank cells in column {@code j} by (log-aware) linear interpolation
 *  between the bracketing known samples; out-of-range rows are left blank. */
function interpolateColumn(rows: FillRow[], j: number, known: { x: number; y: number }[], table: FunctionTableSpec): void {
  for (const row of rows) {
    const x = Number(row.x)
    if (row.x.trim() === '' || !Number.isFinite(x)) continue
    if ((row.ys[j] ?? '').trim() !== '') continue
    if (x < known[0].x || x > known[known.length - 1].x) continue
    let hi = 1
    while (known[hi].x < x) hi++
    const x0 = scaleVal(known[hi - 1].x, table.xLog)
    const x1 = scaleVal(known[hi].x, table.xLog)
    const y0 = scaleVal(known[hi - 1].y, table.yLog)
    const y1 = scaleVal(known[hi].y, table.yLog)
    const t = x1 === x0 ? 0 : (scaleVal(x, table.xLog) - x0) / (x1 - x0)
    row.ys[j] = fmtCell(unscaleVal(y0 + t * (y1 - y0), table.yLog))
  }
}

/** Rows sorted ascending by numeric x (blank x rows sink to the bottom). */
export function sortFunctionRows(table: FunctionTableSpec): FunctionTableSpec {
  const rows = [...table.rows].sort((a, b) => {
    const xa = a.x.trim() === '' ? Number.POSITIVE_INFINITY : Number(a.x)
    const xb = b.x.trim() === '' ? Number.POSITIVE_INFINITY : Number(b.x)
    return (Number.isFinite(xa) ? xa : Number.POSITIVE_INFINITY)
      - (Number.isFinite(xb) ? xb : Number.POSITIVE_INFINITY)
  })
  return { ...table, rows }
}

/** Shared 6-significant-figure formatter for table cells. Exported so the
 *  read-only grid formats solver-computed values exactly like the input cells
 *  it renders them alongside. */
export function fmt6(v: number): string {
  return Number.parseFloat(v.toPrecision(6)).toString()
}

/**
 * The string a read-only parametric/ODE grid should paint for one cell.
 *
 * <p>`rows` carries only what was supplied as INPUT, so a code PARAMETRIC table
 * painted from `rows` alone shows its sweep column and nothing else — every
 * solved column stays blank no matter how many runs succeeded. The solved
 * numbers arrive separately in `results` and are merged here.
 *
 * <p>Same rule as `paramComputedValue` in spreadsheet/tableBinding.ts: a
 * computed value shows only where the row solved AND the input was left blank,
 * so the solver's echo of an input never overwrites what the user typed. Lives
 * here, not in the grid component, so it is reachable from a unit test without
 * mounting a canvas — and so the two grids cannot drift apart silently.
 */
export function readOnlyCellText(
  row: ParamRow | undefined,
  outcome: TableRowResult | undefined,
  name: string,
): string {
  const draft = row?.values[name] ?? ''
  if (draft.trim() !== '') return draft
  const computed = outcome?.success ? outcome.values[name] : undefined
  return computed !== undefined && Number.isFinite(computed) ? String(computed) : draft
}

/** Sanitizes free text (axis labels, column headers) into an identifier:
 *  strips `[unit]` suffixes, collapses non-word runs to `_`, and falls back
 *  when the result does not start with a letter. Shared by the digitizer
 *  export and the D10 composition features (sweep→function, CSV→function). */
export function identifier(raw: string, fallback: string): string {
  const cleaned = raw.replace(/\[[^[\]]*\]/g, '').trim().replace(/\W+/g, '_').replace(/^_|_$/g, '')
  return /^[A-Za-z]/.test(cleaned) ? cleaned : fallback
}

/**
 * Builds a Function Table from digitized curves (Story 8.7 "Send to Function
 * Table"): the x grid is the union of every curve's x samples, each curve
 * fills its own rows, and the gaps left by differing samples are
 * interpolated right away via fillMissingCells.
 */
export function functionTableFromDigitizer(input: {
  existing: TableSpec[]
  xName: string
  yName: string
  xLog: boolean
  yLog: boolean
  curves: { param: string; points: { x: number; y: number }[] }[]
}): FunctionTableSpec {
  const count = input.existing.filter((t) => t.kind === 'function').length
  const rows = alignedCurveRows(input.curves.map((curve) => curve.points.map((p) => [p.x, p.y])))
  const is1D = input.curves.length <= 1
  const table: FunctionTableSpec = {
    id: newTableId(),
    kind: 'function',
    name: identifier(input.yName, '').toLowerCase() || `func${count + 1}`,
    argName: identifier(input.xName, 'x'),
    paramName: is1D ? '' : 'param',
    xLog: input.xLog,
    yLog: input.yLog,
    columns: input.curves.map((c) => c.param),
    rows,
    is1D,
  }
  return fillMissingCells(table)
}

/** Align exact numeric X values once per curve. Exact duplicates keep the first sample. */
function alignedCurveRows(curves: number[][][]): CurveRow[] {
  const indexes = curves.map((points) => {
    const byX = new Map<number, number>()
    for (const [x, y] of points) if (!byX.has(x)) byX.set(x, y)
    return byX
  })
  const xs = [...new Set(indexes.flatMap((index) => [...index.keys()]))].sort((a, b) => a - b)
  return xs.map((x) => ({
    x: String(x),
    ys: indexes.map((index) => index.has(x) ? String(index.get(x)) : ''),
  }))
}

/** Builds a read-only Function Table spec from a solver-wire DTO (a TABLE
 * block the backend parsed out of the editor text). The x grid is the union of
 * every curve's x samples; each curve fills its own rows. */
function functionTableFromDto(dto: FunctionTableDto): FunctionTableSpec {
  const is1D = dto.curves.length <= 1 && (dto.curves[0]?.param == null)
  const rows = alignedCurveRows(dto.curves.map((curve) => curve.points))
  return {
    // Stable id keyed by name so the table keeps its identity across solves.
    id: `code-${dto.name.toLowerCase()}`,
    kind: 'function',
    name: dto.name,
    argName: dto.argNames[0] ?? 'x',
    paramName: is1D ? '' : (dto.argNames[1] ?? 'param'),
    xLog: dto.xLog,
    yLog: dto.yLog,
    columns: dto.curves.map((c) => (c.param == null ? '' : String(c.param))),
    rows,
    is1D,
    source: 'code',
  }
}

/** Builds a read-only Parametric Table spec from a PARAMETRIC ... END block. */
export function paramTableFromDto(dto: ParametricTableDto): ParamTableSpec {
  const rows: ParamRow[] = dto.rows.map((row) => {
    const values: Record<string, string> = {}
    dto.vars.forEach((v, j) => {
      const cell = row[j]
      values[v] = cell == null ? '' : String(cell)
    })
    return { id: crypto.randomUUID(), values }
  })
  return {
    id: `code-param-${dto.name.toLowerCase()}`,
    kind: 'parametric',
    name: dto.name,
    vars: dto.vars,
    rows: rows.length > 0 ? rows : [newParamRow()],
    results: [],
    stats: null,
    checkResult: null,
    checkMessage: '',
    source: 'code',
  }
}

/** Maps a solved ODE Table (from a DYNAMIC block) onto a read-only parametric
 * table: the columns become the table variables and the sampled trajectory the
 * rows, so it renders in the Tables window beside Parametric Tables and is a
 * data source in the Plots window (x = time, y = a state/output) with no extra
 * plot code. The data is solver-populated, so the rows are display-only. */
function odeTableFromDto(dto: OdeTableDto): ParamTableSpec {
  const rows: ParamRow[] = dto.rows.map((row) => {
    const values: Record<string, string> = {}
    dto.vars.forEach((v, j) => {
      const cell = row[j]
      values[v] = cell == null ? '' : String(cell)
    })
    return { id: crypto.randomUUID(), values }
  })
  const columnUnits: Record<string, string> = {}
  dto.vars.forEach((v, j) => {
    const u = dto.units?.[j] ?? ''
    if (u) columnUnits[v] = u
  })
  return {
    id: `code-ode-${dto.name.toLowerCase()}`,
    kind: 'parametric',
    name: dto.name,
    vars: dto.vars,
    rows: rows.length > 0 ? rows : [newParamRow()],
    results: [],
    stats: null,
    checkResult: null,
    checkMessage: '',
    source: 'code',
    origin: 'ode',
    columnUnits,
  }
}

/** Replaces the code-sourced tables in the list with the freshly parsed set
 * from the latest solve/check, leaving GUI tables untouched. */
export function mergeCodeTables(
  existing: TableSpec[],
  functionDtos: FunctionTableDto[] | undefined,
  parametricDtos?: ParametricTableDto[] | undefined,
  odeDtos?: OdeTableDto[] | undefined,
): TableSpec[] {
  const guiTables = existing.filter((t) => t.source !== 'code')
  const codeFunctionTables = (functionDtos ?? []).map(functionTableFromDto)
  const codeParamTables = (parametricDtos ?? []).map(paramTableFromDto)
  const codeOdeTables = (odeDtos ?? []).map(odeTableFromDto)
  return [...guiTables, ...codeFunctionTables, ...codeParamTables, ...codeOdeTables]
}

/** Makes an independent, editable GUI copy of a code-defined table, decoupled
 * from the editor text. The copy is renamed to avoid clashing with the
 * code-defined original (which still wins in the solver by its text name). */
export function duplicateAsEditable(table: TableSpec): TableSpec {
  const name = `${table.name}_copy`
  if (table.kind === 'function') {
    return {
      ...table,
      id: newTableId(),
      name,
      rows: table.rows.map((r) => ({ x: r.x, ys: [...r.ys] })),
      columns: [...table.columns],
      source: 'gui',
    }
  }
  return {
    ...table,
    id: newTableId(),
    name,
    rows: table.rows.map((r) => ({ id: crypto.randomUUID(), values: { ...r.values } })),
    results: [],
    stats: null,
    checkResult: null,
    checkMessage: '',
    source: 'gui',
  }
}

const TABLES_KEY = 'frees.tables'

export function loadTables(): TableSpec[] {
  try {
    const raw = localStorage.getItem(TABLES_KEY)
    if (!raw) return []
    // Persisted JSON of unknown shape; migrated below and cast back to TableSpec
    // at this deserialization boundary.
    const tables = JSON.parse(raw) as Array<Record<string, unknown>>
    // Migrate 'curve' kind to 'function' kind, and run results/check state are transient.
    return tables.map((t) => {
      let mapped = t
      if (t.kind === 'curve') {
        mapped = { ...t, kind: 'function' }
      }
      return mapped.kind === 'parametric'
        ? { ...mapped, results: [], stats: null, checkResult: null, checkMessage: '' }
        : mapped
    }) as unknown as TableSpec[]
  } catch {
    return []
  }
}

export function saveTables(tables: TableSpec[]) {
  try {
    // Code-sourced tables are re-derived from the editor text on each solve;
    // never persist them (they would otherwise show stale before a re-solve).
    const slim = tables
      .filter((t) => t.source !== 'code')
      .map((t) =>
        t.kind === 'parametric'
          ? { ...t, results: [], stats: null, checkResult: null, checkMessage: '' }
          : t,
      )
    localStorage.setItem(TABLES_KEY, JSON.stringify(slim))
  } catch {
    // storage unavailable: tables still work for the session
  }
}
