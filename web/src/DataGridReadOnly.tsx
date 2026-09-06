import { useCallback, useMemo, useState } from 'react'
import {
  CompactSelection,
  DataEditor,
  GridCell,
  GridCellKind,
  GridColumn,
  GridSelection,
  Item,
  Theme as GdgTheme,
} from '@glideapps/glide-data-grid'
import '@glideapps/glide-data-grid/dist/index.css'
import { Button, Group, Stack, Text, useComputedColorScheme, useMantineTheme } from '@mantine/core'
import { IconChartLine } from '@tabler/icons-react'
import { useElementSize } from '@mantine/hooks'
import { fmt6, ParamRow, readOnlyCellText } from './tables'
import { TableRowResult } from './api'
import { VariableDraft } from './VariableInfoModal'
import { displayVar } from './varDisplay'

const EMPTY_SELECTION: GridSelection = {
  columns: CompactSelection.empty(),
  rows: CompactSelection.empty(),
}

// ---------------------------------------------------------------------------
// Read-only, virtualized data grid for solver-produced tables (code PARAMETRIC
// blocks and DYNAMIC/ODE trajectories). These can be very wide (every system
// variable becomes a column) and very tall (one row per ODE sample point), so
// they are rendered through glide-data-grid — a canvas grid that only paints
// the visible window — instead of the per-cell Mantine inputs used for small,
// editable parametric tables. The grid is themed from the Mantine tokens so it
// tracks the app's dark/light color scheme.
// ---------------------------------------------------------------------------

interface Props {
  vars: string[]
  rows: ParamRow[]
  /**
   * Per-row solve outcomes, when the table has been run.
   *
   * <p>`rows` holds only what the user (or the code block) supplied as INPUT,
   * so a code PARAMETRIC table painted from `rows` alone shows its sweep column
   * and nothing else — the solved columns stay blank however many runs
   * succeeded. The solved numbers live here instead, and are merged in per cell
   * exactly as the editable workbook does it, via the shared
   * {@link paramComputedValue}: a computed value appears only where the row
   * solved AND the input was left blank, so a user-supplied input is never
   * overwritten by the solver's echo of it.
   */
  results?: TableRowResult[]
  /** Per-variable display metadata; only `units` is used here (header label). */
  varDrafts: Record<string, VariableDraft>
  /** Per-column SI units for ODE/code tables (column name → unit). Preferred
   * over `varDrafts` since ODE columns are not solved scalars in `varDrafts`. */
  columnUnits?: Record<string, string>
  /** Open a new X-Y plot from the column selection: the first column is the
   * x-axis (time for ODE tables) and the selected columns are the y-axis. */
  onPlotColumns?: (xVar: string, yVars: string[]) => void
}

/**
 * Map glide's theme onto Mantine tokens so a grid matches the rest of the app
 * and follows the active color scheme. Header text uses the same teal the
 * Mantine table headers used; cell backgrounds use the dark surface shades.
 * Exported for the other glide grids (the Tables workbook, the per-table
 * read-only windows).
 */
export function useGlideTheme(): Partial<GdgTheme> {
  const theme = useMantineTheme()
  const dark = useComputedColorScheme('dark') === 'dark'
  return useMemo<Partial<GdgTheme>>(() => {
    const c = theme.colors
    return {
      accentColor: c.teal[6],
      accentLight: dark ? 'rgba(56, 178, 172, 0.18)' : 'rgba(56, 178, 172, 0.12)',
      textDark: dark ? c.dark[0] : c.gray[9],
      textMedium: dark ? c.dark[1] : c.gray[7],
      textLight: dark ? c.dark[2] : c.gray[5],
      textHeader: c.teal[4],
      textHeaderSelected: c.teal[3],
      bgCell: dark ? c.dark[7] : theme.white,
      bgCellMedium: dark ? c.dark[6] : c.gray[0],
      bgHeader: dark ? c.dark[6] : c.gray[1],
      bgHeaderHovered: dark ? c.dark[5] : c.gray[2],
      bgHeaderHasFocus: dark ? c.dark[5] : c.gray[2],
      bgBubble: dark ? c.dark[5] : c.gray[1],
      bgBubbleSelected: c.teal[7],
      bgSearchResult: dark ? c.yellow[9] : c.yellow[2],
      borderColor: dark ? 'rgba(255, 255, 255, 0.08)' : 'rgba(0, 0, 0, 0.08)',
      horizontalBorderColor: dark ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.05)',
      drilldownBorder: dark ? 'rgba(255, 255, 255, 0.2)' : 'rgba(0, 0, 0, 0.2)',
      fontFamily: theme.fontFamilyMonospace,
      baseFontStyle: '12px',
      headerFontStyle: '600 12px',
      editorFontSize: '12px',
      cellHorizontalPadding: 8,
      cellVerticalPadding: 4,
    }
  }, [theme, dark])
}

export default function DataGridReadOnly({ vars, rows, results, varDrafts, columnUnits, onPlotColumns }: Readonly<Props>) {
  const { ref, width, height } = useElementSize()
  const gridTheme = useGlideTheme()

  // Columns are user-resizable; remember overrides keyed by column id (var name).
  const [widthOverrides, setWidthOverrides] = useState<Record<string, number>>({})
  const columns = useMemo<GridColumn[]>(
    () =>
      vars.map((name) => {
        const units = columnUnits?.[name] ?? varDrafts[name]?.units
        // Demangle flat solver names for display only: component port members are
        // stored as `brg$port$t` but shown dotted (`brg.port.t`), matching how the
        // rest of the app renders them. The column id / data key stays the raw
        // name so cell lookup and plot wiring are unaffected.
        const label = displayVar(name)
        const title = units ? `${label} [${units}]` : label
        return {
          id: name,
          title,
          width: widthOverrides[name] ?? Math.max(96, title.length * 8 + 28),
        }
      }),
    [vars, varDrafts, columnUnits, widthOverrides],
  )

  // Keep clipboard data exact; round only the painted cell.
  const getCellContent = useCallback(
    ([col, row]: Item): GridCell => {
      const name = vars[col]
      const value = readOnlyCellText(rows[row], results?.[row], name)
      return {
        kind: GridCellKind.Text,
        data: value,
        displayData: value.trim() !== '' && Number.isFinite(Number(value)) ? fmt6(Number(value)) : value,
        allowOverlay: false,
        contentAlign: 'right',
      }
    },
    [vars, rows, results],
  )

  const onColumnResize = useCallback((column: GridColumn, newSize: number) => {
    if (column.id) setWidthOverrides((w) => ({ ...w, [column.id as string]: newSize }))
  }, [])

  // Column selection drives the "Plot curve" action: the first column is the
  // x-axis (time for ODE tables) and the other selected columns are the y-axis.
  const [selection, setSelection] = useState<GridSelection>(EMPTY_SELECTION)
  const xVar = vars[0]
  const selectedY = useMemo(
    () => selection.columns.toArray().map((i) => vars[i]).filter((v) => v && v !== xVar),
    [selection, vars, xVar],
  )

  return (
    <Stack gap={6} style={{ flex: 1, minHeight: 0 }}>
      {onPlotColumns && (
        <Group gap="sm">
          <Button
            size="xs"
            variant="light"
            leftSection={<IconChartLine size={14} />}
            disabled={selectedY.length === 0}
            onClick={() => onPlotColumns(xVar, selectedY)}
          >
            Plot curve
          </Button>
          <Text size="xs" c="dimmed">
            {selectedY.length === 0
              ? `Select one or more columns (click a header, Ctrl/Cmd-click to add) to plot them vs ${displayVar(xVar)}.`
              : `Plot ${selectedY.map(displayVar).join(', ')} vs ${displayVar(xVar)}.`}
          </Text>
        </Group>
      )}
      <div ref={ref} style={{ flex: 1, minHeight: 0, width: '100%', position: 'relative' }}>
        {width > 0 && height > 0 && (
          <DataEditor
            theme={gridTheme}
            columns={columns}
            rows={rows.length}
            getCellContent={getCellContent}
            width={width}
            height={height}
            rowMarkers="number"
            smoothScrollX
            smoothScrollY
            getCellsForSelection
            gridSelection={selection}
            onGridSelectionChange={setSelection}
            onColumnResize={onColumnResize}
          />
        )}
      </div>
    </Stack>
  )
}
