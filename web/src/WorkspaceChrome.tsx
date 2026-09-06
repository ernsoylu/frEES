import { helpUrl } from './helpUrl'
import {
  ActionIcon,
  Anchor,
  Badge,
  Button,
  Checkbox,
  Group,
  Menu,
  Popover,
  Stack,
  Text,
  Title,
  Tooltip,
  UnstyledButton,
  useComputedColorScheme,
  useMantineColorScheme,
} from '@mantine/core'
import {
  IconAdjustments,
  IconChartGridDots,
  IconChartHistogram,
  IconCrosshair,
  IconTargetArrow,
  IconSitemap,
  IconChartLine,
  IconChecks,
  IconChevronDown,
  IconLayoutSidebarLeftCollapse,
  IconLayoutSidebarLeftExpand,
  IconCode,
  IconDatabase,
  IconDeviceFloppy,
  IconFile,
  IconFilePlus,
  IconFolderOpen,
  IconHelp,
  IconInfoCircle,
  IconLayoutGrid,
  IconLayoutColumns,
  IconMathFunction,
  IconPointFilled,
  IconX,
  IconMoon,
  IconSun,
  IconPencil,
  IconPlus,
  IconPlayerPlayFilled,
  IconPlayerStopFilled,
  IconSettings,
  IconTable,
  IconTerminal2,
  IconTrash,
  IconSearch,
  IconVariable,
  IconLink,
  IconPrinter,
} from '@tabler/icons-react'
import { spotlight } from '@mantine/spotlight'
import { useState } from 'react'
import { CheckResponse, SolveResponse, TableRowResult } from './api'
import type { LayoutPerspective } from './workspace/WorkspaceDock'
import { withStableKeys } from './format'
import { FUNCTION_CATEGORIES } from './functionCatalog'

// ---------------------------------------------------------------------------
// Left icon rail: views on top, tool windows at the bottom (VS Code style).
// ---------------------------------------------------------------------------

// Each view carries a short `label` (shown when the rail is expanded) and a
// longer `tip` (the hover tooltip when collapsed).
const VIEWS = [
  { value: 'equations', label: 'Editor', tip: 'Editor', icon: IconCode },
  { value: 'table', label: 'Tables', tip: 'Tables — parametric runs & curve functions', icon: IconTable },
  { value: 'plots', label: 'Plots', tip: 'Plots — X-Y, property & psychrometric', icon: IconChartLine },
  {
    value: 'digitizer',
    label: 'Digitizer',
    tip: 'Graph Digitizer — extract curves from chart images',
    icon: IconChartGridDots,
  },
  {
    value: 'schematic',
    label: 'Schematic',
    tip: 'Schematic — auto-rendered component network',
    icon: IconSitemap,
  },
  { value: 'terminal', label: 'Terminal', tip: 'Terminal — REPL evaluated against the workspace', icon: IconTerminal2 },
]

const RAIL_EXPANDED_KEY = 'frees.railExpanded'

interface RailProps {
  active: string
  /** Window kinds currently open in the dock (drives the open-state dot). */
  openKinds?: string[]
  /** Specific window ids open (e.g. "diagram:<id>") for per-instance marks. */
  openIds?: string[]
  /** Plots (X-Y, property, psychrometric) available to open as windows. */
  plots?: { id: string; name: string; tag?: string; deletable?: boolean }[]
  /** Number of plot windows currently open (badge on the Plots icon). */
  plotCount?: number
  /** Tables available to open as individual windows. */
  workspaceTables?: { id: string; name: string; deletable?: boolean }[]
  /** Number of table windows currently open (badge on the Tables icon). */
  tableCount?: number
  onSelect: (view: string) => void
  onClose?: (view: string) => void
  /** Rebuild the dock as a named layout preset (shows the Layout menu). */
  onApplyLayout?: (perspective: LayoutPerspective) => void
  onOpenPlot?: (id: string) => void
  onNewPlot?: (kind: 'xy' | 'property' | 'psychro') => void
  onDeletePlot?: (id: string) => void
  onOpenTable?: (id: string) => void
  onNewTable?: (kind: 'parametric' | 'function-1d' | 'function-2d') => void
  onDeleteTable?: (id: string) => void
  /** Open the singleton Fluid State table window. */
  onOpenStates?: () => void
  onPreferences: () => void
  onAbout: () => void
}

// Sidebar launcher for a multi-instance kind (Diagram, Plots): a menu listing
// every instance (open each as its own dock window) plus a "New …" action. A
// badge shows how many windows of that kind are currently open.
function InstanceLauncher({
  expanded,
  active,
  count,
  idPrefix,
  label,
  newActions,
  emptyLabel,
  icon,
  items,
  openIds,
  onOpen,
  onDelete,
}: Readonly<{
  expanded: boolean
  active: boolean
  count: number
  idPrefix: string
  label: string
  newActions: { label: string; onClick: () => void }[]
  emptyLabel: string
  icon: React.ReactNode
  items: { id: string; name: string; tag?: string; deletable?: boolean }[]
  openIds: Set<string>
  onOpen?: (id: string) => void
  onDelete?: (id: string) => void
}>) {
  const variant = active ? 'light' : 'subtle'
  const color = active ? 'teal' : 'gray'
  const target = expanded ? (
    <Button
      variant={variant}
      color={color}
      justify="flex-start"
      fullWidth
      size="sm"
      radius="md"
      leftSection={icon}
      rightSection={
        count > 0 ? (
          <Badge size="xs" variant="filled" circle>
            {count}
          </Badge>
        ) : null
      }
      aria-label={`${label} windows`}
    >
      {label}
    </Button>
  ) : (
    <div style={{ position: 'relative', display: 'inline-flex' }}>
      <ActionIcon variant={variant} color={color} size={40} radius="md" aria-label={`${label} windows`}>
        {icon}
      </ActionIcon>
      {count > 0 && (
        <Badge
          size="xs"
          variant="filled"
          circle
          style={{ position: 'absolute', top: -3, right: -3, pointerEvents: 'none' }}
        >
          {count}
        </Badge>
      )}
    </div>
  )
  return (
    <Menu position="right-start" shadow="md" width={230} withinPortal>
      <Menu.Target>{target}</Menu.Target>
      <Menu.Dropdown>
        <Menu.Label>{label} windows</Menu.Label>
        {items.length === 0 && <Menu.Item disabled>{emptyLabel}</Menu.Item>}
        {items.map((it) => (
          <Menu.Item
            key={it.id}
            onClick={() => onOpen?.(it.id)}
            leftSection={
              openIds.has(`${idPrefix}${it.id}`) ? (
                <IconPointFilled size={10} style={{ color: 'var(--mantine-color-teal-5)' }} />
              ) : (
                <span style={{ display: 'inline-block', width: 10 }} />
              )
            }
            rightSection={
              <Group gap={4} wrap="nowrap" align="center">
                {it.tag && (
                  <Text size="9px" c="dimmed">
                    {it.tag}
                  </Text>
                )}
                {it.deletable && onDelete && (
                  <ActionIcon
                    component="span"
                    size="xs"
                    variant="subtle"
                    color="red"
                    aria-label={`Delete ${it.name}`}
                    onClick={(e) => {
                      e.stopPropagation()
                      onDelete(it.id)
                    }}
                  >
                    <IconTrash size={12} />
                  </ActionIcon>
                )}
              </Group>
            }
          >
            {it.name}
          </Menu.Item>
        ))}
        <Menu.Divider />
        {newActions.map((a) => (
          <Menu.Item key={a.label} leftSection={<IconPlus size={14} />} onClick={a.onClick}>
            {a.label}
          </Menu.Item>
        ))}
      </Menu.Dropdown>
    </Menu>
  )
}

// One rail entry. Collapsed → an icon button with a hover tooltip; expanded →
// a full-width button with the icon and a text label. `href` turns it into a
// link (used by Help); otherwise it is a click action.
function RailEntry({
  icon,
  label,
  tip,
  active,
  open,
  expanded,
  onClick,
  onClose,
  href,
}: Readonly<{
  icon: React.ReactNode
  label: string
  tip: string
  active?: boolean
  /** Whether the corresponding dock window is currently open. */
  open?: boolean
  expanded: boolean
  onClick?: () => void
  /** When provided and the window is open, shows a close affordance. */
  onClose?: () => void
  href?: string
}>) {
  const variant = active ? 'light' : 'subtle'
  const color = active ? 'teal' : 'gray'
  // A small dot marks windows that are open in the dock (so the rail doubles
  // as a window list); the focused window also gets the blue "active" styling.
  const dot = open ? (
    <IconPointFilled
      size={10}
      style={{ color: 'var(--mantine-color-teal-5)', flexShrink: 0 }}
    />
  ) : null

  if (expanded) {
    const shared = {
      variant,
      color,
      justify: 'flex-start' as const,
      fullWidth: true,
      size: 'sm' as const,
      radius: 'md' as const,
      leftSection: icon,
      rightSection: open && onClose ? (
        <UnstyledButton
          component="span"
          aria-label={`Close ${label}`}
          onClick={(e) => {
            e.stopPropagation()
            onClose()
          }}
          style={{ display: 'inline-flex', color: 'var(--mantine-color-dimmed)' }}
        >
          <IconX size={14} />
        </UnstyledButton>
      ) : dot,
      'aria-label': tip,
    }
    return href ? (
      <Button component="a" href={href} target="_blank" {...shared}>
        {label}
      </Button>
    ) : (
      <Button onClick={onClick} {...shared}>
        {label}
      </Button>
    )
  }

  const shared = { variant, color, size: 40, radius: 'md' as const, 'aria-label': tip }
  const inner = href ? (
    <ActionIcon component="a" href={href} target="_blank" {...shared}>
      {icon}
    </ActionIcon>
  ) : (
    <ActionIcon onClick={onClick} {...shared}>
      {icon}
    </ActionIcon>
  )
  // Collapsed: overlay a tiny dot in the corner for open windows.
  const button = (
    <div style={{ position: 'relative', display: 'inline-flex' }}>
      {inner}
      {open && (
        <IconPointFilled
          size={9}
          style={{
            position: 'absolute',
            top: 1,
            right: 1,
            color: 'var(--mantine-color-teal-5)',
            pointerEvents: 'none',
          }}
        />
      )}
    </div>
  )
  return (
    <Tooltip label={tip} position="right" openDelay={300}>
      {button}
    </Tooltip>
  )
}

export function Rail({
  active,
  openKinds = [],
  openIds = [],
  plots,
  plotCount = 0,
  workspaceTables,
  tableCount = 0,
  onSelect,
  onClose,
  onApplyLayout,
  onOpenPlot,
  onNewPlot,
  onDeletePlot,
  onOpenTable,
  onNewTable,
  onDeleteTable,
  onOpenStates,
  onPreferences,
  onAbout,
}: Readonly<RailProps>) {
  const openSet = new Set(openKinds)
  const openIdSet = new Set(openIds)
  const [expanded, setExpanded] = useState(
    () => localStorage.getItem(RAIL_EXPANDED_KEY) === 'true',
  )
  const toggle = () => {
    setExpanded((e) => {
      const next = !e
      localStorage.setItem(RAIL_EXPANDED_KEY, String(next))
      return next
    })
  }

  // Light/dark toggle. Mantine persists the choice to localStorage via its
  // default color-scheme manager.
  const { setColorScheme } = useMantineColorScheme()
  const computedScheme = useComputedColorScheme('dark')
  const toggleScheme = () =>
    setColorScheme(computedScheme === 'dark' ? 'light' : 'dark')

  const iconSize = 22
  const tools = [
    { label: 'Preferences', tip: 'Preferences', icon: IconSettings, onClick: onPreferences },
    { label: 'About', tip: 'About', icon: IconInfoCircle, onClick: onAbout },
  ]

  return (
    <Stack
      justify="space-between"
      p={6}
      w={expanded ? 200 : undefined}
      style={{ borderRight: '1px solid var(--mantine-color-default-border)', flexShrink: 0 }}
    >
      <Stack gap={4}>
        <RailEntry
          icon={
            expanded ? (
              <IconLayoutSidebarLeftCollapse size={iconSize} stroke={1.6} />
            ) : (
              <IconLayoutSidebarLeftExpand size={iconSize} stroke={1.6} />
            )
          }
          label="Collapse"
          tip={expanded ? 'Collapse sidebar' : 'Expand sidebar'}
          expanded={expanded}
          onClick={toggle}
        />
        {VIEWS.map((view) =>
          view.value === 'plots' && plots ? (
            <InstanceLauncher
              key={view.value}
              expanded={expanded}
              active={active === 'plot'}
              count={plotCount}
              idPrefix="plot:"
              label="Plots"
              newActions={[
                { label: 'New X-Y plot', onClick: () => onNewPlot?.('xy') },
                { label: 'New property diagram', onClick: () => onNewPlot?.('property') },
                { label: 'New psychrometric chart', onClick: () => onNewPlot?.('psychro') },
              ]}
              emptyLabel="No plots yet"
              icon={<IconChartLine size={iconSize} stroke={1.6} />}
              items={plots}
              openIds={openIdSet}
              onOpen={onOpenPlot}
              onDelete={onDeletePlot}
            />
          ) : view.value === 'table' && workspaceTables ? (
            <InstanceLauncher
              key={view.value}
              expanded={expanded}
              active={active === 'table'}
              count={tableCount}
              idPrefix="table:"
              label="Tables"
              newActions={[
                { label: 'New parametric table', onClick: () => onNewTable?.('parametric') },
                { label: 'New function table (1-arg)', onClick: () => onNewTable?.('function-1d') },
                { label: 'New function table (curve)', onClick: () => onNewTable?.('function-2d') },
                { label: 'Add fluid state table', onClick: () => onOpenStates?.() },
              ]}
              emptyLabel="No tables yet"
              icon={<IconTable size={iconSize} stroke={1.6} />}
              items={workspaceTables}
              openIds={openIdSet}
              onOpen={onOpenTable}
              onDelete={onDeleteTable}
            />
          ) : (
            <RailEntry
              key={view.value}
              icon={<view.icon size={iconSize} stroke={1.6} />}
              label={view.label}
              tip={view.tip}
              active={active === view.value}
              open={openSet.has(view.value)}
              expanded={expanded}
              onClick={() => onSelect(view.value)}
              onClose={onClose ? () => onClose(view.value) : undefined}
            />
          ),
        )}
      </Stack>
      <Stack gap={4}>
        {onApplyLayout && (
          <Menu position="right-start" shadow="md" width={220} withinPortal>
            <Menu.Target>
              {expanded ? (
                <Button
                  variant="subtle"
                  color="gray"
                  justify="flex-start"
                  fullWidth
                  size="sm"
                  radius="md"
                  leftSection={<IconLayoutGrid size={iconSize} stroke={1.6} />}
                  aria-label="Window layout presets"
                >
                  Layout
                </Button>
              ) : (
                <ActionIcon variant="subtle" color="gray" size={40} radius="md" aria-label="Window layout presets">
                  <IconLayoutGrid size={iconSize} stroke={1.6} />
                </ActionIcon>
              )}
            </Menu.Target>
            <Menu.Dropdown>
              <Menu.Label>Window layout</Menu.Label>
              <Menu.Item
                leftSection={<IconLayoutGrid size={14} />}
                onClick={() => onApplyLayout('default')}
              >
                Default view
              </Menu.Item>
              <Menu.Item
                leftSection={<IconChartLine size={14} />}
                onClick={() => onApplyLayout('results')}
              >
                Results focused
              </Menu.Item>
              <Menu.Item
                leftSection={<IconLayoutColumns size={14} />}
                onClick={() => onApplyLayout('split')}
              >
                Split view
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
        )}
        {tools.map((tool) => (
          <RailEntry
            key={tool.label}
            icon={<tool.icon size={iconSize} stroke={1.6} />}
            label={tool.label}
            tip={tool.tip}
            expanded={expanded}
            onClick={tool.onClick}
          />
        ))}
        <RailEntry
          icon={
            computedScheme === 'dark' ? (
              <IconSun size={iconSize} stroke={1.6} />
            ) : (
              <IconMoon size={iconSize} stroke={1.6} />
            )
          }
          label={computedScheme === 'dark' ? 'Light mode' : 'Dark mode'}
          tip={computedScheme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
          expanded={expanded}
          onClick={toggleScheme}
        />
        <RailEntry
          icon={<IconHelp size={iconSize} stroke={1.6} />}
          label="Help"
          tip="Help"
          expanded={expanded}
          href={helpUrl()}
        />
      </Stack>
    </Stack>
  )
}

// ---------------------------------------------------------------------------
// Status pill: one compact badge summarizing the latest Check/Solve/Table
// outcome; the full message and unit warnings live in its popover.
// ---------------------------------------------------------------------------

interface PillContent {
  color: string
  label: string
  message: string
  warnings: string[]
  /** Deep link into the /help portal for this failure, when we recognize it. */
  helpHref?: string
}

// Map a diagnostic message to the documentation page that explains it. The
// portal pages are hash-addressable (/help#topic), so a failure is one click
// from its cause-and-fix entry in the Errors & Diagnostics index.
function helpHrefForMessage(msg: string): string {
  const m = msg.toLowerCase()
  if (/component|connect|port|domain|stream|fluid family/.test(m)) return helpUrl('#comp-troubleshooting')
  return helpUrl('#errors')
}

function solvePill(result: SolveResponse): PillContent {
  if (!result.success) {
    const message = result.error || 'Solve failed'
    return {
      color: 'red',
      label: 'Solve failed',
      message,
      warnings: [],
      helpHref: helpHrefForMessage(message),
    }
  }
  const warnings = result.unitWarnings
  return {
    color: warnings.length > 0 ? 'yellow' : 'green',
    label: warnings.length > 0 ? `Solved · ${warnings.length} warnings` : 'Solved',
    message: 'Solve successful',
    warnings,
  }
}

function checkPill(checkResult: CheckResponse): PillContent {
  if (!checkResult.solvable) {
    return {
      color: 'red',
      label: 'Check errors',
      message: checkResult.message,
      warnings: checkResult.unitWarnings,
      helpHref: helpHrefForMessage(checkResult.message),
    }
  }
  const warnings = checkResult.unitWarnings
  return {
    color: warnings.length > 0 ? 'yellow' : 'green',
    label: warnings.length > 0 ? `Check OK · ${warnings.length} warnings` : 'Check OK',
    message: checkResult.message,
    warnings,
  }
}

function tablePill(
  results: TableRowResult[],
  checkResult: CheckResponse | null,
  checkMessage: string,
): PillContent | null {
  if (results.length > 0) {
    const solved = results.filter((r) => r.success).length
    const allSolved = solved === results.length
    return {
      color: allSolved ? 'green' : 'red',
      label: `${solved}/${results.length} runs solved`,
      message: allSolved
        ? 'Every table run solved.'
        : 'Failed runs are marked in the table — hover a ✗ cell for the reason.',
      warnings: [],
    }
  }
  if (checkResult || checkMessage) {
    const ok = checkResult?.solvable === true
    return {
      color: ok ? 'green' : 'red',
      label: ok ? 'Table check OK' : 'Table check errors',
      message: checkMessage || (ok ? 'Ready to solve.' : 'The table cannot be solved.'),
      warnings: checkResult?.unitWarnings ?? [],
    }
  }
  return null
}

function StatusPill({ pill, hint }: Readonly<{ pill: PillContent | null; hint: string }>) {
  if (!pill) {
    return (
      <Text size="xs" c="dimmed" visibleFrom="sm">
        {hint}
      </Text>
    )
  }
  return (
    <Popover width={420} position="bottom-end" withArrow shadow="md">
      <Popover.Target>
        <UnstyledButton aria-label="Status details">
          <Badge color={pill.color} variant="light" size="lg" radius="sm" style={{ cursor: 'pointer', textTransform: 'none' }}>
            {pill.label}
          </Badge>
        </UnstyledButton>
      </Popover.Target>
      <Popover.Dropdown>
        <Stack gap={6}>
          <Text size="sm" style={{ whiteSpace: 'pre-wrap' }}>
            {pill.message}
          </Text>
          {pill.warnings.length > 0 && (
            <Stack gap={2}>
              {withStableKeys(pill.warnings).map((w) => (
                <Text size="xs" c="yellow.5" key={w.key}>
                  ⚠ {w.value}
                </Text>
              ))}
            </Stack>
          )}
          {pill.helpHref && (
            <Anchor href={pill.helpHref} target="_blank" size="xs">
              Help with this error →
            </Anchor>
          )}
        </Stack>
      </Popover.Dropdown>
    </Popover>
  )
}

// ---------------------------------------------------------------------------
// Top bar: brand, Check / Solve (context-aware for the parametric table),
// solve options menu, and the status pill. Replaces the old header row and
// both bottom action bars.
// ---------------------------------------------------------------------------

interface TopBarProps {
  isTable: boolean
  checking: boolean
  solving: boolean
  /** Engine completion, 0…1, while a solve or a table run is in flight; `null`
   *  when nothing is running. Painted as the Solve button's own background. */
  solveProgress: number | null
  solvable: boolean
  findAll: boolean
  complexMode: boolean
  checkResult: CheckResponse | null
  result: SolveResponse | null
  tableChecking: boolean
  tableSolving: boolean
  tableCheckResult: CheckResponse | null
  tableCheckMessage: string
  tableResults: TableRowResult[]
  onCheck: () => void
  onSolve: () => void
  onStop?: () => void
  onCheckTable: () => void
  onSolveTable: () => void
  onFindAllChange: (checked: boolean) => void
  onComplexModeChange: (checked: boolean) => void
  projectName: string
  onRenameProject: () => void
  onNewProject: () => void
  onOpenProject: () => void
  /** File menu: the browser-resident project library (Phase 11, D4). */
  onOpenLibrary: () => void
  onSaveProject: () => void
  onSaveProjectAs: () => void
  onInsertFunction: (snippet: string) => void
  onInsertComponent: () => void
  onOpenExamples: () => void
  /** File menu: copy a self-contained #share= link carrying the document. */
  onShareLink: () => void
  /** File menu: open the printable calculation report (browser print-to-PDF). */
  onPrintReport: () => void
  /** Whether a successful solve exists for the report to print. */
  canPrintReport: boolean
  /** Tools menu: open the auxiliary Inspector edge panel. */
  onOpenInspector: () => void
  /** Tools menu: open the Variable Explorer and the REPL Terminal dock windows. */
  onOpenWorkspace: () => void
  onOpenTerminal: () => void
  /** Tools menu: equation tools (also reachable from the left rail). */
  onVariableInfo: () => void
  /** Tools → Analysis: the D5-clipped affordances, re-added as their
   *  surfaces wired (Monte Carlo in Wave B2; Min/Max, Curve Fit and
   *  Parameter Estimation in Wave B3). */
  onMonteCarlo: () => void
  onMinMax: () => void
  onCurveFit: () => void
  onParameterFit: () => void
  onPidTuner: () => void
}

function solveTooltipFor(canSolve: boolean, isTable: boolean): string {
  if (!canSolve) return 'Check & Solve (F2)'
  return isTable ? 'Solve every table run (F2)' : 'Solve the system (F2)'
}

function statusPillFor(props: Readonly<TopBarProps>) {
  if (props.isTable) {
    return tablePill(props.tableResults, props.tableCheckResult, props.tableCheckMessage)
  }
  if (props.result) return solvePill(props.result)
  if (props.checkResult) return checkPill(props.checkResult)
  return null
}

export function TopBar(props: Readonly<TopBarProps>) {
  const { isTable } = props
  const canSolve = isTable ? props.tableCheckResult?.solvable === true : props.solvable
  const solveTooltip = solveTooltipFor(canSolve, isTable)
  const pill = statusPillFor(props)
  const solveBusy = isTable ? props.tableSolving : props.solving
  // `null` is "running, nothing reported yet" — an empty bar, not a missing
  // one, so the button does not jump from full gradient to 0 % on the first
  // report. Clamped because the width is going straight into CSS.
  const solvePercent = Math.min(100, Math.max(0, (props.solveProgress ?? 0) * 100))
  // While it runs, the tooltip carries the number the bar shows — the only
  // place the percentage is available as text.
  const solveTooltipText = solveBusy
    ? `Solving… ${Math.round(solvePercent)}%`
    : solveTooltip
  const hint = isTable
    ? 'Configure columns, fill values, then Check (F4) · Solve (F2)'
    : 'Check (F4) · Solve (F2)'

  return (
    // Wrap (not nowrap) so the Check/Solve action group drops to a second row
    // on narrow viewports instead of overflowing off-screen — the solver
    // trigger must always stay reachable.
    <Group justify="space-between" wrap="wrap" gap="sm">
      <Group gap="sm" wrap="nowrap" align="center">
        <Title order={3} c="teal.4">
          frees
        </Title>
        <Menu position="bottom-start" shadow="md" width={220}>
          <Menu.Target>
            <Button
              variant="subtle"
              color="gray"
              size="xs"
              leftSection={<IconFile size={14} />}
              rightSection={<IconChevronDown size={12} />}
            >
              File
            </Button>
          </Menu.Target>
          <Menu.Dropdown>
            <Menu.Label>{props.projectName}.frees</Menu.Label>
            <Menu.Item leftSection={<IconFilePlus size={14} />} onClick={props.onNewProject}>
              New Project
            </Menu.Item>
            <Menu.Item leftSection={<IconFolderOpen size={14} />} onClick={props.onOpenProject}>
              Open Project…
            </Menu.Item>
            <Menu.Item leftSection={<IconDatabase size={14} />} onClick={props.onOpenLibrary}>
              Browser Projects…
            </Menu.Item>
            <Menu.Item leftSection={<IconLayoutGrid size={14} />} onClick={props.onOpenExamples}>
              Open Example…
            </Menu.Item>
            <Menu.Item leftSection={<IconLink size={14} />} onClick={props.onShareLink}>
              Copy Share Link
            </Menu.Item>
            <Menu.Item leftSection={<IconPrinter size={14} />} onClick={props.onPrintReport} disabled={!props.canPrintReport}>
              Print Report…
            </Menu.Item>
            <Menu.Divider />
            <Menu.Item leftSection={<IconDeviceFloppy size={14} />} onClick={props.onSaveProject}>
              Save Project
            </Menu.Item>
            <Menu.Item leftSection={<IconDeviceFloppy size={14} />} onClick={props.onSaveProjectAs}>
              Save Project As…
            </Menu.Item>
          </Menu.Dropdown>
        </Menu>

        <Menu position="bottom-start" shadow="md" width={250}>
          <Menu.Target>
            <Button
              variant="subtle"
              color="gray"
              size="xs"
              leftSection={<IconMathFunction size={14} />}
              rightSection={<IconChevronDown size={12} />}
            >
              Functions
            </Button>
          </Menu.Target>
          <Menu.Dropdown>
            {FUNCTION_CATEGORIES.map((cat) => (
              <Menu.Sub key={cat.category}>
                <Menu.Sub.Target>
                  <Menu.Sub.Item>{cat.category}</Menu.Sub.Item>
                </Menu.Sub.Target>
                <Menu.Sub.Dropdown>
                  {cat.items.map((item) => {
                    const tip = [
                      item.description,
                      item.usage ? `e.g. ${item.usage}` : null,
                    ]
                      .filter(Boolean)
                      .join('\n')
                    const menuItem = (
                      <Menu.Item onClick={() => props.onInsertFunction(item.snippet)}>
                        {item.label}
                      </Menu.Item>
                    )
                    return tip ? (
                      <Tooltip
                        key={item.label}
                        label={<div style={{ whiteSpace: 'pre-line' }}>{tip}</div>}
                        position="right"
                        multiline
                        w={260}
                        withArrow
                        openDelay={400}
                      >
                        {menuItem}
                      </Tooltip>
                    ) : (
                      <div key={item.label}>{menuItem}</div>
                    )
                  })}
                </Menu.Sub.Dropdown>
              </Menu.Sub>
            ))}
          </Menu.Dropdown>
        </Menu>

        <Menu position="bottom-start" shadow="md" width={200}>
          <Menu.Target>
            <Button
              variant="subtle"
              color="gray"
              size="xs"
              leftSection={<IconAdjustments size={14} />}
              rightSection={<IconChevronDown size={12} />}
            >
              Tools
            </Button>
          </Menu.Target>
          <Menu.Dropdown>
            <Menu.Label>Modeling</Menu.Label>
            <Menu.Item leftSection={<IconLayoutGrid size={14} />} onClick={props.onInsertComponent}>
              Component Wizard
            </Menu.Item>
            <Menu.Divider />
            <Menu.Label>Equation tools</Menu.Label>
            <Menu.Item leftSection={<IconVariable size={14} />} onClick={props.onVariableInfo}>
              Variable Information
            </Menu.Item>
            <Menu.Divider />
            <Menu.Label>Analysis</Menu.Label>
            <Menu.Item leftSection={<IconTargetArrow size={14} />} onClick={props.onMinMax}>
              Min / Max (optimize)
            </Menu.Item>
            <Menu.Item leftSection={<IconMathFunction size={14} />} onClick={props.onCurveFit}>
              Curve Fit
            </Menu.Item>
            <Menu.Item leftSection={<IconChartHistogram size={14} />} onClick={props.onMonteCarlo}>
              Monte Carlo Uncertainty
            </Menu.Item>
            <Menu.Item leftSection={<IconAdjustments size={14} />} onClick={props.onPidTuner}>
              PID Tuner
            </Menu.Item>
            <Menu.Item leftSection={<IconCrosshair size={14} />} onClick={props.onParameterFit}>
              Parameter Estimation
            </Menu.Item>
            <Menu.Divider />
            <Menu.Label>Auxiliary panels</Menu.Label>
            <Menu.Item leftSection={<IconVariable size={14} />} onClick={props.onOpenWorkspace}>
              Variable Explorer
            </Menu.Item>
            <Menu.Item leftSection={<IconTerminal2 size={14} />} onClick={props.onOpenTerminal}>
              Terminal
            </Menu.Item>
            <Menu.Item leftSection={<IconAdjustments size={14} />} onClick={props.onOpenInspector}>
              Inspector
            </Menu.Item>
          </Menu.Dropdown>
        </Menu>

        <Tooltip label="Command palette (Ctrl+K)">
          <ActionIcon
            variant="subtle"
            color="gray"
            onClick={spotlight.open}
            aria-label="Open command palette"
          >
            <IconSearch size={16} />
          </ActionIcon>
        </Tooltip>

        <Tooltip label="Rename project">
          <UnstyledButton
            onClick={props.onRenameProject}
            visibleFrom="sm"
            aria-label={`Rename project (currently ${props.projectName}.frees)`}
            style={{
              padding: '2px 8px',
              borderRadius: '4px',
              cursor: 'pointer',
              border: '1px solid var(--mantine-color-default-border)',
              backgroundColor: 'light-dark(var(--mantine-color-gray-1), var(--mantine-color-dark-6))',
              display: 'flex',
              alignItems: 'center',
              gap: 6,
            }}
          >
            <Text size="xs" c="gray.5" style={{ fontFamily: 'monospace' }}>
              {props.projectName}.frees
            </Text>
            {/* Edit affordance: signals the pill opens the rename dialog rather
                than being an inline text field. */}
            <IconPencil size={12} color="var(--mantine-color-dimmed)" />
          </UnstyledButton>
        </Tooltip>
        <Text c="dimmed" size="xs" visibleFrom="lg">
          free solver
        </Text>
      </Group>

      <Group gap="xs" wrap="nowrap">
        <StatusPill pill={pill} hint={hint} />
        <Tooltip label={`${isTable ? 'Check every table run' : 'Compile and check'} (F4)`}>
          <Button
            variant="default"
            size="xs"
            leftSection={<IconChecks size={14} />}
            onClick={isTable ? props.onCheckTable : props.onCheck}
            loading={isTable ? props.tableChecking : props.checking}
          >
            Check
          </Button>
        </Tooltip>
        <Tooltip label={solveTooltipText}>
          <Button.Group>
            {/* The primary action of the whole app — a gradient sets it apart
                from the neutral Check and the rest of the toolbar.

                While it spins, that gradient doubles as the progress bar: the
                done fraction keeps the full-strength gradient and the rest is
                dimmed by an overlay layer. Painted with `background-size`
                rather than a colour stop because size is animatable and
                gradient stops are not — that is what makes the bar glide
                between the engine's ~60 ms reports instead of stepping.
                Mantine's loading state only adds a 15 %-opacity blurred veil
                (`Button.css` `::before`), so the bar stays legible under it. */}
            <Button
              size="xs"
              variant="gradient"
              gradient={{ from: 'teal.7', to: 'cyan.6', deg: 90 }}
              leftSection={<IconPlayerPlayFilled size={13} />}
              onClick={isTable ? props.onSolveTable : props.onSolve}
              loading={solveBusy}
              style={
                solveBusy
                  ? {
                      backgroundImage:
                        'linear-gradient(rgba(0, 0, 0, 0.45), rgba(0, 0, 0, 0.45)), ' +
                        'linear-gradient(90deg, var(--mantine-color-teal-7), var(--mantine-color-cyan-6))',
                      backgroundRepeat: 'no-repeat, no-repeat',
                      backgroundPosition: 'right center, left center',
                      backgroundSize: `${100 - solvePercent}% 100%, 100% 100%`,
                      transition: 'background-size 150ms linear',
                    }
                  : undefined
              }
              // `aria-busy` and nothing else: `aria-valuenow` needs
              // `role="progressbar"`, and taking the button's role away to get
              // it would cost more than the number is worth. The percentage
              // reaches assistive tech through the tooltip below instead.
              aria-busy={solveBusy}
            >
              Solve
            </Button>
            <Menu position="bottom-end" shadow="md">
              <Menu.Target>
                <Button size="xs" px={6} variant="light" aria-label="Solve options">
                  <IconChevronDown size={14} />
                </Button>
              </Menu.Target>
              <Menu.Dropdown p="sm">
                <Stack gap="xs">
                  {/* The all-roots search is implemented in the engine
                      (`analysis/allroots.rs`) but not yet reachable from a
                      solve request, so the box is held disabled rather than
                      silently returning the single root it always returned. */}
                  <Checkbox
                    size="xs"
                    label="Find all solutions"
                    description="Not available in the browser engine yet"
                    disabled
                    checked={props.findAll}
                    onChange={(e) => props.onFindAllChange(e.currentTarget.checked)}
                  />
                  <Checkbox
                    size="xs"
                    label="Complex mode"
                    checked={props.complexMode}
                    onChange={(e) => props.onComplexModeChange(e.currentTarget.checked)}
                  />
                </Stack>
              </Menu.Dropdown>
            </Menu>
          </Button.Group>
        </Tooltip>
        {solveBusy && props.onStop && (
          <Tooltip label="Stop computation (cancels solve and resets worker)">
            <Button
              size="xs"
              variant="filled"
              color="red"
              leftSection={<IconPlayerStopFilled size={13} />}
              onClick={props.onStop}
            >
              Stop
            </Button>
          </Tooltip>
        )}
      </Group>
    </Group>
  )
}
