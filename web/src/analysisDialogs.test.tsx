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
