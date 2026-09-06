import { useEffect, useState } from 'react'
import { Button, Stack, Tabs, Text } from '@mantine/core'
import { StateTableDto, TableRowResult, VariableResult, getFluids } from './api'
import { ParamRow, TableSpec } from './tables'
import { PlotKind, PlotSpec } from './plots/types'
import { detectStates } from './plots/stateTable'
import { resolvePlotSource } from './plots/sources'
import PlotCard from './plots/PlotCard'
import PlotConfigModal from './plots/PlotConfigModal'

interface Props {
  /** Which plot kinds this window shows and creates; the spec list is
   * shared, so the Plots and Thermodynamics windows each see their slice. */
  kinds: PlotKind[]
  emptyHint: string
  tables?: TableSpec[]
  plots: PlotSpec[]
  onPlotsChange: (plots: PlotSpec[]) => void
  solvedVariables: VariableResult[]
  /** Declared STATE TABLE blocks, for fluid-aware per-circuit state overlays. */
  stateTableDefs?: StateTableDto[]
  cyclePath?: Record<string, number>[]
  tableVars: string[]
  rows: ParamRow[]
  results: TableRowResult[]
  /** Per-column SI units of the active table (ODE/code), for axis annotation. */
  tableUnits?: Record<string, string>
  activePlotId?: string | null
  onActivePlotIdChange?: (id: string | null) => void
  hideHeader?: boolean
  exportTrigger?: { format: string; timestamp: number } | null
  /** When set, render only this one plot and hide the plot-tab strip + Add
   *  (used when each plot is its own dock window). */
  singlePlotId?: string
}

/**
 * A plot window: any number of plots of the given kinds, each with its own
 * configuration, format options and export menu.
 */
export default function PlotTab({
  kinds,
  tables = [],
  emptyHint,
  plots,
  onPlotsChange,
  solvedVariables,
  stateTableDefs,
  cyclePath,
  tableVars,
  rows,
  results,
  tableUnits,
  activePlotId,
  onActivePlotIdChange,
  hideHeader = false,
  exportTrigger = null,
  singlePlotId,
}: Readonly<Props>) {
  const visible = plots.filter((p) => kinds.includes(p.kind))
  const [fluids, setFluids] = useState<string[]>([])
  const [localActivePlot, setLocalActivePlot] = useState<string | null>(null)
  const [editing, setEditing] = useState<PlotSpec | null>(null)
  const [adding, setAdding] = useState(false)

  const activePlot = activePlotId === undefined ? (localActivePlot ?? visible[0]?.id ?? null) : activePlotId
  const setActivePlot = (id: string | null) => {
    if (onActivePlotIdChange) {
      onActivePlotIdChange(id)
    } else {
      setLocalActivePlot(id)
    }
  }

  const needsFluids = editing?.kind === 'property' || (adding && kinds.includes('property'))
  useEffect(() => {
    if (needsFluids) void getFluids().then(setFluids)
  }, [needsFluids])

  useEffect(() => {
    if (visible.length > 0 && (activePlot === null || !visible.some((p) => p.id === activePlot))) {
      setActivePlot(visible[0].id)
    }
  }, [visible, activePlot])

  const states = detectStates(solvedVariables)

  function addPlot(spec: PlotSpec) {
    onPlotsChange([...plots, spec])
    setActivePlot(spec.id)
    setAdding(false)
  }

  function updatePlot(spec: PlotSpec) {
    onPlotsChange(plots.map((p) => (p.id === spec.id ? spec : p)))
    setEditing(null)
  }

  function removePlot(id: string) {
    onPlotsChange(plots.filter((p) => p.id !== id))
    if (activePlot === id) {
      const remaining = visible.find((p) => p.id !== id)
      setActivePlot(remaining?.id ?? null)
    }
  }

  const current = singlePlotId
    ? (visible.find((p) => p.id === singlePlotId) ?? null)
    : (visible.find((p) => p.id === activePlot) ?? visible[0] ?? null)

  const source = current ? resolvePlotSource(current, tables, solvedVariables) : undefined
  const sourceTable = source?.kind === 'table' ? tables.find((t) => t.id === source.tableId) : undefined
  const boundTable = sourceTable?.kind === 'parametric' ? sourceTable : undefined
  const boundSpec = current && { ...current, source: source?.kind === 'table' && !boundTable ? undefined : source }

  return (
    <Stack gap="sm" style={{ flex: 1, minHeight: 0 }}>
      {(adding || editing) && (
        <PlotConfigModal
          spec={editing}
          allowedKinds={kinds}
          defaultName={editing ? editing.name : `Plot ${visible.length + 1}`}
          fluids={fluids}
          tables={tables}
          tableVars={tableVars}
          hasStates={states.indices.length > 0}
          stateTables={stateTableDefs}
          onSave={editing ? updatePlot : addPlot}
          onClose={() => {
            setAdding(false)
            setEditing(null)
          }}
        />
      )}

      {current === null && (
        <Stack gap="xs" align="center" justify="center" style={{ flex: 1 }}>
          <Text size="sm" c="dimmed">
            {emptyHint}
          </Text>
          <Button size="xs" onClick={() => setAdding(true)}>
            Add Plot
          </Button>
        </Stack>
      )}

      {current !== null && (
        <PlotCard
          key={current.id}
          spec={boundSpec!}
          states={states}
          cyclePath={cyclePath}
          tableRows={boundTable?.rows ?? []}
          tableResults={boundTable?.results ?? []}
          variables={solvedVariables}
          tableUnits={boundTable?.columnUnits}
          stateTableDefs={stateTableDefs}
          onConfigure={() => setEditing(boundSpec)}
          onRemove={() => removePlot(current.id)}
          hideHeader={hideHeader}
          exportTrigger={exportTrigger}
          leftSection={
            singlePlotId ? undefined : (
              <Tabs
                value={current.id}
                onChange={(id) => id && setActivePlot(id)}
                variant="pills"
                styles={{ tab: { height: 26, fontSize: 12, padding: '0 8px' } }}
              >
                <Tabs.List>
                  {visible.map((p) => (
                    <Tabs.Tab key={p.id} value={p.id}>
                      {p.name}
                    </Tabs.Tab>
                  ))}
                </Tabs.List>
              </Tabs>
            )
          }
          rightSection={
            singlePlotId ? undefined : (
              <Button size="xs" onClick={() => setAdding(true)}>
                Add Plot
              </Button>
            )
          }
        />
      )}
    </Stack>
  )
}
