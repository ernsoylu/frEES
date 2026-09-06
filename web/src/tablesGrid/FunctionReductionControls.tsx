import { Group, Radio, Stack, Text, TextInput } from '@mantine/core'
import {
  formatComposeCounts,
  type ComposeResult,
  type ReductionChoice,
} from './composeTables'
import { TABLE_MAX_ROWS } from './tableGridModel'

interface Props {
  result: ComposeResult
  reduction: ReductionChoice | null
  onReduction: (value: ReductionChoice | null) => void
  xMin: string
  xMax: string
  onRange: (min: string, max: string) => void
}

export default function FunctionReductionControls({
  result,
  reduction,
  onReduction,
  xMin,
  xMax,
  onRange,
}: Readonly<Props>) {
  const overCap = result.uniqueCount > TABLE_MAX_ROWS || result.needsReduction
  const peak = result.droppedPeak
  return (
    <Stack gap={6}>
      <Text size="xs" c="dimmed">
        {formatComposeCounts(result)}
      </Text>
      {overCap && (
        <>
          <Text size="xs" c="orange">
            {result.uniqueCount.toLocaleString()} unique points exceed the{' '}
            {TABLE_MAX_ROWS.toLocaleString()}-row function-table cap
            {peak
              ? ` — uniform thinning would drop a peak of ${peak.y} at x=${peak.x}`
              : ''}
            . Trim the range, accept the approximation, or cancel.
          </Text>
          <Radio.Group
            value={reduction ?? ''}
            onChange={(v) => onReduction(v === '' ? null : (v as ReductionChoice))}
          >
            <Stack gap={6}>
              <Radio value="trim" label="Trim x range so the remaining points fit" />
              <Radio
                value="decimate"
                label={`Accept uniform thinning to ${TABLE_MAX_ROWS.toLocaleString()} rows`}
              />
            </Stack>
          </Radio.Group>
          {reduction === 'trim' && (
            <Group grow>
              <TextInput
                label="x min"
                value={xMin}
                onChange={(e) => onRange(e.currentTarget.value, xMax)}
                spellCheck={false}
              />
              <TextInput
                label="x max"
                value={xMax}
                onChange={(e) => onRange(xMin, e.currentTarget.value)}
                spellCheck={false}
              />
            </Group>
          )}
        </>
      )}
    </Stack>
  )
}
