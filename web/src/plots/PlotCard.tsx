import { useEffect, useMemo, useState } from 'react'
import { Alert, Button, Group, Loader, Menu, Text } from '@mantine/core'
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
import PlotlyChart from './PlotlyChart'

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
  onConfigure: () => void
  onRemove: () => void
  leftSection?: React.ReactNode
  rightSection?: React.ReactNode
  hideHeader?: boolean
  exportTrigger?: { format: string; timestamp: number } | null
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
  const { states, cyclePath, variables = [], diagram, psychart, stateTableDefs, theme } = inputs
  // When the plot targets one declared STATE TABLE circuit, overlay only that
  // circuit's states (else all detected states).
  const overlayStates = (name?: string | null): StateTable => {
    if (!name || !stateTableDefs?.length) return states
    return detectStateTables(variables, stateTableDefs).find((t) => t.name === name) ?? states
  }
  if (spec.kind === 'property' && diagram) {
    return buildPropertyFigure(diagram, spec.property, spec.format, overlayStates(spec.property.stateTable), theme, cyclePath)
  }
  if (spec.kind === 'psychro' && psychart) {
    return buildPsychroFigure(psychart, spec.psychro, spec.format, overlayStates(spec.psychro.stateTable), theme, cyclePath)
  }
  if (spec.kind === 'xy' && !spec.source) return null
  if (spec.kind === 'xy' && (spec.xy.xVar || spec.xy.chartType === 'histogram') && spec.xy.yVars.length > 0 && (spec.xy.chartType !== 'surface3d' || spec.xy.zVar)) {
    return buildXyFigureFromSpec(spec, inputs, spec.xy.chartType === 'histogram' ? '' : spec.xy.xVar!)
  }
  if (spec.kind === 'bode' && spec.control.omega && spec.control.mag && spec.control.phase) {
    const omega = getArrayValues(variables, spec.control.omega)
    const mag = getArrayValues(variables, spec.control.mag)
    const phase = getArrayValues(variables, spec.control.phase)
    return buildBodeFigure(omega, mag, phase, spec.format, theme)
  }
  if (spec.kind === 'nyquist' && spec.control.real && spec.control.imag) {
    const real = getArrayValues(variables, spec.control.real)
    const imag = getArrayValues(variables, spec.control.imag)
    return buildNyquistFigure(real, imag, spec.format, theme)
  }
  if (spec.kind === 'nichols' && spec.control.mag && spec.control.phase) {
    const mag = getArrayValues(variables, spec.control.mag)
    const phase = getArrayValues(variables, spec.control.phase)
    return buildNicholsFigure(mag, phase, spec.format, theme)
  }
  if (spec.kind === 'polezero' && spec.control.pr && spec.control.pi) {
    const pr = getArrayValues(variables, spec.control.pr)
    const pi = getArrayValues(variables, spec.control.pi)
    const zr = getArrayValues(variables, spec.control.zr)
    const zi = getArrayValues(variables, spec.control.zi)
    return buildPoleZeroFigure(pr, pi, zr, zi, spec.format, theme)
  }
  if (spec.kind === 'rootlocus' && spec.control.pr && spec.control.pi) {
    const cpr = getMatrixValues(variables, spec.control.pr)
    const cpi = getMatrixValues(variables, spec.control.pi)
    const zr = getArrayValues(variables, spec.control.zr)
    const zi = getArrayValues(variables, spec.control.zi)
    return buildRootLocusFigure(cpr, cpi, zr, zi, spec.format, theme)
  }
  return null
}

/** Builds the XY figure: series from parametric-table rows (or solved arrays as a
 *  fallback), with unit-annotated axis labels. */
function buildXyFigureFromSpec(spec: PlotSpec, inputs: FigureInputs, xVar: string): PlotlyFigure {
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
  onRemove,
  leftSection,
  rightSection,
  hideHeader = false,
  exportTrigger = null,
}: Readonly<Props>) {
  const { diagram, psychart, loading, error } = useDiagramData(spec)
  const [exporting, setExporting] = useState(false)
  const [exportError, setExportError] = useState<string | null>(null)
  const [publicationStyle, setPublicationStyle] = useState(true)

  useEffect(() => {
    if (exportTrigger) {
      // Narrow the free-form trigger string to a known export format value.
      const fmt = EXPORT_FORMATS.find((f) => f.value === exportTrigger.format)?.value
      if (fmt) void onExport(fmt)
    }
  }, [exportTrigger])

  const figure = useMemo(
    () => buildFigure(spec, { states, cyclePath, tableRows, tableResults, variables, tableUnits, diagram, psychart, stateTableDefs, theme: 'dark' }),
    [spec, states, cyclePath, tableRows, tableResults, variables, tableUnits, diagram, psychart, stateTableDefs],
  )

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
    })
    if (!exportFigure) return
    setExporting(true)
    setExportError(null)
    try {
      await exportPlot(exportFigure, format, spec.name.replace(/\s+/g, '_'))
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
            <Button variant="default" size="xs" onClick={onConfigure}>
              Configure
            </Button>
            <Menu shadow="md">
              <Menu.Target>
                <Button variant="default" size="xs" loading={exporting}>
                  Export
                </Button>
              </Menu.Target>
              <Menu.Dropdown>
                {EXPORT_FORMATS.map((f) => (
                  <Menu.Item key={f.value} onClick={() => void onExport(f.value)}>
                    {f.label}
                  </Menu.Item>
                ))}
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
            <Button variant="subtle" color="red" size="xs" onClick={onRemove}>
              Remove
            </Button>
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
      {loading && (
        <Group gap="xs">
          <Loader size="xs" />
          <Text size="sm" c="dimmed">
            Computing property curves…
          </Text>
        </Group>
      )}
      {!loading && !error && figure === null && (
        <Text size="sm" c="dimmed">
          {spec.kind === 'xy'
            ? 'Choose an explicit data source and the required channels in Configure. Missing or ambiguous sources are never replaced by the active table.'
            : 'No data yet.'}
        </Text>
      )}
      {figure !== null && (
        <div style={{ flex: 1, minHeight: 0 }}>
          {/* Fill the tile exactly — no 380px floor that would overflow a short
              window — and let the ResizeObserver in PlotlyChart keep it fitted. */}
          <PlotlyChart figure={figure} minHeight={0} />
        </div>
      )}
    </>
  )
}
