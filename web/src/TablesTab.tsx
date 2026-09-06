import { Button, Group, Stack, Text, Tooltip } from '@mantine/core'
import { lazy, Suspense, useState } from 'react'
import { duplicateAsEditable, FunctionTableSpec, TableSpec } from './tables'
import { VariableDraft } from './VariableInfoModal'

// Solver-produced (read-only) tables can be huge; render them through a
// virtualized canvas grid, code-split so glide-data-grid stays out of the
// initial bundle and only loads when such a table is opened.
const DataGridReadOnly = lazy(() => import('./DataGridReadOnly'))
// Sweep → Function (Wave H): also code-split — it is only needed on demand.
const CreateFunctionModal = lazy(() => import('./tablesGrid/CreateFunctionModal'))

// ---------------------------------------------------------------------------
// Read-only table window: code-defined PARAMETRIC tables and solved ODE
// trajectories, each in its own dock window ("table:<id>"). Editable tables
// (GUI parametric + all function/lookup tables) live in the Tables workbook
// (tablesGrid/TablesGridTab) — the Mantine editors this file used to host
// were retired in Phase 4 of the unification plan.
// ---------------------------------------------------------------------------

interface Props {
  tables: TableSpec[]
  /** The one table this window renders. */
  singleTableId: string
  varDrafts: Record<string, VariableDraft>
  /** Open a new X-Y plot from the column selection (x = first column / time,
   * y = the selected columns). */
  onPlotColumns?: (xVar: string, yVars: string[]) => void
  /** Make an editable GUI copy (decoupled from the editor text); the copy
   * opens in the Tables workbook. */
  onCopyToEditable?: (copy: TableSpec) => void
  /** Sweep → Function (Wave H): receives the Function Table specs produced
   * from this table's columns; the host applies replace-vs-add and opens the
   * Tables workbook. */
  onCreateFunctionTables?: (specs: FunctionTableSpec[]) => void
}

export default function TablesTab(props: Readonly<Props>) {
  const { tables, singleTableId } = props
  const active = tables.find((t) => t.id === singleTableId) ?? null
  const [createFnOpen, setCreateFnOpen] = useState(false)

  if (!active) {
    return (
      <Text size="sm" c="dimmed" mt="md">
        Table not found — it may have been removed or renamed by the last solve.
      </Text>
    )
  }

  if (active.kind !== 'parametric' || active.source !== 'code') {
    // Editable tables are hosted in the Tables workbook; this window kind
    // only ever backs read-only code/ODE tables.
    return (
      <Text size="sm" c="dimmed" mt="md">
        This table lives in the Tables window now.
      </Text>
    )
  }

  return (
    <Stack gap="xs" style={{ flex: 1, minHeight: 0 }}>
      <Group gap="xs" wrap="nowrap" align="center">
        <Text size="xs" c="dimmed" style={{ flex: 1 }}>
          {active.origin === 'ode'
            ? 'Solved trajectory from a DYNAMIC (ODE) block — virtualized read-only grid (columns are SI-solver values; drag column edges to resize, click-drag to select/copy).'
            : 'Defined in code (PARAMETRIC … END) — virtualized read-only grid. Run it with Solve Table.'}
        </Text>
        {props.onCreateFunctionTables && active.vars.length >= 2 && (
          <Tooltip label="Turn two columns of this table into a Function Table callable in equations">
            <Button size="compact-xs" variant="default" onClick={() => setCreateFnOpen(true)}>
              Create function…
            </Button>
          </Tooltip>
        )}
        {props.onCopyToEditable && (
          <Tooltip label="Make an editable copy in the Tables workbook (decoupled from the editor text)">
            <Button
              size="compact-xs"
              variant="default"
              onClick={() => props.onCopyToEditable?.(duplicateAsEditable(active))}
            >
              Editable copy
            </Button>
          </Tooltip>
        )}
      </Group>
      {active.stats?.converged === false && <Text c="orange" size="xs">
        Table not converged after {active.stats.passes} passes ({active.stats.termination}); values are provisional.
      </Text>}
      <Suspense fallback={<Text size="sm" c="dimmed">Loading grid…</Text>}>
        <DataGridReadOnly
          vars={active.vars}
          rows={active.rows}
          results={active.results}
          varDrafts={props.varDrafts}
          columnUnits={active.columnUnits}
          onPlotColumns={props.onPlotColumns}
        />
      </Suspense>
      {createFnOpen && props.onCreateFunctionTables && (
        <Suspense fallback={null}>
          <CreateFunctionModal
            table={active}
            tables={tables}
            onCreate={props.onCreateFunctionTables}
            onClose={() => setCreateFnOpen(false)}
          />
        </Suspense>
      )}
    </Stack>
  )
}
