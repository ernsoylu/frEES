import { useEffect, useMemo, useState } from 'react'
import {
  ActionIcon,
  Alert,
  Badge,
  Button,
  Group,
  Loader,
  Menu,
  Modal,
  Paper,
  ScrollArea,
  Table,
  Text,
  Tooltip,
} from '@mantine/core'
import {
  IconArrowRight,
  IconChartBar,
  IconChevronLeft,
  IconChevronRight,
  IconTable,
  IconX,
} from '@tabler/icons-react'
import type { PlotlyFigure } from 'plotly.js/lib/core'
import {
  DiagramResponse,
  PsychartResponse,
  StateTableDto,
  TableRowResult,
  VariableResult,
  getPropertyDiagram,
  getPsychrometricChart,
} from '../api'
import { ParamRow } from '../tables'
import { displayVar } from '../varDisplay'
import { PlotSpec } from './types'
import { StateTable, detectStateTables } from './stateTable'
import {
  PlotTheme,
  XYSeries,
  buildPropertyFigure,
  buildPsychroFigure,
  buildXYFigure,
  buildBodeFigure,
  buildNicholsFigure,
  buildNyquistFigure,
  buildPoleZeroFigure,
  buildRootLocusFigure,
} from './figure'
import { EXPORT_FORMATS, exportPlot } from './exportPlot'
import PlotlyChart, { type PlotPointClickEvent } from './PlotlyChart'

export interface PlotCursor {
  traceIndex: number
  traceName: string
  pointIndex: number
  x: number
  y: number
  sampleId?: string
}

export interface TraceStatistics {
  traceName: string
  count: number
  valid: number
  missing: number
  minX: number
  maxX: number
  minY: number
  maxY: number
  meanY: number
  sumY: number
}

export function formatPlotValue(v: number | undefined | null): string {
  if (v === undefined || v === null || !Number.isFinite(v)) return '—'
  if (Math.abs(v) >= 1e5 || (Math.abs(v) < 1e-3 && v !== 0)) {
    return v.toExponential(4)
  }
  return v.toLocaleString(undefined, { maximumFractionDigits: 4 })
}

export function computeTraceStats(trace: unknown): TraceStatistics | null {
  const t = trace as { name?: string; x?: unknown[]; y?: unknown[] } | undefined
  if (!t || !Array.isArray(t.x) || !Array.isArray(t.y)) return null
  const xs = t.x
  const ys = t.y
  const n = Math.max(xs.length, ys.length)
  let valid = 0
  let minX = Infinity
  let maxX = -Infinity
  let minY = Infinity
  let maxY = -Infinity
  let sumY = 0

  for (let i = 0; i < n; i++) {
    const x = xs[i]
    const y = ys[i]
    if (typeof x === 'number' && Number.isFinite(x) && typeof y === 'number' && Number.isFinite(y)) {
      valid++
      if (x < minX) minX = x
      if (x > maxX) maxX = x
      if (y < minY) minY = y
      if (y > maxY) maxY = y
      sumY += y
    }
  }

  return {
    traceName: t.name || 'Trace',
    count: n,
    valid,
    missing: n - valid,
    minX: valid > 0 ? minX : NaN,
    maxX: valid > 0 ? maxX : NaN,
    minY: valid > 0 ? minY : NaN,
    maxY: valid > 0 ? maxY : NaN,
    meanY: valid > 0 ? sumY / valid : NaN,
    sumY,
  }
}

interface Props {
  spec: PlotSpec
  states: StateTable
  cyclePath?: Record<string, number>[]
  tableRows: ParamRow[]
  tableResults: TableRowResult[]
  /** Flat solved variables; the data source for XY plots that reference solved
   * arrays (e.g. x = speed[1:N]) rather than a parametric table. */
  variables?: VariableResult[]
  /** Per-column SI units for the active read-only table (ODE/code), used to
   * unit-annotate axis labels when the columns are not solved scalars. */
  tableUnits?: Record<string, string>
  /** Declared STATE TABLE blocks, for overlaying a single circuit's states. */
  stateTableDefs?: StateTableDto[]
  onDuplicate?: () => void
  onConfigure: () => void
  onRemove: () => void
  leftSection?: React.ReactNode
  rightSection?: React.ReactNode
  hideHeader?: boolean
  exportTrigger?: { format: string; timestamp: number } | null
  onSelectRow?: (tableId: string | undefined, rowId: string) => void
}

/** Value of one variable in one run: solved value or the typed input. */
function runValue(
  row: ParamRow,
  result: TableRowResult | undefined,
  name: string,
): number | undefined {
  if (result && !result.success) return undefined
  const solved = result?.success ? result.values[name] : undefined
  if (solved !== undefined) return Number.isFinite(solved) ? solved : undefined
  const raw = (row.values[name] ?? '').trim()
  if (raw === '') return undefined
  const value = Number(raw)
  return Number.isFinite(value) ? value : undefined
}

function buildXYSeries(
  rows: ParamRow[],
  results: TableRowResult[],
  xVar: string,
  yVars: string[],
  zVar?: string | null,
  sizeVar?: string | null,
  axis: 'y' | 'y2' = 'y',
): XYSeries[] {
  return yVars.map((yVar) => {
    const x: number[] = []
    const y: number[] = []
    const z: number[] = []
    const size: number[] = []
    rows.forEach((row, i) => {
      const xValue = xVar ? runValue(row, results[i], xVar) : i
      const yValue = runValue(row, results[i], yVar)
      const zValue = zVar ? runValue(row, results[i], zVar) : undefined
      const sizeValue = sizeVar ? runValue(row, results[i], sizeVar) : undefined

      const hasX = xValue !== undefined
      const hasY = yValue !== undefined
      const hasZ = !zVar || zValue !== undefined
      const hasSize = !sizeVar || (sizeValue !== undefined && sizeValue >= 0)

      const valid = hasX && hasY && hasZ && hasSize
      x.push(valid ? xValue : Number.NaN)
      y.push(valid ? yValue : Number.NaN)
      if (zVar) z.push(valid ? zValue! : Number.NaN)
      if (sizeVar) size.push(valid ? sizeValue! : Number.NaN)
    })
    return {
      name: yVar,
      sampleIds: rows.map((row) => row.id),
      x,
      y,
      z: zVar ? z : undefined,
      size: sizeVar ? size : undefined,
      axis,
    }
  })
}

/** Collects the elements of a solved array variable (base[1], base[2], …) into
 * a dense, index-ordered list. Returns the value at each 1-based index. */
function arrayValues(variables: VariableResult[], base: string): Map<number, number> {
  const out = new Map<number, number>()
  const prefix = `${base.toLowerCase()}[`
  for (const v of variables) {
    const name = v.name.toLowerCase()
    if (!name.startsWith(prefix) || !name.endsWith(']')) continue
    const inner = v.name.substring(prefix.length, v.name.length - 1)
    const idx = Number(inner)
    if (Number.isInteger(idx)) out.set(idx, v.value)
  }
  return out
}

/** Builds XY series from solved array variables: x = base[i] vs each y base.
 * Points are emitted only for indices present in both the x and y arrays. */
function buildArrayXYSeries(
  variables: VariableResult[],
  xVar: string,
  yVars: string[],
  axis: 'y' | 'y2' = 'y',
  zVar?: string | null,
  sizeVar?: string | null,
): XYSeries[] {
  const channels = [...new Set([xVar, ...yVars, zVar, sizeVar].filter((name): name is string => !!name))]
  const arrays = channels.map((name) => arrayValues(variables, name))
  const indices = [...new Set(arrays.flatMap((array) => [...array.keys()]))].sort((a, b) => a - b)
  const rows: ParamRow[] = []
  indices.forEach((index, i) => {
    if (i > 0 && index > indices[i - 1] + 1) rows.push({ id: `gap-${index}`, values: {} })
    rows.push({ id: String(index), values: Object.fromEntries(channels.map((name, j) => [name, arrays[j].has(index) ? String(arrays[j].get(index)) : ''])) })
  })
  return buildXYSeries(rows, [], xVar, yVars, zVar, sizeVar, axis)
}

export function useDiagramData(spec: PlotSpec) {
  const [diagram, setDiagram] = useState<DiagramResponse | null>(null)
  const [psychart, setPsychart] = useState<PsychartResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const { kind } = spec
  const { fluid, diagram: diagramType } = spec.property
  const { pressureKPa, tMinC, tMaxC } = spec.psychro

  useEffect(() => {
    if (kind !== 'property' && kind !== 'psychro') return
    let cancelled = false
    setLoading(true)
    setError(null)
    const request =
      kind === 'property'
        ? getPropertyDiagram(fluid, diagramType).then((d) => {
            if (!cancelled) setDiagram(d)
          })
        : getPsychrometricChart(
            pressureKPa * 1000,
            tMinC + 273.15,
            tMaxC + 273.15,
          ).then((c) => {
            if (!cancelled) setPsychart(c)
          })
    request
      .catch((e: unknown) => {
        if (!cancelled) setError(String(e instanceof Error ? e.message : e))
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [kind, fluid, diagramType, pressureKPa, tMinC, tMaxC])

  return { diagram, psychart, loading, error }
}

export interface FigureInputs {
  states: StateTable
  cyclePath?: Record<string, number>[]
  tableRows: ParamRow[]
  tableResults: TableRowResult[]
  /** Flat solved variables, used as the XY data source when no parametric
   * table rows are available (plots referencing solved arrays). */
  variables?: VariableResult[]
  /** Per-column SI units for the active read-only table (ODE/code), whose
   * columns are not solved scalars — used to unit-annotate the axis labels. */
  tableUnits?: Record<string, string>
  diagram: DiagramResponse | null
  psychart: PsychartResponse | null
  /** Declared STATE TABLE blocks, so a plot can overlay just one circuit. */
  stateTableDefs?: StateTableDto[]
  theme: PlotTheme
  revision?: string | number
}

function getArrayValues(variables: VariableResult[], base: string | null): number[] {
  if (!base) return []
  const map = arrayValues(variables, base)
  const indices = [...map.keys()].sort((a, b) => a - b)
  return indices.map((i) => map.get(i) as number)
}

function getMatrixValues(variables: VariableResult[], base: string | null): number[][] {
  if (!base) return []
  const prefix = `${base.toLowerCase()}[`
  const cells: { i: number; j: number; val: number }[] = []
  let maxI = 0
  let maxJ = 0
  for (const v of variables) {
    const name = v.name.toLowerCase()
    if (!name.startsWith(prefix) || !name.endsWith(']')) continue
    const inner = v.name.substring(prefix.length, v.name.length - 1)
    const parts = inner.split(',')
    if (parts.length === 2) {
      const i = Number(parts[0])
      const j = Number(parts[1])
      if (Number.isInteger(i) && Number.isInteger(j)) {
        cells.push({ i, j, val: v.value })
        if (i > maxI) maxI = i
        if (j > maxJ) maxJ = j
      }
    }
  }
  if (maxI === 0 || maxJ === 0) return []
  const res: number[][] = Array.from({ length: maxI }, () => new Array(maxJ).fill(0))
  for (const cell of cells) {
    res[cell.i - 1][cell.j - 1] = cell.val
  }
  return res
}

export function buildFigure(spec: PlotSpec, inputs: FigureInputs): PlotlyFigure | null {
  const { states, cyclePath, variables = [], diagram, psychart, stateTableDefs, theme, revision } = inputs
  // When the plot targets one declared STATE TABLE circuit, overlay only that
  // circuit's states (else all detected states). If the named circuit is missing,
  // do not fall back silently to all detected states across circuits.
  const overlayStates = (name?: string | null): StateTable => {
    if (!name) return states
    if (!stateTableDefs?.length) return { indices: [], columns: [], values: {} }
    return detectStateTables(variables, stateTableDefs).find((t) => t.name === name) ?? { indices: [], columns: [], values: {} }
  }
  if (spec.kind === 'property' && diagram) {
    const selectedCircuit = spec.property.stateTable
      ? stateTableDefs?.find((s) => s.name === spec.property.stateTable)
      : undefined
    const fluidMatches = selectedCircuit
      ? (!selectedCircuit.fluid || selectedCircuit.fluid.toLowerCase() === spec.property.fluid.toLowerCase())
      : (!stateTableDefs || stateTableDefs.length <= 1 || stateTableDefs.every((s) => !s.fluid || s.fluid.toLowerCase() === spec.property.fluid.toLowerCase()))
    const effectiveCyclePath = fluidMatches ? cyclePath : undefined
    return buildPropertyFigure(diagram, spec.property, spec.format, overlayStates(spec.property.stateTable), theme, effectiveCyclePath, revision)
  }
  if (spec.kind === 'psychro' && psychart) {
    return buildPsychroFigure(psychart, spec.psychro, spec.format, overlayStates(spec.psychro.stateTable), theme, undefined, revision)
  }
  if (spec.kind === 'xy' && !spec.source) return null
  if (spec.kind === 'xy' && (spec.xy.xVar || spec.xy.chartType === 'histogram') && spec.xy.yVars.length > 0 && (spec.xy.chartType !== 'surface3d' || spec.xy.zVar)) {
    return buildXyFigureFromSpec(spec, inputs, spec.xy.chartType === 'histogram' ? '' : spec.xy.xVar!, revision)
  }
  if (spec.kind === 'bode' && spec.control.omega && spec.control.mag && spec.control.phase) {
    const omega = getArrayValues(variables, spec.control.omega)
    const mag = getArrayValues(variables, spec.control.mag)
    const phase = getArrayValues(variables, spec.control.phase)
    return buildBodeFigure(omega, mag, phase, spec.format, theme, revision)
  }
  if (spec.kind === 'nyquist' && spec.control.real && spec.control.imag) {
    const real = getArrayValues(variables, spec.control.real)
    const imag = getArrayValues(variables, spec.control.imag)
    return buildNyquistFigure(real, imag, spec.format, theme, revision)
  }
  if (spec.kind === 'nichols' && spec.control.mag && spec.control.phase) {
    const mag = getArrayValues(variables, spec.control.mag)
    const phase = getArrayValues(variables, spec.control.phase)
    return buildNicholsFigure(mag, phase, spec.format, theme, revision)
  }
  if (spec.kind === 'polezero' && spec.control.pr && spec.control.pi) {
    const pr = getArrayValues(variables, spec.control.pr)
    const pi = getArrayValues(variables, spec.control.pi)
    const zr = getArrayValues(variables, spec.control.zr)
    const zi = getArrayValues(variables, spec.control.zi)
    return buildPoleZeroFigure(pr, pi, zr, zi, spec.format, theme, revision)
  }
  if (spec.kind === 'rootlocus' && spec.control.pr && spec.control.pi) {
    const cpr = getMatrixValues(variables, spec.control.pr)
    const cpi = getMatrixValues(variables, spec.control.pi)
    const zr = getArrayValues(variables, spec.control.zr)
    const zi = getArrayValues(variables, spec.control.zi)
    return buildRootLocusFigure(cpr, cpi, zr, zi, spec.format, theme, revision)
  }
  return null
}

/** Builds the XY figure: series from parametric-table rows (or solved arrays as a
 *  fallback), with unit-annotated axis labels. */
function buildXyFigureFromSpec(spec: PlotSpec, inputs: FigureInputs, xVar: string, revision?: string | number): PlotlyFigure {
  const { tableRows, tableResults, variables = [], theme } = inputs
  // Use solved array variables when there is no parametric table, or when the
  // table exists but has not been run yet (results empty). Fall back to
  // parametric-table rows only when the table was actually executed — OR when
  // the rows already carry the requested series data even though there are no
  // run results, which is the case for read-only code PARAMETRIC tables and
  // DYNAMIC/ODE trajectories (their values live in the rows, not in `results`).
  const outcomes = spec.source?.kind === 'table' && spec.source.data === 'solved'
    ? tableRows.map((_, i) => tableResults[i] ?? { success: false, values: {}, error: null }) : []
  const useArrays = spec.source?.kind === 'arrays'
  const series = useArrays
    ? buildArrayXYSeries(variables, xVar, spec.xy.yVars, 'y', spec.xy.zVar, spec.xy.sizeVar)
    : buildXYSeries(tableRows, outcomes, xVar, spec.xy.yVars, spec.xy.zVar, spec.xy.sizeVar)
  if (spec.xy.y2Vars && spec.xy.y2Vars.length > 0) {
    series.push(
      ...(useArrays
        ? buildArrayXYSeries(variables, xVar, spec.xy.y2Vars, 'y2')
        : buildXYSeries(tableRows, outcomes, xVar, spec.xy.y2Vars, null, null, 'y2')),
    )
  }
  // Append each axis variable's unit (from the solved variables — same unit a
  // table column displays) to the default axis labels, unless disabled.
  const showUnits = spec.format.showUnits !== false
  const unitOf = (name: string): string => {
    const v = variables.find((x) => x.name.toLowerCase() === name.toLowerCase())
    // Solved scalars carry their unit; ODE/code-table columns are not scalars,
    // so fall back to the table's per-column SI units.
    return v?.units ?? inputs.tableUnits?.[name] ?? ''
  }
  const withUnit = (label: string, unit: string) =>
    showUnits && unit ? `${label} [${unit}]` : label
  // For a single shared unit across all Y vars, label the axis with it.
  const yUnits = new Set(spec.xy.yVars.map(unitOf).filter(Boolean))
  const yUnit = yUnits.size === 1 ? [...yUnits][0] : ''
  return buildXYFigure(
    series,
    spec.format,
    withUnit(displayVar(xVar), unitOf(xVar)),
    withUnit(spec.xy.yVars.map(displayVar).join(', '), yUnit),
    theme,
    spec.xy,
    revision,
  )
}

export default function PlotCard({
  spec,
  states,
  cyclePath,
  tableRows,
  tableResults,
  variables = [],
  tableUnits,
  stateTableDefs,
  onConfigure,
  onDuplicate,
  onRemove,
  leftSection,
  rightSection,
  hideHeader = false,
  exportTrigger = null,
  onSelectRow,
}: Readonly<Props>) {
  const { diagram, psychart, loading, error } = useDiagramData(spec)
  const [exporting, setExporting] = useState(false)
  const [exportError, setExportError] = useState<string | null>(null)
  const [publicationStyle, setPublicationStyle] = useState(true)
  const [viewResetKey, setViewResetKey] = useState(0)
  const [exportViewMode, setExportViewMode] = useState<'current' | 'full'>('current')

  // Phase 10D: Persistent 1-cursor & 2-cursor inspection, deltas/slope, raw data stats, and accessible table
  const [cursor1, setCursor1] = useState<PlotCursor | null>(null)
  const [cursor2, setCursor2] = useState<PlotCursor | null>(null)
  const [dualCursor, setDualCursor] = useState(false)
  const [activeCursorTarget, setActiveCursorTarget] = useState<1 | 2>(1)
  const [showDataTable, setShowDataTable] = useState(false)
  const [showStats, setShowStats] = useState(false)

  const circuitWarning = useMemo(() => {
    if (spec.kind !== 'property') return null
    if (spec.property.stateTable) {
      const exists = stateTableDefs?.some((s) => s.name === spec.property.stateTable)
      if (stateTableDefs && stateTableDefs.length > 0 && !exists) {
        return `State table "${spec.property.stateTable}" not found. No states overlaid.`
      }
      const circuit = stateTableDefs?.find((s) => s.name === spec.property.stateTable)
      if (circuit?.fluid && circuit.fluid.toLowerCase() !== spec.property.fluid.toLowerCase()) {
        return `Circuit "${spec.property.stateTable}" fluid (${circuit.fluid}) does not match diagram fluid (${spec.property.fluid}); cycle path suppressed.`
      }
    } else if (cyclePath && stateTableDefs && stateTableDefs.length > 1) {
      const distinctFluids = new Set(stateTableDefs.map((s) => s.fluid?.toLowerCase()).filter(Boolean))
      if (distinctFluids.size > 1) {
        return `Multiple circuits with different fluids detected; cycle path suppressed for ambiguous diagram.`
      }
    }
    return null
  }, [spec, stateTableDefs, cyclePath])

  useEffect(() => {
    if (exportTrigger) {
      // Narrow the free-form trigger string to a known export format value.
      const fmt = EXPORT_FORMATS.find((f) => f.value === exportTrigger.format)?.value
      if (fmt) void onExport(fmt)
    }
  }, [exportTrigger])

  const figure = useMemo(
    () =>
      buildFigure(spec, {
        states,
        cyclePath,
        tableRows,
        tableResults,
        variables,
        tableUnits,
        diagram,
        psychart,
        stateTableDefs,
        theme: 'dark',
        revision: viewResetKey,
      }),
    [spec, states, cyclePath, tableRows, tableResults, variables, tableUnits, diagram, psychart, stateTableDefs, viewResetKey],
  )

  const handlePointClick = (event: PlotPointClickEvent) => {
    const newCursor: PlotCursor = {
      traceIndex: event.traceIndex,
      traceName: event.traceName || `Trace ${event.traceIndex + 1}`,
      pointIndex: event.pointIndex,
      x: event.x,
      y: event.y,
      sampleId: event.sampleId,
    }
    if (dualCursor && activeCursorTarget === 2) {
      setCursor2(newCursor)
    } else {
      setCursor1(newCursor)
      if (dualCursor && !cursor2) {
        setActiveCursorTarget(2)
      }
    }
  }

  const stepSample = (direction: -1 | 1) => {
    const target = (dualCursor && activeCursorTarget === 2 && cursor2) ? cursor2 : (cursor1 ?? { traceIndex: 0, pointIndex: 0, x: 0, y: 0, traceName: '' })
    if (!figure) return
    const trace = figure.data[target.traceIndex] as { x?: unknown[]; y?: unknown[]; customdata?: unknown[]; name?: string } | undefined
    if (!trace || !Array.isArray(trace.x) || trace.x.length === 0) return

    const newIndex = Math.max(0, Math.min(trace.x.length - 1, target.pointIndex + direction))
    const newX = trace.x[newIndex]
    const newY = trace.y?.[newIndex]
    const sampleId =
      (Array.isArray(trace.customdata) ? trace.customdata[newIndex] : undefined) ??
      (spec.source?.kind === 'table' ? tableRows[newIndex]?.id : undefined)

    const updatedCursor: PlotCursor = {
      traceIndex: target.traceIndex,
      traceName: target.traceName || trace.name || `Trace ${target.traceIndex + 1}`,
      pointIndex: newIndex,
      x: typeof newX === 'number' ? newX : Number(newX),
      y: typeof newY === 'number' ? newY : Number(newY),
      sampleId: typeof sampleId === 'string' ? sampleId : undefined,
    }

    if (dualCursor && activeCursorTarget === 2) {
      setCursor2(updatedCursor)
    } else {
      setCursor1(updatedCursor)
    }
  }

  const deltaX = cursor1 && cursor2 ? cursor2.x - cursor1.x : null
  const deltaY = cursor1 && cursor2 ? cursor2.y - cursor1.y : null
  const slope = deltaX !== null && deltaY !== null ? (deltaX !== 0 ? deltaY / deltaX : null) : null

  const activeTraceIndex = cursor1 ? cursor1.traceIndex : 0
  const activeTrace = figure?.data[activeTraceIndex] as { name?: string; x?: unknown[]; y?: unknown[]; customdata?: unknown[] } | undefined
  const activeStats = useMemo(() => computeTraceStats(activeTrace), [activeTrace])

  // Overlay persistent cursors on the figure layout
  const displayedFigure = useMemo(() => {
    if (!figure) return null
    if (!cursor1 && !cursor2) return figure
    const shapes = [...(figure.layout.shapes ?? [])]
    const annotations = [...(figure.layout.annotations ?? [])]

    if (cursor1 && Number.isFinite(cursor1.x) && Number.isFinite(cursor1.y)) {
      shapes.push({
        type: 'line',
        x0: cursor1.x,
        x1: cursor1.x,
        y0: 0,
        y1: 1,
        yref: 'paper',
        line: { color: '#339af0', width: 1.5, dash: 'dash' },
      })
      annotations.push({
        x: cursor1.x,
        y: cursor1.y,
        text: 'C1',
        showarrow: true,
        arrowhead: 2,
        ax: 0,
        ay: -25,
        bgcolor: '#1971c2',
        bordercolor: '#339af0',
        font: { color: '#ffffff', size: 10 },
      })
    }

    if (cursor2 && Number.isFinite(cursor2.x) && Number.isFinite(cursor2.y)) {
      shapes.push({
        type: 'line',
        x0: cursor2.x,
        x1: cursor2.x,
        y0: 0,
        y1: 1,
        yref: 'paper',
        line: { color: '#ff922b', width: 1.5, dash: 'dot' },
      })
      annotations.push({
        x: cursor2.x,
        y: cursor2.y,
        text: 'C2',
        showarrow: true,
        arrowhead: 2,
        ax: 0,
        ay: -25,
        bgcolor: '#e8590c',
        bordercolor: '#ff922b',
        font: { color: '#ffffff', size: 10 },
      })
    }

    return {
      ...figure,
      layout: {
        ...figure.layout,
        shapes,
        annotations,
      },
    }
  }, [figure, cursor1, cursor2])

  async function onExport(format: (typeof EXPORT_FORMATS)[number]['value']) {
    const theme: PlotTheme = publicationStyle ? 'light' : 'dark'
    const exportFigure = buildFigure(spec, {
      states,
      cyclePath,
      tableRows,
      tableResults,
      variables,
      tableUnits,
      diagram,
      psychart,
      stateTableDefs,
      theme,
      revision: viewResetKey,
    })
    if (!exportFigure) return
    setExporting(true)
    setExportError(null)
    try {
      await exportPlot(exportFigure, format, spec.name.replace(/\s+/g, '_'), {
        viewMode: exportViewMode,
        background: publicationStyle ? '#ffffff' : undefined,
      })
    } catch (e) {
      setExportError(String(e instanceof Error ? e.message : e))
    } finally {
      setExporting(false)
    }
  }

  return (
    <>
      {!hideHeader && (
        <Group justify="space-between" mb="xs" wrap="nowrap" align="center">
          <Group gap="xs" style={{ flex: 1 }} wrap="nowrap">
            {leftSection}
            {spec.fromCode ? <>
              <Badge size="xs">Code-owned</Badge>
              <Button variant="default" size="xs" onClick={onDuplicate}>Duplicate as editable</Button>
            </> : <Button variant="default" size="xs" onClick={onConfigure}>Configure</Button>}
            <Button variant="default" size="xs" onClick={() => setViewResetKey((k) => k + 1)}>
              Fit data
            </Button>
            <Menu shadow="md">
              <Menu.Target>
                <Button variant="default" size="xs" loading={exporting}>
                  Export
                </Button>
              </Menu.Target>
              <Menu.Dropdown>
                <Menu.Label>Format</Menu.Label>
                {EXPORT_FORMATS.map((f) => (
                  <Menu.Item key={f.value} onClick={() => void onExport(f.value)}>
                    {f.label}
                  </Menu.Item>
                ))}
                <Menu.Divider />
                <Menu.Label>Export Scope</Menu.Label>
                <Menu.Item
                  onClick={() => setExportViewMode('current')}
                  rightSection={exportViewMode === 'current' ? '✓' : undefined}
                >
                  Current view
                </Menu.Item>
                <Menu.Item
                  onClick={() => setExportViewMode('full')}
                  rightSection={exportViewMode === 'full' ? '✓' : undefined}
                >
                  Full data view (autoscale)
                </Menu.Item>
                <Menu.Divider />
                <Menu.Item
                  onClick={() => setPublicationStyle((v) => !v)}
                  rightSection={publicationStyle ? '✓' : undefined}
                >
                  Publication style (white)
                </Menu.Item>
              </Menu.Dropdown>
            </Menu>
          </Group>
          <Group gap="xs" wrap="nowrap">
            {!spec.fromCode && <Button variant="subtle" color="red" size="xs" onClick={onRemove}>Remove</Button>}
            {rightSection}
          </Group>
        </Group>
      )}

      {error && (
        <Alert color="red" mb="xs">
          {error}
        </Alert>
      )}
      {exportError && (
        <Alert color="orange" mb="xs" withCloseButton onClose={() => setExportError(null)}>
          Export failed: {exportError}
        </Alert>
      )}
      {circuitWarning && (
        <Alert color="yellow" mb="xs">
          {circuitWarning}
        </Alert>
      )}
      {!!spec.codeDiagnostics?.length && (
        <Alert color="yellow" mb="xs">
          {spec.codeDiagnostics.join(' ')}
        </Alert>
      )}
      {loading && (
        <Group gap="xs">
          <Loader size="xs" />
          <Text size="sm" c="dimmed">
            Computing property curves…
          </Text>
        </Group>
      )}
      {!loading && !error && displayedFigure === null && (
        <Text size="sm" c="dimmed">
          {spec.kind === 'xy'
            ? 'Choose an explicit data source and the required channels in Configure. Missing or ambiguous sources are never replaced by the active table.'
            : 'No data yet.'}
        </Text>
      )}
      {displayedFigure !== null && (
        <div style={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
          <div style={{ flex: 1, minHeight: 220, position: 'relative' }}>
            {/* Fill the tile exactly and let the ResizeObserver in PlotlyChart keep it fitted. */}
            <PlotlyChart figure={displayedFigure} minHeight={0} onPointClick={handlePointClick} />
          </div>

          {/* Inspection & Measurement Bar */}
          <Paper
            p="xs"
            mt={4}
            withBorder
            style={{ backgroundColor: 'var(--mantine-color-body)', fontSize: 12, flexShrink: 0 }}
            role="region"
            aria-label="Plot inspection and cursor measurement"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === 'ArrowLeft') {
                e.preventDefault()
                stepSample(-1)
              } else if (e.key === 'ArrowRight') {
                e.preventDefault()
                stepSample(1)
              }
            }}
          >
            <Group justify="space-between" wrap="wrap" gap="xs">
              <Group gap="xs" wrap="wrap" align="center">
                <Group gap={4} wrap="nowrap">
                  <Button
                    size="compact-xs"
                    variant={!dualCursor ? 'filled' : 'default'}
                    color="blue"
                    onClick={() => {
                      setDualCursor(false)
                      setCursor2(null)
                      setActiveCursorTarget(1)
                    }}
                  >
                    1-Cursor
                  </Button>
                  <Button
                    size="compact-xs"
                    variant={dualCursor ? 'filled' : 'default'}
                    color="orange"
                    onClick={() => {
                      setDualCursor(true)
                      setActiveCursorTarget(2)
                    }}
                  >
                    2-Cursor (Δ)
                  </Button>
                </Group>

                {cursor1 ? (
                  <Badge
                    color="blue"
                    variant={activeCursorTarget === 1 && dualCursor ? 'filled' : 'light'}
                    style={{ cursor: dualCursor ? 'pointer' : 'default' }}
                    onClick={() => dualCursor && setActiveCursorTarget(1)}
                    title={dualCursor ? 'Click to select Cursor 1 for sample stepping' : undefined}
                  >
                    C1: Pt #{cursor1.pointIndex + 1} | X: {formatPlotValue(cursor1.x)} | Y: {formatPlotValue(cursor1.y)}
                  </Badge>
                ) : (
                  <Text size="xs" c="dimmed">Click plot point to inspect</Text>
                )}

                {dualCursor && (
                  cursor2 ? (
                    <Badge
                      color="orange"
                      variant={activeCursorTarget === 2 ? 'filled' : 'light'}
                      style={{ cursor: 'pointer' }}
                      onClick={() => setActiveCursorTarget(2)}
                      title="Click to select Cursor 2 for sample stepping"
                    >
                      C2: Pt #{cursor2.pointIndex + 1} | X: {formatPlotValue(cursor2.x)} | Y: {formatPlotValue(cursor2.y)}
                    </Badge>
                  ) : (
                    <Text size="xs" c="dimmed">Click 2nd point for C2</Text>
                  )
                )}

                {dualCursor && cursor1 && cursor2 && (
                  <Badge color="grape" variant="outline">
                    ΔX: {formatPlotValue(deltaX)} | ΔY: {formatPlotValue(deltaY)} | Slope: {slope !== null ? formatPlotValue(slope) : (deltaX === 0 ? 'vertical' : '—')}
                  </Badge>
                )}

                {(cursor1 || cursor2) && (
                  <Group gap={4} wrap="nowrap">
                    <Tooltip label="Step to previous sample (ArrowLeft)">
                      <ActionIcon size="xs" variant="default" aria-label="Previous sample" onClick={() => stepSample(-1)}>
                        <IconChevronLeft size={12} />
                      </ActionIcon>
                    </Tooltip>
                    <Tooltip label="Step to next sample (ArrowRight)">
                      <ActionIcon size="xs" variant="default" aria-label="Next sample" onClick={() => stepSample(1)}>
                        <IconChevronRight size={12} />
                      </ActionIcon>
                    </Tooltip>
                  </Group>
                )}

                {/* Link to table row */}
                {cursor1?.sampleId && onSelectRow && (
                  <Button
                    size="compact-xs"
                    variant="subtle"
                    leftSection={<IconArrowRight size={12} />}
                    onClick={() => onSelectRow(spec.source?.kind === 'table' ? spec.source.tableId : undefined, cursor1.sampleId!)}
                    title="Highlight and inspect this row in the table"
                  >
                    Row {cursor1.sampleId} in table
                  </Button>
                )}
              </Group>

              <Group gap="xs" wrap="nowrap">
                <Button
                  size="compact-xs"
                  variant={showStats ? 'filled' : 'default'}
                  leftSection={<IconChartBar size={12} />}
                  onClick={() => setShowStats((s) => !s)}
                >
                  Stats
                </Button>
                <Button
                  size="compact-xs"
                  variant="default"
                  leftSection={<IconTable size={12} />}
                  onClick={() => setShowDataTable(true)}
                >
                  Raw data
                </Button>
                {(cursor1 || cursor2) && (
                  <ActionIcon
                    size="xs"
                    variant="subtle"
                    color="gray"
                    aria-label="Clear cursors"
                    title="Clear cursors"
                    onClick={() => {
                      setCursor1(null)
                      setCursor2(null)
                      setActiveCursorTarget(1)
                    }}
                  >
                    <IconX size={12} />
                  </ActionIcon>
                )}
              </Group>
            </Group>

            {/* Statistics drawer */}
            {showStats && activeStats && (
              <Paper p="xs" mt="xs" withBorder style={{ backgroundColor: 'var(--mantine-color-default-hover)' }}>
                <Group justify="space-between" mb={4}>
                  <Text size="xs" fw={600}>
                    Raw Data Statistics — {activeStats.traceName}
                  </Text>
                  <Text size="xs" c="dimmed">
                    Scope: {activeStats.count} total samples ({activeStats.valid} valid, {activeStats.missing} missing/NaN)
                  </Text>
                </Group>
                <Group gap="md" wrap="wrap">
                  <Text size="xs">
                    <b>X Range:</b> [{formatPlotValue(activeStats.minX)}, {formatPlotValue(activeStats.maxX)}]
                  </Text>
                  <Text size="xs">
                    <b>Y Range:</b> [{formatPlotValue(activeStats.minY)}, {formatPlotValue(activeStats.maxY)}]
                  </Text>
                  <Text size="xs">
                    <b>Y Mean:</b> {formatPlotValue(activeStats.meanY)}
                  </Text>
                  <Text size="xs">
                    <b>Y Sum:</b> {formatPlotValue(activeStats.sumY)}
                  </Text>
                </Group>
              </Paper>
            )}
          </Paper>

          {/* Accessible raw data table modal */}
          <Modal
            opened={showDataTable}
            onClose={() => setShowDataTable(false)}
            title={`Raw Data: ${spec.name}`}
            size="lg"
          >
            {activeTrace && Array.isArray(activeTrace.x) ? (
              <ScrollArea style={{ maxHeight: 400 }}>
                <Table striped highlightOnHover withTableBorder role="table" aria-label={`Raw data for ${spec.name}`}>
                  <Table.Thead>
                    <Table.Tr>
                      <Table.Th scope="col">#</Table.Th>
                      <Table.Th scope="col">Sample / Row</Table.Th>
                      <Table.Th scope="col">X ({spec.format.xLabel || spec.xy?.xVar || 'X'})</Table.Th>
                      <Table.Th scope="col">Y ({spec.format.yLabel || spec.xy?.yVars?.join(', ') || 'Y'})</Table.Th>
                      <Table.Th scope="col">Status</Table.Th>
                    </Table.Tr>
                  </Table.Thead>
                  <Table.Tbody>
                    {activeTrace.x.map((xVal: unknown, idx: number) => {
                      const yVal = (activeTrace.y as unknown[])?.[idx]
                      const isValid = typeof xVal === 'number' && Number.isFinite(xVal) && typeof yVal === 'number' && Number.isFinite(yVal)
                      const sampleId = (Array.isArray(activeTrace.customdata) ? (activeTrace.customdata as unknown[])[idx] : undefined) ?? (spec.source?.kind === 'table' ? tableRows[idx]?.id : undefined)
                      return (
                        <Table.Tr key={idx}>
                          <Table.Td>{idx + 1}</Table.Td>
                          <Table.Td>{typeof sampleId === 'string' ? sampleId : `pt-${idx + 1}`}</Table.Td>
                          <Table.Td>{formatPlotValue(typeof xVal === 'number' ? xVal : Number(xVal))}</Table.Td>
                          <Table.Td>{formatPlotValue(typeof yVal === 'number' ? yVal : Number(yVal))}</Table.Td>
                          <Table.Td>
                            {isValid ? (
                              <Badge color="teal" size="xs">Valid</Badge>
                            ) : (
                              <Badge color="red" size="xs">Missing / NaN</Badge>
                            )}
                          </Table.Td>
                        </Table.Tr>
                      )
                    })}
                  </Table.Tbody>
                </Table>
              </ScrollArea>
            ) : (
              <Text size="sm" c="dimmed">No raw tabular series available for this plot.</Text>
            )}
          </Modal>
        </div>
      )}
    </>
  )
}
