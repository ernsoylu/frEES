import { useState, useEffect, useRef } from 'react'
import {
  Badge,
  Button,
  Collapse,
  Group,
  Modal,
  NumberInput,
  Stack,
  Text,
  Textarea,
  TextInput,
  Select,
  SegmentedControl,
} from '@mantine/core'
import { curveFit, CURVE_FIT_FAILURE, CurveFitResponse } from './api'
import {
  FIT_TEMPLATES,
  fittedModelInsertText,
  LOSS_OPTIONS,
  LossName,
  MONO_INPUT,
  parseNumberList,
  validateSigma,
} from './curveFitShared'
import { FitResultView } from './FitResultView'
import { TableSpec } from './tables'

function parseValueAndUnit(str: string): { value: number; unit?: string } | null {
  const trimmed = str.trim()
  if (trimmed === '') return null
  const regex = /^(-?(?:\d+(?:\.\d+)?|\.\d+)(?:[eE][-+]?\d+)?)(?:\s*\[([^\]]*)\])?$/
  const match = regex.exec(trimmed)
  if (!match) return null
  return {
    value: Number(match[1]),
    unit: match[2] || undefined,
  }
}

/** What a data source hands the fit: the two axes, the optional per-point
 *  uncertainties, and the first unit seen on each axis. */
interface FitData {
  x: number[]
  y: number[]
  sigma?: number[]
  xUnit?: string
  yUnit?: string
}

/** Parses (x, y[, sigma]) string tuples into numeric arrays plus the first unit
 *  seen on each axis; rows that don't parse to numbers are skipped. A row whose
 *  sigma cell is missing or non-numeric drops out of the sigma vector, which
 *  the caller then rejects as a partial column rather than guessing. */
function collectXyPoints(
  rawPairs: [string | undefined, string | undefined, (string | undefined)?][],
): FitData {
  const x: number[] = []
  const y: number[] = []
  const sigma: number[] = []
  let sawSigmaColumn = false
  let xUnit: string | undefined
  let yUnit: string | undefined
  for (const [xValRaw, yValRaw, sigmaRaw] of rawPairs) {
    if (xValRaw === undefined || yValRaw === undefined) continue
    const parsedX = parseValueAndUnit(xValRaw)
    const parsedY = parseValueAndUnit(yValRaw)
    if (!parsedX || !parsedY) continue
    x.push(parsedX.value)
    y.push(parsedY.value)
    if (sigmaRaw !== undefined) {
      sawSigmaColumn = true
      const parsedSigma = parseValueAndUnit(sigmaRaw)
      if (parsedSigma) sigma.push(parsedSigma.value)
    }
    if (xUnit === undefined && parsedX.unit) xUnit = parsedX.unit
    if (yUnit === undefined && parsedY.unit) yUnit = parsedY.unit
  }
  return {
    x,
    y,
    sigma: sawSigmaColumn && sigma.length === x.length ? sigma : undefined,
    xUnit,
    yUnit,
  }
}

function getDefaultParameterUnit(
  paramName: string,
  template: string,
  xUnit: string,
  yUnit: string
): string {
  if (!xUnit && !yUnit) return ''
  const xu = xUnit || '1'
  const yu = yUnit || '1'

  if (template.startsWith('Linear')) return linearParamUnit(paramName, xUnit, xu, yu, yUnit)
  if (template.startsWith('Quadratic')) return quadraticParamUnit(paramName, xUnit, xu, yu, yUnit)
  if (template.startsWith('Cubic')) return cubicParamUnit(paramName, xUnit, xu, yu, yUnit)
  if (template.startsWith('Exponential')) return exponentialParamUnit(paramName, xUnit, xu, yUnit)
  if (template.startsWith('Logarithmic')) return paramName === 'a' || paramName === 'b' ? yUnit : ''
  if (template.startsWith('Power')) return paramName === 'c' ? yUnit : ''
  return ''
}

function linearParamUnit(paramName: string, xUnit: string, xu: string, yu: string, yUnit: string): string {
  if (paramName === 'a') return xUnit ? `${yu}/${xu}` : yu
  if (paramName === 'b') return yUnit
  return ''
}

function quadraticParamUnit(paramName: string, xUnit: string, xu: string, yu: string, yUnit: string): string {
  if (paramName === 'a') return xUnit ? `${yu}/${xu}^2` : yu
  if (paramName === 'b') return xUnit ? `${yu}/${xu}` : yu
  if (paramName === 'c') return yUnit
  return ''
}

function cubicParamUnit(paramName: string, xUnit: string, xu: string, yu: string, yUnit: string): string {
  if (paramName === 'a') return xUnit ? `${yu}/${xu}^3` : yu
  if (paramName === 'b') return xUnit ? `${yu}/${xu}^2` : yu
  if (paramName === 'c') return xUnit ? `${yu}/${xu}` : yu
  if (paramName === 'd') return yUnit
  return ''
}

function exponentialParamUnit(paramName: string, xUnit: string, xu: string, yUnit: string): string {
  if (paramName === 'a') return yUnit
  if (paramName === 'b') return xUnit ? `1/${xu}` : ''
  if (paramName === 'c') return yUnit
  return ''
}

interface Props {
  tables?: TableSpec[]
  defaultTableId?: string | null
  onClose: () => void
  onInsertEquation?: (eq: string) => void
}

/**
 * Calculate > Curve Fit: Levenberg-Marquardt least-squares fitting of a model
 * equation's parameters to observed (x, y) data (Story 9.7).
 */
export default function CurveFitModal({
  tables = [],
  defaultTableId,
  onClose,
  onInsertEquation,
}: Readonly<Props>) {
  const [templateKey, setTemplateKey] = useState<string>('custom')
  const [model, setModel] = useState('y = a * exp(-b * x) + c')
  const [yVariable, setYVariable] = useState('y')
  const [xVariable, setXVariable] = useState('x')
  const [parameters, setParameters] = useState('a, b, c')
  const [guesses, setGuesses] = useState('')
  const [lowerBounds, setLowerBounds] = useState('')
  const [upperBounds, setUpperBounds] = useState('')
  const [loss, setLoss] = useState<LossName>('linear')
  const [fScale, setFScale] = useState<number | ''>('')
  const [confidence, setConfidence] = useState<number | ''>(0.95)
  const [sigmaColumnName, setSigmaColumnName] = useState<string>('')
  const [advancedOpen, setAdvancedOpen] = useState(false)
  const [running, setRunning] = useState(false)
  const [validation, setValidation] = useState<string | null>(null)
  const [result, setResult] = useState<CurveFitResponse | null>(null)
  // The data the displayed result was actually fitted to. Editing the data box
  // after a fit must not silently re-point the plot at numbers the fit never
  // saw, so the plot reads this and not the live parse.
  const [fitted, setFitted] = useState<FitData | null>(null)

  // Units states
  const [inferredXUnit, setInferredXUnit] = useState<string>('')
  const [inferredYUnit, setInferredYUnit] = useState<string>('')
  const [parameterUnits, setParameterUnits] = useState<Record<string, string>>({})

  // Table selection states
  const [dataMode, setDataMode] = useState<'manual' | 'table'>('manual')
  const [selectedTableId, setSelectedTableId] = useState<string | null>(null)
  const [xColumnName, setXColumnName] = useState<string>('')
  const [yColumnName, setYColumnName] = useState<string>('')
  const [data, setData] = useState('')

  useEffect(() => {
    if (defaultTableId && tables.length > 0) {
      const table = tables.find((t) => t.id === defaultTableId)
      if (table) {
        setSelectedTableId(defaultTableId)
        setDataMode('table')
        if (table.kind === 'parametric') {
          const vars = table.vars
          setXColumnName(vars[0] || '')
          setYColumnName(vars[1] || vars[0] || '')
        } else {
          setXColumnName(table.argName || 'x')
          setYColumnName('0')
        }
      }
    }
  }, [defaultTableId, tables])

  function handleTemplateChange(val: string | null) {
    const key = val || 'custom'
    setTemplateKey(key)
    if (key === 'custom') return
    const found = FIT_TEMPLATES.find((t) => t.name === key)
    if (found) {
      setModel(found.equation)
      setYVariable(found.yVariable)
      setXVariable(found.xVariable)
      setParameters(found.parameters)
    }
  }

  function handleTableChange(tableId: string | null) {
    setSelectedTableId(tableId)
    if (!tableId) return
    const table = tables.find((t) => t.id === tableId)
    if (!table) return
    if (table.kind === 'parametric') {
      const vars = table.vars
      setXColumnName(vars[0] || '')
      setYColumnName(vars[1] || vars[0] || '')
    } else {
      setXColumnName(table.argName || 'x')
      setYColumnName('0')
    }
  }

  function parseData(): FitData | string {
    const x: number[] = []
    const y: number[] = []
    const sigma: number[] = []
    let xUnit: string | undefined
    let yUnit: string | undefined

    for (const rawLine of data.split('\n')) {
      const line = rawLine.trim()
      if (line === '') continue

      // Regex matching floating point numbers optionally followed by [unit]
      const regex = /(-?\d*\.?\d+(?:[eE][-+]?\d+)?)(?:\s*\[([^\]]*)\])?/g
      const matches = Array.from(line.matchAll(regex))
      if (matches.length < 2) {
        return `Each data line needs at least two numbers (x y), got: "${line}"`
      }

      const xi = Number(matches[0][1])
      const xUi = matches[0][2] || undefined
      const yi = Number(matches[1][1])
      const yUi = matches[1][2] || undefined

      if (!Number.isFinite(xi) || !Number.isFinite(yi)) {
        return `Not a numeric data pair: "${line}"`
      }
      x.push(xi)
      y.push(yi)
      // A third number on the line is that point's uncertainty. Optional, but
      // all-or-nothing: a sigma on some rows and not others is a mistake, not
      // a default, so it is refused rather than silently filled in.
      if (matches.length > 2) {
        const si = Number(matches[2][1])
        if (!Number.isFinite(si)) return `Not a numeric uncertainty: "${line}"`
        sigma.push(si)
      }
      if (xUnit === undefined && xUi) xUnit = xUi
      if (yUnit === undefined && yUi) yUnit = yUi
    }
    if (x.length < 2) return 'At least two data points are required.'
    if (sigma.length !== 0 && sigma.length !== x.length) {
      return `Give an uncertainty on every data line or on none (got ${sigma.length} of ${x.length}).`
    }
    return { x, y, sigma: sigma.length > 0 ? sigma : undefined, xUnit, yUnit }
  }

  function extractDataFromTable(): FitData | string {
    if (!selectedTableId) return 'No table selected.'
    const table = tables.find((t) => t.id === selectedTableId)
    if (!table) return 'Selected table not found.'

    let pairs: [string | undefined, string | undefined, (string | undefined)?][]
    if (table.kind === 'parametric') {
      if (!xColumnName || !yColumnName) {
        return 'Please select both X and Y columns.'
      }
      pairs = table.rows.map((row) => [
        row.values[xColumnName],
        row.values[yColumnName],
        sigmaColumnName ? row.values[sigmaColumnName] : undefined,
      ])
    } else {
      const yColIndex = Number(yColumnName)
      if (Number.isNaN(yColIndex) || yColIndex < 0 || yColIndex >= table.columns.length) {
        return 'Please select a valid Y column.'
      }
      const sigmaColIndex = sigmaColumnName === '' ? -1 : Number(sigmaColumnName)
      pairs = table.rows.map((row) => [
        row.x,
        row.ys[yColIndex],
        sigmaColIndex >= 0 ? row.ys[sigmaColIndex] : undefined,
      ])
    }

    const result = collectXyPoints(pairs)
    if (result.x.length < 2) {
      return 'Selected table columns must contain at least two numeric data points.'
    }
    if (sigmaColumnName !== '' && !result.sigma) {
      return 'The uncertainty column must be numeric on every row used by the fit.'
    }
    return result
  }

  // Reactive unit inference update
  const prevSourceRef = useRef<string>('')
  useEffect(() => {
    const parsed = dataMode === 'manual' ? parseData() : extractDataFromTable()
    if (typeof parsed !== 'string') {
      const sourceKey = `${dataMode}-${selectedTableId}-${xColumnName}-${yColumnName}-${data.length}`
      if (prevSourceRef.current !== sourceKey) {
        prevSourceRef.current = sourceKey
        setInferredXUnit(parsed.xUnit || '')
        setInferredYUnit(parsed.yUnit || '')
      }
    }
  }, [data, dataMode, selectedTableId, xColumnName, yColumnName, tables])

  // Parameter default units propagation
  useEffect(() => {
    const paramList = parameters
      .split(',')
      .map((p) => p.trim())
      .filter((p) => p !== '')
    const newUnits = { ...parameterUnits }
    let changed = false
    for (const p of paramList) {
      if (!newUnits[p]) {
        const defUnit = getDefaultParameterUnit(p, templateKey, inferredXUnit, inferredYUnit)
        if (defUnit) {
          newUnits[p] = defUnit
          changed = true
        }
      }
    }
    if (changed) {
      setParameterUnits(newUnits)
    }
  }, [parameters, templateKey, inferredXUnit, inferredYUnit])

  async function run() {
    setResult(null)
    setFitted(null)
    const paramList = parameters
      .split(',')
      .map((p) => p.trim())
      .filter((p) => p !== '')
    if (model.trim() === '' || yVariable.trim() === '' || xVariable.trim() === '') {
      setValidation('Model equation, dependent and independent variables are required.')
      return
    }
    if (paramList.length === 0) {
      setValidation('List at least one parameter to fit (comma-separated).')
      return
    }
    const parsed = dataMode === 'manual' ? parseData() : extractDataFromTable()
    if (typeof parsed === 'string') {
      setValidation(parsed)
      return
    }
    const initialGuess = parseNumberList(guesses, paramList.length, 'Initial guesses')
    if (typeof initialGuess === 'string') {
      setValidation(initialGuess)
      return
    }
    const lower = parseNumberList(lowerBounds, paramList.length, 'Lower bounds')
    if (typeof lower === 'string') {
      setValidation(lower)
      return
    }
    const upper = parseNumberList(upperBounds, paramList.length, 'Upper bounds')
    if (typeof upper === 'string') {
      setValidation(upper)
      return
    }
    // The engine takes bounds as a pair or not at all; catching the half-filled
    // case here says which side is missing, which the engine cannot.
    if (Boolean(lower) !== Boolean(upper)) {
      setValidation(`Give both bounds or neither — the ${lower ? 'upper' : 'lower'} bounds are empty.`)
      return
    }
    const sigmaProblem = validateSigma(parsed.sigma)
    if (sigmaProblem) {
      setValidation(sigmaProblem)
      return
    }
    setValidation(null)
    if (running) return
    setRunning(true)
    try {
      const response = await curveFit({
        model,
        yVariable: yVariable.trim(),
        xVariable: xVariable.trim(),
        parameters: paramList,
        xData: parsed.x,
        yData: parsed.y,
        initialGuess,
        sigma: parsed.sigma,
        lowerBounds: lower,
        upperBounds: upper,
        loss,
        fScale: typeof fScale === 'number' ? fScale : undefined,
        confidence: typeof confidence === 'number' ? confidence : undefined,
      })
      setResult(response)
      setFitted(parsed)
    } catch (err) {
      setResult({ ...CURVE_FIT_FAILURE, error: String(err) })
    } finally {
      setRunning(false)
    }
  }

  function insertToEditor() {
    if (!result || !result.success || !onInsertEquation) return
    onInsertEquation(
      fittedModelInsertText(
        templateKey,
        model,
        result.parameterNames,
        result.fittedParameters,
        parameterUnits,
      ),
    )
    onClose()
  }

  return (
    <Modal opened onClose={onClose} title="Curve Fit — Least Squares (Levenberg-Marquardt)" centered size="xl">
      <Text size="sm" c="dimmed" mb="md">
        Fits the model parameters to the observed data by minimizing the sum of
        squared residuals. Choose an equation template (or write your own custom one)
        and select the data source.
      </Text>

      <Stack gap="sm">
        <Select
          label="Model equation template"
          placeholder="Select a template or write custom"
          data={[
            { value: 'custom', label: 'Custom equation' },
            ...FIT_TEMPLATES.map((t) => ({ value: t.name, label: t.name })),
          ]}
          value={templateKey}
          onChange={handleTemplateChange}
          allowDeselect={false}
        />

        <TextInput
          label="Model equation"
          description="The dependent variable must appear alone on one side"
          value={model}
          onChange={(e) => {
            setModel(e.currentTarget.value)
            setTemplateKey('custom')
          }}
          spellCheck={false}
          styles={MONO_INPUT}
        />

        <Group grow>
          <TextInput
            label="Dependent variable"
            value={yVariable}
            onChange={(e) => {
              setYVariable(e.currentTarget.value)
              setTemplateKey('custom')
            }}
            spellCheck={false}
            styles={MONO_INPUT}
          />
          <TextInput
            label="Independent variable"
            value={xVariable}
            onChange={(e) => {
              setXVariable(e.currentTarget.value)
              setTemplateKey('custom')
            }}
            spellCheck={false}
            styles={MONO_INPUT}
          />
          <TextInput
            label="Parameters to fit"
            value={parameters}
            onChange={(e) => {
              setParameters(e.currentTarget.value)
              setTemplateKey('custom')
            }}
            spellCheck={false}
            styles={MONO_INPUT}
          />
        </Group>

        <Group grow>
          <TextInput
            label="Y (Dependent) Unit"
            placeholder="e.g. kPa"
            value={inferredYUnit}
            onChange={(e) => setInferredYUnit(e.currentTarget.value)}
            styles={MONO_INPUT}
          />
          <TextInput
            label="X (Independent) Unit"
            placeholder="e.g. m"
            value={inferredXUnit}
            onChange={(e) => setInferredXUnit(e.currentTarget.value)}
            styles={MONO_INPUT}
          />
          <div style={{ flex: 1 }} />
        </Group>

        {/* Always offered, even with no tables in the project. Hiding the
            control entirely made the table source undiscoverable — you had to
            already know it existed to go and create a table first. */}
        <Group justify="space-between" align="center" mt="xs">
          <Text size="sm" fw={500}>Data points source</Text>
          <SegmentedControl
            value={dataMode}
            onChange={(v) => {
              setDataMode(v as 'manual' | 'table')
              setValidation(null)
            }}
            data={[
              { value: 'manual', label: 'Enter manually' },
              { value: 'table', label: 'Select from table', disabled: tables.length === 0 },
            ]}
          />
        </Group>
        {tables.length === 0 && (
          <Text size="xs" c="dimmed">
            No tables in this project yet — add a parametric or function table and its
            columns become selectable here.
          </Text>
        )}

        {dataMode === 'manual' ? (
          <Textarea
            label="Data points"
            placeholder={'0.0 [m]  5.1 [kPa]\n1.0 [m]  3.2 [kPa]\n2.0 [m]  2.1 [kPa]'}
            value={data}
            onChange={(e) => setData(e.currentTarget.value)}
            autosize
            minRows={5}
            maxRows={12}
            spellCheck={false}
            styles={MONO_INPUT}
          />
        ) : (
          <Stack gap="xs">
            <Select
              label="Select Table"
              placeholder="Choose a table..."
              data={tables.map((t) => ({
                value: t.id,
                label: t.name + (t.kind === 'parametric' ? ' (Parametric)' : ' (Function)'),
              }))}
              value={selectedTableId}
              onChange={handleTableChange}
              allowDeselect={false}
            />

            {selectedTableId && (() => {
              const table = tables.find((t) => t.id === selectedTableId)
              if (!table) return null

              if (table.kind === 'parametric') {
                return (
                  <Group grow>
                    <Select
                      label="X Column (Independent)"
                      data={table.vars.map((v) => ({ value: v, label: v }))}
                      value={xColumnName}
                      onChange={(v) => v && setXColumnName(v)}
                      allowDeselect={false}
                    />
                    <Select
                      label="Y Column (Dependent)"
                      data={table.vars.map((v) => ({ value: v, label: v }))}
                      value={yColumnName}
                      onChange={(v) => v && setYColumnName(v)}
                      allowDeselect={false}
                    />
                  </Group>
                )
              } else {
                const yCols = table.columns.map((col, idx) => ({
                  value: String(idx),
                  label: table.is1D ? 'y' : (table.paramName ? `${table.paramName} = ${col}` : `Col ${idx + 1} (${col})`),
                }))
                return (
                  <Group grow>
                    <TextInput
                      label="X Column (Independent)"
                      value={table.argName || 'x'}
                      readOnly
                      disabled
                    />
                    <Select
                      label="Y Column (Dependent)"
                      data={yCols}
                      value={yColumnName}
                      onChange={(v) => v && setYColumnName(v)}
                      allowDeselect={false}
                    />
                  </Group>
                )
              }
            })()}

            {selectedTableId && (() => {
              const parsed = extractDataFromTable()
              if (typeof parsed === 'string') {
                return (
                  <Text c="orange" size="xs">
                    {parsed}
                  </Text>
                )
              }
              return (
                <Group gap="xs">
                  <Badge color="teal" variant="light">
                    ✓ Extracted {parsed.x.length} data points
                  </Badge>
                  <Text size="xs" c="dimmed" style={{ maxWidth: '450px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    Points: {parsed.x.map((xi, idx) => `(${xi}, ${parsed.y[idx]})`).join(', ')}
                  </Text>
                </Group>
              )
            })()}
          </Stack>
        )}

        <TextInput
          label="Initial guesses (optional)"
          description="Comma-separated, one per parameter; defaults to 1"
          value={guesses}
          onChange={(e) => setGuesses(e.currentTarget.value)}
          spellCheck={false}
          styles={MONO_INPUT}
        />

        <Group justify="space-between" align="center" mt="xs">
          <Text size="sm" fw={500}>
            Weighting, bounds &amp; robustness
          </Text>
          <Button
            variant="subtle"
            size="compact-sm"
            onClick={() => setAdvancedOpen((o) => !o)}
          >
            {advancedOpen ? 'Hide' : 'Show'}
          </Button>
        </Group>

        <Collapse expanded={advancedOpen}>
          <Stack gap="sm">
            <Text size="xs" c="dimmed">
              All optional. Left alone, this is an ordinary unweighted least-squares fit
              with no box constraints — exactly what it was before.
            </Text>

            {dataMode === 'manual' ? (
              <Text size="xs" c="dimmed">
                <b>Uncertainties:</b> add a third number on each data line to give that
                point&apos;s standard deviation. Supply one on every line or on none. With
                uncertainties the reported standard errors become absolute and reduced χ²
                becomes meaningful; without them the errors are scaled by the observed
                residual spread.
              </Text>
            ) : (
              selectedTableId && (() => {
                const table = tables.find((t) => t.id === selectedTableId)
                if (!table) return null
                const options =
                  table.kind === 'parametric'
                    ? table.vars.map((v) => ({ value: v, label: v }))
                    : table.columns.map((col, idx) => ({
                        value: String(idx),
                        label: table.paramName ? `${table.paramName} = ${col}` : `Col ${idx + 1} (${col})`,
                      }))
                return (
                  <Select
                    label="Uncertainty column (optional)"
                    description="Per-point standard deviations; all values must be positive"
                    placeholder="None — unweighted fit"
                    data={options}
                    value={sigmaColumnName === '' ? null : sigmaColumnName}
                    onChange={(v) => setSigmaColumnName(v ?? '')}
                    clearable
                  />
                )
              })()
            )}

            <Group grow>
              <TextInput
                label="Lower bounds (optional)"
                description="Comma-separated, one per parameter"
                value={lowerBounds}
                onChange={(e) => setLowerBounds(e.currentTarget.value)}
                spellCheck={false}
                styles={MONO_INPUT}
              />
              <TextInput
                label="Upper bounds (optional)"
                description="Send both sides or neither"
                value={upperBounds}
                onChange={(e) => setUpperBounds(e.currentTarget.value)}
                spellCheck={false}
                styles={MONO_INPUT}
              />
            </Group>

            <Group grow align="flex-start">
              <Select
                label="Loss function"
                description="Robust losses down-weight outliers"
                data={LOSS_OPTIONS.map((o) => ({ value: o.value, label: o.label }))}
                value={loss}
                onChange={(v) => setLoss((v as LossName) ?? 'linear')}
                allowDeselect={false}
              />
              <NumberInput
                label="Outlier scale (optional)"
                description={
                  loss === 'linear'
                    ? 'Only used by a robust loss'
                    : 'Residual size treated as an outlier; blank estimates it'
                }
                value={fScale}
                onChange={(v) => setFScale(typeof v === 'number' ? v : '')}
                disabled={loss === 'linear'}
                min={0}
                step={0.1}
              />
              <NumberInput
                label="Confidence level"
                description="For the reported bands"
                value={confidence}
                onChange={(v) => setConfidence(typeof v === 'number' ? v : '')}
                min={0.5}
                max={0.999}
                step={0.01}
                decimalScale={3}
              />
            </Group>
          </Stack>
        </Collapse>

        {validation && (
          <Text c="red" size="sm">
            {validation}
          </Text>
        )}

        {result && (
          <FitResultView
            result={result}
            parameterUnits={parameterUnits}
            setParameterUnits={setParameterUnits}
            xData={fitted?.x}
            yData={fitted?.y}
            xLabel={inferredXUnit ? `${xVariable} [${inferredXUnit}]` : xVariable}
            yLabel={inferredYUnit ? `${yVariable} [${inferredYUnit}]` : yVariable}
          />
        )}

        <Group justify="flex-end" mt="xs">
          <Button variant="default" onClick={onClose}>
            Close
          </Button>
          {result && result.success && onInsertEquation && (
            <Button color="teal" onClick={insertToEditor}>
              Copy to Editor
            </Button>
          )}
          <Button onClick={run} loading={running}>
            Fit
          </Button>
        </Group>
      </Stack>
    </Modal>
  )
}

