import { tableInputIssues } from '../tableValidation'
// tablesGrid/TablesGridTab.tsx
//
// The Tables workbook, rebuilt on glide-data-grid (decision D10: the Univer
// spreadsheet engine is removed; the read-only table windows already render
// through glide, so this adds zero vendor bytes). One window hosts every
// function/lookup table and GUI parametric table; TableSpec stays the only
// document — the grid renders specs directly, so the whole materialized-sheet
// layer (celldata, protection rules, debounced sheet→spec sync, the sync FSM)
// is gone. Edits write through to the specs synchronously, which is what
// keeps the flushTablesWorkbook() contract: a just-committed cell is part of
// the very next solve.
//
// Deliberately not reproduced (per D10): the generic Univer ribbon, number
// formats, the fx formula bar and cell formulas. Stored spec.formulas from
// old files are surfaced read-only (ƒ marker + hint line) and are replaced by
// the typed literal when the cell is edited — never silently dropped, never
// evaluated.

import { useEffect, useLayoutEffect, useRef, useState, type Dispatch, type SetStateAction } from 'react'
import {
  DataEditor,
  GridCellKind,
  type EditableGridCell,
  type EditListItem,
  type GridCell,
  type GridColumn,
  type GridKeyEventArgs,
  type GridSelection,
  type Item,
  type Theme as GdgTheme,
} from '@glideapps/glide-data-grid'
import '@glideapps/glide-data-grid/dist/index.css'
import {
  ActionIcon,
  Badge,
  Button,
  Checkbox,
  Code,
  Group,
  Menu,
  Stack,
  Text,
  TextInput,
  Tooltip,
} from '@mantine/core'
import { useElementSize } from '@mantine/hooks'
import {
  IconArrowsSort,
  IconChartGridDots,
  IconDownload,
  IconFileTypeCsv,
  IconMathFunction,
  IconPlus,
  IconSparkles,
  IconTable,
  IconTrash,
} from '@tabler/icons-react'
import {
  fillMissingCells,
  FunctionTableSpec,
  newFunctionTable,
  newParamTable,
  ParamTableSpec,
  sortFunctionRows,
  TableSpec,
} from '../tables'
import { useGlideTheme } from '../DataGridReadOnly'
import { tablesWorkbookSync } from './tablesWorkbookBridge'
import { downloadValuesAsCsv } from './csv'
import { applyFunctionSpecs } from './composeTables'
import CreateFunctionModal from './CreateFunctionModal'
import ImportCsvModal from './ImportCsvModal'
import {
  appendRow,
  applyCellEdit,
  applyCellEdits,
  restoreUserEdit,
  applyPaste,
  boundColumnCount,
  cellViewAt,
  clearCells,
  csvExportComments,
  csvValuesFor,
  gridRowCount,
  headerTitles,
  isHostedTable,
  removeLastRow,
  storedFormulaList,
  TABLE_MAX_ROWS,
} from './tableGridModel'

interface Props {
  tables: TableSpec[]
  activeTableId: string | null
  onTablesChange: Dispatch<SetStateAction<TableSpec[]>>
  onActiveTableIdChange: (id: string | null) => void
  /** Opens ConfigureTableModal for a parametric table (App-level modal). */
  onConfigureTable?: (tableId: string) => void
  /** Opens the fill-column-values modal for a parametric table variable. */
  onAlterColumn?: (tableId: string, varName: string) => void
}

function hostedOf(tables: TableSpec[]): TableSpec[] {
  return tables.filter(isHostedTable)
}

/** Solver-computed cells (green) and failed-run markers (red) — the same two
 * colors the bound sheets used. */
const COMPUTED_GREEN = '#69db7c'
const FAILED_RED = '#fa5252'

interface UndoEntry {
  tableId: string
  before: TableSpec
  after: TableSpec | null
}

const UNDO_LIMIT = 100

export default function TablesGridTab({
  tables,
  activeTableId,
  onTablesChange,
  onActiveTableIdChange,
  onConfigureTable,
  onAlterColumn,
}: Readonly<Props>) {
  const hosted = hostedOf(tables)
  const active = hosted.find((t) => t.id === activeTableId) ?? hosted[0] ?? null

  const { ref: sizeRef, width, height } = useElementSize()
  const gridTheme = useGlideTheme()
  const [warnings, setWarnings] = useState<Record<string, string>>({})
  const [widthOverrides, setWidthOverrides] = useState<Record<string, number>>({})
  const [renamingId, setRenamingId] = useState<string | null>(null)
  const [renameDraft, setRenameDraft] = useState('')
  // Sweep → Function (Wave H): the table id the create-function dialog targets.
  const [createFnFor, setCreateFnFor] = useState<string | null>(null)
  // CSV → Function (D11): the Data Analyzer's CSV import, relocated here.
  const [importCsvOpen, setImportCsvOpen] = useState(false)

  // The synchronous view of the table list. Grid commits update it in the
  // same tick (before React re-renders), which is what lets
  // flushTablesWorkbook() hand the solve handler fresh specs from its own
  // closure (contract b of the old workbook, kept — without the debounce
  // that made a flush necessary in the first place; it still covers the
  // click-to-Solve race where the overlay editor commits on mousedown).
  const tablesRef = useRef(tables)
  useLayoutEffect(() => {
    tablesRef.current = tables
  })

  // Undo/redo for USER edits only, as spec snapshots. Solver write-backs and
  // code-table refreshes arrive as external prop changes and never enter it.
  const undoStack = useRef<UndoEntry[]>([])
  const redoStack = useRef<UndoEntry[]>([])

  useEffect(() => {
    tablesWorkbookSync.flush = () => hostedOf(tablesRef.current)
    tablesWorkbookSync.openCsvImport = () => setImportCsvOpen(true)
    // A request raised before this lazily-loaded tab mounted (Spotlight's
    // "Import CSV…" opens the window and asks in the same tick). Deferred a
    // microtask so mounting the grid is not also a state update.
    if (tablesWorkbookSync.csvImportPending) {
      tablesWorkbookSync.csvImportPending = false
      queueMicrotask(() => setImportCsvOpen(true))
    }
    return () => {
      tablesWorkbookSync.flush = null
      tablesWorkbookSync.openCsvImport = null
    }
  }, [])

  // NOTE: handlers and derived grid props below are deliberately plain
  // functions, not useCallback/useMemo — the hooks-v7 compiler lint cannot
  // preserve manual memoization over the tablesRef write-through pattern, and
  // React Compiler is not adopted here (see eslint.config.mjs). The grid
  // repaints only its visible window, so per-render identities are fine.

  /** Replaces one spec in both the synchronous ref and React state. */
  const commitSpec = (before: TableSpec, after: TableSpec, pushUndo: boolean) => {
    if (before === after) return
    tablesRef.current = tablesRef.current.map((t) => (t.id === before.id ? after : t))
    redoStack.current = []
    if (pushUndo) {
      undoStack.current.push({ tableId: before.id, before, after })
      if (undoStack.current.length > UNDO_LIMIT) undoStack.current.shift()
      redoStack.current = []
    }
    onTablesChange((prev) => prev.map((t) => (t.id === before.id ? after : t)))
  }

  const setWarning = (specId: string, warning: string | null) => {
    setWarnings((prev) => {
      if (warning) return { ...prev, [specId]: warning }
      if (!(specId in prev)) return prev
      const next = { ...prev }
      delete next[specId]
      return next
    })
  }

  /** The current spec by id from the synchronous view (props may be a render
   * behind a just-committed edit). */
  const liveSpec = (id: string | undefined | null): TableSpec | null =>
    (id && hostedOf(tablesRef.current).find((t) => t.id === id)) || null

  const undo = () => {
    const e = undoStack.current.pop()
    if (!e) return
    const current = liveSpec(e.tableId)
    if (e.after && (!current || current.rows !== e.after.rows)) return
    const restored = e.after && current ? restoreUserEdit(current, e.after, e.before) : e.before
    redoStack.current.push(e)
    tablesRef.current = current ? tablesRef.current.map((t) => t.id === e.tableId ? restored : t) : [...tablesRef.current, restored]
    onTablesChange(tablesRef.current)
    onActiveTableIdChange(e.tableId)
  }

  const redo = () => {
    const e = redoStack.current.pop()
    const current = e && liveSpec(e.tableId)
    if (!e || !current || current.rows !== e.before.rows) return
    undoStack.current.push(e)
    tablesRef.current = e.after
      ? tablesRef.current.map((t) => t.id === e.tableId ? restoreUserEdit(t, e.before, e.after!) : t)
      : tablesRef.current.filter((t) => t.id !== e.tableId)
    onTablesChange(tablesRef.current)
    onActiveTableIdChange(e.tableId)
  }

  // -------------------------------------------------------------------------
  // Grid wiring for the active table

  const readOnly = active?.source === 'code'
  const isFunction = active?.kind === 'function'

  const headerRowTheme: Partial<GdgTheme> = {
    bgCell: gridTheme.bgHeader,
    baseFontStyle: '600 12px',
    textDark: gridTheme.textHeader,
  }

  const columns: GridColumn[] = active
    ? headerTitles(active).map((title, c) => {
        const key = `${active.id}:${c}`
        const base =
          c === 0 && active.kind === 'parametric' ? 64 : Math.max(96, title.length * 8 + 28)
        return { id: key, title, width: widthOverrides[key] ?? base }
      })
    : []

  const rowCount = active ? gridRowCount(active) : 0

  const getCellContent = ([col, row]: Item): GridCell => {
    const spec = active
    const empty: GridCell = {
      kind: GridCellKind.Text,
      data: '',
      displayData: '',
      allowOverlay: false,
    }
    if (!spec || row >= gridRowCount(spec) || col >= boundColumnCount(spec)) return empty
    const view = cellViewAt(spec, row, col)
    const themeOverride: Partial<GdgTheme> | undefined =
      view.kind === 'header'
        ? headerRowTheme
        : view.kind === 'computed'
          ? { textDark: COMPUTED_GREEN }
          : view.failed
            ? { textDark: FAILED_RED }
            : undefined
    return {
      kind: GridCellKind.Text,
      data: view.text,
      // Legacy formulas are surfaced, never evaluated (D10): the ƒ marker
      // shows the cell carries one; the hint line lists the texts.
      displayData: view.formula ? `${view.text} ƒ`.trim() : view.text,
      allowOverlay: view.editable,
      readonly: !view.editable,
      contentAlign: view.kind === 'header' ? 'left' : 'right',
      themeOverride,
    }
  }

  const handleCellEdited = (cell: Item, newValue: EditableGridCell) => {
    if (newValue.kind !== GridCellKind.Text) return
    const spec = liveSpec(active?.id)
    if (!spec) return
    const [col, row] = cell
    const result = applyCellEdit(spec, row, col, newValue.data)
    if (!result.changed) return
    commitSpec(spec, result.spec, true)
    setWarning(
      spec.id,
      result.errorCells.length > 0
        ? `Formula error in ${result.errorCells.join(', ')} — those cells were stored blank and are excluded from the solver.`
        : null,
    )
  }

  /** Batched edits (glide's fill handle / fill-right / fill-down): applied
   * through the same per-cell rule as typing — the Run column and header
   * labels simply refuse — and committed as ONE spec change so a 20-cell fill
   * is one undo step. Returning true suppresses the per-cell fallback. */
  const handleCellsEdited = (newValues: readonly EditListItem[]): boolean | void => {
    if (newValues.length <= 1) return // single edits take the onCellEdited path
    const before = liveSpec(active?.id)
    if (!before) return true
    const result = applyCellEdits(before, newValues.flatMap(({ location: [col, gridRow], value }) =>
      value.kind === GridCellKind.Text ? [{ gridRow, col, text: value.data }] : []))
    const errorCells = result.errorCells
    if (result.changed) commitSpec(before, result.spec, true)
    setWarning(
      before.id,
      errorCells.length > 0
        ? `Formula error in ${errorCells.join(', ')} — those cells were stored blank and are excluded from the solver.`
        : null,
    )
    return true
  }

  const handlePaste = (target: Item, values: readonly (readonly string[])[]): boolean => {
    const spec = liveSpec(active?.id)
    if (!spec) return false
    const [col, row] = target
    const result = applyPaste(spec, row, col, values)
    if (result.changed) commitSpec(spec, result.spec, true)
    let warning: string | null = null
    if (result.truncated) {
      warning = `Table row limit reached — data past ${TABLE_MAX_ROWS} rows was dropped.`
    } else if (result.errorCells.length > 0) {
      warning = `Formula error in ${result.errorCells.join(', ')} — those cells were stored blank and are excluded from the solver.`
    } else if (result.outOfRegion) {
      warning = 'Content outside the table columns was clipped (columns are schema — use the toolbar).'
    } else if (result.intoReadOnly) {
      warning =
        spec.kind === 'parametric'
          ? 'The Run column is managed by the table — pasted run numbers were ignored (use Add Row to add runs).'
          : 'Header labels are managed by the toolbar — pasted header content was ignored.'
    }
    setWarning(spec.id, warning)
    return false // handled entirely here; glide must not apply it cell-by-cell
  }

  const handleDelete = (selection: GridSelection): boolean => {
    const spec = liveSpec(active?.id)
    if (!spec) return false
    const cells: { gridRow: number; col: number }[] = []
    const rows = gridRowCount(spec)
    const cols = boundColumnCount(spec)
    const addRect = (r: { x: number; y: number; width: number; height: number }) => {
      for (let row = r.y; row < r.y + r.height; row++) {
        for (let col = r.x; col < r.x + r.width; col++) {
          if (row < rows && col < cols) cells.push({ gridRow: row, col })
        }
      }
    }
    if (selection.current) {
      addRect(selection.current.range)
      for (const r of selection.current.rangeStack) addRect(r)
    }
    for (const col of selection.columns) addRect({ x: col, y: 0, width: 1, height: rows })
    for (const row of selection.rows) addRect({ x: 0, y: row, width: cols, height: 1 })
    if (cells.length === 0) return false
    const result = clearCells(spec, cells)
    if (result.changed) {
      commitSpec(spec, result.spec, true)
      setWarning(spec.id, null)
    }
    return false // handled here (row-trim rules live in the model)
  }

  const handleRowAppended = () => {
    const spec = liveSpec(active?.id)
    if (!spec) return
    const next = appendRow(spec)
    if (next !== spec) commitSpec(spec, next, true)
  }

  const handleKeyDown = (e: GridKeyEventArgs) => {
    const mod = e.ctrlKey || e.metaKey
    if (!mod) return
    const k = e.key.toLowerCase()
    if (k === 'z' && !e.shiftKey) {
      e.cancel()
      undo()
    } else if (k === 'y' || (k === 'z' && e.shiftKey)) {
      e.cancel()
      redo()
    }
  }

  const onColumnResize = (column: GridColumn, newSize: number) => {
    if (column.id) setWidthOverrides((w) => ({ ...w, [column.id as string]: newSize }))
  }

  // -------------------------------------------------------------------------
  // Toolbar actions (schema lives here, not in cells)

  const activeFn: FunctionTableSpec | null = active?.kind === 'function' ? active : null
  const activeParam: ParamTableSpec | null = active?.kind === 'parametric' ? active : null

  /** Schema/metadata edits from toolbar inputs (name, argName, log flags…) —
   * committed without undo entries so per-keystroke text edits don't flood
   * the stack. */
  const updateActiveFn = (patch: Partial<FunctionTableSpec>) => {
    const spec = liveSpec(activeFn?.id)
    if (!spec || spec.kind !== 'function') return
    commitSpec(spec, { ...spec, ...patch }, false)
  }

  const transformActive = (update: (t: TableSpec) => TableSpec, pushUndo = true) => {
    const spec = liveSpec(active?.id)
    if (!spec) return
    try {
      const next = update(spec)
      if (next !== spec) commitSpec(spec, next, pushUndo)
      setWarning(spec.id, null)
    } catch (error) {
      setWarning(spec.id, error instanceof Error ? error.message : String(error))
    }
  }

  const addTable = (kind: 'function-1d' | 'function-2d' | 'parametric') => {
    let created: TableSpec | null = null
    onTablesChange((prev) => {
      created =
        kind === 'parametric' ? newParamTable(prev) : newFunctionTable(prev, kind === 'function-1d')
      return [...prev, created]
    })
    requestAnimationFrame(() => {
      if (created) onActiveTableIdChange(created.id)
    })
  }

  /** Applies produced function specs (replace same-named GUI tables in
   * place, append the rest — composeTables.applyFunctionSpecs) and focuses
   * the first one. Same structural-change pattern as addTable. */
  const handleCreateFunctions = (specs: FunctionTableSpec[]) => {
    let firstId: string | null = null
    onTablesChange((prev) => {
      const applied = applyFunctionSpecs(prev, specs)
      firstId = applied.ids[0] ?? null
      return applied.tables
    })
    requestAnimationFrame(() => {
      if (firstId) onActiveTableIdChange(firstId)
    })
  }

  const removeTable = (id: string) => {
    const before = liveSpec(id)
    if (!before) return
    undoStack.current.push({ tableId: id, before, after: null })
    redoStack.current = []
    tablesRef.current = tablesRef.current.filter((t) => t.id !== id)
    onTablesChange((prev) => prev.filter((t) => t.id !== id))
    if (activeTableId === id) onActiveTableIdChange(hosted.find((t) => t.id !== id)?.id ?? null)
  }

  const commitRename = () => {
    if (renamingId) {
      const spec = liveSpec(renamingId)
      const name = renameDraft.trim()
      if (spec && name && name !== spec.name) commitSpec(spec, { ...spec, name }, false)
    }
    setRenamingId(null)
  }

  const handleExportCsv = (mode: 'exact' | 'display') => {
    if (!active) return
    const suffix = mode === 'display' ? '.display.csv' : '.csv'
    downloadValuesAsCsv(
      csvValuesFor(active, mode),
      `${active.name || 'table'}${suffix}`,
      csvExportComments(active, mode),
    )
  }

  const multiCurve = (activeFn?.columns.length ?? 0) > 1
  const callSignature = activeFn
    ? multiCurve
      ? `${activeFn.name || 'name'}(${activeFn.argName || 'x'}, ${activeFn.paramName || 'param'})`
      : `${activeFn.name || 'name'}(${activeFn.argName || 'x'})`
    : ''
  const legacyFormulas = active ? storedFormulaList(active) : []

  return (
    <Group align="stretch" gap={0} style={{ flex: 1, minHeight: 0, height: '100%' }} wrap="nowrap">
      {/* Navigation list of all hosted tables. */}
      <Stack
        gap={4}
        p="xs"
        style={{ width: 180, borderRight: '1px solid var(--mantine-color-default-border)', overflowY: 'auto' }}
      >
        <Group justify="space-between" wrap="nowrap">
          <Text size="xs" fw={700} c="dimmed">
            Tables
          </Text>
          <Menu position="bottom-end" shadow="md">
            <Menu.Target>
              <ActionIcon size="sm" variant="light" aria-label="Export active table as CSV" disabled={!active}>
                <IconDownload size={14} />
              </ActionIcon>
            </Menu.Target>
            <Menu.Dropdown>
              <Menu.Item leftSection={<IconFileTypeCsv size={14} />} onClick={() => handleExportCsv('exact')}>
                Exact values (round-trip)
              </Menu.Item>
              <Menu.Item leftSection={<IconFileTypeCsv size={14} />} onClick={() => handleExportCsv('display')}>
                Formatted report (6 significant digits)
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
          <Menu position="bottom-start" shadow="md">
            <Menu.Target>
              <ActionIcon size="sm" variant="light" aria-label="Add table">
                <IconPlus size={14} />
              </ActionIcon>
            </Menu.Target>
            <Menu.Dropdown>
              <Menu.Item leftSection={<IconTable size={14} />} onClick={() => addTable('parametric')}>
                Parametric Table — solve the system once per row
              </Menu.Item>
              <Menu.Item leftSection={<IconChartGridDots size={14} />} onClick={() => addTable('function-1d')}>
                Function Table (without Curve)
              </Menu.Item>
              <Menu.Item leftSection={<IconChartGridDots size={14} />} onClick={() => addTable('function-2d')}>
                Function Table (with Curve family)
              </Menu.Item>
              <Menu.Divider />
              <Menu.Item leftSection={<IconFileTypeCsv size={14} />} onClick={() => setImportCsvOpen(true)}>
                Import CSV… — a Function Table from measured data
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
        </Group>
        {hosted.map((t) => (
          <Group
            key={t.id}
            gap={4}
            px={6}
            py={2}
            wrap="nowrap"
            style={{
              cursor: 'pointer',
              borderRadius: 4,
              background: active?.id === t.id ? 'var(--mantine-color-default-hover)' : undefined,
            }}
            onClick={() => onActiveTableIdChange(t.id)}
            onDoubleClick={() => {
              if (t.source !== 'code') {
                setRenamingId(t.id)
                setRenameDraft(t.name)
              }
            }}
          >
            {t.kind === 'parametric' ? (
              <IconTable size={13} color="var(--mantine-color-teal-4)" />
            ) : (
              <IconChartGridDots size={13} color="var(--mantine-color-teal-4)" />
            )}
            {renamingId === t.id ? (
              <TextInput
                size="xs"
                value={renameDraft}
                autoFocus
                onChange={(e) => setRenameDraft(e.currentTarget.value)}
                onBlur={commitRename}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') commitRename()
                  if (e.key === 'Escape') setRenamingId(null)
                }}
                style={{ flex: 1 }}
              />
            ) : (
              <Text size="xs" style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {t.name}
              </Text>
            )}
            {t.source === 'code' ? (
              <Tooltip label="Defined by a TABLE block in the editor (read-only)">
                <Badge size="xs" variant="light" color="grape">
                  code
                </Badge>
              </Tooltip>
            ) : (
              <ActionIcon
                size="xs"
                variant="subtle"
                color="red"
                aria-label={`Delete ${t.name}`}
                onClick={(e) => {
                  e.stopPropagation()
                  removeTable(t.id)
                }}
              >
                <IconTrash size={11} />
              </ActionIcon>
            )}
          </Group>
        ))}
        {hosted.length === 0 && (
          <Text size="xs" c="dimmed">
            No tables yet. Add a Parametric Table to run the system over value sets, or a Function
            Table to turn tabulated data into a function callable from the equations — typed in,
            swept from a solved table, or imported from a .csv.
          </Text>
        )}
      </Stack>

      <Stack gap={4} p="xs" style={{ flex: 1, minWidth: 0 }}>
        {activeFn && (
          <Group gap="xs" align="flex-end" wrap="wrap">
            <TextInput
              size="xs"
              label="Function name"
              value={activeFn.name}
              onChange={(e) => updateActiveFn({ name: e.currentTarget.value })}
              readOnly={readOnly}
              w={130}
            />
            <TextInput
              size="xs"
              label="Argument (X column)"
              value={activeFn.argName}
              onChange={(e) => updateActiveFn({ argName: e.currentTarget.value })}
              readOnly={readOnly}
              w={130}
            />
            {!activeFn.is1D && (
              <TextInput
                size="xs"
                label="Curve parameter"
                placeholder="e.g. T"
                value={activeFn.paramName}
                onChange={(e) => updateActiveFn({ paramName: e.currentTarget.value })}
                readOnly={readOnly}
                disabled={!multiCurve}
                w={110}
              />
            )}
            <Checkbox
              size="xs"
              label="log X"
              checked={activeFn.xLog}
              onChange={(e) => updateActiveFn({ xLog: e.currentTarget.checked })}
              disabled={readOnly}
              mb={6}
            />
            <Checkbox
              size="xs"
              label="log Y"
              checked={activeFn.yLog}
              onChange={(e) => updateActiveFn({ yLog: e.currentTarget.checked })}
              disabled={readOnly}
              mb={6}
            />
            {!readOnly && (
              <>
                {!activeFn.is1D && (
                  <Button
                    size="compact-xs"
                    variant="default"
                    mb={4}
                    onClick={() =>
                      transformActive((t) =>
                        t.kind === 'function'
                          ? {
                              ...t,
                              columns: [...t.columns, ''],
                              rows: t.rows.map((r) => ({ ...r, ys: [...r.ys, ''] })),
                            }
                          : t,
                      )
                    }
                  >
                    Add curve
                  </Button>
                )}
                {!activeFn.is1D && (
                  <Button
                    size="compact-xs"
                    variant="default"
                    mb={4}
                    disabled={activeFn.columns.length <= 1}
                    onClick={() =>
                      transformActive((t) =>
                        t.kind === 'function'
                          ? {
                              ...t,
                              columns: t.columns.slice(0, -1),
                              rows: t.rows.map((r) => ({ ...r, ys: r.ys.slice(0, -1) })),
                            }
                          : t,
                      )
                    }
                  >
                    Remove curve
                  </Button>
                )}
                <Button
                  size="compact-xs"
                  variant="default"
                  mb={4}
                  leftSection={<IconArrowsSort size={13} />}
                  onClick={() => transformActive((t) => (t.kind === 'function' ? sortFunctionRows(t) : t))}
                >
                  Sort
                </Button>
                <Tooltip label="Interpolate blank cells from each curve's known points (log-aware)">
                  <Button
                    size="compact-xs"
                    mb={4}
                    leftSection={<IconSparkles size={13} />}
                    onClick={() => transformActive((t) => (t.kind === 'function' ? fillMissingCells(t) : t))}
                  >
                    Fill missing
                  </Button>
                </Tooltip>
              </>
            )}
            <Text size="xs" c="dimmed" mb={6}>
              Use in equations: <Code>U = {callSignature}</Code>
            </Text>
          </Group>
        )}
        {activeParam && (
          <Group gap="xs" align="center" wrap="wrap">
            <Button size="xs" variant="default" onClick={() => onConfigureTable?.(activeParam.id)}>
              Configure Columns
            </Button>
            <Button size="xs" variant="default" onClick={() => transformActive(appendRow)}>
              Add Row
            </Button>
            <Button
              size="xs"
              variant="default"
              disabled={activeParam.rows.length <= 1}
              onClick={() => transformActive(removeLastRow)}
            >
              Remove Row
            </Button>
            {activeParam.vars.length > 0 && (
              <Menu position="bottom-start" shadow="md">
                <Menu.Target>
                  <Button size="xs" variant="default">
                    Fill Column…
                  </Button>
                </Menu.Target>
                <Menu.Dropdown>
                  {activeParam.vars.map((name) => (
                    <Menu.Item key={name} onClick={() => onAlterColumn?.(activeParam.id, name)}>
                      {name}
                    </Menu.Item>
                  ))}
                </Menu.Dropdown>
              </Menu>
            )}
            {activeParam.vars.length >= 2 && (
              <Tooltip label="Turn two columns of this table into a Function Table callable in equations">
                <Button
                  size="xs"
                  variant="default"
                  leftSection={<IconMathFunction size={13} />}
                  onClick={() => setCreateFnFor(activeParam.id)}
                >
                  Create function…
                </Button>
              </Tooltip>
            )}
            {activeParam.results.length > 0 && (
              <Button
                size="xs"
                variant="subtle"
                onClick={() =>
                  transformActive(
                    (t) => (t.kind === 'parametric' ? { ...t, results: [] } : t),
                    false,
                  )
                }
              >
                Clear Results
              </Button>
            )}
            <Text size="xs" c="dimmed">
              {activeParam.vars.length === 0
                ? 'Run Check first, then Configure Columns to choose the table variables. Blank cells are solved per run (shown green).'
                : activeParam.results.length > 0
                  ? `${activeParam.results.filter((r) => r.success).length}/${activeParam.results.length} runs solved — computed cells are green; typing over one makes it an input.`
                  : 'Fill cells for independent variables; blank cells are solved for each run (Run Table in the top bar).'}
            </Text>
          </Group>
        )}
        {active && warnings[active.id] && (
          <Text size="xs" c="yellow.5">
            {warnings[active.id]}
          </Text>
        )}
        {active && tableInputIssues(active).length > 0 && <Text c="red" size="xs">{tableInputIssues(active).slice(0, 12).join('; ')}</Text>}
        {activeParam && <Text size="xs">Run status: {activeParam.runStatus ?? 'not-run'}</Text>}
        {activeParam?.stats && <Text size="xs" c={activeParam.stats.converged === false ? 'orange' : 'dimmed'}>
          {activeParam.stats.solved}/{activeParam.stats.runs} completed; {activeParam.stats.failed} failed; {activeParam.stats.notRun ?? 0} not run.
          {' '}{activeParam.stats.passes ?? 1} passes — {activeParam.stats.termination ?? 'completed'}.
          {activeParam.stats.converged === false && ' Values are provisional; the table has not converged.'}
        </Text>}
        {activeParam?.results.some((r) => r.error) && <Text size="xs" c="red">
          {activeParam.results.flatMap((r, i) => r.error ? [`Run ${i + 1}: ${r.error}`] : []).join('; ')}
        </Text>}
        {legacyFormulas.length > 0 && (
          <Text size="xs" c="orange.4">
            Stored formulas (ƒ) are shown read-only and no longer recalculate:{' '}
            {legacyFormulas
              .slice(0, 4)
              .map(({ ref, formula }) => `${ref} = ${formula}`)
              .join('; ')}
            {legacyFormulas.length > 4 ? `; … ${legacyFormulas.length - 4} more` : ''}
            {'. '}Editing a cell replaces its formula with the typed value; use Fill Column for
            ranges.
          </Text>
        )}
        <div ref={sizeRef} style={{ flex: 1, minHeight: 0, width: '100%', position: 'relative' }}>
          {active && width > 0 && height > 0 && (
            <DataEditor
              key={active.id}
              theme={gridTheme}
              columns={columns}
              rows={rowCount}
              getCellContent={getCellContent}
              width={width}
              height={height}
              headerHeight={isFunction ? 0 : 36}
              rowMarkers="none"
              smoothScrollX
              smoothScrollY
              getCellsForSelection
              fillHandle={!readOnly}
              onCellEdited={readOnly ? undefined : handleCellEdited}
              onCellsEdited={readOnly ? undefined : handleCellsEdited}
              onPaste={readOnly ? false : handlePaste}
              onDelete={readOnly ? undefined : handleDelete}
              onKeyDown={handleKeyDown}
              onColumnResize={onColumnResize}
              {...(!readOnly
                ? {
                    trailingRowOptions: {
                      tint: true,
                      hint: 'Add row…',
                      sticky: false,
                    },
                    onRowAppended: handleRowAppended,
                  }
                : {})}
            />
          )}
          {!active && (
            <Text size="xs" c="dimmed" p="md">
              Select or add a table on the left.
            </Text>
          )}
        </div>
      </Stack>

      {(() => {
        const source = tables.find((t) => t.id === createFnFor)
        if (!source || source.kind !== 'parametric') return null
        return (
          <CreateFunctionModal
            table={source}
            tables={tables}
            onCreate={handleCreateFunctions}
            onClose={() => setCreateFnFor(null)}
          />
        )
      })()}

      {importCsvOpen && (
        <ImportCsvModal
          tables={tables}
          onCreate={(spec) => handleCreateFunctions([spec])}
          onClose={() => setImportCsvOpen(false)}
        />
      )}
    </Group>
  )
}
