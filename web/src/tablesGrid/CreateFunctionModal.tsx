import { tableInputIssues } from '../tableValidation'
// tablesGrid/CreateFunctionModal.tsx
//
// Sweep → Function (Wave H, the flagship composition feature): turns columns
// of any parametric-kind table (GUI, code PARAMETRIC, or a solved ODE
// trajectory) into Function Tables instantly callable from equations.
// Pick the x column, one or more y columns (one 1-D function per pick), or a
// single y plus a family-parameter column (one 2-D function). All conversion
// logic lives in composeTables.ts; this file is only the dialog.

import { useMemo, useState } from 'react'
import {
  Button,
  Code,
  Group,
  Modal,
  MultiSelect,
  Select,
  Stack,
  Text,
  TextInput,
} from '@mantine/core'
import { FunctionTableSpec, identifier, ParamTableSpec, TableSpec } from '../tables'
import {
  checkFunctionName,
  ComposeResult,
  functionSpecFromParamColumns,
  NameCheck,
} from './composeTables'
import { FunctionNameHints, FunctionPrecedenceNote } from './FunctionNameHints'
import { TABLE_MAX_ROWS } from './tableGridModel'

const NONE = '__none__'

interface Props {
  /** The source table (any origin: gui, code, or a solved ODE trajectory). */
  table: ParamTableSpec
  /** Full table list, for name-collision checks. */
  tables: TableSpec[]
  /** Receives the produced specs; the host applies replace-vs-add. */
  onCreate: (specs: FunctionTableSpec[]) => void
  onClose: () => void
}

function defaultName(yVar: string): string {
  return identifier(yVar, 'f').toLowerCase()
}

interface Pick {
  yVar: string
  name: string
  check: NameCheck
  result: ComposeResult
}

export default function CreateFunctionModal({
  table,
  tables,
  onCreate,
  onClose,
}: Readonly<Props>) {
  const inputIssues = tableInputIssues(table, false)
  const vars = table.vars
  const [xVar, setXVar] = useState<string>(vars[0] ?? '')
  const [yVars, setYVars] = useState<string[]>(vars.length > 1 ? [vars[1]] : [])
  const [familyVar, setFamilyVar] = useState<string>(NONE)
  const [nameEdits, setNameEdits] = useState<Record<string, string>>({})

  const family = yVars.length === 1 && familyVar !== NONE ? familyVar : null
  const nameOf = (yVar: string) => nameEdits[yVar] ?? defaultName(yVar)

  const picks: Pick[] = useMemo(
    () => inputIssues.length ? [] :
      yVars
        .filter((y) => y !== xVar && vars.includes(y))
        .map((yVar) => {
          const name = nameOf(yVar)
          return {
            yVar,
            name,
            check: checkFunctionName(tables, name),
            result: functionSpecFromParamColumns({
              table,
              xVar,
              yVar,
              familyVar: family,
              name: name.trim(),
            }),
          }
        }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [table, tables, xVar, yVars, family, nameEdits],
  )

  const duplicateNames = new Set(
    picks
      .map((p) => p.name.trim().toLowerCase())
      .filter((n, i, all) => all.indexOf(n) !== i),
  )
  const invalid =
    picks.length === 0 ||
    xVar === '' ||
    picks.some(
      (p) =>
        !p.check.ok ||
        duplicateNames.has(p.name.trim().toLowerCase()) ||
        p.result.usedRows === 0,
    )
  const anyReplace = picks.some((p) => p.check.replacesGui)

  const create = () => {
    if (invalid) return
    onCreate(picks.map((p) => p.result.spec))
    onClose()
  }

  const yOptions = vars.filter((v) => v !== xVar)
  const familyOptions = vars.filter((v) => v !== xVar && !yVars.includes(v))

  return (
    <Modal opened onClose={onClose} title="Create Function from Table Columns" centered size="lg">
      <Text size="sm" c="dimmed" mb="md">
        Turns columns of “{table.name}” into a Function Table callable from equations. Each row
        uses the typed input where present, otherwise the solved value; failed or incomplete rows
        are skipped.
      </Text>

      <Stack gap="sm">
        {inputIssues.length > 0 && <Text c="red" size="sm">{inputIssues.slice(0, 12).join('; ')}</Text>}
        <Group grow align="flex-start">
          <Select
            label="X column (lookup argument)"
            data={vars}
            value={xVar}
            onChange={(v) => {
              if (!v) return
              setXVar(v)
              setYVars((prev) => prev.filter((y) => y !== v))
              setFamilyVar((prev) => (prev === v ? NONE : prev))
            }}
            allowDeselect={false}
          />
          <MultiSelect
            label="Y column(s) — one function per pick"
            data={yOptions}
            value={yVars}
            onChange={(next) => {
              setYVars(next)
              if (next.length !== 1) setFamilyVar(NONE)
              else setFamilyVar((prev) => (next.includes(prev) ? NONE : prev))
            }}
            searchable
          />
        </Group>

        {yVars.length === 1 && (
          <Select
            label="Family parameter column (optional)"
            description="Distinct values of this column become the curve family — a 2-D function name(x, param)."
            data={[{ value: NONE, label: '(none — 1-D function)' }, ...familyOptions]}
            value={familyVar}
            onChange={(v) => setFamilyVar(v ?? NONE)}
            allowDeselect={false}
          />
        )}

        {picks.map((p) => {
          const dup = duplicateNames.has(p.name.trim().toLowerCase())
          const { usedRows, skippedRows, decimated } = p.result
          const signature = family
            ? `${p.name.trim() || 'name'}(${p.result.spec.argName}, ${p.result.spec.paramName})`
            : `${p.name.trim() || 'name'}(${p.result.spec.argName})`
          return (
            <Stack key={p.yVar} gap={4}>
              <TextInput
                label={picks.length > 1 ? `Function name for ${p.yVar}` : 'Function name'}
                value={nameOf(p.yVar)}
                onChange={(e) => {
                  const value = e.currentTarget.value
                  setNameEdits((prev) => ({ ...prev, [p.yVar]: value }))
                }}
                error={p.check.error ?? (dup ? 'Duplicate name among the picked columns.' : null)}
                spellCheck={false}
                styles={{ input: { fontFamily: 'var(--mantine-font-family-monospace)' } }}
              />
              <Text size="xs" c={usedRows === 0 ? 'red' : 'dimmed'}>
                {usedRows === 0
                  ? 'No usable rows — every row is failed or incomplete for these columns.'
                  : `${usedRows} row${usedRows === 1 ? '' : 's'} used · ${skippedRows} skipped (failed or incomplete)` +
                    (family ? ` · ${p.result.spec.columns.length} curves` : '') +
                    (decimated ? ` · thinned to ${TABLE_MAX_ROWS} rows (table row cap)` : '')}
              </Text>
              {usedRows > 0 && (
                <Text size="xs" c="dimmed">
                  Use in equations: <Code>U = {signature}</Code>
                </Text>
              )}
              <FunctionNameHints name={p.name} check={p.check} />
            </Stack>
          )
        })}

        {picks.length === 0 && (
          <Text size="xs" c="dimmed">
            Pick at least one Y column.
          </Text>
        )}

        <FunctionPrecedenceNote />

        <Group justify="flex-end" mt="xs">
          <Button variant="default" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={create} disabled={invalid} color={anyReplace ? 'yellow' : undefined}>
            {anyReplace
              ? 'Create (replace existing)'
              : picks.length > 1
                ? `Create ${picks.length} functions`
                : 'Create function'}
          </Button>
        </Group>
      </Stack>
    </Modal>
  )
}
