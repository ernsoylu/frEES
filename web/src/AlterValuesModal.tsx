import { useState } from 'react'
import {
  Button,
  Group,
  Modal,
  SegmentedControl,
  Stack,
  Text,
  TextInput,
} from '@mantine/core'

interface Props {
  variable: string
  rowCount: number
  initialFirst: string
  initialLast: string
  onApply: (values: number[]) => void
  onClose: () => void
}

/**
 * "Alter Values": fills a parametric-table column from first to last
 * row with a linear or logarithmic progression (spreadsheet-style fill).
 */
export default function AlterValuesModal({
  variable,
  rowCount,
  initialFirst,
  initialLast,
  onApply,
  onClose,
}: Readonly<Props>) {
  const [first, setFirst] = useState(initialFirst)
  const [last, setLast] = useState(initialLast)
  const [mode, setMode] = useState<'linear' | 'log'>('linear')
  const [error, setError] = useState<string | null>(null)

  function apply() {
    const a = Number(first)
    const b = Number(last)
    if (first.trim() === '' || !Number.isFinite(a)) {
      setError('First value must be a number.')
      return
    }
    if (last.trim() === '' || !Number.isFinite(b)) {
      setError('Last value must be a number.')
      return
    }
    if (mode === 'log' && (a === 0 || b === 0 || a * b < 0)) {
      setError('Logarithmic fill requires nonzero values with the same sign.')
      return
    }

    const values: number[] = []
    for (let i = 0; i < rowCount; i++) {
      const t = rowCount === 1 ? 0 : i / (rowCount - 1)
      const value = mode === 'linear' ? a + (b - a) * t : a * Math.pow(b / a, t)
      values.push(value)
    }
    onApply(values)
  }

  return (
    <Modal opened onClose={onClose} title={`Alter Values — ${variable}`} centered size="lg">
      <Text size="sm" c="dimmed" mb="md">
        Fills all {rowCount} rows of <strong>{variable}</strong> from the first
        to the last value.
      </Text>

      <Stack gap="sm">
        <SegmentedControl
          fullWidth
          value={mode}
          onChange={(v) => {
            setMode(v as 'linear' | 'log')
            setError(null)
          }}
          data={[
            { value: 'linear', label: 'Linear' },
            { value: 'log', label: 'Logarithmic' },
          ]}
        />
        <TextInput
          label="First row value"
          value={first}
          onChange={(e) => {
            setFirst(e.currentTarget.value)
            setError(null)
          }}
          spellCheck={false}
          styles={{ input: { fontFamily: 'var(--mantine-font-family-monospace)' } }}
        />
        <TextInput
          label="Last row value"
          value={last}
          onChange={(e) => {
            setLast(e.currentTarget.value)
            setError(null)
          }}
          spellCheck={false}
          styles={{ input: { fontFamily: 'var(--mantine-font-family-monospace)' } }}
        />
      </Stack>

      {error && (
        <Text c="red" size="sm" mt="sm">
          {error}
        </Text>
      )}

      <Group justify="flex-end" mt="lg" gap="xs">
        <Button variant="default" onClick={onClose}>
          Cancel
        </Button>
        <Button onClick={apply}>Apply</Button>
      </Group>
    </Modal>
  )
}
