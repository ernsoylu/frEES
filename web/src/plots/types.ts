/** Plot model shared by the Plots tab, the config modal and exports. */

export type PlotKind = 'xy' | 'property' | 'psychro' | 'bode' | 'nyquist' | 'nichols' | 'polezero' | 'rootlocus'

export const DIAGRAM_TYPES = [
  { value: 'T-s', label: 'T-s (temperature–entropy)' },
  { value: 'P-h', label: 'log P-h (pressure–enthalpy)' },
  { value: 'P-v', label: 'P-v (pressure–volume)' },
  { value: 'T-v', label: 'T-v (temperature–volume)' },
  { value: 'h-s', label: 'h-s (Mollier)' },
  { value: 'P-T', label: 'P-T (saturation curve)' },
]

/** Axis properties of each property diagram type. */
export function diagramAxes(diagram: string): { x: string; y: string } {
  switch (diagram) {
    case 'P-h':
      return { x: 'h', y: 'P' }
    case 'P-v':
      return { x: 'v', y: 'P' }
    case 'T-v':
      return { x: 'v', y: 'T' }
    case 'h-s':
      return { x: 's', y: 'h' }
    case 'P-T':
      return { x: 'T', y: 'P' }
    default:
      return { x: 's', y: 'T' }
  }
}

/** Presentation options shared by every plot kind. */
export interface PlotFormat {
  title: string
  xLabel: string
  yLabel: string
  /** Display unit ids per axis (see plots/units.ts); null = default. */
  xUnit: string | null
  yUnit: string | null
  xLog: boolean | null
  yLog: boolean | null
  grid: boolean
  legend: boolean
  /** XY plots: append each axis variable's unit (e.g. "Vair [m^3/s]") to the
   *  axis title, taken from the solved variable units. Defaults to on. */
  showUnits?: boolean
  legendAlign?: 'left' | 'center' | 'right'
  /** Title of the secondary (right) Y axis used by XYConfig.y2Vars. */
  y2Label?: string
  fontSize: number
  celsius: boolean
  xMin?: number | null
  xMax?: number | null
  yMin?: number | null
  yMax?: number | null
  xTick?: number | null
  yTick?: number | null
  lineColors?: Record<string, string>
}

export type ChartType = 'line' | 'bar' | 'pie' | 'histogram' | 'scatter' | 'surface3d'

export interface XYConfig {
  xVar: string | null
  yVars: string[]
  /** Variables plotted against a secondary (right) Y axis. */
  y2Vars?: string[]
  chartType?: ChartType
  zVar?: string | null
  sizeVar?: string | null
}

export interface PropertyConfig {
  fluid: string
  diagram: string
  quality: boolean
  isolines: boolean
  overlayStates: boolean
  connectStates: boolean
  closeCycle: boolean
  /** Name of the declared STATE TABLE block whose states to overlay; null
   * overlays every detected state. Selecting one also sets the fluid. */
  stateTable?: string | null
}

export interface PsychroConfig {
  pressureKPa: number
  tMinC: number
  tMaxC: number
  wetBulb: boolean
  enthalpy: boolean
  volume: boolean
  overlayStates: boolean
  connectStates: boolean
  /** Name of the declared STATE TABLE block whose states to overlay; null
   * overlays every detected state. */
  stateTable?: string | null
}

export interface ControlConfig {
  omega: string | null
  mag: string | null
  phase: string | null
  real: string | null
  imag: string | null
  pr: string | null
  pi: string | null
  zr: string | null
  zi: string | null
}

export type PlotSource = { kind: 'arrays' } | { kind: 'table'; tableId: string; data: 'inputs' | 'solved' }

export interface PlotSpec {
  source?: PlotSource
  resultRevision?: number
  id: string
  name: string
  kind: PlotKind
  xy: XYConfig
  property: PropertyConfig
  psychro: PsychroConfig
  control: ControlConfig
  format: PlotFormat
  /** True when the plot was declared in the editor text with a PLOT ... END
   * block rather than created in the GUI. Code plots are regenerated on every
   * solve and never persisted with the project. */
  fromCode?: boolean
}

export function defaultFormat(kind: PlotKind): PlotFormat {
  return {
    title: '',
    xLabel: '',
    yLabel: '',
    xUnit: null,
    yUnit: null,
    xLog: null,
    yLog: null,
    grid: true,
    legend: true,
    showUnits: true,
    fontSize: 13,
    celsius: kind === 'psychro',
    xMin: null,
    xMax: null,
    yMin: null,
    yMax: null,
    xTick: null,
    yTick: null,
    lineColors: {},
  }
}

export function newPlotSpec(kind: PlotKind, name: string): PlotSpec {
  return {
    id: crypto.randomUUID(),
    name,
    kind,
    xy: { xVar: null, yVars: [], chartType: 'line', zVar: null, sizeVar: null },
    property: {
      fluid: 'Water',
      diagram: 'T-s',
      quality: true,
      isolines: true,
      overlayStates: true,
      connectStates: true,
      closeCycle: false,
      stateTable: null,
    },
    psychro: {
      pressureKPa: 101.325,
      tMinC: 0,
      tMaxC: 50,
      wetBulb: true,
      enthalpy: true,
      volume: false,
      overlayStates: true,
      connectStates: false,
      stateTable: null,
    },
    control: {
      omega: null,
      mag: null,
      phase: null,
      real: null,
      imag: null,
      pr: null,
      pi: null,
      zr: null,
      zi: null,
    },
    format: defaultFormat(kind),
  }
}

/** A GUI snapshot of a code plot, with a collision-free name. */
export function editablePlotCopy(spec: PlotSpec, plots: PlotSpec[]): PlotSpec {
  const occupied = new Set(plots.map((p) => p.name.toLowerCase()))
  const base = `${spec.name}_copy`
  let name = base
  for (let i = 2; occupied.has(name.toLowerCase()); i++) name = `${base}${i}`
  return { ...spec, id: crypto.randomUUID(), name, fromCode: false }
}
