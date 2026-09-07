import { useMemo, useState } from 'react'
import {
  Alert,
  Button,
  Group,
  Modal,
  Select,
  Stack,
  Table,
  Text,
  TextInput,
} from '@mantine/core'
import { parameterFit, type ParameterFitResult, type StopCriteria, type VariableInfo, type FunctionTableDto } from './api'
import type { FunctionTableSpec, TableSpec } from './tables'
import { formatValue } from './format'

interface Bounds {
  initial: string
  lower: string
  upper: string
}

/**
 * Replaces the document's assignment of each fitted parameter with its fitted
 * value — the frontend twin of the terminal-override replace rule. A
 * parameter with no assignment line gets one appended.
 */
export function applyFittedParameters(text: string, names: string[], values: number[]): string {
  let out = text
  names.forEach((name, i) => {
    const value = formatValue(values[i])
    const pattern = new RegExp(`^(\\s*)${name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\s*=[^;\\n]*`, 'im')
    if (pattern.test(out)) {
      out = out.replace(pattern, `$1${name} = ${value}`)
    } else {
      out = `${out.trim()}\n${name} = ${value}`
    }
  })
  return out
}

/** The (t, v) pairs of a 1-D function table, in row order. Non-numeric rows
 *  are dropped — the Import CSV… path already skipped them, but a hand-edited
 *  table can carry blanks. */
function measuredSeries(spec: FunctionTableSpec): { t: number[]; v: number[] } {
  const t: number[] = []
  const v: number[] = []
  for (const row of spec.rows) {
    const x = Number(row.x)
    const y = Number(row.ys[0])
    if (!Number.isFinite(x) || !Number.isFinite(y)) continue
    t.push(x)
    v.push(y)
  }
  return { t, v }
}

/**
 * Parameter estimation: fit chosen document parameters so a DYNAMIC column
 * matches a measured series.
 *
 * The measured side used to be an analyzer channel; since D11 removed the
 * Data Analyzer it is a 1-D **function table** — which is what Import CSV…
 * produces, so a recording still reaches this dialog in two clicks, and the
 * same table is simultaneously callable from the equations.
 */
export default function ParameterFitModal({
  opened,
  onClose,
  text,
  stopCriteria,
  variableInfo,
  getFunctionTables,
  tables,
  onApply,
}: Readonly<{
  opened: boolean
  onClose: () => void
  text: string
  stopCriteria: StopCriteria
  variableInfo: VariableInfo[]
  getFunctionTables: () => FunctionTableDto[]
  tables: TableSpec[]
  onApply: (nextText: string) => void
}>) {
  // Measured source: every single-curve function table (imported from a CSV,
  // digitized, swept, or typed). A curve family has no single y per x, so it
  // cannot be a measured trace.
  const measuredTables = useMemo(
    () =>
      tables.filter(
        (t): t is FunctionTableSpec => t.kind === 'function' && t.columns.length === 1,
      ),
    [tables],
  )
  const measuredOptions = useMemo(
    () =>
      measuredTables.map((t) => ({
        value: t.id,
        label: `${t.name} (${t.rows.length} point${t.rows.length === 1 ? '' : 's'})`,
      })),
    [measuredTables],
  )

  // Fit target: any solved DYNAMIC table column (time column excluded).
  const targetOptions = useMemo(() => {
    const out: { value: string; label: string }[] = []
    for (const t of tables) {
      if (t.kind !== 'parametric' || t.origin !== 'ode') continue
      const block = t.id.replace(/^code-ode-/, '')
      for (const v of t.vars.slice(1)) {
        out.push({ value: `${block}|${v}`, label: `${block}: ${v}` })
      }
    }
    return out
  }, [tables])

  const [measSel, setMeasSel] = useState<string | null>(null)
  const [targetSel, setTargetSel] = useState<string | null>(null)
  const [paramText, setParamText] = useState('')
  const [bounds, setBounds] = useState<Record<string, Bounds>>({})
  const [running, setRunning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<ParameterFitResult | null>(null)

  const paramNames = paramText
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s.length > 0)

  const setBound = (name: string, key: keyof Bounds, value: string) =>
    setBounds((prev) => ({
      ...prev,
      [name]: { ...(prev[name] ?? { initial: '', lower: '', upper: '' }), [key]: value },
    }))

  const run = async () => {
    setError(null)
    setResult(null)
    if (measSel === null || targetSel === null || paramNames.length === 0) {
      setError('Pick a measured channel, a fit target, and at least one parameter.')
      return
    }
    const initial: number[] = []
    const lower: number[] = []
    const upper: number[] = []
    for (const name of paramNames) {
      const b = bounds[name]
      const i = Number(b?.initial)
      const lo = Number(b?.lower)
      const hi = Number(b?.upper)
      if (!Number.isFinite(i) || !Number.isFinite(lo) || !Number.isFinite(hi) || lo >= hi) {
        setError(`Bounds for ${name} must be finite numbers with lower < upper.`)
        return
      }
      initial.push(i)
      lower.push(lo)
      upper.push(hi)
    }
    const measured = measuredTables.find((t) => t.id === measSel)
    const raw = measured ? measuredSeries(measured) : null
    if (!raw || raw.t.length === 0) {
      setError('The measured function table has no numeric points.')
      return
    }
    const [odeBlock, column] = targetSel.split('|')
    setRunning(true)
    try {
      const r = await parameterFit({
        text,
        stopCriteria,
        variableInfo,
        functionTables: getFunctionTables(),
        parameters: paramNames,
        initial,
        lower,
        upper,
        odeBlock,
        column,
        measuredT: raw.t,
        measuredV: raw.v,
      })
      if (r.success) setResult(r)
      else setError(r.error ?? 'Parameter fit failed.')
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setRunning(false)
    }
  }

  return (
    <Modal opened={opened} onClose={onClose} title="Parameter Estimation" size="lg" centered>
      <Stack gap="sm">
        <Text size="sm" c="dimmed">
          Fits the chosen document parameters so a DYNAMIC column matches a measured series —
          each trial re-solves the model and the residuals are reduced on the measured raster.
          The measured side is a single-curve function table: import one from a .csv in the
          Tables window (Import CSV…).
        </Text>
        <Group grow>
          <Select
            label="Measured series (function table)"
            searchable
            data={measuredOptions}
            value={measSel}
            onChange={setMeasSel}
            placeholder={
              measuredOptions.length === 0
                ? 'Import a CSV as a function table first'
                : 'Pick a function table'
            }
          />
          <Select
            label="Fit target (DYNAMIC column)"
            searchable
            data={targetOptions}
            value={targetSel}
            onChange={setTargetSel}
            placeholder={targetOptions.length === 0 ? 'Solve a DYNAMIC document first' : 'Pick a column'}
          />
        </Group>
        <TextInput
          label="Parameters to fit (comma-separated document assignments)"
          placeholder="k, ua, x0"
          value={paramText}
          onChange={(e) => setParamText(e.currentTarget.value)}
        />
        {paramNames.length > 0 && (
          <Table withTableBorder fz="sm">
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Parameter</Table.Th>
                <Table.Th>Initial</Table.Th>
                <Table.Th>Lower</Table.Th>
                <Table.Th>Upper</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {paramNames.map((name) => (
                <Table.Tr key={name}>
                  <Table.Td ff="monospace">{name}</Table.Td>
                  {(['initial', 'lower', 'upper'] as const).map((key) => (
                    <Table.Td key={key}>
                      <TextInput
                        size="xs"
                        aria-label={`${key} of ${name}`}
                        value={bounds[name]?.[key] ?? ''}
                        onChange={(e) => setBound(name, key, e.currentTarget.value)}
                      />
                    </Table.Td>
                  ))}
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        )}
        <Group>
          <Button onClick={run} loading={running}>
            Fit
          </Button>
        </Group>
        {error && (
          <Alert color="red" title="Parameter fit failed">
            {error}
          </Alert>
        )}
        {result && (
          <>
            <Text size="sm">
              RMSE {formatValue(result.initialRmse)} → <b>{formatValue(result.rmse)}</b> in{' '}
              {result.evaluations} evaluations
              {result.truncated ? ' (stopped at the time budget)' : ''}.
            </Text>
            <Table withTableBorder maw={420} fz="sm">
              <Table.Tbody>
                {result.parameterNames.map((name, i) => (
                  <Table.Tr key={name}>
                    <Table.Td ff="monospace">{name}</Table.Td>
                    <Table.Td ta="right" ff="monospace">
                      {formatValue(result.fittedValues[i])}
                    </Table.Td>
                  </Table.Tr>
                ))}
              </Table.Tbody>
            </Table>
            <Group>
              <Button
                variant="light"
                onClick={() => {
                  onApply(applyFittedParameters(text, result.parameterNames, result.fittedValues))
                  onClose()
                }}
              >
                Apply to editor
              </Button>
            </Group>
          </>
        )}
      </Stack>
    </Modal>
  )
}
