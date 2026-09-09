// FitResultView.tsx — the shared curve-fit result rendering (error alert, or
// the convergence badge + parameter table + uncertainty diagnostics), used by
// CurveFitModal and the Digitizer's fit dialog. Kept component-only
// (react-refresh); the shared constants and pure helpers live in
// curveFitShared.ts.

import { Alert, Badge, Group, Stack, Table, Text, TextInput, Tooltip } from '@mantine/core'
import { CurveFitResponse } from './api'
import { formatValue } from './format'
import { MONO_INPUT, buildFitFigure } from './curveFitShared'
import PlotlyChart from './plots/PlotlyChart'

/** A number the engine could not produce renders as an em dash, never as 0 —
 *  the whole point of the boundary sending `null` is that the value does not
 *  exist, and a zero standard error would read as certainty. */
function orDash(value: number | null | undefined, digits?: number) {
  if (value === null || value === undefined || !Number.isFinite(value)) return '—'
  return digits === undefined ? formatValue(value) : value.toPrecision(digits)
}

/** Fit outcome rendering: error alert, or the convergence badge, parameter
 * table with standard errors, uncertainty diagnostics and the band plot.
 *
 * The unit column renders only when `setParameterUnits` is provided (the
 * digitizer's data is unitless — it omits the editor). The plot renders only
 * when `xData` and `yData` are provided. */
export function FitResultView({
  result,
  parameterUnits,
  setParameterUnits,
  xData,
  yData,
  xLabel = 'x',
  yLabel = 'y',
}: Readonly<{
  result: CurveFitResponse
  parameterUnits?: Record<string, string>
  setParameterUnits?: (units: Record<string, string>) => void
  xData?: readonly number[]
  yData?: readonly number[]
  xLabel?: string
  yLabel?: string
}>) {
  if (!result.success) {
    return (
      <Alert color="red" variant="light" p="xs">
        <Text size="sm" style={{ whiteSpace: 'pre-wrap' }}>
          {result.error}
        </Text>
      </Alert>
    )
  }
  const units = parameterUnits ?? {}
  const figure =
    xData && yData ? buildFitFigure(result, xData, xLabel, yLabel, yData) : null
  const confidencePct = Math.round(result.confidence * 100)
  const anyAtBound = result.atBound.some(Boolean)

  return (
    <Stack gap="xs">
      <Group gap="xs">
        <Badge color="green" variant="light" leftSection="✓">
          Fit converged
        </Badge>
        <Text size="xs" c="dimmed">
          {result.iterations} iterations · R² = {formatValue(result.rSquared)} · RMSE ={' '}
          {formatValue(result.rmse)}
        </Text>
      </Group>

      {result.unidentifiable && (
        <Alert color="orange" variant="light" p="xs">
          <Text size="sm">
            The data does not determine every parameter separately — the Jacobian has rank{' '}
            {result.rank} of {result.parameterNames.length}. The fitted values below still
            minimise the residuals, but standard errors are withheld rather than invented:
            any number there would describe a parameter the data cannot pin down. Drop a
            parameter, or add data that distinguishes them.
          </Text>
        </Alert>
      )}

      <Table striped highlightOnHover>
        <Table.Thead>
          <Table.Tr>
            <Table.Th style={{ width: '110px' }}>Parameter</Table.Th>
            <Table.Th style={{ width: '160px' }}>Fitted value</Table.Th>
            <Table.Th style={{ width: '150px' }}>Std. error</Table.Th>
            {setParameterUnits && <Table.Th>Unit (optional)</Table.Th>}
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {result.parameterNames.map((name, i) => (
            <Table.Tr key={name}>
              <Table.Td ff="monospace">
                {name}
                {result.atBound[i] && (
                  <Tooltip
                    withArrow
                    multiline
                    w={300}
                    label={
                      'This parameter sits on one of its bounds. Its standard error was ' +
                      'computed as though it were free, so it overstates how well the data ' +
                      'pins it down.'
                    }
                  >
                    <Badge size="xs" color="yellow" variant="light" ml={6}>
                      at bound
                    </Badge>
                  </Tooltip>
                )}
              </Table.Td>
              <Table.Td ff="monospace" c="green.4">
                {formatValue(result.fittedParameters[i])}
              </Table.Td>
              <Table.Td ff="monospace" c="dimmed">
                ± {orDash(result.parameterStdErrors[i])}
              </Table.Td>
              {setParameterUnits && (
                <Table.Td>
                  <TextInput
                    size="xs"
                    placeholder="e.g. kPa"
                    value={units[name] || ''}
                    onChange={(e) => {
                      setParameterUnits({
                        ...units,
                        [name]: e.currentTarget.value,
                      })
                    }}
                    styles={MONO_INPUT}
                  />
                </Table.Td>
              )}
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>

      <Group gap="lg">
        <Text size="xs" c="dimmed">
          Residual dof <b>{result.residualDof}</b>
        </Text>
        <Text size="xs" c="dimmed">
          Jacobian rank{' '}
          <b>
            {result.rank}/{result.parameterNames.length}
          </b>
        </Text>
        <Tooltip
          withArrow
          multiline
          w={320}
          label={
            'Ratio of largest to smallest singular value of the Jacobian. Large means ' +
            'the parameters trade off against each other, so the individual standard ' +
            'errors are near-meaningless even though they are finite.'
          }
        >
          <Text size="xs" c="dimmed">
            Condition number <b>{orDash(result.conditionNumber, 4)}</b>
          </Text>
        </Tooltip>
        <Tooltip
          withArrow
          multiline
          w={320}
          label={
            'Chi-square per degree of freedom, over the weighted residuals. Far from 1 ' +
            'means the uncertainties you supplied do not describe the observed scatter. ' +
            'Shown only when uncertainties were given — without them there is no ' +
            'absolute scale to compare against.'
          }
        >
          <Text size="xs" c="dimmed">
            Reduced χ² <b>{orDash(result.reducedChiSquare, 4)}</b>
          </Text>
        </Tooltip>
        {anyAtBound && (
          <Text size="xs" c="yellow">
            Some parameters are on a bound
          </Text>
        )}
      </Group>

      {figure && (
        <Stack gap={4}>
          <PlotlyChart figure={figure} minHeight={300} />
          <Text size="xs" c="dimmed">
            Bands are {confidencePct}%. The inner band is the confidence band on the fitted
            curve; the outer is the prediction band, where a new measurement would fall.
            Both are local-linear approximations.
          </Text>
        </Stack>
      )}
    </Stack>
  )
}
