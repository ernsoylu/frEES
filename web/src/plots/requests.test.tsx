import { renderHook } from '@testing-library/react'
import { expect, it, vi } from 'vitest'
import { useDiagramData } from './PlotCard'
import { newPlotSpec, type PlotKind } from './types'
import { getPropertyDiagram, getPsychrometricChart } from '../api'
vi.mock('../api', () => ({ getPropertyDiagram: vi.fn(), getPsychrometricChart: vi.fn() }))
it('all control kinds and XY issue zero thermo requests', () => {
  for (const kind of ['xy', 'bode', 'nyquist', 'nichols', 'polezero', 'rootlocus'] as PlotKind[]) {
    const hook = renderHook(() => useDiagramData(newPlotSpec(kind, kind)))
    hook.unmount()
  }
  expect(getPropertyDiagram).not.toHaveBeenCalled()
  expect(getPsychrometricChart).not.toHaveBeenCalled()
})
