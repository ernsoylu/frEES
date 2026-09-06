import { useState, useMemo, useEffect } from 'react'
import {
  Autocomplete,
  Badge,
  Box,
  Button,
  Code,
  Divider,
  Grid,
  Group,
  Modal,
  Paper,
  ScrollArea,
  Select,
  Stack,
  Text,
  TextInput,
  Title,
  UnstyledButton,
} from '@mantine/core'
import { COMPONENT_CATALOG, ComponentSpec, ComponentParam } from './componentCatalog'
import { COMPONENT_OVERRIDES, ComponentSection } from './componentOverrides'
import {
  generateComponentText,
  suggestInstanceName,
  instanceNameError,
  missingRequiredParams,
  activeParams,
  selectedVariant,
  assembleBlock,
  paramValueError,
  inactiveDraftParams,
  KNOWN_FLUIDS,
  ParamValues,
} from './componentText'
import { isUaParam, UaResult } from './uaCorrelation'
import UaBuilderModal from './UaBuilderModal'
import MapBuilderModal from './MapBuilderModal'

interface Props {
  opened: boolean
  onClose: () => void
  /** Insert the generated component block onto a fresh line in the editor. */
  onInsert: (block: string) => void
  /** Instance names already in the document (case-insensitive uniqueness). */
  existingNames?: readonly string[]
}

// Friendly labels for the library keys parsed from `category: Component (<lib>)`.
const LIBRARY_LABELS: Record<string, string> = {
  ac: 'Air-Conditioning / Refrigeration',
  control: 'Control',
  electrical: 'Electrical / Battery',
  fluid: 'Thermofluid',
  heat: 'Heat Transfer',
  hydraulic: 'Hydraulic (Oil)',
  liquid: 'Liquid Cooling',
  mechanical: 'Mechanical',
  moistair: 'Moist Air (HVAC)',
  pneumatic: 'Pneumatic (Gas)',
  powertrain: 'Powertrain',
  twophase: 'Two-Phase',
}
const libraryLabel = (lib: string) => LIBRARY_LABELS[lib] ?? lib

// domain$ is an internal connector-type guard with a std-library default — not a
// user-facing field. It is omitted from the form (and emitted as its default).
const HIDDEN_PARAMS = new Set(['domain$'])

// Group a component's params into labeled sections: use the override's sections
// (filtered to params that exist), then sweep any remaining params into "Other".
function buildSections(spec: ComponentSpec): ComponentSection[] {
  const visible = spec.params.filter((p) => !HIDDEN_PARAMS.has(p.name))
  const override = COMPONENT_OVERRIDES[spec.type]
  if (!override?.sections) {
    return [{ title: 'Parameters', params: visible.map((p) => p.name) }]
  }
  const claimed = new Set<string>()
  const sections: ComponentSection[] = []
  for (const s of override.sections) {
    const params = s.params.filter((name) => visible.some((p) => p.name === name))
    params.forEach((name) => claimed.add(name))
    if (params.length) sections.push({ title: s.title, params })
  }
  const leftover = visible.filter((p) => !claimed.has(p.name)).map((p) => p.name)
  if (leftover.length) sections.push({ title: 'Other', params: leftover })
  return sections
}

export default function ComponentWizardModal({ opened, onClose, onInsert, existingNames = [] }: Readonly<Props>) {
  const [search, setSearch] = useState('')
  const [library, setLibrary] = useState<string | null>(null)
  const [selectedType, setSelectedType] = useState<string | null>(null)
  const [instanceName, setInstanceName] = useState('')
  const [values, setValues] = useState<ParamValues>({})
  // Preamble blocks (correlation helpers / TABLE blocks) keyed by the param they
  // feed; prepended above the component line on insert.
  const [preamble, setPreamble] = useState<Record<string, string>>({})
  const [uaBuilderFor, setUaBuilderFor] = useState<string | null>(null)
  const [mapBuilderFor, setMapBuilderFor] = useState<string | null>(null)

  const libraries = useMemo(() => {
    const libs = Array.from(new Set(COMPONENT_CATALOG.map((c) => c.library))).sort((a, b) =>
      a.localeCompare(b),
    )
    return libs.map((lib) => ({ value: lib, label: libraryLabel(lib) }))
  }, [])

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase()
    return COMPONENT_CATALOG.filter((c) => {
      if (library && c.library !== library) return false
      if (!q) return true
      return (
        c.type.toLowerCase().includes(q) ||
        c.summary.toLowerCase().includes(q) ||
        c.tags.some((t) => t.toLowerCase().includes(q))
      )
    })
  }, [search, library])

  const spec = useMemo(
    () => COMPONENT_CATALOG.find((c) => c.type === selectedType) ?? null,
    [selectedType],
  )

  // Reset the form whenever a different component is selected.
  useEffect(() => {
    if (!spec) return
    setInstanceName(suggestInstanceName(spec.type, existingNames))
    setValues({})
    setPreamble({})
  }, [spec, existingNames])

  const preview = spec ? generateComponentText(spec, instanceName, values) : ''
  const missing = spec ? missingRequiredParams(spec, values) : []
  const nameErr = instanceNameError(instanceName, existingNames)
  const fieldErrors = spec
    ? Object.fromEntries(
        activeParams(spec, values)
          .map((p) => [p.name, paramValueError(p, values[p.name] ?? '')] as const)
          .filter(([, err]) => err),
      )
    : {}
  const drafts = spec ? inactiveDraftParams(spec, values) : []
  const canAdd = !!spec && !nameErr && missing.length === 0 && Object.keys(fieldErrors).length === 0

  // Names of params active for the current variant (drives show/hide + which
  // preamble blocks are emitted).
  const activeNames = useMemo(
    () => new Set(spec ? activeParams(spec, values).map((p) => p.name) : []),
    [spec, values],
  )

  // The full block: preamble for active params (in param order) + the component line.
  const fullBlock = useMemo(() => {
    if (!spec) return ''
    const pre = spec.params.filter((p) => activeNames.has(p.name) && preamble[p.name]).map((p) => preamble[p.name])
    return assembleBlock(pre, generateComponentText(spec, instanceName, values))
  }, [spec, activeNames, preamble, instanceName, values])

  const setParam = (name: string, value: string) =>
    setValues((prev) => ({ ...prev, [name]: value }))

  const handleAdd = () => {
    if (!spec || !canAdd) return
    onInsert(fullBlock)
    onClose()
  }

  const sections = spec ? buildSections(spec) : []
  const override = spec ? COMPONENT_OVERRIDES[spec.type] : undefined

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title="Insert a Component"
      size="calc(100vw - 40px)"
      centered
    >
      <Grid gap="md">
        {/* ── Left: browse / pick a component ───────────────────────────── */}
        <Grid.Col span={{ base: 12, md: 4 }}>
          <Stack gap="xs">
            <Select
              placeholder="All libraries"
              data={libraries}
              value={library}
              onChange={setLibrary}
              clearable
              searchable
            />
            <TextInput
              placeholder="Search by name, tag, or description…"
              value={search}
              onChange={(e) => setSearch(e.currentTarget.value)}
            />
            <Text size="xs" c="dimmed">
              {filtered.length} component{filtered.length === 1 ? '' : 's'}
            </Text>
            <ScrollArea h="calc(100vh - 280px)" type="auto">
              <Stack gap={4} pr="sm">
                {filtered.map((c) => (
                  <UnstyledButton
                    key={c.type}
                    onClick={() => setSelectedType(c.type)}
                  >
                    <Paper
                      withBorder
                      p="xs"
                      radius="md"
                      bg={c.type === selectedType ? 'var(--mantine-color-blue-light)' : undefined}
                    >
                      <Group justify="space-between" wrap="nowrap" align="flex-start">
                        <Text fw={600} size="sm">{c.type}</Text>
                        <Badge variant="light" size="xs" style={{ flexShrink: 0 }}>
                          {libraryLabel(c.library)}
                        </Badge>
                      </Group>
                      <Text size="xs" c="dimmed" lineClamp={2}>{c.summary}</Text>
                    </Paper>
                  </UnstyledButton>
                ))}
              </Stack>
            </ScrollArea>
          </Stack>
        </Grid.Col>

        {/* ── Right: configure the selected component ───────────────────── */}
        <Grid.Col span={{ base: 12, md: 8 }}>
          {!spec ? (
            <Box pt="xl" ta="center">
              <Text c="dimmed">Select a component from the list to configure it.</Text>
            </Box>
          ) : (
            <Stack gap="sm">
              <Group justify="space-between" align="flex-start" wrap="nowrap">
                <Box>
                  <Title order={4}>{spec.type}</Title>
                  <Text size="sm" c="dimmed">{spec.summary}</Text>
                  {override?.notes && (
                    <Text size="xs" c="dimmed" mt={4}>{override.notes}</Text>
                  )}
                </Box>
                <Badge variant="light" style={{ flexShrink: 0 }}>{libraryLabel(spec.library)}</Badge>
              </Group>

              {/* Ports — read-only; the user wires these with connect(...). */}
              {spec.ports.length > 0 && (
                <Box>
                  <Text size="xs" fw={600} c="dimmed" mb={4}>
                    Ports — wire with connect(…)
                  </Text>
                  <Group gap={6}>
                    {spec.ports.map((port) => (
                      <Badge key={port} variant="outline" size="sm">{port}</Badge>
                    ))}
                  </Group>
                </Box>
              )}

              {/* Illustration placeholder (real art is a follow-up). */}
              {override?.illustration && (
                <Paper
                  withBorder
                  radius="md"
                  h={120}
                  bg="var(--mantine-color-dark-6)"
                  style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}
                >
                  <Text size="xs" c="dimmed">▭ illustration: {override.illustration}</Text>
                </Paper>
              )}

              <Divider />

              <TextInput
                label="Instance name"
                description="A unique identifier for this component instance."
                value={instanceName}
                onChange={(e) => setInstanceName(e.currentTarget.value)}
                error={nameErr ?? undefined}
                w={260}
              />

              {sections.map((section) => {
                const names = section.params.filter((name) => activeNames.has(name))
                if (names.length === 0) return null
                return (
                  <Box key={section.title}>
                    <Text size="xs" fw={700} tt="uppercase" c="dimmed" mb={6}>
                      {section.title}
                    </Text>
                    <Grid gap="xs">
                      {names.map((name) => {
                        const p = spec.params.find((x) => x.name === name)
                        if (!p) return null
                        return (
                          <Grid.Col key={name} span={{ base: 12, sm: 6 }}>
                            <ParamField
                              param={p}
                              value={values[name] ?? ''}
                              onChange={(v) => setParam(name, v)}
                              error={fieldErrors[name] || undefined}
                              defaultVariant={p.isSelector ? selectedVariant(spec, values) : null}
                              onBuildUa={isUaParam(p.name, p.unit) ? () => setUaBuilderFor(name) : undefined}
                              onBuildMap={p.isMap ? () => setMapBuilderFor(name) : undefined}
                            />
                          </Grid.Col>
                        )
                      })}
                    </Grid>
                  </Box>
                )
              })}

              <Divider />

              {drafts.length > 0 && (
                <Text size="xs" c="dimmed">
                  Inactive for this variant (kept as a draft, omitted from generated code): {drafts.join(', ')}
                </Text>
              )}

              <Box>
                <Text size="xs" fw={600} c="dimmed" mb={4}>Preview</Text>
                <Code block>{fullBlock || preview}</Code>
              </Box>

              <Group justify="flex-end" gap="sm">
                <Button variant="default" onClick={onClose}>Cancel</Button>
                <Button onClick={handleAdd} disabled={!canAdd}>Add to System</Button>
              </Group>
            </Stack>
          )}
        </Grid.Col>
      </Grid>

      {uaBuilderFor && (
        <UaBuilderModal
          opened={!!uaBuilderFor}
          onClose={() => setUaBuilderFor(null)}
          instanceName={instanceName}
          onConfirm={(r: UaResult) => {
            setParam(uaBuilderFor, r.uaVar)
            setPreamble((prev) => ({ ...prev, [uaBuilderFor]: r.lines.join('\n') }))
          }}
        />
      )}
      {mapBuilderFor && (
        <MapBuilderModal
          opened={!!mapBuilderFor}
          onClose={() => setMapBuilderFor(null)}
          defaultName={`${(instanceName || 'map').toLowerCase()}Map`}
          onConfirm={(r) => {
            setParam(mapBuilderFor, r.tableName)
            setPreamble((prev) => ({ ...prev, [mapBuilderFor]: r.block }))
          }}
        />
      )}
    </Modal>
  )
}

// A single parameter input. Selector params (model$) render a clearable Select of
// their variant values; everything else is a text field (values may be numbers,
// strings, or references to other solver variables, e.g. UA=UA_chl_r — so a
// numeric input would be too restrictive). The unit, when known, is shown in the
// label and appended to the emitted text.
function ParamField({
  param,
  value,
  onChange,
  error,
  defaultVariant,
  onBuildUa,
  onBuildMap,
}: Readonly<{
  param: ComponentParam
  value: string
  onChange: (v: string) => void
  error?: string
  defaultVariant?: string | null
  onBuildUa?: () => void
  onBuildMap?: () => void
}>) {
  const label = (
    <Group gap={4} component="span">
      <Text component="span" size="sm">{param.name}</Text>
      {param.unit && <Text component="span" size="xs" c="dimmed">[{param.unit}]</Text>}
      {!param.required && <Text component="span" size="xs" c="dimmed">(optional)</Text>}
    </Group>
  )

  if (param.isSelector && param.values.length > 0) {
    const declared = (param.defaultValue || defaultVariant || '').trim()
    return (
      <Select
        label={label}
        description={param.description || undefined}
        data={param.values}
        value={value || null}
        onChange={(v) => onChange(v ?? '')}
        placeholder={declared ? `default (${declared})` : 'default'}
        clearable
        error={error}
      />
    )
  }

  if (param.isString && /fluid\$/i.test(param.name)) {
    return (
      <Box>
        <Autocomplete
          label={label}
          description={param.description || undefined}
          value={value}
          onChange={onChange}
          data={KNOWN_FLUIDS}
          placeholder="Water, R134a, INCOMP::MEG[0.50], …"
          error={error}
        />
      </Box>
    )
  }

  const builder = onBuildUa
    ? <Button size="compact-xs" variant="light" onClick={onBuildUa}>ƒ Compute UA…</Button>
    : onBuildMap
      ? <Button size="compact-xs" variant="light" onClick={onBuildMap}>Build a map…</Button>
      : null

  return (
    <Box>
      <TextInput
        label={label}
        description={param.description || undefined}
        value={value}
        onChange={(e) => onChange(e.currentTarget.value)}
        placeholder={param.isString ? 'name' : 'value or variable'}
        error={error}
      />
      {builder && <Group mt={4}>{builder}</Group>}
    </Box>
  )
}
