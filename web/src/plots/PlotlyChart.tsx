import { useEffect, useRef, useState } from 'react'
import { Alert, Button, Stack, Text } from '@mantine/core'
import type { PlotlyFigure } from 'plotly.js/lib/core'

/**
 * Renders a pre-built Plotly figure. Plotly is loaded on demand so the
 * main bundle does not carry the charting library.
 */
export interface PlotPointClickEvent {
  traceIndex: number
  pointIndex: number
  x: number
  y: number
  sampleId?: string
  traceName?: string
}

export default function PlotlyChart({
  figure,
  minHeight = 380,
  onPointClick,
}: Readonly<{
  figure: PlotlyFigure
  minHeight?: number
  onPointClick?: (point: PlotPointClickEvent) => void
}>) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [renderError, setRenderError] = useState<string | null>(null)
  const [retryKey, setRetryKey] = useState(0)
  const renderSeq = useRef(0)
  const onPointClickRef = useRef(onPointClick)
  onPointClickRef.current = onPointClick

  useEffect(() => {
    let cancelled = false
    const seq = ++renderSeq.current
    async function render() {
      try {
        const { default: Plotly } = await import('./plotlyBundle')
        const el = containerRef.current
        if (cancelled || el === null || seq !== renderSeq.current) return
        await Plotly.react(el, figure.data, figure.layout, {
          responsive: true,
          displaylogo: false,
        })
        if (el) {
          const elAny = el as unknown as {
            removeAllListeners?: (event: string) => void
            on?: (event: string, handler: (data: unknown) => void) => void
          }
          elAny.removeAllListeners?.('plotly_click')
          elAny.on?.('plotly_click', (data: unknown) => {
            if (!onPointClickRef.current) return
            const d = data as {
              points?: Array<{
                curveNumber?: number
                pointIndex?: number
                x?: number
                y?: number
                customdata?: unknown
                data?: { name?: string; customdata?: unknown[] }
              }>
            }
            const pt = d?.points?.[0]
            if (!pt) return
            const sampleId =
              pt.customdata ??
              (Array.isArray(pt.data?.customdata) ? pt.data.customdata[pt.pointIndex ?? 0] : undefined)
            onPointClickRef.current({
              traceIndex: pt.curveNumber ?? 0,
              pointIndex: pt.pointIndex ?? 0,
              x: typeof pt.x === 'number' ? pt.x : Number(pt.x),
              y: typeof pt.y === 'number' ? pt.y : Number(pt.y),
              sampleId: typeof sampleId === 'string' ? sampleId : undefined,
              traceName: pt.data?.name || '',
            })
          })
        }
        if (!cancelled && seq === renderSeq.current) {
          setRenderError(null)
        }
      } catch (e: unknown) {
        if (!cancelled && seq === renderSeq.current) {
          setRenderError(e instanceof Error ? e.message : String(e))
        }
      }
    }
    void render()
    return () => {
      cancelled = true
    }
  }, [figure, retryKey])

  // Handle WebGL context loss for 3D plots without erasing user's configuration
  useEffect(() => {
    const el = containerRef.current
    if (!el) return
    const handleContextLost = (e: Event) => {
      e.preventDefault()
      // Trigger a re-render/retry to recreate the WebGL context
      setRetryKey((k) => k + 1)
    }
    el.addEventListener('webglcontextlost', handleContextLost, true)
    return () => {
      el.removeEventListener('webglcontextlost', handleContextLost, true)
    }
  }, [])

  // Plotly's `responsive` config only tracks the *window*; it does not react to
  // the panel/tile being resized within the dockview workspace. Observe the
  // container directly and resize the plot so it always fills its tile (no
  // scrollbars when shrunk, no empty margins when grown).
  useEffect(() => {
    const el = containerRef.current
    if (el === null) return
    let plotly: typeof import('./plotlyBundle').default | null = null
    let frame = 0
    void import('./plotlyBundle').then(({ default: P }) => {
      plotly = P
    })
    if (typeof ResizeObserver === 'undefined') return
    const observer = new ResizeObserver(() => {
      cancelAnimationFrame(frame)
      frame = requestAnimationFrame(() => {
        if (plotly && containerRef.current) {
          try {
            plotly.Plots.resize(containerRef.current)
          } catch {
            // ignore resize during transition/unmount
          }
        }
      })
    })
    observer.observe(el)
    return () => {
      observer.disconnect()
      cancelAnimationFrame(frame)
    }
  }, [])

  useEffect(() => {
    const el = containerRef.current
    return () => {
      if (el) {
        void import('./plotlyBundle').then(({ default: Plotly }) => {
          try {
            Plotly.purge(el)
          } catch {
            // ignore cleanup errors during unmount
          }
        })
      }
    }
  }, [])

  if (renderError) {
    return (
      <Stack align="center" justify="center" p="md" style={{ width: '100%', height: '100%', minHeight }}>
        <Alert color="red" title="Failed to render plot" style={{ maxWidth: 450 }}>
          <Text size="xs" mb="xs">
            {renderError}
          </Text>
          <Button
            size="xs"
            variant="outline"
            color="red"
            onClick={() => {
              setRenderError(null)
              setRetryKey((k) => k + 1)
            }}
          >
            Retry
          </Button>
        </Alert>
      </Stack>
    )
  }

  return (
    <div
      ref={containerRef}
      data-testid="plotly-chart"
      style={{ width: '100%', height: '100%', minHeight }}
    />
  )
}
