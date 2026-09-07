import { cleanup, fireEvent, render, screen, within } from '@testing-library/react'
import { MantineProvider } from '@mantine/core'
import { afterEach, expect, it, vi } from 'vitest'
import { DEFAULT_STOP_CRITERIA } from './api'
import MinMaxModal from './MinMaxModal'
import ParameterFitModal from './ParameterFitModal'

HTMLElement.prototype.scrollIntoView = () => {}

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  Reflect.deleteProperty(document, 'fonts')
})

it('opens analysis dialogs with invalid table drafts and reports validation on Run', async () => {
  vi.stubGlobal('matchMedia', () => ({ matches: false, addEventListener() {}, removeEventListener() {} }))
  vi.stubGlobal('ResizeObserver', class { observe() {} unobserve() {} disconnect() {} })
  Object.defineProperty(document, 'fonts', { configurable: true, value: new EventTarget() })
  const getFunctionTables = vi.fn(() => { throw new Error('lookup: Row 1, x: invalid number') })
  const shared = {
    text: 'y = x * x', stopCriteria: DEFAULT_STOP_CRITERIA,
    variableInfo: [], getFunctionTables, onClose: () => {},
  }
  const fit = render(<MantineProvider env="test">
    <ParameterFitModal {...shared} opened tables={[]} onApply={() => {}} />
  </MantineProvider>)
  expect(getFunctionTables).not.toHaveBeenCalled()
  fit.unmount()
  render(<MantineProvider env="test">
    <MinMaxModal {...shared} variables={['x', 'y']} complexMode={false} unitSystem="SI" />
  </MantineProvider>)
  expect(getFunctionTables).not.toHaveBeenCalled()
  fireEvent.click(screen.getByLabelText('Objective variable', { selector: 'input' }))
  fireEvent.click(within(screen.getByLabelText('Objective variable', { selector: '[role=listbox]' })).getByText('y'))
  fireEvent.click(screen.getByLabelText('Independent (varied) variables', { selector: 'input' }))
  fireEvent.click(within(screen.getByLabelText('Independent (varied) variables', { selector: '[role=listbox]' })).getByText('x'))
  fireEvent.change(screen.getByLabelText('Lower bound of x'), { target: { value: '-1' } })
  fireEvent.change(screen.getByLabelText('Upper bound of x'), { target: { value: '1' } })
  fireEvent.click(screen.getByRole('button', { name: 'Minimize', hidden: true }))
  expect(await screen.findByText('lookup: Row 1, x: invalid number')).toBeInTheDocument()
  expect(getFunctionTables).toHaveBeenCalledOnce()
}, 15_000)

it('supports interactive Pareto point click, inspection and loading into document', async () => {
  vi.stubGlobal('matchMedia', () => ({ matches: false, addEventListener() {}, removeEventListener() {} }))
  vi.stubGlobal('ResizeObserver', class { observe() {} unobserve() {} disconnect() {} })
  Object.defineProperty(document, 'fonts', { configurable: true, value: new EventTarget() })

  const onApply = vi.fn()
  const shared = {
    text: 'x = 1\ny = 2',
    stopCriteria: DEFAULT_STOP_CRITERIA,
    variableInfo: [],
    getFunctionTables: () => [],
    onClose: () => {},
    onApply,
  }

  const optimizeMultiSpy = vi.spyOn(await import('./api'), 'optimizeMulti').mockResolvedValue({
    success: true,
    error: null,
    decisionNames: ['x'],
    objectiveNames: ['f1', 'f2'],
    evaluations: 160,
    front: [
      { objectives: [1.5, 8.0], decisions: [2.5] },
      { objectives: [3.0, 4.0], decisions: [5.0] },
    ],
  })

  render(
    <MantineProvider env="test">
      <MinMaxModal {...shared} variables={['x', 'f1', 'f2']} complexMode={false} unitSystem="SI" />
    </MantineProvider>,
  )

  // Switch to Multi-objective mode
  fireEvent.click(screen.getByText('Multi-objective (Pareto)'))

  // Select objectives
  fireEvent.click(screen.getByLabelText('Objective variables (2 or more)', { selector: 'input' }))
  fireEvent.click(within(screen.getByLabelText('Objective variables (2 or more)', { selector: '[role=listbox]' })).getByText('f1'))
  fireEvent.click(within(screen.getByLabelText('Objective variables (2 or more)', { selector: '[role=listbox]' })).getByText('f2'))

  // Select decision variable
  fireEvent.click(screen.getByLabelText('Independent (varied) variables', { selector: 'input' }))
  fireEvent.click(within(screen.getByLabelText('Independent (varied) variables', { selector: '[role=listbox]' })).getByText('x'))

  // Set bounds
  fireEvent.change(screen.getByLabelText('Lower bound of x'), { target: { value: '0' } })
  fireEvent.change(screen.getByLabelText('Upper bound of x'), { target: { value: '10' } })

  // Click Find Pareto front
  fireEvent.click(screen.getByRole('button', { name: 'Find Pareto front' }))

  // Verify Pareto results are shown
  expect(await screen.findByText('2 Pareto-optimal points')).toBeInTheDocument()
  expect(screen.getByText('160 evaluations')).toBeInTheDocument()

  // Initially no point is selected
  expect(screen.queryByText(/Selected Point #/)).not.toBeInTheDocument()

  // Click row 1 in the table
  const pointRow = screen.getByText('2.5').closest('tr')!
  fireEvent.click(pointRow)

  // Now inspection card appears
  expect(await screen.findByText('Selected Point #1')).toBeInTheDocument()
  expect(screen.getByText('f1 = 1.5')).toBeInTheDocument()
  expect(screen.getByText('f2 = 8')).toBeInTheDocument()
  expect(screen.getByText('x = 2.5')).toBeInTheDocument()

  // Click Load Point into Document
  fireEvent.click(screen.getByRole('button', { name: 'Load Point into Document' }))
  expect(onApply).toHaveBeenCalledWith('x = 2.5\ny = 2')
  expect(await screen.findByText('✓ Applied to Document')).toBeInTheDocument()

  optimizeMultiSpy.mockRestore()
}, 15_000)
