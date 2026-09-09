// Shared pieces of the curve-fit UI, used by CurveFitModal (Calculate >
// Curve Fit) and the Digitizer's "Fit curve…" dialog (Wave H): the model
// template list and the fitted-model editor text. The shared result
// rendering lives beside this file in FitResultView.tsx (components only,
// for react-refresh). Extracted rather than duplicated so the two fit
// surfaces cannot drift.

import type { PlotlyFigure } from 'plotly.js/lib/core'
import { formatValue } from './format'
import type { CurveFitResponse } from './api'

export const MONO_INPUT = {
  input: { fontFamily: 'var(--mantine-font-family-monospace)' },
}

export interface FitTemplate {
  name: string
  equation: string
  yVariable: string
  xVariable: string
  parameters: string
}

export const FIT_TEMPLATES: FitTemplate[] = [
  {
    name: 'Linear (y = a * x + b)',
    equation: 'y = a * x + b',
    yVariable: 'y',
    xVariable: 'x',
    parameters: 'a, b',
  },
  {
    name: 'Quadratic (y = a * x^2 + b * x + c)',
    equation: 'y = a * x^2 + b * x + c',
    yVariable: 'y',
    xVariable: 'x',
    parameters: 'a, b, c',
  },
  {
    name: 'Cubic (y = a * x^3 + b * x^2 + c * x + d)',
    equation: 'y = a * x^3 + b * x^2 + c * x + d',
    yVariable: 'y',
    xVariable: 'x',
    parameters: 'a, b, c, d',
  },
  {
    name: 'Exponential (y = a * exp(b * x))',
    equation: 'y = a * exp(b * x)',
    yVariable: 'y',
    xVariable: 'x',
    parameters: 'a, b',
  },
  {
    name: 'Exponential with offset (y = a * exp(b * x) + c)',
    equation: 'y = a * exp(b * x) + c',
    yVariable: 'y',
    xVariable: 'x',
    parameters: 'a, b, c',
  },
  {
    name: 'Logarithmic (y = a * ln(x) + b)',
    equation: 'y = a * ln(x) + b',
    yVariable: 'y',
    xVariable: 'x',
    parameters: 'a, b',
  },
  {
    name: 'Power (y = a * x^b)',
    equation: 'y = a * x^b',
    yVariable: 'y',
    xVariable: 'x',
    parameters: 'a, b',
  },
  {
    name: 'Power with offset (y = a * x^b + c)',
    equation: 'y = a * x^b + c',
    yVariable: 'y',
    xVariable: 'x',
    parameters: 'a, b, c',
  },
]

/** A template's model equation rewritten to the caller's variable names
 * (used by the digitizer fit, whose axes carry their own names). Single-pass
 * whole-word substitution so `y → x` / `x → y` swaps cannot cascade. */
export function templateModelFor(template: FitTemplate, xVar: string, yVar: string): string {
  return template.equation.replace(/\b[xy]\b/g, (m) => (m === 'x' ? xVar : yVar))
}

/** The editor text "Insert equation"/"Copy to Editor" produces: a comment
 * naming the template, one `param = value [unit]` line per fitted parameter,
 * then the model equation itself. */
export function fittedModelInsertText(
  templateKey: string,
  model: string,
  parameterNames: readonly string[],
  fittedParameters: readonly number[],
  parameterUnits: Record<string, string> = {},
): string {
  const paramEquations = parameterNames.map((name, i) => {
    const val = formatValue(fittedParameters[i])
    const unit = parameterUnits[name]?.trim()
    const unitStr = unit ? ` [${unit}]` : ''
    return `${name} = ${val}${unitStr}`
  })
  return [
    `{ Fitted Model: ${templateKey === 'custom' ? 'Custom' : templateKey} }`,
    ...paramEquations,
    model.trim(),
  ].join('\n')
}

/** The robust losses `curve_fit` accepts, in increasing order of how hard they
 *  push outliers away. `linear` is ordinary least squares. */
export const LOSS_OPTIONS = [
  { value: 'linear', label: 'Least squares (no outlier handling)' },
  { value: 'soft_l1', label: 'Soft L1 (mild)' },
  { value: 'huber', label: 'Huber (moderate)' },
  { value: 'cauchy', label: 'Cauchy (aggressive)' },
] as const

export type LossName = (typeof LOSS_OPTIONS)[number]['value']

/** A comma-separated numeric list, or an error message naming what is wrong.
 *  Empty input is `undefined` — the field is optional, not invalid. */
export function parseNumberList(
  raw: string,
  expected: number,
  label: string,
): number[] | undefined | string {
  if (raw.trim() === '') return undefined
  const values = raw.split(',').map((v) => Number(v.trim()))
  if (values.some((v) => !Number.isFinite(v))) {
    return `${label} must be numbers, comma-separated.`
  }
  if (values.length !== expected) {
    return `${label} needs one value per parameter (got ${values.length}, expected ${expected}).`
  }
  return values
}

/** Per-point sigmas parsed from the optional third column of the manual data
 *  box. All must be positive — the engine refuses zero and negative, and
 *  saying so here costs a round trip less. */
export function validateSigma(sigma: number[] | undefined): string | null {
  if (!sigma) return null
  if (sigma.some((s) => !(s > 0) || !Number.isFinite(s))) {
    return 'Uncertainties must be finite and greater than zero.'
  }
  return null
}

/** Points sorted by x, so a band drawn as a filled ribbon does not zig-zag
 *  back across itself when the data arrives unsorted. */
function sortedByX(
  x: readonly number[],
  ...series: readonly (readonly (number | null)[])[]
): { x: number[]; series: (number | null)[][] } {
  const order = x.map((_, i) => i).sort((a, b) => x[a] - x[b])
  return {
    x: order.map((i) => x[i]),
    series: series.map((s) => order.map((i) => s[i] ?? null)),
  }
}

/** The fit picture: observed points, the fitted curve, and the confidence and
 *  prediction ribbons. Returns `null` when there is nothing to draw.
 *
 *  The bands come back `null` per point when the fit could not support them
 *  (rank-deficient, or no residual degrees of freedom), and a ribbon is simply
 *  omitted in that case rather than drawn flat against the curve. */
export function buildFitFigure(
  result: CurveFitResponse,
  xData: readonly number[],
  xLabel: string,
  yLabel: string,
  observed: readonly number[],
): PlotlyFigure | null {
  if (!result.success || xData.length === 0 || result.fittedValues.length !== xData.length) {
    return null
  }
  const { x, series } = sortedByX(
    xData,
    observed,
    result.fittedValues,
    result.confidenceBandLo,
    result.confidenceBandHi,
    result.predictionBandLo,
    result.predictionBandHi,
  )
  const [obs, fitted, ciLo, ciHi, piLo, piHi] = series
  const hasBand = (lo: (number | null)[]) => lo.some((v) => v !== null)

  const ribbon = (
    lo: (number | null)[],
    hi: (number | null)[],
    name: string,
    color: string,
  ) => [
    { x, y: hi, type: 'scatter', mode: 'lines', name, line: { width: 0 }, showlegend: false,
      hoverinfo: 'skip' },
    { x, y: lo, type: 'scatter', mode: 'lines', name, line: { width: 0 }, fill: 'tonexty',
      fillcolor: color, hoverinfo: 'skip' },
  ]

  const data = [
    ...(hasBand(piLo) ? ribbon(piLo, piHi, 'Prediction band', 'rgba(77, 171, 247, 0.13)') : []),
    ...(hasBand(ciLo) ? ribbon(ciLo, ciHi, 'Confidence band', 'rgba(77, 171, 247, 0.28)') : []),
    { x, y: fitted, type: 'scatter', mode: 'lines', name: 'Fit',
      line: { color: '#4dabf7', width: 2 } },
    { x, y: obs, type: 'scatter', mode: 'markers', name: 'Observed',
      marker: { color: '#f783ac', size: 7 } },
  ]

  return {
    data,
    layout: {
      margin: { l: 56, r: 16, t: 8, b: 44 },
      xaxis: { title: { text: xLabel } },
      yaxis: { title: { text: yLabel } },
      showlegend: true,
      legend: { orientation: 'h', y: -0.22 },
    },
  } as PlotlyFigure
}
