import { lazy, Suspense, useDeferredValue, useMemo, useState } from 'react'
import {
  Badge,
  Group,
  Paper,
  Stack,
  Table,
  Text,
  TextInput,
  ThemeIcon,
  ActionIcon,
  Tooltip,
} from '@mantine/core'
import {
  IconAdjustments,
  IconAdjustmentsHorizontal,
  IconChevronLeft,
  IconChevronRight,
  IconComponents,
  IconPencil,
  IconSearch,
  IconTable,
  IconVariable,
} from '@tabler/icons-react'
import { Button } from '@mantine/core'
import { ComponentParamResult, ComponentResult, SolveResponse, VariableResult } from './api'
import SolveDiagnostics from './SolveDiagnostics'
import { formatValue } from './format'

/**
 * The advanced Workspace (array-language-style variable window): a live, typed view of
 * the current solved state. Scalars list in a sortable table with value, type,
 * unit and uncertainty; vectors and matrices group under expandable rows that
 * reveal their grid. Updates whenever the document is re-solved (its source is
 * the same `result.variables` the rest of the app reads).
 */

import { group, ArrayGroup } from './workspaceData'

// Beyond these sizes the Mantine tables (one DOM node per cell) are replaced by
// lazy-loaded virtualized canvas grids, so render cost stops scaling with the
// solved system. Below them the richer DOM tables (badges, hover) are kept.
const EMPTY_NAMES: Set<string> = new Set()
const VIRTUALIZE_SCALARS_AT = 200
const VIRTUALIZE_CELLS_AT = 400
const ScalarGrid = lazy(() => import('./WorkspaceGrids').then((m) => ({ default: m.ScalarGrid })))
const MatrixGrid = lazy(() => import('./WorkspaceGrids').then((m) => ({ default: m.MatrixGrid })))
const gridFallback = <Text size="xs" c="dimmed">Loading grid…</Text>

function typeLabel(g: ArrayGroup): string {
  return g.is2D ? `${g.rows.length}×${g.cols.length} Matrix` : `${g.rows.length}×1 Vector`
}

/**
 * A solved COMPONENT instance presented as a datasheet: its given inputs (the
 * parameter bindings it was built with) alongside its computed outputs (the
 * solver variables, which flatten to dotted port-member names like `chlr.out.h`
 * / `cmp.in.mdot`). The workspace would otherwise scatter the members across the
 * scalar list and hide the parameters entirely — here they regroup under the
 * instance name with the component type as a pill, the same affordance
 * arrays/matrices already get.
 */
export interface ComponentGroup {
  name: string
  /** Component type (e.g. `TwoPhaseEvaporatorUA`), from the backend metadata. */
  type?: string
  /** Input parameter bindings (`UA=UA_chl_r`, `SH=5`, `fluid$=R1234yf`). */
  params: ComponentParamResult[]
  /** Computed port/output members, with the instance prefix stripped from the label. */
  members: { v: VariableResult; label: string }[]
}

/**
 * Splits scalars into plain scalars and component instances. The instance list
 * (`instances`, from the backend) is authoritative — it carries every
 * component's type and parameters, so even an instance with no output variables
 * still shows its inputs. Dotted scalar names (`inst.member`) attach their
 * member to the matching instance; in frees only COMPONENT port/output members
 * display with a dot, so an unrecognized prefix still forms its own group rather
 * than leaking into the scalar list. Fully empty instances (no params, no
 * members) are dropped.
 */
export function groupComponents(
  scalars: VariableResult[],
  instances: ComponentResult[],
): { plain: VariableResult[]; components: ComponentGroup[] } {
  const plain: VariableResult[] = []
  const comps = new Map<string, ComponentGroup>()
  // Seed from the authoritative instance list (keyed lowercase: frees names are
  // case-insensitive and members display lowercased).
  for (const c of instances) {
    comps.set(c.name.toLowerCase(), { name: c.name, type: c.type, params: c.params ?? [], members: [] })
  }
  for (const v of scalars) {
    const dot = v.name.indexOf('.')
    if (dot <= 0) {
      plain.push(v)
      continue
    }
    const inst = v.name.slice(0, dot)
    const key = inst.toLowerCase()
    let c = comps.get(key)
    if (!c) {
      c = { name: inst, params: [], members: [] }
      comps.set(key, c)
    }
    c.members.push({ v, label: v.name.slice(dot + 1) })
  }
  for (const c of comps.values()) {
    c.members.sort((a, b) => a.label.localeCompare(b.label))
  }
  return {
    plain,
    components: Array.from(comps.values())
      .filter((c) => c.members.length > 0 || c.params.length > 0)
      .sort((a, b) => a.name.localeCompare(b.name)),
  }
}

function uncertaintyText(v: VariableResult): string {
  return v.uncertainty != null && v.uncertainty !== 0 ? `± ${formatValue(v.uncertainty)}` : ''
}

function ScalarTable({ scalars, replNames, pinnedNames, pinnableNames, onPin }: Readonly<{
  scalars: VariableResult[]
  replNames: Set<string>
  pinnedNames: Set<string>
  pinnableNames: Set<string>
  onPin?: (v: VariableResult) => void
}>) {
  return (
    // The variable names + 5 columns have an intrinsic min-width that exceeds a
    // narrow dock/edge panel; scroll horizontally inside the panel rather than
    // letting the table overflow and get clipped. `type="native"` is a plain
    // overflow-x:auto block whose min-content collapses to 0 — so the table's
    // min-width can't push the (row-reverse) edge group wider than its slot and
    // clip the panel on the left.
    <Table.ScrollContainer type="native" minWidth={380}>
    <Table striped highlightOnHover>
      <Table.Thead>
        <Table.Tr>
          <Table.Th>Name</Table.Th>
          <Table.Th>Value</Table.Th>
          <Table.Th>Type</Table.Th>
          <Table.Th>Units</Table.Th>
          <Table.Th>Uncertainty</Table.Th>
        </Table.Tr>
      </Table.Thead>
      <Table.Tbody>
        {scalars.map((v) => (
          <Table.Tr key={v.name}>
            <Table.Td style={{ textTransform: 'none' }}>
              <Group gap={6} wrap="nowrap">
                {onPin && pinnableNames.has(v.name.toLowerCase()) && (
                  <Tooltip
                    label={
                      pinnedNames.has(v.name.toLowerCase())
                        ? `${v.name} is pinned to a slider`
                        : `Pin ${v.name} to a slider`
                    }
                  >
                    <ActionIcon
                      size="xs"
                      variant="subtle"
                      color={pinnedNames.has(v.name.toLowerCase()) ? 'teal' : 'gray'}
                      aria-label={`Pin ${v.name} to a slider`}
                      disabled={pinnedNames.has(v.name.toLowerCase())}
                      onClick={() => onPin(v)}
                    >
                      <IconAdjustmentsHorizontal size={13} />
                    </ActionIcon>
                  </Tooltip>
                )}
                {v.name}
                {replNames.has(v.name.toLowerCase()) && (
                  <Badge variant="light" color="teal" size="xs" title="Defined in the terminal">repl</Badge>
                )}
              </Group>
            </Table.Td>
            <Table.Td ff="monospace" c="green.4">{formatValue(v.value)}</Table.Td>
            <Table.Td><Text size="xs" c="dimmed">Scalar</Text></Table.Td>
            <Table.Td ff="monospace" c="dimmed">{v.units || <span title="dimensionless">—</span>}</Table.Td>
            <Table.Td ff="monospace" c="dimmed">{uncertaintyText(v) || '—'}</Table.Td>
          </Table.Tr>
        ))}
      </Table.Tbody>
    </Table>
    </Table.ScrollContainer>
  )
}

function ArrayRow({ g }: Readonly<{ g: ArrayGroup }>) {
  const [open, setOpen] = useState(false)
  return (
    <Paper withBorder p={0} radius="sm" style={{ overflow: 'hidden' }}>
      <Group
        justify="space-between"
        wrap="nowrap"
        px="sm"
        py={6}
        className="frees-row-toggle"
        style={{ cursor: 'pointer' }}
        onClick={() => setOpen((o) => !o)}
      >
        <Group gap="xs" wrap="nowrap">
          <ActionIcon variant="subtle" color="gray" size="sm" aria-label={open ? 'Collapse' : 'Expand'}>
            <IconChevronRight
              size={14}
              style={{ transform: open ? 'rotate(90deg)' : 'none', transition: 'transform 120ms' }}
            />
          </ActionIcon>
          <Text size="sm" fw={600} style={{ textTransform: 'none' }}>{g.name}</Text>
          <Badge variant="light" size="xs">{typeLabel(g)}</Badge>
        </Group>
        {g.units && <Text size="xs" c="dimmed" ff="monospace">[{g.units}]</Text>}
      </Group>
      {open && g.cells.size > VIRTUALIZE_CELLS_AT && (
        <div style={{ padding: 8 }}>
          <Suspense fallback={gridFallback}>
            <MatrixGrid g={g} />
          </Suspense>
        </div>
      )}
      {open && g.cells.size <= VIRTUALIZE_CELLS_AT && (
        <div style={{ overflowX: 'auto', padding: 8 }}>
          {g.is2D ? (
            <Table withTableBorder withColumnBorders striped>
              <Table.Thead>
                <Table.Tr>
                  <Table.Th style={{ textAlign: 'center' }}>r\c</Table.Th>
                  {g.cols.map((c) => <Table.Th key={c} style={{ textAlign: 'center' }}>{c}</Table.Th>)}
                </Table.Tr>
              </Table.Thead>
              <Table.Tbody>
                {g.rows.map((r) => (
                  <Table.Tr key={r}>
                    <Table.Td fw={700} style={{ textAlign: 'center' }}>{r}</Table.Td>
                    {g.cols.map((c) => (
                      <Table.Td key={c} ff="monospace" style={{ textAlign: 'right' }}>
                        {(() => { const cell = g.cells.get(`${r},${c}`); return cell ? formatValue(cell.value) : '—' })()}
                      </Table.Td>
                    ))}
                  </Table.Tr>
                ))}
              </Table.Tbody>
            </Table>
          ) : (
            <Table withTableBorder withColumnBorders striped>
              <Table.Thead>
                <Table.Tr>
                  <Table.Th style={{ textAlign: 'center', width: 80 }}>Index</Table.Th>
                  <Table.Th style={{ textAlign: 'center' }}>Value</Table.Th>
                </Table.Tr>
              </Table.Thead>
              <Table.Tbody>
                {g.rows.map((r) => (
                  <Table.Tr key={r}>
                    <Table.Td fw={700} style={{ textAlign: 'center' }}>{r}</Table.Td>
                    <Table.Td ff="monospace" style={{ textAlign: 'right' }}>
                      {(() => { const cell = g.cells.get(`${r}`); return cell ? formatValue(cell.value) : '—' })()}
                    </Table.Td>
                  </Table.Tr>
                ))}
              </Table.Tbody>
            </Table>
          )}
        </div>
      )}
    </Paper>
  )
}

function ComponentRow({
  c,
  replNames,
  onTunePid,
}: Readonly<{ c: ComponentGroup; replNames: Set<string>; onTunePid?: (c: ComponentGroup) => void }>) {
  const [open, setOpen] = useState(false)
  return (
    <Paper withBorder p={0} radius="sm" style={{ overflow: 'hidden' }}>
      <Group
        justify="space-between"
        wrap="nowrap"
        px="sm"
        py={6}
        className="frees-row-toggle"
        style={{ cursor: 'pointer' }}
        onClick={() => setOpen((o) => !o)}
      >
        <Group gap="xs" wrap="nowrap" style={{ minWidth: 0 }}>
          <ActionIcon variant="subtle" color="gray" size="sm" aria-label={open ? 'Collapse' : 'Expand'}>
            <IconChevronRight
              size={14}
              style={{ transform: open ? 'rotate(90deg)' : 'none', transition: 'transform 120ms' }}
            />
          </ActionIcon>
          <Text size="sm" fw={600} style={{ textTransform: 'none' }} truncate>{c.name}</Text>
          {c.type && (
            <Badge variant="light" color="grape" size="xs" style={{ flexShrink: 0, textTransform: 'none' }}>
              {c.type}
            </Badge>
          )}
        </Group>
        <Group gap="xs" wrap="nowrap" style={{ flexShrink: 0 }}>
          {c.type === 'SigPID' && onTunePid && (
            <Tooltip label="Auto-tune this PID's gains (PID Tuner)">
              <Button
                size="compact-xs"
                variant="light"
                color="teal"
                leftSection={<IconAdjustments size={12} />}
                onClick={(e) => {
                  e.stopPropagation()
                  onTunePid(c)
                }}
              >
                Tune…
              </Button>
            </Tooltip>
          )}
          <Text size="xs" c="dimmed" ff="monospace">
            {c.params.length > 0 && `${c.params.length} par`}
            {c.params.length > 0 && c.members.length > 0 && ' · '}
            {c.members.length > 0 && `${c.members.length} var`}
          </Text>
        </Group>
      </Group>
      {open && (
        <Stack gap={8} p={8}>
          {c.params.length > 0 && (
            <div style={{ overflowX: 'auto' }}>
              <Text size="xs" fw={700} c="dimmed" tt="uppercase" lts="0.05em" mb={4}>Parameters</Text>
              <Table withTableBorder striped>
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>Parameter</Table.Th>
                    <Table.Th>Binding</Table.Th>
                    <Table.Th>Value</Table.Th>
                    <Table.Th>Units</Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {c.params.map((p) => (
                    <Table.Tr key={p.name}>
                      <Table.Td style={{ textTransform: 'none' }}>{p.name}</Table.Td>
                      <Table.Td ff="monospace" c="grape.3" style={{ textTransform: 'none' }}>{p.ref}</Table.Td>
                      <Table.Td ff="monospace" c={p.value != null ? 'green.4' : 'dimmed'}>
                        {p.value != null ? formatValue(p.value) : '—'}
                      </Table.Td>
                      <Table.Td ff="monospace" c="dimmed">{p.units || '—'}</Table.Td>
                    </Table.Tr>
                  ))}
                </Table.Tbody>
              </Table>
            </div>
          )}
          {c.members.length > 0 && (
            <div style={{ overflowX: 'auto' }}>
              <Text size="xs" fw={700} c="dimmed" tt="uppercase" lts="0.05em" mb={4}>Results</Text>
              <Table withTableBorder striped highlightOnHover>
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>Variable</Table.Th>
                    <Table.Th>Value</Table.Th>
                    <Table.Th>Units</Table.Th>
                    <Table.Th>Uncertainty</Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {c.members.map(({ v, label }) => (
                    <Table.Tr key={v.name}>
                      <Table.Td style={{ textTransform: 'none' }}>
                        <Group gap={6} wrap="nowrap">
                          {label}
                          {replNames.has(v.name.toLowerCase()) && (
                            <Badge variant="light" color="teal" size="xs" title="Defined in the terminal">repl</Badge>
                          )}
                        </Group>
                      </Table.Td>
                      <Table.Td ff="monospace" c="green.4">{formatValue(v.value)}</Table.Td>
                      <Table.Td ff="monospace" c="dimmed">{v.units || <span title="dimensionless">—</span>}</Table.Td>
                      <Table.Td ff="monospace" c="dimmed">{uncertaintyText(v) || '—'}</Table.Td>
                    </Table.Tr>
                  ))}
                </Table.Tbody>
              </Table>
            </div>
          )}
        </Stack>
      )}
    </Paper>
  )
}

interface Props {
  variables: VariableResult[]
  /** Lowercased names of variables defined/changed in the REPL (badged in the table). */
  replNames?: Set<string>
  /** Backend component metadata — supplies each instance's type pill and parameter datasheet. */
  components?: ComponentResult[]
  /** Opens the Variable Information modal (guesses, bounds, units, uncertainty). */
  onEdit?: () => void
  /** Opens the PID Tuner for a selected SigPID component instance. */
  onTunePid?: (c: ComponentGroup) => void
  /** Opens the PID Tuner for a selected SigPID component instance. */
  /** Last solve response — feeds the Diagnostics section (stats, blocks,
   *  residuals; opens itself when the solve failed). */
  diagnostics?: SolveResponse | null
  /** Lowercased names already pinned to the slider strip. */
  pinnedNames?: Set<string>
  /** Lowercased names the document assigns a literal — only these are pinnable. */
  pinnableNames?: Set<string>
  /** Pin a variable to the slider strip (absent = the affordance is hidden). */
  onPin?: (v: VariableResult) => void
  /** The slider strip itself, rendered above the variable list. */
  sliderStrip?: React.ReactNode
  /** Total number of solutions found (for multi-solution exploration). */
  solutionsCount?: number
  /** Active solution index (0-based). */
  selectedSolutionIndex?: number
  /** Callback when switching between solutions. */
  onSelectSolutionIndex?: (index: number) => void
}

export default function Workspace({
  variables,
  replNames,
  components: instances,
  onEdit,
  onTunePid,
  diagnostics,
  pinnedNames,
  pinnableNames,
  onPin,
  sliderStrip,
  solutionsCount,
  selectedSolutionIndex,
  onSelectSolutionIndex,
}: Readonly<Props>) {
  const [query, setQuery] = useState('')
  // The input stays urgent (every keystroke paints immediately); the heavy
  // filter + regroup below trails behind at transition priority, so typing in
  // the search box never blocks on a large workspace.
  const deferredQuery = useDeferredValue(query)
  const repl = replNames ?? new Set<string>()

  const { plain, components, groups } = useMemo(() => {
    const q = deferredQuery.trim().toLowerCase()
    const filtered = q ? variables.filter((v) => v.name.toLowerCase().includes(q)) : variables
    const grouped = group(filtered)
    // A component matches the filter by its name, type, or any parameter; its
    // members are whatever survived the variable filter above.
    const inst = (instances ?? []).filter(
      (c) =>
        !q ||
        c.name.toLowerCase().includes(q) ||
        c.type.toLowerCase().includes(q) ||
        c.params.some((p) => p.name.toLowerCase().includes(q) || p.ref.toLowerCase().includes(q)),
    )
    const { plain, components } = groupComponents(grouped.scalars, inst)
    return { plain, components, groups: grouped.groups }
  }, [variables, deferredQuery, instances])

  const empty = variables.length === 0

  return (
    // Slightly darker than the editor canvas (same tint as the REPL terminal)
    // so the tool surfaces read as distinct from the main document.
    <Paper
      withBorder
      p="md"
      h="100%"
      style={{
        overflowY: 'auto',
        backgroundColor: 'light-dark(var(--mantine-color-gray-0), var(--mantine-color-dark-8))',
      }}
    >
      <SolveDiagnostics response={diagnostics ?? null} />
      {/* Wrap (not nowrap) so in a narrow dock/edge panel the filter + Edit drop
          below the title instead of squeezing it into a clipped two-line wrap. */}
      <Group justify="space-between" mb="sm" gap="xs" wrap="wrap">
        <Group gap="xs" wrap="nowrap" style={{ flex: '1 1 auto', minWidth: 0 }}>
          <ThemeIcon variant="light" size="sm"><IconVariable size={14} /></ThemeIcon>
          <Text fw={600} c="teal.4" truncate>Variable Explorer</Text>
          {!empty && <Badge variant="light" size="sm" style={{ flexShrink: 0 }}>{variables.length}</Badge>}
        </Group>
        <Group gap="xs" wrap="nowrap" style={{ flex: '1 1 auto' }}>
          <TextInput
            size="xs"
            style={{ flex: 1, minWidth: 0 }}
            placeholder="Filter variables…"
            leftSection={<IconSearch size={13} />}
            value={query}
            onChange={(e) => setQuery(e.currentTarget.value)}
            aria-label="Filter workspace variables"
          />
          {onEdit && (
            <Button
              size="xs"
              variant="light"
              leftSection={<IconPencil size={14} />}
              onClick={onEdit}
            >
              Edit
            </Button>
          )}
        </Group>
      </Group>

      {solutionsCount !== undefined && solutionsCount > 1 && (
        <Group gap="xs" mb="sm" align="center">
          <Text size="xs" fw={500} c="dimmed">Solution:</Text>
          <ActionIcon
            size="xs"
            variant="subtle"
            disabled={(selectedSolutionIndex ?? 0) <= 0}
            onClick={() => onSelectSolutionIndex?.((selectedSolutionIndex ?? 0) - 1)}
            aria-label="Previous solution"
          >
            <IconChevronLeft size={14} />
          </ActionIcon>
          <Badge size="sm" variant="filled" color="blue">
            {(selectedSolutionIndex ?? 0) + 1} of {solutionsCount}
          </Badge>
          <ActionIcon
            size="xs"
            variant="subtle"
            disabled={(selectedSolutionIndex ?? 0) >= solutionsCount - 1}
            onClick={() => onSelectSolutionIndex?.((selectedSolutionIndex ?? 0) + 1)}
            aria-label="Next solution"
          >
            <IconChevronRight size={14} />
          </ActionIcon>
        </Group>
      )}

      {empty ? (
        <Text c="dimmed" size="sm">
          Solve the document to populate the workspace. Variables, arrays and
          matrices from the last solve appear here with their value, type, unit
          and uncertainty.
        </Text>
      ) : (
        <Stack gap="md">
          {sliderStrip}
          {plain.length > 0 &&
            (plain.length > VIRTUALIZE_SCALARS_AT ? (
              <Suspense fallback={gridFallback}>
                <ScalarGrid scalars={plain} replNames={repl} />
              </Suspense>
            ) : (
              <ScalarTable scalars={plain} replNames={repl} pinnedNames={pinnedNames ?? EMPTY_NAMES} pinnableNames={pinnableNames ?? EMPTY_NAMES} onPin={onPin} />
            ))}
          {components.length > 0 && (
            <Stack gap="xs">
              <Group gap={6}>
                <IconComponents size={13} />
                <Text size="xs" fw={700} c="dimmed" tt="uppercase" lts="0.05em">Components</Text>
              </Group>
              {components.map((c) => (
                <ComponentRow key={c.name} c={c} replNames={repl} onTunePid={onTunePid} />
              ))}
            </Stack>
          )}
          {groups.length > 0 && (
            <Stack gap="xs">
              <Group gap={6}>
                <IconTable size={13} />
                <Text size="xs" fw={700} c="dimmed" tt="uppercase" lts="0.05em">Arrays & Matrices</Text>
              </Group>
              {groups.map((g) => <ArrayRow key={g.name} g={g} />)}
            </Stack>
          )}
          {plain.length === 0 && components.length === 0 && groups.length === 0 && (
            <Text c="dimmed" size="sm">No variables match “{deferredQuery}”.</Text>
          )}
        </Stack>
      )}
    </Paper>
  )
}
