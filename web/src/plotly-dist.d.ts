/**
 * Minimal typings for the custom Plotly partial bundle (plots/plotlyBundle.ts).
 * plotly.js ships no type definitions for its lib/ entry points.
 */
declare module 'plotly.js/lib/core' {
  export interface PlotlyLineStyle {
    color?: string
    width?: number
    dash?: 'solid' | 'dot' | 'dash' | 'dashdot'
  }

  export interface PlotlyMarkerStyle {
    color?: string
    size?: number | number[]
    symbol?: string
    line?: { width?: number; color?: string }
  }

  export interface PlotlyTrace {
    type: 'scatter' | 'bar' | 'pie' | 'histogram' | 'mesh3d' | 'scatter3d'
    mode?: string
    name: string
    x?: (number | null)[]
    y?: (number | null)[]
    z?: (number | null)[]
    labels?: string[]
    values?: number[]
    intensity?: (number | null)[]
    colorscale?: string
    opacity?: number
    line?: PlotlyLineStyle
    marker?: PlotlyMarkerStyle
    text?: string[]
    textposition?: string
    textfont?: { color?: string; size?: number }
    showlegend?: boolean
    hoverinfo?: string
    connectgaps?: boolean
    yaxis?: string
    /** Histogram bin-count hint (histogram plots, Monte Carlo). */
    nbinsx?: number
    uid?: string
    customdata?: (string | number)[]
  }

  export interface PlotlyAxisLayout {
    title?: { text: string } | string
    type?: 'linear' | 'log'
    gridcolor?: string
    zerolinecolor?: string
    color?: string
    showgrid?: boolean
    linecolor?: string
    mirror?: boolean
    ticks?: 'outside' | 'inside' | ''
    exponentformat?: 'power' | 'e' | 'none'
    domain?: number[]
    scaleanchor?: string
    range?: (number | null)[]
    autorange?: boolean | 'reversed'
    dtick?: number
    overlaying?: string
    side?: 'left' | 'right' | 'top' | 'bottom'
  }

  export interface PlotlyLayout {
    uirevision?: string | number | boolean
    annotations?: Record<string, unknown>[]
    title?: { text: string; font?: { size?: number } }
    paper_bgcolor?: string
    plot_bgcolor?: string
    font?: { color?: string; family?: string; size?: number }
    margin?: { t?: number; r?: number; b?: number; l?: number }
    xaxis?: PlotlyAxisLayout
    yaxis?: PlotlyAxisLayout
    yaxis2?: PlotlyAxisLayout
    showlegend?: boolean
    legend?: {
      orientation?: 'h' | 'v'
      font?: { size?: number }
      bgcolor?: string
    }
    barmode?: 'group' | 'stack' | 'overlay' | 'relative'
    scene?: {
      xaxis?: PlotlyAxisLayout
      yaxis?: PlotlyAxisLayout
      zaxis?: PlotlyAxisLayout
    }
    /** Layout shapes (lines/rects); used for the embedded-chart playback marker. */
    shapes?: Record<string, unknown>[]
  }

  export interface PlotlyConfig {
    responsive?: boolean
    displaylogo?: boolean
  }

  export interface PlotlyFigure {
    data: PlotlyTrace[]
    layout: PlotlyLayout
  }

  export interface PlotlyImageOptions {
    format: 'svg' | 'png' | 'jpeg'
    width: number
    height: number
    scale?: number
  }

  const Plotly: {
    react: (
      el: HTMLElement,
      data: PlotlyTrace[],
      layout?: PlotlyLayout,
      config?: PlotlyConfig,
    ) => Promise<unknown>
    purge: (el: HTMLElement) => void
    /** Re-layout a plot to fit its container's current size. */
    Plots: { resize: (el: HTMLElement) => void }
    toImage: (
      figure: PlotlyFigure | HTMLElement,
      options: PlotlyImageOptions,
    ) => Promise<string>
    /** Register trace-type modules into the core bundle. */
    register: (modules: unknown[]) => void
  }
  export default Plotly
}

declare module 'plotly.js/lib/bar' {
  const traceModule: unknown
  export default traceModule
}
declare module 'plotly.js/lib/pie' {
  const traceModule: unknown
  export default traceModule
}
declare module 'plotly.js/lib/histogram' {
  const traceModule: unknown
  export default traceModule
}
declare module 'plotly.js/lib/mesh3d' {
  const traceModule: unknown
  export default traceModule
}
