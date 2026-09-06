import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { MantineProvider } from '@mantine/core'
import PlotlyChart from './PlotlyChart'
import type { PlotlyFigure } from 'plotly.js/lib/core'

const mockReact = vi.fn()
const mockPurge = vi.fn()
const mockResize = vi.fn()

vi.mock('./plotlyBundle', () => ({
  default: {
    react: (...args: unknown[]) => mockReact(...args),
    purge: (...args: unknown[]) => mockPurge(...args),
    Plots: {
      resize: (...args: unknown[]) => mockResize(...args),
    },
  },
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

describe('PlotlyChart error handling and lifecycle', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  const dummyFigure: PlotlyFigure = {
    data: [],
    layout: {},
  }

  it('renders error boundary with Retry button on render failure', async () => {
    mockReact.mockRejectedValueOnce(new Error('WebGL context unavailable'))

    render(
      <MantineProvider>
        <PlotlyChart figure={dummyFigure} />
      </MantineProvider>,
    )

    await waitFor(() => {
      expect(screen.getByText('Failed to render plot')).toBeInTheDocument()
      expect(screen.getByText('WebGL context unavailable')).toBeInTheDocument()
    })

    const retryBtn = screen.getByRole('button', { name: 'Retry' })
    expect(retryBtn).toBeInTheDocument()

    // When retrying, mock succeeds
    mockReact.mockResolvedValueOnce(undefined)
    fireEvent.click(retryBtn)

    await waitFor(() => {
      expect(screen.queryByText('Failed to render plot')).not.toBeInTheDocument()
      expect(mockReact).toHaveBeenCalledTimes(2)
    })
  })

  it('handles webglcontextlost and purges on unmount', async () => {
    mockReact.mockResolvedValue(undefined)

    const { unmount } = render(
      <MantineProvider>
        <PlotlyChart figure={dummyFigure} />
      </MantineProvider>,
    )

    await waitFor(() => {
      expect(mockReact).toHaveBeenCalledTimes(1)
    })

    const chartEl = screen.getByTestId('plotly-chart')
    expect(chartEl).toBeInTheDocument()

    // Dispatch webglcontextlost
    const event = new Event('webglcontextlost', { bubbles: true, cancelable: true })
    chartEl.dispatchEvent(event)

    await waitFor(() => {
      expect(mockReact).toHaveBeenCalledTimes(2)
    })

    unmount()
    await waitFor(() => {
      expect(mockPurge).toHaveBeenCalledWith(chartEl)
    })
  })
})
