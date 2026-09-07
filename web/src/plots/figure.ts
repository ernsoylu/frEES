import type {
  PlotlyAxisLayout,
  PlotlyFigure,
  PlotlyLayout,
  PlotlyTrace,
} from 'plotly.js/lib/core'
import { DiagramCurve, DiagramResponse, PsychartResponse } from '../api'
import { PlotFormat, PropertyConfig, PsychroConfig, XYConfig } from './types'
import { StateTable, statesForAxes } from './stateTable'
import { UnitChoice, resolveUnit } from './units'
import { displayVar } from '../varDisplay'

/**
 * Builds Plotly figures for every plot kind in one place so the on-screen
 * dark theme and the publication light theme stay consistent.
 */

export type PlotTheme = 'dark' | 'light'

interface ThemeColors {
  font: string
  grid: string
  zero: string
  dome: string
  states: string
}

const THEMES: Record<PlotTheme, ThemeColors> = {
  dark: {
    font: '#c1c2c5',
    grid: '#373A40',
    zero: '#5c5f66',
    dome: '#e9ecef',
    states: '#ffa94b',
  },
  light: {
    font: '#212529',
    grid: '#d4d7da',
    zero: '#9aa0a6',
    dome: '#212529',
    states: '#e8590c',
  },
}

interface FamilyStyle {
  color: string
  width: number
  dash: 'solid' | 'dot' | 'dash' | 'dashdot'
}

const FAMILY_STYLES: Record<string, FamilyStyle> = {
  quality: { color: '#868e96', width: 1, dash: 'dot' },
  isobar: { color: '#4dabf7', width: 1, dash: 'solid' },
  isotherm: { color: '#ff6b6b', width: 1, dash: 'solid' },
  isentrope: { color: '#38d9a9', width: 1, dash: 'dash' },
  rh: { color: '#4dabf7', width: 1, dash: 'solid' },
  wetbulb: { color: '#38d9a9', width: 1, dash: 'dash' },
  enthalpy: { color: '#ffa94b', width: 1, dash: 'dot' },
  volume: { color: '#b197fc', width: 1, dash: 'dashdot' },
}

/** Axis title with the active display unit, e.g. "P [kPa]". */
function axisTitle(property: string, unit: UnitChoice): string {
  const name = property === 'w' ? 'Humidity ratio ω' : property
  return `${name} [${unit.label}]`
}

function transformAxis(
  values: (number | null)[],
  scale: number,
  offset: number,
): (number | null)[] {
  return values.map((value) => (value === null ? null : value * scale + offset))
}

function curveTrace(
  curve: DiagramCurve,
  style: FamilyStyle,
  xScale: number,
  xOffset: number,
  yScale: number,
  yOffset: number,
): PlotlyTrace {
  return {
    type: 'scatter',
    mode: 'lines',
    name: curve.label,
    x: transformAxis(curve.x, xScale, xOffset),
    y: transformAxis(curve.y, yScale, yOffset),
    line: { color: style.color, width: style.width, dash: style.dash },
    showlegend: false,
    hoverinfo: 'name+x+y',
  }
}

/** Axis range bound in plot coordinates; log axes take the exponent. Nonpositive values are rejected. */
function rangeValue(value: number | null | undefined, log: boolean): number | null {
  if (value === null || value === undefined) return null
  if (log) {
    if (value <= 0) return null
    return Math.log10(value)
  }
  return value
}

function axisLayout(
  label: string,
  log: boolean,
  format: PlotFormat,
  colors: ThemeColors,
  min: number | null | undefined,
  max: number | null | undefined,
  tick: number | null | undefined,
): PlotlyAxisLayout {
  const layout: PlotlyAxisLayout = {
    title: { text: label },
    type: log ? 'log' : 'linear',
    gridcolor: colors.grid,
    zerolinecolor: colors.zero,
    color: colors.font,
    showgrid: format.grid,
    exponentformat: 'power',
  }

  const rMin = rangeValue(min, log)
  const rMax = rangeValue(max, log)
  if (rMin !== null || rMax !== null) {
    layout.range = [rMin, rMax]
  }

  if (tick !== null && tick !== undefined && tick > 0) {
    layout.dtick = tick
  }

  return layout
}

function baseLayout(
  format: PlotFormat,
  xLabel: string,
  yLabel: string,
  xLog: boolean,
  yLog: boolean,
  theme: PlotTheme,
  revision?: string | number,
): PlotlyLayout {
  const colors = THEMES[theme]
  const background = theme === 'dark' ? 'rgba(0,0,0,0)' : '#ffffff'
  const revPart = revision !== undefined ? `_${revision}` : ''
  return {
    title: format.title ? { text: format.title } : undefined,
    uirevision: `${xLog}_${yLog}_${format.xUnit ?? ''}_${format.yUnit ?? ''}${revPart}`,
    paper_bgcolor: background,
    plot_bgcolor: background,
    font: { color: colors.font, size: format.fontSize },
    margin: { t: format.title ? 48 : 24, r: 16, b: 56, l: 64 },
    xaxis: axisLayout(format.xLabel || xLabel, xLog, format, colors, format.xMin, format.xMax, format.xTick),
    yaxis: axisLayout(format.yLabel || yLabel, yLog, format, colors, format.yMin, format.yMax, format.yTick),
    showlegend: format.legend,
    legend: legendLayout(format.legendAlign),
  }
}

/** Horizontal legend anchored per the configured alignment (default center). */
function legendLayout(align: 'left' | 'center' | 'right' | undefined) {
  const x = align === 'left' ? 0 : align === 'right' ? 1 : 0.5
  const xanchor = align ?? 'center'
  return { orientation: 'h' as const, bgcolor: 'rgba(0,0,0,0)', x, xanchor }
}

/** Shared layout chrome for the control-system figures (Nyquist, pole-zero,
 *  Nichols, root locus). They differ only in their axis-title defaults, whether
 *  the Y axis is locked to the X scale (`squareAspect`), and whether a legend is
 *  shown at all (`legend: false` ⇒ no legend; otherwise gated on format.legend). */
function controlAxesLayout(
  format: PlotFormat,
  theme: PlotTheme,
  opts: { xLabel: string; yLabel: string; squareAspect?: boolean; legend?: boolean },
  revision?: string | number,
): PlotlyLayout {
  const colors = THEMES[theme]
  const background = theme === 'dark' ? 'rgba(0,0,0,0)' : '#ffffff'
  const layout: PlotlyLayout = {
    title: format.title ? { text: format.title } : undefined,
    uirevision: revision !== undefined ? `control_${revision}` : 'control',
    paper_bgcolor: background,
    plot_bgcolor: background,
    font: { color: colors.font, size: format.fontSize },
    margin: { t: format.title ? 48 : 24, r: 24, b: 56, l: 64 },
    xaxis: {
      title: format.xLabel || opts.xLabel,
      color: colors.font,
      gridcolor: colors.grid,
      zerolinecolor: colors.zero,
      showgrid: format.grid,
    },
    yaxis: {
      title: format.yLabel || opts.yLabel,
      ...(opts.squareAspect ? { scaleanchor: 'x' as const } : {}),
      color: colors.font,
      gridcolor: colors.grid,
      zerolinecolor: colors.zero,
      showgrid: format.grid,
    },
    showlegend: opts.legend === false ? false : format.legend,
  }
  if (opts.legend !== false) {
    layout.legend = legendLayout(format.legendAlign)
  }
  return layout
}

interface StateOverlay {
  xProperty: string
  yProperty: string
  connect: boolean
  close: boolean
}

/** Line trace joining the state points: the solver's cycle path when present, else the points in order. */
function connectionTrace(
  overlay: StateOverlay,
  points: { x: number; y: number }[],
  cyclePath: Record<string, number>[] | undefined,
  color: string,
  xUnit: UnitChoice,
  yUnit: UnitChoice,
): PlotlyTrace | null {
  let name = 'Schematic Connections'
  let linePoints = overlay.close && points.length > 2 ? [...points, points[0]] : points

  if (cyclePath && cyclePath.length > 0) {
    name = 'Cycle Path'
    linePoints = cyclePath
      .map((pt) => ({ x: pt[overlay.xProperty], y: pt[overlay.yProperty] }))
      .filter((p): p is { x: number; y: number } => p.x !== undefined && p.y !== undefined)
    if (linePoints.length === 0) return null
  }

  return {
    type: 'scatter',
    mode: 'lines',
    name,
    uid: cyclePath && cyclePath.length > 0 ? 'cycle_path' : 'schematic_connections',
    x: linePoints.map((p) => p.x * xUnit.scale + xUnit.offset),
    y: linePoints.map((p) => p.y * yUnit.scale + yUnit.offset),
    line: { color, width: 2 },
    showlegend: false,
    hoverinfo: 'none',
  }
}

function stateTraces(
  overlay: StateOverlay,
  states: StateTable,
  colors: ThemeColors,
  xUnit: UnitChoice,
  yUnit: UnitChoice,
  customStateColor?: string,
  cyclePath?: Record<string, number>[],
): PlotlyTrace[] {
  const points = statesForAxes(states, overlay.xProperty, overlay.yProperty)
  if (points.length === 0) return []

  const traces: PlotlyTrace[] = []
  const stateColor = customStateColor || colors.states

  if (overlay.connect) {
    const connection = connectionTrace(overlay, points, cyclePath, stateColor, xUnit, yUnit)
    if (connection) {
      traces.push(connection)
    }
  }

  traces.push({
    type: 'scatter',
    mode: 'markers+text',
    name: 'States',
    uid: 'states',
    x: points.map((p) => p.x * xUnit.scale + xUnit.offset),
    y: points.map((p) => p.y * yUnit.scale + yUnit.offset),
    marker: { color: stateColor, size: 9 },
    text: points.map((p) => String(p.index)),
    textposition: 'top right',
    textfont: { color: stateColor },
    showlegend: true,
  })

  return traces
}

export function buildPropertyFigure(
  diagram: DiagramResponse,
  config: PropertyConfig,
  format: PlotFormat,
  states: StateTable,
  theme: PlotTheme,
  cyclePath?: Record<string, number>[],
  revision?: string | number,
): PlotlyFigure {
  const colors = THEMES[theme]
  const xUnit = resolveUnit(diagram.xProperty, format.xUnit, format.celsius)
  const yUnit = resolveUnit(diagram.yProperty, format.yUnit, format.celsius)

  const traces: PlotlyTrace[] = []
  for (const curve of diagram.isolines) {
    if (curve.family === 'quality' && !config.quality) continue
    if (curve.family !== 'quality' && !config.isolines) continue
    const style = FAMILY_STYLES[curve.family] ?? FAMILY_STYLES.isobar
    const trace = curveTrace(curve, style, xUnit.scale, xUnit.offset, yUnit.scale, yUnit.offset)
    trace.uid = `${curve.family}_${curve.label}`
    traces.push(trace)
  }
  for (const dome of diagram.dome) {
    traces.push({
      ...curveTrace(
        dome,
        { color: colors.dome, width: 2.5, dash: 'solid' },
        xUnit.scale,
        xUnit.offset,
        yUnit.scale,
        yUnit.offset,
      ),
      uid: 'dome',
      showlegend: true,
    })
  }
  if (diagram.markers && diagram.markers.length > 0) {
    traces.push({
      type: 'scatter',
      mode: 'markers+text',
      name: 'Critical point',
      uid: 'critical_markers',
      x: diagram.markers.map((m) => m.x * xUnit.scale + xUnit.offset),
      y: diagram.markers.map((m) => m.y * yUnit.scale + yUnit.offset),
      marker: {
        symbol: 'diamond',
        size: 9,
        color: colors.dome,
      },
      text: diagram.markers.map((m) => m.label),
      textposition: 'top center',
      textfont: { color: colors.font, size: 10 },
      showlegend: true,
    })
  }
  if (config.overlayStates) {
    const customStateColor = format.lineColors?.['states']
    traces.push(
      ...stateTraces(
        {
          xProperty: diagram.xProperty,
          yProperty: diagram.yProperty,
          connect: config.connectStates,
          close: config.closeCycle,
        },
        states,
        colors,
        xUnit,
        yUnit,
        customStateColor,
        cyclePath,
      ),
    )
  }

  const layout = baseLayout(
    format,
    axisTitle(diagram.xProperty, xUnit),
    axisTitle(diagram.yProperty, yUnit),
    format.xLog ?? diagram.xLog,
    format.yLog ?? diagram.yLog,
    theme,
    revision,
  )
  layout.title ??= { text: `${diagram.fluid}` }
  return { data: traces, layout }
}

export function buildPsychroFigure(
  chart: PsychartResponse,
  config: PsychroConfig,
  format: PlotFormat,
  states: StateTable,
  theme: PlotTheme,
  cyclePath?: Record<string, number>[],
  revision?: string | number,
): PlotlyFigure {
  const colors = THEMES[theme]
  const xUnit = resolveUnit('T', format.xUnit, format.celsius)
  const yUnit = resolveUnit('w', format.yUnit, false)
  const traces: PlotlyTrace[] = []
  chart.curves.forEach((curve, i) => {
    if (curve.family === 'wetbulb' && !config.wetBulb) return
    if (curve.family === 'enthalpy' && !config.enthalpy) return
    if (curve.family === 'volume' && !config.volume) return
    const saturation = curve.family === 'saturation'
    const style: FamilyStyle = saturation
      ? { color: colors.dome, width: 2.5, dash: 'solid' }
      : (FAMILY_STYLES[curve.family] ?? FAMILY_STYLES.rh)
    const trace = curveTrace(curve, style, xUnit.scale, xUnit.offset, yUnit.scale, yUnit.offset)
    trace.uid = `${curve.family}_${i}`
    trace.showlegend = saturation
    traces.push(trace)
  })
  if (config.overlayStates) {
    const customStateColor = format.lineColors?.['states']
    traces.push(
      ...stateTraces(
        { xProperty: 'T', yProperty: 'w', connect: config.connectStates, close: false },
        states,
        colors,
        xUnit,
        yUnit,
        customStateColor,
        cyclePath,
      ),
    )
  }
  const layout = baseLayout(
    format,
    `Dry-bulb temperature [${xUnit.label}]`,
    axisTitle('w', yUnit),
    format.xLog ?? false,
    format.yLog ?? false,
    theme,
    revision,
  )
  layout.title ??= {
    text: `Psychrometric chart — ${(chart.pressure / 1000).toFixed(2)} kPa`,
  }
  return { data: traces, layout }
}

export interface XYSeries {
  sampleIds?: string[]
  name: string
  x: number[]
  y: number[]
  z?: number[]
  size?: number[]
  /** Which Y axis the series belongs to; 'y2' is the secondary right axis. */
  axis?: 'y' | 'y2'
}

function scaleSizes(values: number[]): number[] {
  const min = Math.min(...values)
  const max = Math.max(...values)
  if (max === min) return values.map(() => 15)
  return values.map((v) => 8 + ((v - min) / (max - min)) * 32)
}

export function buildXYFigure(
  series: XYSeries[],
  format: PlotFormat,
  xLabel: string,
  yLabel: string,
  theme: PlotTheme,
  config?: XYConfig,
  revision?: string | number,
): PlotlyFigure {
  const chartType = config?.chartType || 'line'
  const traces: PlotlyTrace[] = []
  const counts = series.map((s) => {
    const valid = s.y.filter((y, i) => Number.isFinite(y) && (chartType === 'histogram' || Number.isFinite(s.x[i]))).length
    return `${displayVar(s.name)}: ${valid} valid / ${s.y.length - valid} skipped`
  })
  if (chartType !== 'line') series = series.map((s) => {
    const indices = s.y.flatMap((y, i) => Number.isFinite(y) && (chartType === 'histogram' || Number.isFinite(s.x[i])) ? [i] : [])
    return { ...s, x: indices.map((i) => s.x[i]), y: indices.map((i) => s.y[i]), z: s.z && indices.map((i) => s.z![i]), size: s.size && indices.map((i) => s.size![i]), sampleIds: s.sampleIds && indices.map((i) => s.sampleIds![i]) }
  })

  if (chartType === 'pie') {
    if (series.length > 0) {
      const s = series[0]
      traces.push({
        type: 'pie',
        labels: s.x.map(String),
        values: s.y,
        name: displayVar(s.name),
        uid: s.name,
        textposition: 'inside',
        hoverinfo: 'label+value+percent',
      })
    }
  } else if (chartType === 'histogram') {
    series.forEach((s) => {
      traces.push({
        type: 'histogram',
        x: s.y,
        name: displayVar(s.name),
        uid: s.name,
        opacity: 0.75,
        marker: { color: format.lineColors?.[s.name] || undefined },
      })
    })
  } else if (chartType === 'bar') {
    series.forEach((s) => {
      traces.push({
        type: 'bar',
        name: displayVar(s.name),
        uid: s.name,
        x: s.x,
        y: s.y,
        marker: { color: format.lineColors?.[s.name] || undefined },
        ...(s.axis === 'y2' ? { yaxis: 'y2' } : {}),
      } as PlotlyTrace)
    })
  } else if (chartType === 'scatter') {
    series.forEach((s) => {
      const markerSize = s.size && s.size.length > 0 ? scaleSizes(s.size) : 10
      const style = format.traceStyles?.[s.name]
      traces.push({
        type: 'scatter',
        mode: 'markers',
        name: displayVar(s.name),
        uid: s.name,
        x: s.x,
        y: s.y,
        marker: {
          size: markerSize,
          color: format.lineColors?.[s.name] || undefined,
          symbol: style?.markerSymbol || undefined,
        },
        ...(s.axis === 'y2' ? { yaxis: 'y2' } : {}),
      } as PlotlyTrace)
    })
  } else if (chartType === 'surface3d') {
    series.forEach((s) => {
      if (s.z && s.z.length >= 3 && s.x.some((x, i) =>
        i > 1 && (s.x[1] - s.x[0]) * (s.y[i] - s.y[0]) !== (s.y[1] - s.y[0]) * (x - s.x[0]))) {
        traces.push({
          type: 'mesh3d',
          name: displayVar(s.name),
          uid: s.name,
          x: s.x,
          y: s.y,
          z: s.z,
          intensity: s.z,
          colorscale: 'Viridis',
          opacity: 0.8,
        })
      }
    })
  } else {
    // Default: 'line'
    series.forEach((s) => {
      const style = format.traceStyles?.[s.name]
      traces.push({
        type: 'scatter',
        mode: 'lines+markers',
        connectgaps: false,
        customdata: s.sampleIds,
        name: displayVar(s.name),
        uid: s.name,
        x: s.x,
        y: s.y,
        line: {
          color: format.lineColors?.[s.name] || undefined,
          dash: style?.dash || undefined,
        },
        marker: style?.markerSymbol ? { symbol: style.markerSymbol } : undefined,
        ...(s.axis === 'y2' ? { yaxis: 'y2' } : {}),
      } as PlotlyTrace)
    })
  }

  const layout = baseLayout(
    format,
    chartType === 'histogram' ? 'Value' : (format.xLabel || xLabel),
    chartType === 'histogram' ? 'Frequency' : (format.yLabel || yLabel),
    chartType === 'histogram' ? false : (format.xLog ?? false),
    chartType === 'histogram' ? false : (format.yLog ?? false),
    theme,
    revision,
  )

  if (chartType !== 'line') layout.annotations = [{ text: counts.join('; '), xref: 'paper', yref: 'paper', x: 0, y: 1.08, showarrow: false }]

  if (format.annotations && format.annotations.length > 0) {
    layout.shapes = layout.shapes ?? []
    layout.annotations = layout.annotations ?? []
    for (const ann of format.annotations) {
      if (ann.type === 'hline') {
        layout.shapes.push({
          type: 'line',
          y0: ann.value,
          y1: ann.value,
          xref: 'paper',
          x0: 0,
          x1: 1,
          line: { color: ann.color || '#ff6b6b', dash: ann.dash || 'dash', width: 1.5 },
        })
        layout.annotations.push({
          text: ann.text,
          xref: 'paper',
          x: 1,
          y: ann.value,
          showarrow: false,
          xanchor: 'right',
          yanchor: 'bottom',
          font: { color: ann.color || '#ff6b6b', size: 11 },
        })
      } else if (ann.type === 'vline') {
        layout.shapes.push({
          type: 'line',
          x0: ann.value,
          x1: ann.value,
          yref: 'paper',
          y0: 0,
          y1: 1,
          line: { color: ann.color || '#4dabf7', dash: ann.dash || 'dash', width: 1.5 },
        })
        layout.annotations.push({
          text: ann.text,
          yref: 'paper',
          y: 1,
          x: ann.value,
          showarrow: false,
          xanchor: 'left',
          yanchor: 'top',
          font: { color: ann.color || '#4dabf7', size: 11 },
        })
      }
    }
  }

  if (chartType === 'bar') {
    layout.barmode = 'group'
  }

  // Secondary right Y axis for the series tagged 'y2'.
  if (traces.some((t) => t.yaxis === 'y2')) {
    const colors = THEMES[theme]
    const y2Names = series.filter((s) => s.axis === 'y2').map((s) => displayVar(s.name))
    layout.yaxis2 = {
      title: { text: format.y2Label || y2Names.join(', ') },
      overlaying: 'y',
      side: 'right',
      type: 'linear',
      color: colors.font,
      showgrid: false,
      exponentformat: 'power',
    }
    layout.margin = { ...layout.margin, r: 64 }
  }

  if (chartType === 'surface3d') {
    layout.scene = {
      xaxis: { title: format.xLabel || xLabel, color: THEMES[theme].font, gridcolor: THEMES[theme].grid },
      yaxis: { title: format.yLabel || yLabel, color: THEMES[theme].font, gridcolor: THEMES[theme].grid },
      zaxis: { title: config?.zVar || 'Z', color: THEMES[theme].font, gridcolor: THEMES[theme].grid },
    }
  }

  return { data: traces, layout }
}

export function buildBodeFigure(
  omega: number[],
  mag: number[],
  phase: number[],
  format: PlotFormat,
  theme: PlotTheme,
  revision?: string | number,
): PlotlyFigure {
  const colors = THEMES[theme]
  const background = theme === 'dark' ? 'rgba(0,0,0,0)' : '#ffffff'
  const traces: PlotlyTrace[] = [
    {
      type: 'scatter',
      mode: 'lines',
      name: 'Magnitude',
      uid: 'bode_magnitude',
      x: omega,
      y: mag,
      yaxis: 'y2',
      line: { color: '#4dabf7', width: 2 },
      showlegend: false,
    },
    {
      type: 'scatter',
      mode: 'lines',
      name: 'Phase',
      uid: 'bode_phase',
      x: omega,
      y: phase,
      yaxis: 'y',
      line: { color: '#ff6b6b', width: 2 },
      showlegend: false,
    },
  ]

  const layout: PlotlyLayout = {
    title: format.title ? { text: format.title } : undefined,
    uirevision: revision !== undefined ? `bode_${revision}` : 'bode',
    paper_bgcolor: background,
    plot_bgcolor: background,
    font: { color: colors.font, size: format.fontSize },
    margin: { t: format.title ? 48 : 24, r: 24, b: 56, l: 64 },
    xaxis: {
      title: format.xLabel || 'Frequency [rad/s]',
      type: 'log',
      color: colors.font,
      gridcolor: colors.grid,
      zerolinecolor: colors.zero,
      showgrid: format.grid,
    },
    yaxis: {
      title: 'Phase [deg]',
      domain: [0.0, 0.45],
      color: colors.font,
      gridcolor: colors.grid,
      zerolinecolor: colors.zero,
      showgrid: format.grid,
    },
    yaxis2: {
      title: 'Magnitude [dB]',
      domain: [0.55, 1.0],
      color: colors.font,
      gridcolor: colors.grid,
      zerolinecolor: colors.zero,
      showgrid: format.grid,
    },
    showlegend: false,
  }

  return { data: traces, layout }
}

/** Open-loop gain (dB) along a constant closed-loop magnitude (M) contour. */
function nicholsMLocus(mDb: number): { x: (number | null)[]; y: (number | null)[] } {
  const m = Math.pow(10, mDb / 20)
  const x: (number | null)[] = []
  const y: (number | null)[] = []
  for (let deg = -359.5; deg <= -0.5; deg += 0.5) {
    const th = (deg * Math.PI) / 180
    const cos = Math.cos(th)
    let a: number | null = null
    if (Math.abs(1 - m * m) < 1e-9) {
      if (cos < -1e-6) a = -1 / (2 * cos)
    } else {
      const disc = 1 - m * m * Math.sin(th) * Math.sin(th)
      if (disc >= 0) {
        const sq = m * Math.sqrt(disc)
        const denom = 1 - m * m
        const a1 = (m * m * cos + sq) / denom
        const a2 = (m * m * cos - sq) / denom
        a = a1 > 0 ? a1 : a2 > 0 ? a2 : null
      }
    }
    pushNichols(x, y, deg, a)
  }
  return { x, y }
}

/** Open-loop gain (dB) along a constant closed-loop phase (N) contour. */
function nicholsNLocus(alphaDeg: number): { x: (number | null)[]; y: (number | null)[] } {
  const x: (number | null)[] = []
  const y: (number | null)[] = []
  for (let deg = -359.5; deg <= -0.5; deg += 0.5) {
    const th = (deg * Math.PI) / 180
    const tanPhi = Math.tan(th - (alphaDeg * Math.PI) / 180)
    const denom = Math.sin(th) - tanPhi * Math.cos(th)
    const a = Math.abs(denom) > 1e-9 ? tanPhi / denom : null
    pushNichols(x, y, deg, a && a > 0 ? a : null)
  }
  return { x, y }
}

function pushNichols(x: (number | null)[], y: (number | null)[], deg: number, a: number | null) {
  if (a !== null && a > 0 && Number.isFinite(a)) {
    const db = 20 * Math.log10(a)
    if (db >= -40 && db <= 40) {
      x.push(deg)
      y.push(db)
      return
    }
  }
  x.push(null)
  y.push(null)
}

const NICHOLS_M_DB = [6, 3, 1, 0.5, 0, -1, -3, -6, -12, -20]
const NICHOLS_N_DEG = [-5, -30, -60, -90, -120, -150, -180, -210, -240, -270, -300, -330, -355]

/**
 * Nichols chart: open-loop magnitude (dB) versus phase (deg), parametric in
 * frequency, overlaid on the standard M (closed-loop magnitude) and N
 * (closed-loop phase) grid.
 */
export function buildNicholsFigure(
  mag: number[],
  phase: number[],
  format: PlotFormat,
  theme: PlotTheme,
  revision?: string | number,
): PlotlyFigure {
  const gridColor = theme === 'dark' ? 'rgba(255,255,255,0.18)' : 'rgba(0,0,0,0.15)'

  const traces: PlotlyTrace[] = []
  for (const mDb of NICHOLS_M_DB) {
    const locus = nicholsMLocus(mDb)
    traces.push({
      type: 'scatter', mode: 'lines', name: `M ${mDb} dB`, uid: `nichols_m_${mDb}`, x: locus.x, y: locus.y,
      line: { color: gridColor, width: 1 }, hoverinfo: 'skip', showlegend: false,
    })
  }
  for (const aDeg of NICHOLS_N_DEG) {
    const locus = nicholsNLocus(aDeg)
    traces.push({
      type: 'scatter', mode: 'lines', name: `N ${aDeg}°`, uid: `nichols_n_${aDeg}`, x: locus.x, y: locus.y,
      line: { color: gridColor, width: 1, dash: 'dot' }, hoverinfo: 'skip', showlegend: false,
    })
  }
  // The −1 critical point sits at (−180 deg, 0 dB).
  traces.push({
    type: 'scatter', mode: 'markers', name: 'Critical point', uid: 'nichols_critical', x: [-180], y: [0],
    marker: { color: '#ff6b6b', size: 9, symbol: 'x' }, hoverinfo: 'skip', showlegend: false,
  })
  // The open-loop locus.
  traces.push({
    type: 'scatter', mode: 'lines+markers', name: 'Open loop', uid: 'nichols_open_loop',
    x: phase, y: mag,
    line: { color: '#4dabf7', width: 2 }, marker: { color: '#4dabf7', size: 4 },
    showlegend: false,
  })

  const layout = controlAxesLayout(format, theme, {
    xLabel: 'Open-loop phase [deg]',
    yLabel: 'Open-loop gain [dB]',
    legend: false,
  }, revision)

  return { data: traces, layout }
}

export function buildNyquistFigure(
  real: number[],
  imag: number[],
  format: PlotFormat,
  theme: PlotTheme,
  revision?: string | number,
): PlotlyFigure {
  const traces: PlotlyTrace[] = [
    {
      type: 'scatter',
      mode: 'lines',
      name: 'Nyquist Curve',
      uid: 'nyquist_curve',
      x: real,
      y: imag,
      line: { color: '#38d9a9', width: 2 },
      hoverinfo: 'x+y',
    },
    {
      type: 'scatter',
      mode: 'markers',
      name: 'Critical Point (-1+j0)',
      uid: 'nyquist_critical',
      x: [-1.0],
      y: [0.0],
      marker: { symbol: 'x', color: '#ff6b6b', size: 12 },
      hoverinfo: 'name',
    },
  ]

  const layout = controlAxesLayout(format, theme, {
    xLabel: 'Real Axis',
    yLabel: 'Imaginary Axis',
    squareAspect: true,
  }, revision)

  return { data: traces, layout }
}

export function buildPoleZeroFigure(
  pr: number[],
  pi: number[],
  zr: number[],
  zi: number[],
  format: PlotFormat,
  theme: PlotTheme,
  revision?: string | number,
): PlotlyFigure {
  const background = theme === 'dark' ? 'rgba(0,0,0,0)' : '#ffffff'
  const traces: PlotlyTrace[] = []

  if (pr.length > 0) {
    traces.push({
      type: 'scatter',
      mode: 'markers',
      name: 'Poles',
      uid: 'pz_poles',
      x: pr,
      y: pi,
      marker: { symbol: 'x', size: 10, color: '#ff6b6b' },
      hoverinfo: 'name+x+y',
    })
  }

  if (zr.length > 0) {
    traces.push({
      type: 'scatter',
      mode: 'markers',
      name: 'Zeros',
      uid: 'pz_zeros',
      x: zr,
      y: zi,
      marker: {
        symbol: 'circle',
        size: 10,
        color: background,
        line: { width: 2, color: '#4dabf7' },
      },
      hoverinfo: 'name+x+y',
    })
  }

  const layout = controlAxesLayout(format, theme, {
    xLabel: 'Real Axis [1/s]',
    yLabel: 'Imaginary Axis [rad/s]',
    squareAspect: true,
  }, revision)

  return { data: traces, layout }
}

export function buildRootLocusFigure(
  cpr: number[][],
  cpi: number[][],
  zr: number[],
  zi: number[],
  format: PlotFormat,
  theme: PlotTheme,
  revision?: string | number,
): PlotlyFigure {
  const background = theme === 'dark' ? 'rgba(0,0,0,0)' : '#ffffff'
  const traces: PlotlyTrace[] = []

  const M = cpr.length
  const N = M > 0 ? cpr[0].length : 0

  // 1. Draw trajectories for each branch
  for (let j = 0; j < N; j++) {
    const bx: number[] = []
    const by: number[] = []
    for (let i = 0; i < M; i++) {
      bx.push(cpr[i][j])
      by.push(cpi[i][j])
    }
    traces.push({
      type: 'scatter',
      mode: 'lines',
      name: `Branch ${j + 1}`,
      uid: `rl_branch_${j + 1}`,
      x: bx,
      y: by,
      line: { width: 2 },
      hoverinfo: 'name+x+y',
      showlegend: false,
    })
  }

  // 2. Draw open-loop poles (K=0) as 'x'
  if (N > 0) {
    const px: number[] = []
    const py: number[] = []
    for (let j = 0; j < N; j++) {
      px.push(cpr[0][j])
      py.push(cpi[0][j])
    }
    traces.push({
      type: 'scatter',
      mode: 'markers',
      name: 'Open-loop Poles',
      uid: 'rl_poles',
      x: px,
      y: py,
      marker: { symbol: 'x', size: 10, color: '#ff6b6b' },
      hoverinfo: 'name+x+y',
    })
  }

  // 3. Draw open-loop zeros as 'o'
  if (zr.length > 0) {
    traces.push({
      type: 'scatter',
      mode: 'markers',
      name: 'Open-loop Zeros',
      uid: 'rl_zeros',
      x: zr,
      y: zi,
      marker: {
        symbol: 'circle',
        size: 10,
        color: background,
        line: { width: 2, color: '#4dabf7' },
      },
      hoverinfo: 'name+x+y',
    })
  }

  const layout = controlAxesLayout(format, theme, {
    xLabel: 'Real Axis [1/s]',
    yLabel: 'Imaginary Axis [rad/s]',
    squareAspect: true,
  }, revision)

  return { data: traces, layout }
}

