import { useMemo, useState } from 'react'
import {
  ActionIcon,
  Button,
  Checkbox,
  Group,
  Modal,
  Stack,
  Text,
  TextInput,
} from '@mantine/core'
import { IconSearch, IconX } from '@tabler/icons-react'

interface Props {
  variables: string[]
  selected: string[]
  onSave: (selected: string[]) => void
  onClose: () => void
}

export default function ConfigureTableModal({
  variables,
  selected,
  onSave,
  onClose,
}: Readonly<Props>) {
  const [local, setLocal] = useState<string[]>(selected)
  const [query, setQuery] = useState('')
  const [selectedFirst, setSelectedFirst] = useState(false)

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    let list = q ? variables.filter((v) => v.toLowerCase().includes(q)) : [...variables]
    if (selectedFirst) {
      list.sort((a, b) => {
        const aSel = local.includes(a)
        const bSel = local.includes(b)
        if (aSel === bSel) return 0
        return aSel ? -1 : 1
      })
    }
    return list
  }, [variables, query, selectedFirst, local])

  function toggle(name: string) {
    setLocal((prev) =>
      prev.includes(name)
        ? prev.filter((v) => v !== name)
        : [...prev, name]
    )
  }

  function selectAllVisible() {
    setLocal((prev) => {
      const set = new Set(prev)
      for (const v of filtered) set.add(v)
      return Array.from(set)
    })
  }

  function clearAllVisible() {
    setLocal((prev) => {
      const visibleSet = new Set(filtered)
      return prev.filter((v) => !visibleSet.has(v))
    })
  }

  function save() {
    onSave(local)
  }

  return (
    <Modal opened onClose={onClose} title="Configure Table Columns" centered size="lg">
      <Text size="sm" c="dimmed" mb="xs">
        Select which variables you want to include as columns in the parametric table.
      </Text>

      <Stack gap="xs">
        <TextInput
          placeholder="Search variables by name or component..."
          leftSection={<IconSearch size={14} />}
          rightSection={
            query ? (
              <ActionIcon size="xs" variant="subtle" onClick={() => setQuery('')} aria-label="Clear search">
                <IconX size={12} />
              </ActionIcon>
            ) : null
          }
          value={query}
          onChange={(e) => setQuery(e.currentTarget.value)}
          size="xs"
        />

        <Group justify="space-between" align="center">
          <Group gap="xs">
            <Button size="compact-xs" variant="light" onClick={selectAllVisible} disabled={filtered.length === 0}>
              Select all visible
            </Button>
            <Button size="compact-xs" variant="subtle" onClick={clearAllVisible} disabled={filtered.length === 0}>
              Clear visible
            </Button>
            <Checkbox
              size="xs"
              label="Selected first"
              checked={selectedFirst}
              onChange={(e) => setSelectedFirst(e.currentTarget.checked)}
            />
          </Group>
          <Text size="xs" c="dimmed">
            {local.length} of {variables.length} selected
            {query && ` (${filtered.length} matching)`}
          </Text>
        </Group>

        {variables.length === 0 ? (
          <Text c="dimmed" size="sm" style={{ fontStyle: 'italic' }}>
            No variables detected yet — please check your equations first to find variables.
          </Text>
        ) : filtered.length === 0 ? (
          <Text c="dimmed" size="sm" style={{ fontStyle: 'italic' }}>
            No variables match "{query}".
          </Text>
        ) : (
          <Stack gap="xs" style={{ maxHeight: 300, overflowY: 'auto' }} p={4}>
            {filtered.map((name) => (
              <Checkbox
                key={name}
                label={name}
                checked={local.includes(name)}
                onChange={() => toggle(name)}
              />
            ))}
          </Stack>
        )}
      </Stack>

      <Group justify="flex-end" mt="xl">
        <Button variant="default" onClick={onClose}>
          Cancel
        </Button>
        <Button onClick={save} disabled={variables.length === 0}>
          Save
        </Button>
      </Group>
    </Modal>
  )
}

