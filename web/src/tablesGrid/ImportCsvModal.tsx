// tablesGrid/ImportCsvModal.tsx — Import CSV… → Function Table (decision D11).
//
// The Data Analyzer's CSV import, relocated into the Tables workbook and
// pointed at the destination that made the Analyzer redundant: a measured
// series becomes a GUI Function Table, callable straight from the equations
// (`U = speed(time)`), instead of a strip chart you could only look at.
//
// The conversion is composeTables.functionSpecFromXY — the very function
// Wave H's ChannelFunctionModal called, so the row cap, the uniform
// decimation, the duplicate-x rule and the skipped-pair count are the same
// behaviour they always were. Reading is tablesGrid/csv.parseCsvTable.

import { useMemo, useState } from 'react'
import { Button, Checkbox, Code, FileInput, Group, Modal, Select, Stack, Text, TextInput } from '@mantine/core'
import { IconFileTypeCsv } from '@tabler/icons-react'
import { FunctionTableSpec, identifier, TableSpec } from '../tables'
import { checkFunctionName, functionSpecFromXY, type ReductionChoice } from './composeTables'
import { FunctionNameHints, FunctionPrecedenceNote } from './FunctionNameHints'
import FunctionReductionControls from './FunctionReductionControls'
import { parseCsvTable, type CsvTable, type CsvOptions } from './csv'

/** Whole-file read cap. A function table holds 5 000 rows, so a recording
 *  bigger than this is one to trim before importing — and reading it as a
 *  single string would cost far more memory than the table it produces. */
const MAX_BYTES = 64 * 1024 * 1024

interface Props {
  /** For function-name collision checks (may be empty). */
  tables: TableSpec[]
  onClose: () => void
  onCreate: (spec: FunctionTableSpec) => void
}

interface Loaded {
  fileName: string
  text: string
  table: CsvTable
}

export default function ImportCsvModal({ tables, onClose, onCreate }: Readonly<Props>) {
  const [loaded, setLoaded] = useState<Loaded | null>(null)
  const [reading, setReading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [xIndex, setXIndex] = useState<number | null>(null)
  const [yIndex, setYIndex] = useState<number | null>(null)
  const [name, setName] = useState('')
  const [nameTouched, setNameTouched] = useState(false)
  const [delimiter, setDelimiter] = useState('auto')
  const [parseOptions, setParseOptions] = useState<CsvOptions>({
    header: 'auto',
    decimal: '.',
    unitRow: false,
  })
  const [reduction, setReduction] = useState<ReductionChoice | null>(null)
  const [xMin, setXMin] = useState('')
  const [xMax, setXMax] = useState('')

  const applyParsed = (
    fileName: string,
    text: string,
    nextDelimiter: string,
    nextOptions: CsvOptions,
    prevX: number | null,
    prevY: number | null,
  ) => {
    const table = parseCsvTable(
      text,
      nextDelimiter === 'auto' ? undefined : nextDelimiter,
      nextOptions,
    )
    const numeric = table.columns.filter((c) => c.numericCount > 0)
    const stillNumeric = (index: number | null): index is number =>
      index !== null && (table.columns[index]?.numericCount ?? 0) > 0
    const x = stillNumeric(prevX) ? prevX : (numeric[0]?.index ?? null)
    const y =
      stillNumeric(prevY) && prevY !== x
        ? prevY
        : (numeric.find((c) => c.index !== x)?.index ?? null)
    setLoaded({ fileName, text, table })
    setXIndex(x)
    setYIndex(y)
    if (!nameTouched) {
      const yCol = y === null ? undefined : table.columns[y]
      if (yCol) setName(identifier(yCol.name, 'f').toLowerCase())
    }
  }

  const reparse = (nextDelimiter: string, nextOptions: CsvOptions) => {
    setDelimiter(nextDelimiter)
    setParseOptions(nextOptions)
    setError(null)
    if (!loaded) return
    try {
      applyParsed(loaded.fileName, loaded.text, nextDelimiter, nextOptions, xIndex, yIndex)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }

  /** Columns with at least one number in them — a text column cannot be an
   *  axis, and listing it only invites a confusing empty result. */
  const numericColumns = loaded ? loaded.table.columns.filter((c) => c.numericCount > 0) : []
  const columnAt = (index: number | null) =>
    index === null ? null : (loaded?.table.columns[index] ?? null)
  const xColumn = columnAt(xIndex)
  const yColumn = columnAt(yIndex)

  const pickFile = (file: File | null) => {
    setError(null)
    setLoaded(null)
    setXIndex(null)
    setYIndex(null)
    setReduction(null)
    setXMin('')
    setXMax('')
    if (!file) return
    if (file.size > MAX_BYTES) {
      setError(
        `“${file.name}” is ${(file.size / 1024 / 1024).toFixed(0)} MB — larger than the ` +
          `${MAX_BYTES / 1024 / 1024} MB import limit. Trim or downsample the recording first.`,
      )
      return
    }
    setReading(true)
    file
      .text()
      .then((text) => {
        applyParsed(file.name, text, delimiter, parseOptions, null, null)
      })
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setReading(false))
  }

  const pickY = (value: string | null) => {
    if (value === null) return
    const index = Number(value)
    setYIndex(index)
    const column = loaded?.table.columns[index]
    if (column && !nameTouched) setName(identifier(column.name, 'f').toLowerCase())
  }

  const argName = xColumn ? identifier(xColumn.name, 'x') : 'x'
  const nameCheck = checkFunctionName(tables, name)

  const preview = useMemo(() => {
    if (!xColumn || !yColumn) return null
    const min = xMin.trim() === '' ? Number.NaN : Number(xMin)
    const max = xMax.trim() === '' ? Number.NaN : Number(xMax)
    return functionSpecFromXY({
      name: name.trim(),
      argName,
      xs: xColumn.values,
      ys: yColumn.values,
      reduction: reduction ?? undefined,
      xMin: Number.isFinite(min) ? min : undefined,
      xMax: Number.isFinite(max) ? max : undefined,
    })
  }, [xColumn, yColumn, name, argName, reduction, xMin, xMax])

  const canCreate =
    !error && nameCheck.ok && preview !== null && preview.usedRows > 0 && !preview.needsReduction

  const create = () => {
    if (!canCreate || preview === null) return
    onCreate(preview.spec)
    onClose()
  }

  const columnOptions = numericColumns.map((c) => ({
    value: String(c.index),
    label: `${c.name}${c.unit ? ` [${c.unit}]` : ''} (${c.numericCount.toLocaleString()} numeric)`,
  }))

  return (
    <Modal opened onClose={onClose} title="Import CSV as Function Table" centered size="lg">
      <Text size="sm" c="dimmed" mb="md">
        Turns two columns of a .csv into a Function Table callable from equations (interpolated
        lookup). Blank and non-numeric cells are skipped; duplicate x values keep the first row.
        The file is read in this browser tab and never uploaded.
      </Text>

      <Stack gap="sm">
        <FileInput
          label="CSV file"
          placeholder="Choose a .csv file…"
          accept=".csv,.tsv,.txt,text/csv,text/tab-separated-values"
          leftSection={<IconFileTypeCsv size={16} />}
          onChange={pickFile}
          disabled={reading}
          clearable
        />

        <Group grow>
          <Select
            label="Delimiter"
            value={delimiter}
            data={[
              { value: 'auto', label: 'Auto' },
              { value: ',', label: 'Comma' },
              { value: ';', label: 'Semicolon' },
              { value: '\t', label: 'Tab' },
              { value: '|', label: 'Pipe' },
            ]}
            onChange={(v) => v && reparse(v, parseOptions)}
          />
          <Select
            label="Header row"
            value={parseOptions.header}
            data={[
              { value: 'auto', label: 'Auto' },
              { value: 'yes', label: 'Yes' },
              { value: 'no', label: 'No' },
            ]}
            onChange={(v) =>
              v && reparse(delimiter, { ...parseOptions, header: v as CsvOptions['header'] })
            }
          />
          <Select
            label="Decimal separator"
            value={parseOptions.decimal}
            data={['.', ',']}
            onChange={(v) =>
              v && reparse(delimiter, { ...parseOptions, decimal: v as '.' | ',' })
            }
          />
        </Group>
        <Checkbox
          label="First data record contains units"
          checked={parseOptions.unitRow}
          onChange={(e) =>
            reparse(delimiter, { ...parseOptions, unitRow: e.currentTarget.checked })
          }
        />
        {loaded && (
          <>
            <Text size="xs">Raw preview (first 4,096 characters)</Text>
            <Code block style={{ maxHeight: 120, overflow: 'auto' }}>
              {loaded.text.slice(0, 4096)}
            </Code>
            <Text size="xs">Parsed preview (first five records)</Text>
            <Code block>
              {[
                loaded.table.columns.map((c) => c.name).join(' | '),
                ...Array.from({ length: Math.min(5, loaded.table.rowCount) }, (_, i) =>
                  loaded.table.columns
                    .map((c) =>
                      Number.isFinite(c.values[i]) ? String(c.values[i]) : '(blank/invalid)',
                    )
                    .join(' | '),
                ),
              ].join('\n')}
            </Code>
            {!!loaded.table.rejectedRows?.length && (
              <Text c="orange" size="xs">
                {loaded.table.rejectedRows.length} source issue
                {loaded.table.rejectedRows.length === 1 ? '' : 's'}:{' '}
                {loaded.table.rejectedRows
                  .slice(0, 12)
                  .map((r) => `Record ${r.record}: ${r.reason}`)
                  .join('; ')}
              </Text>
            )}
            <Text size="xs" c="dimmed">
              {loaded.table.rowCount.toLocaleString()} data row
              {loaded.table.rowCount === 1 ? '' : 's'} × {loaded.table.columns.length} column
              {loaded.table.columns.length === 1 ? '' : 's'}
              {loaded.table.headerless
                ? ' — no header row found, columns are named by position.'
                : '.'}
            </Text>

            <Group grow align="flex-start">
              <Select
                label="X column (lookup argument)"
                data={columnOptions.filter((o) => o.value !== String(yIndex))}
                value={xIndex === null ? null : String(xIndex)}
                onChange={(v) => v !== null && setXIndex(Number(v))}
                allowDeselect={false}
                searchable
              />
              <Select
                label="Y column (function values)"
                data={columnOptions.filter((o) => o.value !== String(xIndex))}
                value={yIndex === null ? null : String(yIndex)}
                onChange={pickY}
                allowDeselect={false}
                searchable
              />
            </Group>

            <TextInput
              label="Function name"
              value={name}
              onChange={(e) => {
                setName(e.currentTarget.value)
                setNameTouched(true)
              }}
              error={nameCheck.error}
              spellCheck={false}
              styles={{ input: { fontFamily: 'var(--mantine-font-family-monospace)' } }}
            />

            {preview && (
              <>
                <Text size="xs" c={preview.uniqueCount === 0 ? 'red' : 'dimmed'}>
                  {preview.uniqueCount === 0
                    ? 'No numeric pairs in the selected columns — pick different columns.'
                    : `Use in equations: `}
                  {preview.uniqueCount > 0 && (
                    <Text span size="xs" ff="monospace">
                      U = {name.trim() || 'name'}({argName})
                    </Text>
                  )}
                </Text>
                <FunctionReductionControls
                  result={preview}
                  reduction={reduction}
                  onReduction={setReduction}
                  xMin={xMin}
                  xMax={xMax}
                  onRange={(min, max) => {
                    setXMin(min)
                    setXMax(max)
                  }}
                />
              </>
            )}

            <FunctionNameHints name={name} check={nameCheck} />
            <FunctionPrecedenceNote />
          </>
        )}

        {error && (
          <Text c="red" size="sm">
            {error}
          </Text>
        )}

        <Group justify="flex-end" mt="xs">
          <Button variant="default" onClick={onClose}>
            Cancel
          </Button>
          <Button
            onClick={create}
            disabled={!canCreate}
            loading={reading}
            color={nameCheck.replacesGui ? 'yellow' : undefined}
          >
            {nameCheck.replacesGui ? 'Create (replace existing)' : 'Create function table'}
          </Button>
        </Group>
      </Stack>
    </Modal>
  )
}
