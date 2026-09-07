import type { PlotlyFigure } from 'plotly.js/lib/core'
import { cleanPlotlyFigure } from './figure'

/**
 * Plot file export. SVG/PNG/JPG come straight from Plotly (raster at 4x
 * scale for print resolution). PDF/EPS are clipped (decision D5): they were
 * transcoded server-side by Apache FOP, which has no browser-engine
 * equivalent, and the api.ts stub could only reject. SVG covers the
 * fully-vector need client-side.
 */

export type ExportFormat = 'svg' | 'png' | 'jpg'

export const EXPORT_FORMATS: { value: ExportFormat; label: string }[] = [
  { value: 'svg', label: 'SVG (vector)' },
  { value: 'png', label: 'PNG (high resolution)' },
  { value: 'jpg', label: 'JPG (high resolution)' },
]

const EXPORT_WIDTH = 1200
const EXPORT_HEIGHT = 800
const RASTER_SCALE = 4

export interface ExportPlotOptions {
  /** 'current' preserves active layout; 'full' autoscales all axes to fit all data. */
  viewMode?: 'current' | 'full'
  /** Background color override (e.g. '#ffffff' for publication). */
  background?: string
  width?: number
  height?: number
}

export function prepareFigureForExport(
  figure: PlotlyFigure,
  options?: ExportPlotOptions,
): PlotlyFigure {
  const layout = { ...figure.layout }
  if (options?.background) {
    layout.paper_bgcolor = options.background
    layout.plot_bgcolor = options.background
  }
  if (options?.viewMode === 'full') {
    if (layout.xaxis) {
      layout.xaxis = { ...layout.xaxis, range: undefined, autorange: true }
    }
    if (layout.yaxis) {
      layout.yaxis = { ...layout.yaxis, range: undefined, autorange: true }
    }
    if (layout.yaxis2) {
      layout.yaxis2 = { ...layout.yaxis2, range: undefined, autorange: true }
    }
    if (layout.scene) {
      layout.scene = {
        ...layout.scene,
        xaxis: layout.scene.xaxis ? { ...layout.scene.xaxis, autorange: true } : undefined,
        yaxis: layout.scene.yaxis ? { ...layout.scene.yaxis, autorange: true } : undefined,
        zaxis: layout.scene.zaxis ? { ...layout.scene.zaxis, autorange: true } : undefined,
      }
    }
  }
  return cleanPlotlyFigure({
    data: figure.data,
    layout,
  })
}

async function figureToSvg(
  figure: PlotlyFigure,
  width = EXPORT_WIDTH,
  height = EXPORT_HEIGHT,
): Promise<string> {
  const { default: Plotly } = await import('./plotlyBundle')
  const url = await Plotly.toImage(figure, {
    format: 'svg',
    width,
    height,
  })
  // Plotly returns a data URL: data:image/svg+xml,<percent-encoded svg>
  return decodeURIComponent(url.substring(url.indexOf(',') + 1))
}

function download(href: string, filename: string) {
  const link = document.createElement('a')
  link.href = href
  link.download = filename
  document.body.appendChild(link)
  link.click()
  link.remove()
}

function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob)
  download(url, filename)
  URL.revokeObjectURL(url)
}

export async function exportPlot(
  figure: PlotlyFigure,
  format: ExportFormat,
  baseName: string,
  options?: ExportPlotOptions,
): Promise<void> {
  const fig = prepareFigureForExport(figure, options)
  const filename = `${baseName || 'plot'}.${format}`
  const width = options?.width ?? EXPORT_WIDTH
  const height = options?.height ?? EXPORT_HEIGHT

  if (format === 'svg') {
    const svg = await figureToSvg(fig, width, height)
    downloadBlob(new Blob([svg], { type: 'image/svg+xml' }), filename)
    return
  }
  const { default: Plotly } = await import('./plotlyBundle')
  const url = await Plotly.toImage(fig, {
    format: format === 'jpg' ? 'jpeg' : 'png',
    width,
    height,
    scale: RASTER_SCALE,
  })
  download(url, filename)
}
