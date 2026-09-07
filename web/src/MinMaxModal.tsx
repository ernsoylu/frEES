import { useState } from 'react'
import {
  Alert,
  Badge,
  Button,
  Group,
  Modal,
  MultiSelect,
  NumberInput,
  Paper,
  SegmentedControl,
  Select,
  Stack,
  Table,
  Text,
  Textarea,
  TextInput,
} from '@mantine/core'
import { applyFittedParameters } from './ParameterFitModal'
import {
  optimize,
  optimizeMulti,
  FunctionTableDto,
  MultiObjectiveParams,
  OptimizeMethod,
  OptimizeResponse,
  ParetoResponse,
  StopCriteria,
  UnitSystem,
  VariableInfo,
  VariableResult,
} from './api'
import PlotlyChart from './plots/PlotlyChart'
import { formatValue } from './format'

const MONO_INPUT = {
  input: { fontFamily: 'var(--mantine-font-family-monospace)' },
}

const METHOD_OPTIONS = [
  { value: 'brent', label: "Brent (1-D)" },
  { value: 'nelder-mead', label: 'Nelder-Mead Simplex' },
  { value: 'bobyqa', label: 'BOBYQA' },
]

type Mode = 'single' | 'multi'

interface Bounds {
  lower: string
  upper: string
}

function OptimumLine({
  label,
  variable,
}: Readonly<{ label: string; variable: VariableResult | null }>) {
  if (variable === null) return null
  return (
    <Group gap="xs">
      <Text size="sm" c="dimmed" w={90}>
        {label}
      </Text>
      <Text size="sm" ff="monospace" c="green.4">
        {variable.name} = {formatValue(variable.value)}
      </Text>
      <Text size="sm" ff="monospace" c="dimmed">
        {variable.units || ''}
      </Text>
    </Group>
  )
}

function ResultView({
  result,
  text,
  onApply,
}: Readonly<{
  result: OptimizeResponse
  text: string
  onApply?: (nextText: string) => void
}>) {
  const [applied, setApplied] = useState(false)
  if (!result.success) {
    return (
      <Alert color="red" variant="light" p="xs">
        <Text size="sm" style={{ whiteSpace: 'pre-wrap' }}>
          {result.error}
        </Text>
      </Alert>
    )
  }
  return (
    <Stack gap="xs">
      <Group gap="xs" justify="space-between">
        <Group gap="xs">
          <Badge color="green" variant="light" leftSection="✓">
            Optimum found
          </Badge>
          <Text size="xs" c="dimmed">
            {result.evaluations} objective evaluations
          </Text>
        </Group>
        {onApply && (
          <Button
            size="xs"
            color={applied ? 'teal' : 'green'}
            variant="light"
            onClick={() => {
              const names = result.decisions.map((d) => d.name)
              const values = result.decisions.map((d) => d.value)
              onApply(applyFittedParameters(text, names, values))
              setApplied(true)
              setTimeout(() => setApplied(false), 2500)
            }}
          >
            {applied ? '✓ Applied to Document' : 'Load Optimum into Document'}
          </Button>
        )}
      </Group>
      {result.warning && (
        <Alert color="yellow" variant="light" p="xs">
          <Text size="sm" style={{ whiteSpace: 'pre-wrap' }}>
            {result.warning}
          </Text>
        </Alert>
      )}
      <OptimumLine label="Objective" variable={result.objective} />
      {result.decisions.map((d) => (
        <OptimumLine key={d.name} label="At" variable={d} />
      ))}
      <Table striped highlightOnHover>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Variable</Table.Th>
            <Table.Th>Value</Table.Th>
            <Table.Th>Units</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {result.variables.map((v) => (
            <Table.Tr key={v.name}>
              <Table.Td>{v.name}</Table.Td>
              <Table.Td ff="monospace" c="green.4">
                {formatValue(v.value)}
              </Table.Td>
              <Table.Td ff="monospace" c="dimmed">
                {v.units || '-'}
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Stack>
  )
}

function ParetoView({
  result,
  text,
  onApply,
}: Readonly<{
  result: ParetoResponse
  text: string
  onApply?: (nextText: string) => void
}>) {
  const [selectedPointIndex, setSelectedPointIndex] = useState<number | null>(null)
  const [applied, setApplied] = useState(false)

  if (!result.success) {
    return (
      <Alert color="red" variant="light" p="xs">
        <Text size="sm" style={{ whiteSpace: 'pre-wrap' }}>
          {result.error}
        </Text>
      </Alert>
    )
  }
  const { front, objectiveNames, decisionNames } = result
  const twoObjectives = objectiveNames.length === 2
  const selectedPoint = selectedPointIndex !== null && selectedPointIndex < front.length ? front[selectedPointIndex] : null

  const figure = twoObjectives
    ? {
        data: [
          {
            x: front.map((p) => p.objectives[0]),
            y: front.map((p) => p.objectives[1]),
            type: 'scatter' as const,
            mode: 'markers+lines' as const,
            marker: { color: '#69db7c', size: 8 },
            line: { color: '#2f9e44', width: 1 },
            name: 'Pareto front',
            customdata: front.map((_, i) => i),
          },
          ...(selectedPoint
            ? [
                {
                  x: [selectedPoint.objectives[0]],
                  y: [selectedPoint.objectives[1]],
                  type: 'scatter' as const,
                  mode: 'markers' as const,
                  marker: { color: '#ffa94d', size: 14, symbol: 'diamond', line: { color: '#d9480f', width: 2 } },
                  name: `Point #${selectedPointIndex + 1}`,
                },
              ]
            : []),
        ],
        layout: {
          xaxis: { title: { text: objectiveNames[0] }, gridcolor: '#373a40', zeroline: false },
          yaxis: { title: { text: objectiveNames[1] }, gridcolor: '#373a40', zeroline: false },
          paper_bgcolor: 'transparent',
          plot_bgcolor: 'transparent',
          font: { color: '#c1c2c5' },
          margin: { t: 10, r: 10, b: 44, l: 56 },
          showlegend: false,
        },
      }
    : null

  const handleApply = (point: { decisions: number[] }) => {
    if (!onApply) return
    const nextText = applyFittedParameters(text, decisionNames, point.decisions)
    onApply(nextText)
    setApplied(true)
    setTimeout(() => setApplied(false), 2500)
  }

  return (
    <Stack gap="xs">
      <Group gap="xs" justify="space-between">
        <Group gap="xs">
          <Badge color="green" variant="light" leftSection="✓">
            {front.length} Pareto-optimal points
          </Badge>
          <Text size="xs" c="dimmed">
            {result.evaluations} evaluations
          </Text>
        </Group>
        <Text size="xs" c="dimmed">
          Click a point or row to inspect & load into document
        </Text>
      </Group>

      {figure && (
        <PlotlyChart
          figure={figure}
          minHeight={280}
          onPointClick={(pt) => {
            if (pt.pointIndex >= 0 && pt.pointIndex < front.length) {
              setSelectedPointIndex(pt.pointIndex)
              setApplied(false)
            }
          }}
        />
      )}

      {selectedPoint && (
        <Paper withBorder p="xs" radius="sm" style={{ backgroundColor: 'var(--mantine-color-dark-6)' }}>
          <Stack gap="xs">
            <Group justify="space-between" align="center">
              <Group gap="xs">
                <Badge color="orange" variant="filled">
                  Selected Point #{selectedPointIndex + 1}
                </Badge>
                <Text size="xs" c="dimmed">
                  of {front.length}
                </Text>
              </Group>
              {onApply && (
                <Button
                  size="xs"
                  color={applied ? 'teal' : 'orange'}
                  variant="light"
                  onClick={() => handleApply(selectedPoint)}
                >
                  {applied ? '✓ Applied to Document' : 'Load Point into Document'}
                </Button>
              )}
            </Group>

            <Group gap="md" wrap="wrap">
              <Stack gap={2}>
                <Text size="xs" fw={700} c="dimmed">
                  OBJECTIVES
                </Text>
                <Group gap="xs">
                  {objectiveNames.map((name, j) => (
                    <Badge key={name} variant="outline" color="green">
                      {name} = {formatValue(selectedPoint.objectives[j])}
                    </Badge>
                  ))}
                </Group>
              </Stack>
              <Stack gap={2}>
                <Text size="xs" fw={700} c="dimmed">
                  DECISION VARIABLES
                </Text>
                <Group gap="xs">
                  {decisionNames.map((name, j) => (
                    <Badge key={name} variant="outline" color="cyan">
                      {name} = {formatValue(selectedPoint.decisions[j])}
                    </Badge>
                  ))}
                </Group>
              </Stack>
            </Group>
          </Stack>
        </Paper>
      )}

      {!twoObjectives && (
        <Text size="xs" c="dimmed">
          Plotting is shown for exactly two objectives; the table below lists every point.
        </Text>
      )}

      <Table striped highlightOnHover withTableBorder>
        <Table.Thead>
          <Table.Tr>
            <Table.Th style={{ width: 40 }}>#</Table.Th>
            {objectiveNames.map((name) => (
              <Table.Th key={`o-${name}`}>{name}</Table.Th>
            ))}
            {decisionNames.map((name) => (
              <Table.Th key={`d-${name}`}>{name}</Table.Th>
            ))}
            {onApply && <Table.Th style={{ width: 90 }}>Action</Table.Th>}
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {front.map((p, i) => {
            const isSelected = selectedPointIndex === i
            return (
              <Table.Tr
                key={i}
                onClick={() => {
                  setSelectedPointIndex(i)
                  setApplied(false)
                }}
                style={{
                  cursor: 'pointer',
                  backgroundColor: isSelected ? 'var(--mantine-color-blue-light)' : undefined,
                }}
              >
                <Table.Td>
                  <Text size="xs" c={isSelected ? 'blue.4' : 'dimmed'} fw={isSelected ? 700 : 400}>
                    {i + 1}
                  </Text>
                </Table.Td>
                {p.objectives.map((v, j) => (
                  <Table.Td key={`o${j}`} ff="monospace" c="green.4">
                    {formatValue(v)}
                  </Table.Td>
                ))}
                {p.decisions.map((v, j) => (
                  <Table.Td key={`d${j}`} ff="monospace" c={isSelected ? 'cyan.3' : 'dimmed'} fw={isSelected ? 700 : 400}>
                    {formatValue(v)}
                  </Table.Td>
                ))}
                {onApply && (
                  <Table.Td onClick={(e) => e.stopPropagation()}>
                    <Button
                      size="compact-xs"
                      variant={isSelected ? 'filled' : 'subtle'}
                      color={isSelected ? 'orange' : 'gray'}
                      onClick={() => handleApply(p)}
                    >
                      Load
                    </Button>
                  </Table.Td>
                )}
              </Table.Tr>
            )
          })}
        </Table.Tbody>
      </Table>
    </Stack>
  )
}

interface Props {
  variables: string[]
  text: string
  stopCriteria: StopCriteria
  complexMode: boolean
  variableInfo: VariableInfo[]
  unitSystem: UnitSystem
  getFunctionTables: () => FunctionTableDto[]
  onClose: () => void
  onApply?: (nextText: string) => void
}

/**
 * Calculate > Min/Max: optimization of an objective over independent variables.
 * Single-objective uses Brent / Nelder-Mead / BOBYQA; the Multi-objective tab
 * runs NSGA-II and returns the Pareto front.
 */
export default function MinMaxModal({
  variables,
  text,
  stopCriteria,
  complexMode,
  variableInfo,
  unitSystem,
  getFunctionTables,
  onClose,
  onApply,
}: Readonly<Props>) {
  const [mode, setMode] = useState<Mode>('single')
  const [goal, setGoal] = useState<'minimize' | 'maximize'>('minimize')
  const [objective, setObjective] = useState<string | null>(null)
  const [objectives, setObjectives] = useState<string[]>([])
  const [objGoals, setObjGoals] = useState<Record<string, 'min' | 'max'>>({})
  const [population, setPopulation] = useState<number>(40)
  const [generations, setGenerations] = useState<number>(40)
  const [decisions, setDecisions] = useState<string[]>([])
  const [bounds, setBounds] = useState<Record<string, Bounds>>({})
  const [method, setMethod] = useState<OptimizeMethod>('brent')
  const [constraints, setConstraints] = useState('')
  const [running, setRunning] = useState(false)
  const [validation, setValidation] = useState<string | null>(null)
  const [result, setResult] = useState<OptimizeResponse | null>(null)
  const [pareto, setPareto] = useState<ParetoResponse | null>(null)

  function onDecisionsChange(selected: string[]) {
    setDecisions(selected)
    setBounds((prev) => {
      const next: Record<string, Bounds> = {}
      for (const name of selected) {
        next[name] = prev[name] ?? { lower: '', upper: '' }
      }
      return next
    })
    if (selected.length > 1 && method === 'brent') setMethod('nelder-mead')
    if (selected.length === 1) setMethod('brent')
  }

  function onObjectivesChange(selected: string[]) {
    setObjectives(selected)
    setObjGoals((prev) => {
      const next: Record<string, 'min' | 'max'> = {}
      for (const name of selected) next[name] = prev[name] ?? 'min'
      return next
    })
  }

  function setBound(name: string, key: keyof Bounds, value: string) {
    setBounds((prev) => ({ ...prev, [name]: { ...prev[name], [key]: value } }))
  }

  function constraintLines(): string[] {
    return constraints.split('\n').map((l) => l.trim()).filter((l) => l !== '')
  }

  function validateBounds(): string | null {
    if (decisions.length === 0) return 'Choose at least one independent variable.'
    for (const name of decisions) {
      const b = bounds[name]
      const lo = Number(b?.lower)
      const hi = Number(b?.upper)
      if (!b || b.lower.trim() === '' || b.upper.trim() === '' || !Number.isFinite(lo) || !Number.isFinite(hi)) {
        return `Both bounds of ${name} must be numbers.`
      }
      if (lo >= hi) return `The lower bound of ${name} must be less than the upper bound.`
    }
    return null
  }

  function validateSingle(): string | null {
    if (objective === null) return 'Choose an objective variable.'
    if (decisions.includes(objective)) return 'The objective cannot also be an independent variable.'
    const boundsError = validateBounds()
    if (boundsError) return boundsError
    if (method === 'brent' && decisions.length > 1) {
      return "Brent's method is 1-D only; pick Nelder-Mead or BOBYQA."
    }
    for (const line of constraintLines()) {
      if (!/(<=|>=|=)/.test(line)) return `Constraint "${line}" needs <=, >= or = followed by a number.`
    }
    return null
  }

  function validateMulti(): string | null {
    if (objectives.length < 2) return 'Choose at least two objective variables.'
    for (const name of objectives) {
      if (decisions.includes(name)) return `${name} cannot be both an objective and an independent variable.`
    }
    const boundsError = validateBounds()
    if (boundsError) return boundsError
    for (const line of constraintLines()) {
      if (!/(<=|>=|=)/.test(line)) return `Constraint "${line}" needs <=, >= or = followed by a number.`
    }
    return null
  }

  async function run() {
    const problem = mode === 'single' ? validateSingle() : validateMulti()
    setValidation(problem)
    setResult(null)
    setPareto(null)
    if (problem !== null || running) return
    setRunning(true)
    try {
      if (mode === 'single') {
        const response = await optimize(
          text,
          { ...stopCriteria, complexMode },
          variableInfo,
          unitSystem,
          {
            objective: objective ?? '',
            decisions,
            lowers: decisions.map((name) => Number(bounds[name].lower)),
            uppers: decisions.map((name) => Number(bounds[name].upper)),
            method,
            maximize: goal === 'maximize',
            constraints: constraintLines(),
          },
          getFunctionTables(),
        )
        setResult(response)
      } else {
        const params: MultiObjectiveParams = {
          objectives,
          maximize: objectives.map((name) => objGoals[name] === 'max'),
          decisions,
          lowers: decisions.map((name) => Number(bounds[name].lower)),
          uppers: decisions.map((name) => Number(bounds[name].upper)),
          populationSize: population,
          generations,
          constraints: constraintLines(),
        }
        setPareto(
          await optimizeMulti(
            text,
            { ...stopCriteria, complexMode },
            variableInfo,
            params,
            getFunctionTables(),
          ),
        )
      }
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      if (mode === 'single') {
        setResult({ success: false, error: message, warning: null, objective: null, decision: null, decisions: [], evaluations: 0, variables: [] })
      } else {
        setPareto({ success: false, error: message, decisionNames: [], objectiveNames: [], front: [], evaluations: 0 })
      }
    } finally {
      setRunning(false)
    }
  }

  const decisionTable = decisions.length > 0 && (
    <Table withRowBorders={false} verticalSpacing={4}>
      <Table.Thead>
        <Table.Tr>
          <Table.Th>Variable</Table.Th>
          <Table.Th>Lower bound</Table.Th>
          <Table.Th>Upper bound</Table.Th>
        </Table.Tr>
      </Table.Thead>
      <Table.Tbody>
        {decisions.map((name) => (
          <Table.Tr key={name}>
            <Table.Td ff="monospace">{name}</Table.Td>
            <Table.Td>
              <TextInput
                value={bounds[name]?.lower ?? ''}
                onChange={(e) => setBound(name, 'lower', e.currentTarget.value)}
                spellCheck={false}
                size="xs"
                styles={MONO_INPUT}
                aria-label={`Lower bound of ${name}`}
              />
            </Table.Td>
            <Table.Td>
              <TextInput
                value={bounds[name]?.upper ?? ''}
                onChange={(e) => setBound(name, 'upper', e.currentTarget.value)}
                spellCheck={false}
                size="xs"
                styles={MONO_INPUT}
                aria-label={`Upper bound of ${name}`}
              />
            </Table.Td>
          </Table.Tr>
        ))}
      </Table.Tbody>
    </Table>
  )

  return (
    <Modal opened onClose={onClose} title="Min/Max — Optimization" centered size="xl">
      <Stack gap="sm">
        <SegmentedControl
          value={mode}
          onChange={(value) => setMode(value as Mode)}
          data={[
            { label: 'Single objective', value: 'single' },
            { label: 'Multi-objective (Pareto)', value: 'multi' },
          ]}
        />

        {mode === 'single' ? (
          <>
            <SegmentedControl
              value={goal}
              onChange={(value) => setGoal(value as 'minimize' | 'maximize')}
              data={[
                { label: 'Minimize', value: 'minimize' },
                { label: 'Maximize', value: 'maximize' },
              ]}
            />
            <Select
              label="Objective variable"
              data={variables}
              value={objective}
              onChange={setObjective}
              placeholder="variable to optimize"
              searchable
            />
          </>
        ) : (
          <>
            <MultiSelect
              label="Objective variables (2 or more)"
              data={variables.filter((v) => !decisions.includes(v))}
              value={objectives}
              onChange={onObjectivesChange}
              placeholder={objectives.length === 0 ? 'conflicting objectives to trade off' : undefined}
              searchable
              clearable
            />
            {objectives.map((name) => (
              <Group key={name} gap="sm" justify="space-between">
                <Text size="sm" ff="monospace">{name}</Text>
                <SegmentedControl
                  size="xs"
                  value={objGoals[name] ?? 'min'}
                  onChange={(value) => setObjGoals((prev) => ({ ...prev, [name]: value as 'min' | 'max' }))}
                  data={[
                    { label: 'Minimize', value: 'min' },
                    { label: 'Maximize', value: 'max' },
                  ]}
                />
              </Group>
            ))}
          </>
        )}

        <MultiSelect
          label="Independent (varied) variables"
          data={variables.filter((v) => (mode === 'single' ? v !== objective : !objectives.includes(v)))}
          value={decisions}
          onChange={onDecisionsChange}
          placeholder={decisions.length === 0 ? 'variables to vary' : undefined}
          searchable
          clearable
        />

        {decisionTable}

        {mode === 'single' ? (
          <>
            <Select
              label="Method"
              description={decisions.length > 1 ? "Brent's method only works with a single independent variable." : undefined}
              data={METHOD_OPTIONS.map((option) => ({
                ...option,
                disabled: option.value === 'brent' && decisions.length > 1,
              }))}
              value={method}
              onChange={(value) => setMethod((value as OptimizeMethod) ?? 'brent')}
              allowDeselect={false}
            />
            <Textarea
              label="Constraints (optional)"
              description="One per line: expr <= value, expr >= value, or expr = value."
              placeholder={'r + h <= 20'}
              value={constraints}
              onChange={(e) => setConstraints(e.currentTarget.value)}
              autosize
              minRows={2}
              maxRows={6}
              spellCheck={false}
              styles={MONO_INPUT}
            />
          </>
        ) : (
          <>
            <Group grow>
              <NumberInput
                label="Population size"
                value={population}
                onChange={(v) => setPopulation(typeof v === 'number' ? v : 40)}
                min={8}
                max={200}
                step={8}
              />
              <NumberInput
                label="Generations"
                value={generations}
                onChange={(v) => setGenerations(typeof v === 'number' ? v : 40)}
                min={1}
                max={200}
                step={10}
              />
            </Group>
            <Textarea
              label="Constraints (optional)"
              description="One per line: expr <= value, expr >= value, or expr = value. Infeasible points are dominated by feasible ones."
              placeholder={'mass <= 50'}
              value={constraints}
              onChange={(e) => setConstraints(e.currentTarget.value)}
              autosize
              minRows={2}
              maxRows={6}
              spellCheck={false}
              styles={MONO_INPUT}
            />
          </>
        )}

        {validation && (
          <Text c="red" size="sm">
            {validation}
          </Text>
        )}

        {mode === 'single' && result && <ResultView result={result} text={text} onApply={onApply} />}
        {mode === 'multi' && pareto && <ParetoView result={pareto} text={text} onApply={onApply} />}

        <Group justify="flex-end" mt="xs">
          <Button variant="default" onClick={onClose}>
            Close
          </Button>
          <Button onClick={run} loading={running}>
            {mode === 'multi' ? 'Find Pareto front' : goal === 'maximize' ? 'Maximize' : 'Minimize'}
          </Button>
        </Group>
      </Stack>
    </Modal>
  )
}
