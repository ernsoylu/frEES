// DigitizerFitModal.tsx — Digitizer → Fit → Function (Wave H).
//
// Fits a model curve to the active digitized dataset via the existing
// curve-fit engine surface (api.curveFit, wired since Wave B3), then offers
// two inserts: the fitted analytic form into the editor, and the sampled
// fitted curve as a GUI Function Table (the fit evaluated at the dataset's
// x samples — `fittedValues` from the engine, never a client-side re-eval).
// The template list and result rendering are shared with CurveFitModal
// (curveFitShared.tsx); the raw Send-to-Function-Table flow is untouched.

import { useMemo, useState } from 'react'
import {
  Button,
  Group,
  Modal,
  Select,
  Stack,
  Text,
  TextInput,
} from '@mantine/core'
import { curveFit, CURVE_FIT_FAILURE, CurveFitResponse } from './api'
import {
  FIT_TEMPLATES,
  fittedModelInsertText,
  MONO_INPUT,
  templateModelFor,
} from './curveFitShared'
import { FitResultView } from './FitResultView'
import { checkFunctionName, functionSpecFromXY } from './tablesGrid/composeTables'
import { FunctionNameHints, FunctionPrecedenceNote } from './tablesGrid/FunctionNameHints'
import { FunctionTableSpec, identifier, TableSpec } from './tables'

interface Props {
  datasetName: string
  /** Calibrated (x, y) values of the dataset, sorted ascending by x. */
  points: { x: number; y: number }[]
  xName: string
  yName: string
  xLog: boolean
  yLog: boolean
  /** For function-name collision checks (may be empty). */
  tables: TableSpec[]
  onClose: () => void
  /** Insert the fitted analytic form into the editor. */
  onInsertEquation?: (eq: string) => void
  /** Add the sampled fitted curve as a Function Table. */
  onCreateFunctionTable?: (spec: FunctionTableSpec) => void
}

export default function DigitizerFitModal({
  datasetName,
  points,
  xName,
  yName,
  xLog,
  yLog,
  tables,
  onClose,
  onInsertEquation,
  onCreateFunctionTable,
}: Readonly<Props>) {
  const xVar = identifier(xName, 'x')
  const yVar = identifier(yName, 'y')
  const [templateKey, setTemplateKey] = useState<string>(FIT_TEMPLATES[0].name)
  const [model, setModel] = useState(() => templateModelFor(FIT_TEMPLATES[0], xVar, yVar))
  const [parameters, setParameters] = useState(FIT_TEMPLATES[0].parameters)
  const [guesses, setGuesses] = useState('')
  const [running, setRunning] = useState(false)
  const [validation, setValidation] = useState<string | null>(null)
  const [result, setResult] = useState<CurveFitResponse | null>(null)
  const [tableName, setTableName] = useState(`${identifier(yName, 'f').toLowerCase()}_fit`)

  const xData = useMemo(() => points.map((p) => p.x), [points])
  const yData = useMemo(() => points.map((p) => p.y), [points])
  const nameCheck = checkFunctionName(tables, tableName)

  function handleTemplateChange(val: string | null) {
    const key = val ?? 'custom'
    setTemplateKey(key)
    setResult(null)
    if (key === 'custom') return
    const found = FIT_TEMPLATES.find((t) => t.name === key)
    if (found) {
      setModel(templateModelFor(found, xVar, yVar))
      setParameters(found.parameters)
    }
  }

  async function run() {
    setResult(null)
    const paramList = parameters
      .split(',')
      .map((p) => p.trim())
      .filter((p) => p !== '')
    if (model.trim() === '') {
      setValidation('A model equation is required.')
      return
    }
    if (paramList.length === 0) {
      setValidation('List at least one parameter to fit (comma-separated).')
      return
    }
    if (points.length < 2) {
      setValidation('At least two digitized points are required.')
      return
    }
    let initialGuess: number[] | undefined
    if (guesses.trim() !== '') {
      const values = guesses.split(',').map((g) => Number(g.trim()))
      if (values.some((v) => !Number.isFinite(v)) || values.length !== paramList.length) {
        setValidation('Initial guesses must be one number per parameter (comma-separated).')
        return
      }
      initialGuess = values
    }
    setValidation(null)
    if (running) return
    setRunning(true)
    try {
      const response = await curveFit({
        model,
        yVariable: yVar,
        xVariable: xVar,
        parameters: paramList,
        xData,
        yData,
        initialGuess,
      })
      setResult(response)
    } catch (err) {
      setResult({ ...CURVE_FIT_FAILURE, error: String(err) })
    } finally {
      setRunning(false)
    }
  }

  function insertEquation() {
    if (!result?.success || !onInsertEquation) return
    onInsertEquation(
      fittedModelInsertText(templateKey, model, result.parameterNames, result.fittedParameters),
    )
    onClose()
  }

  function sendAsTable() {
    if (!result?.success || !onCreateFunctionTable || !nameCheck.ok) return
    const converted = functionSpecFromXY({
      name: tableName.trim(),
      argName: xVar,
      xs: xData,
      ys: result.fittedValues,
      xLog,
      yLog,
    })
    if (converted.needsReduction) return
    onCreateFunctionTable(converted.spec)
    onClose()
  }

  return (
    <Modal
      opened
      onClose={onClose}
      title={`Fit Curve — ${datasetName} (Levenberg-Marquardt)`}
      centered
      size="lg"
    >
      <Text size="sm" c="dimmed" mb="md">
        Fits a model to the {points.length} calibrated points of “{datasetName}”
        {' '}({yVar} over {xVar}). After a successful fit, insert the analytic form into the
        editor or send the fitted curve to a Function Table.
      </Text>

      <Stack gap="sm">
        <Select
          label="Model equation template"
          data={[
            ...FIT_TEMPLATES.map((t) => ({ value: t.name, label: t.name })),
            { value: 'custom', label: 'Custom equation' },
          ]}
          value={templateKey}
          onChange={handleTemplateChange}
          allowDeselect={false}
        />
        <Group grow>
          <TextInput
            label="Model equation"
            description={`Dependent: ${yVar} · independent: ${xVar}`}
            value={model}
            onChange={(e) => {
              setModel(e.currentTarget.value)
              setTemplateKey('custom')
              setResult(null)
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
              setResult(null)
            }}
            spellCheck={false}
            styles={MONO_INPUT}
          />
        </Group>
        <TextInput
          label="Initial guesses (optional)"
          description="Comma-separated, one per parameter; defaults to 1"
          value={guesses}
          onChange={(e) => setGuesses(e.currentTarget.value)}
          spellCheck={false}
          styles={MONO_INPUT}
        />

        {validation && (
          <Text c="red" size="sm">
            {validation}
          </Text>
        )}

        {result && <FitResultView result={result} />}

        {result?.success && onCreateFunctionTable && (
          <Stack gap={4}>
            <Group align="flex-end" gap="xs">
              <TextInput
                label="Function table name"
                value={tableName}
                onChange={(e) => setTableName(e.currentTarget.value)}
                error={nameCheck.error}
                spellCheck={false}
                styles={MONO_INPUT}
                style={{ flex: 1 }}
              />
              <Button
                variant="default"
                disabled={!nameCheck.ok}
                color={nameCheck.replacesGui ? 'yellow' : undefined}
                onClick={sendAsTable}
              >
                {nameCheck.replacesGui ? 'Send fit as table (replace)' : 'Send fit as table'}
              </Button>
            </Group>
            <FunctionNameHints name={tableName} check={nameCheck} />
            <FunctionPrecedenceNote />
          </Stack>
        )}

        <Group justify="flex-end" mt="xs">
          <Button variant="default" onClick={onClose}>
            Close
          </Button>
          {result?.success && onInsertEquation && (
            <Button color="teal" onClick={insertEquation}>
              Insert equation
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
