// fitResultView.test.tsx — the Phase 4.2 uncertainty rendering. The point of
// these cases is that a number the engine refused to produce must never reach
// the user as a plausible-looking one.

import { cleanup, render, screen } from '@testing-library/react'
import { MantineProvider } from '@mantine/core'
import { afterEach, expect, it, vi } from 'vitest'
import { CURVE_FIT_FAILURE, CurveFitResponse } from './api'
import { FitResultView } from './FitResultView'

// The chart loads Plotly on demand; the assertions here are about the text
// beside it, so stub it out rather than pulling the library into jsdom.
vi.mock('./plots/PlotlyChart', () => ({
  default: () => <div data-testid="fit-plot" />,
}))

HTMLElement.prototype.scrollIntoView = () => {}
// Mantine's provider reads the colour scheme on mount; jsdom has no
// matchMedia. Same stub analysisDialogs.test.tsx uses.
vi.stubGlobal('matchMedia', () => ({
  matches: false,
  addEventListener() {},
  removeEventListener() {},
}))
afterEach(cleanup)

const ok: CurveFitResponse = {
  ...CURVE_FIT_FAILURE,
  success: true,
  error: null,
  parameterNames: ['a', 'b'],
  fittedParameters: [1.99, 1.04],
  parameterStdErrors: [0.0597, 0.1463],
  parameterCovariance: [
    [0.00356, -0.00712],
    [-0.00712, 0.0214],
  ],
  residuals: [0, 0, 0],
  fittedValues: [1, 2, 3],
  rSquared: 0.999,
  rmse: 0.19,
  iterations: 4,
  residualDof: 3,
  rank: 2,
  conditionNumber: 5.6,
  unidentifiable: false,
  reducedChiSquare: null,
  atBound: [false, false],
  confidence: 0.95,
  confidenceBandLo: [0.9, 1.9, 2.9],
  confidenceBandHi: [1.1, 2.1, 3.1],
  predictionBandLo: [0.8, 1.8, 2.8],
  predictionBandHi: [1.2, 2.2, 3.2],
}

function show(result: CurveFitResponse, withData = true) {
  render(
    <MantineProvider>
      <FitResultView
        result={result}
        xData={withData ? [1, 2, 3] : undefined}
        yData={withData ? [1, 2, 3] : undefined}
      />
    </MantineProvider>,
  )
}

it('shows a standard error beside every fitted parameter', () => {
  show(ok)
  expect(screen.getByText('± 0.0597')).toBeTruthy()
  expect(screen.getByText('± 0.1463')).toBeTruthy()
  expect(screen.getByText(/Residual dof/)).toBeTruthy()
  expect(screen.getByText(/Jacobian rank/)).toBeTruthy()
  expect(screen.getByTestId('fit-plot')).toBeTruthy()
})

it('renders an unavailable standard error as a dash, never as zero', () => {
  show({ ...ok, parameterStdErrors: [null, null], unidentifiable: true, rank: 1 })
  // Both parameters show the dash...
  expect(screen.getAllByText('± —')).toHaveLength(2)
  // ...and nothing anywhere reads as a confident zero.
  expect(screen.queryByText('± 0')).toBeNull()
})

it('explains an unidentifiable fit instead of just withholding numbers', () => {
  show({ ...ok, unidentifiable: true, rank: 1, parameterStdErrors: [null, null] })
  expect(screen.getByText(/does not determine every parameter separately/)).toBeTruthy()
  expect(screen.getByText(/rank 1 of 2/)).toBeTruthy()
})

it('flags a parameter sitting on a bound', () => {
  show({ ...ok, atBound: [true, false] })
  expect(screen.getByText('at bound')).toBeTruthy()
  expect(screen.getByText(/Some parameters are on a bound/)).toBeTruthy()
})

it('shows reduced chi-square only when uncertainties were supplied', () => {
  show(ok)
  expect(screen.getByText('—')).toBeTruthy()
  cleanup()
  show({ ...ok, reducedChiSquare: 2.7342 })
  expect(screen.getByText('2.734')).toBeTruthy()
})

it('omits the plot when there is no data to draw it against', () => {
  show(ok, false)
  expect(screen.queryByTestId('fit-plot')).toBeNull()
})

it('renders the engine error and nothing else when the fit failed', () => {
  show({ ...CURVE_FIT_FAILURE, error: 'Model equation is required.' })
  expect(screen.getByText('Model equation is required.')).toBeTruthy()
  expect(screen.queryByText(/Residual dof/)).toBeNull()
})
