import { useState } from 'react'
import {
  ActionIcon,
  Button,
  Checkbox,
  ColorInput,
  Divider,
  Group,
  Modal,
  MultiSelect,
  NumberInput,
  SegmentedControl,
  Select,
  Stack,
  Text,
  TextInput,
} from '@mantine/core'
import { IconTrash } from '@tabler/icons-react'
import {
  DIAGRAM_TYPES,
  PlotFormat,
  PlotKind,
  PlotSpec,
  PropertyConfig,
  PsychroConfig,
  XYConfig,
  ChartType,
  diagramAxes,
  newPlotSpec,
} from './types'
import { defaultUnitId, isOffsetUnit, unitIdsFor } from './units'
import { StateTableDto } from '../api'
import type { TableSpec } from '../tables'
import { displayVar, varOptions } from '../varDisplay'

interface Props {
  /** Existing spec to edit, or null to create a new plot. */
  spec: PlotSpec | null
  /** Plot kinds this window offers (Plots: xy; Thermodynamics: property/psychro). */
  allowedKinds: PlotKind[]
  defaultName: string
  fluids: string[]
  tableVars: string[]
  tables?: TableSpec[]
  occupiedNames?: string[]
  /** Seed for a new X-Y plot (e.g. opened from a table's column selection):
   * pre-fills the x-axis variable and y-axis variables. */
  initialXy?: { xVar: string; yVars: string[]; tableId?: string }
  hasStates: boolean
  /** Declared STATE TABLE blocks, so property/psychro plots can overlay one
   * specific circuit's states (and adopt its fluid). */
  stateTables?: StateTableDto[]
  onSave: (spec: PlotSpec) => void
  onClose: () => void
}

const CHART_TYPE_OPTIONS = [
  { value: 'line', label: 'Line chart' },
  { value: 'bar', label: 'Bar chart' },
  { value: 'pie', label: 'Pie chart' },
  { value: 'histogram', label: 'Histogram' },
  { value: 'scatter', label: 'Scatter (bubble)' },
  { value: 'surface3d', label: 'Triangulated 3D mesh' },
]

function XYSection({
  config,
  tableVars,
  onChange,
}: Readonly<{
  config: XYConfig
  tableVars: string[]
  onChange: (config: XYConfig) => void
}>) {
  const chartType = config.chartType ?? 'line'
  // Options show demangled labels (brg$port$t → brg.port.t) but keep the raw
  // name as the value so the selection still keys into the table data.
  const varOpts = varOptions(tableVars)

  return (
    <Stack gap="xs">
      <Group grow>
        <Select
          label="Chart Type"
          size="xs"
          data={CHART_TYPE_OPTIONS}
          value={chartType}
          onChange={(val) => {
            onChange({
              ...config,
              chartType: (val as ChartType) ?? 'line',
              zVar: val === 'surface3d' ? config.zVar : null,
              sizeVar: val === 'scatter' ? config.sizeVar : null,
            })
          }}
        />
        <Select
          label={chartType === 'histogram' ? 'Variable (uses Y-axis)' : 'X-axis variable'}
          size="xs"
          data={varOpts}
          value={config.xVar}
          onChange={(xVar) => onChange({ ...config, xVar })}
          searchable
          disabled={chartType === 'histogram'}
        />
      </Group>

      <Group grow>
        <MultiSelect
          label={chartType === 'pie' ? 'Value variable (uses first Y)' : 'Y-axis variables'}
          size="xs"
          data={varOpts}
          value={config.yVars}
          onChange={(yVars) => onChange({ ...config, yVars })}
          maxValues={chartType === 'pie' ? 1 : undefined}
          searchable
        />
        {chartType === 'surface3d' && (
          <Select
            label="Z-axis variable"
            size="xs"
            data={varOpts}
            value={config.zVar ?? null}
            onChange={(zVar) => onChange({ ...config, zVar })}
            searchable
          />
        )}
        {chartType === 'scatter' && (
          <Select
            label="Bubble size variable (optional)"
            size="xs"
            data={varOpts}
            value={config.sizeVar ?? null}
            onChange={(sizeVar) => onChange({ ...config, sizeVar })}
            searchable
            clearable
          />
        )}
      </Group>
      {(chartType === 'line' || chartType === 'bar' || chartType === 'scatter') && (
        <MultiSelect
          label="Right Y-axis variables (optional, dual-Y)"
          size="xs"
          data={varOptions(tableVars.filter((v) => !config.yVars.includes(v)))}
          value={config.y2Vars ?? []}
          onChange={(y2Vars) => onChange({ ...config, y2Vars })}
          searchable
          clearable
        />
      )}
    </Stack>
  )
}

function PropertySection({
  config,
  fluids,
  hasStates,
  stateTables = [],
  onChange,
}: Readonly<{
  config: PropertyConfig
  fluids: string[]
  hasStates: boolean
  stateTables?: StateTableDto[]
  onChange: (config: PropertyConfig) => void
}>) {
  return (
    <Stack gap="xs">
      <Group grow>
        <Select
          label="Fluid"
          size="xs"
          data={fluids}
          value={config.fluid}
          onChange={(fluid) => fluid && onChange({ ...config, fluid })}
          searchable
        />
        <Select
          label="Diagram"
          size="xs"
          data={DIAGRAM_TYPES}
          value={config.diagram}
          onChange={(diagram) => diagram && onChange({ ...config, diagram })}
        />
      </Group>
      <Group gap="md">
        <Checkbox
          label="Quality lines"
          size="xs"
          checked={config.quality}
          onChange={(e) => onChange({ ...config, quality: e.currentTarget.checked })}
        />
        <Checkbox
          label="Isolines (P/T/s)"
          size="xs"
          checked={config.isolines}
          onChange={(e) => onChange({ ...config, isolines: e.currentTarget.checked })}
        />
      </Group>
      <Divider label="State points" labelPosition="left" />
      {!hasStates && stateTables.length === 0 && (
        <Text size="xs" c="dimmed">
          No states detected. Declare a STATE TABLE block (with FLUID = ...) or
          solve a system with numbered properties (h1, s1, T[2], ...); states
          appear here and in the States tab.
        </Text>
      )}
      {stateTables.length > 0 && (
        <Select
          label="State table (circuit)"
          description="Overlay one circuit's states; selecting it also sets the fluid."
          size="xs"
          data={[
            { value: '', label: 'All detected states' },
            ...stateTables.map((s) => ({
              value: s.name,
              label: s.fluid ? `${s.name} — ${s.fluid}` : s.name,
            })),
          ]}
          value={config.stateTable ?? ''}
          onChange={(name) => {
            const picked = stateTables.find((s) => s.name === name)
            onChange({
              ...config,
              stateTable: name ? name : null,
              fluid: picked?.fluid ?? config.fluid,
            })
          }}
          allowDeselect={false}
        />
      )}
      <Group gap="md">
        <Checkbox
          label="Overlay states"
          size="xs"
          checked={config.overlayStates}
          onChange={(e) =>
            onChange({ ...config, overlayStates: e.currentTarget.checked })
          }
        />
        <Checkbox
          label="Connect in order"
          size="xs"
          checked={config.connectStates}
          disabled={!config.overlayStates}
          onChange={(e) =>
            onChange({ ...config, connectStates: e.currentTarget.checked })
          }
        />
        <Checkbox
          label="Close cycle"
          size="xs"
          checked={config.closeCycle}
          disabled={!config.overlayStates || !config.connectStates}
          onChange={(e) => onChange({ ...config, closeCycle: e.currentTarget.checked })}
        />
      </Group>
    </Stack>
  )
}

function PsychroSection({
  config,
  stateTables = [],
  onChange,
}: Readonly<{
  config: PsychroConfig
  stateTables?: StateTableDto[]
  onChange: (config: PsychroConfig) => void
}>) {
  return (
    <Stack gap="xs">
      <Group grow>
        <NumberInput
          label="Pressure [kPa]"
          size="xs"
          value={config.pressureKPa}
          onChange={(v) =>
            onChange({ ...config, pressureKPa: typeof v === 'number' ? v : 101.325 })
          }
          min={2}
        />
        <NumberInput
          label="T min [°C]"
          size="xs"
          value={config.tMinC}
          onChange={(v) =>
            onChange({ ...config, tMinC: typeof v === 'number' ? v : 0 })
          }
        />
        <NumberInput
          label="T max [°C]"
          size="xs"
          value={config.tMaxC}
          onChange={(v) =>
            onChange({ ...config, tMaxC: typeof v === 'number' ? v : 50 })
          }
        />
      </Group>
      <Group gap="md">
        <Checkbox
          label="Wet-bulb lines"
          size="xs"
          checked={config.wetBulb}
          onChange={(e) => onChange({ ...config, wetBulb: e.currentTarget.checked })}
        />
        <Checkbox
          label="Enthalpy lines"
          size="xs"
          checked={config.enthalpy}
          onChange={(e) => onChange({ ...config, enthalpy: e.currentTarget.checked })}
        />
        <Checkbox
          label="Specific volume lines"
          size="xs"
          checked={config.volume}
          onChange={(e) => onChange({ ...config, volume: e.currentTarget.checked })}
        />
      </Group>
      <Divider label="State points" labelPosition="left" />
      <Text size="xs" c="dimmed">
        States with a dry-bulb temperature and a humidity ratio (e.g. T1 and
        w1) are drawn on the chart.
      </Text>
      {stateTables.length > 0 && (
        <Select
          label="State table (circuit)"
          description="Overlay only this circuit's states."
          size="xs"
          data={[
            { value: '', label: 'All detected states' },
            ...stateTables.map((s) => ({
              value: s.name,
              label: s.fluid ? `${s.name} — ${s.fluid}` : s.name,
            })),
          ]}
          value={config.stateTable ?? ''}
          onChange={(name) => onChange({ ...config, stateTable: name ? name : null })}
          allowDeselect={false}
        />
      )}
      <Group gap="md">
        <Checkbox
          label="Overlay states"
          size="xs"
          checked={config.overlayStates}
          onChange={(e) =>
            onChange({ ...config, overlayStates: e.currentTarget.checked })
          }
        />
        <Checkbox
          label="Connect in order"
          size="xs"
          checked={config.connectStates}
          disabled={!config.overlayStates}
          onChange={(e) =>
            onChange({ ...config, connectStates: e.currentTarget.checked })
          }
        />
      </Group>
    </Stack>
  )
}

/** Axis properties whose units can be selected for the current plot. */
function axisProperties(spec: PlotSpec): { x: string; y: string } | null {
  if (spec.kind === 'property') {
    return diagramAxes(spec.property.diagram)
  }
  if (spec.kind === 'psychro') {
    return { x: 'T', y: 'w' }
  }
  return null
}

function XyLineColors({
  spec,
  format,
  onChange,
}: Readonly<{
  spec: PlotSpec
  format: PlotFormat
  onChange: (format: PlotFormat) => void
}>) {
  if (spec.xy.yVars.length === 0) {
    return (
      <Text size="xs" c="dimmed">
        Select Y-axis variables first to configure their colors.
      </Text>
    )
  }
  return (
    <Group gap="xs">
      {spec.xy.yVars.map((yVar) => (
        <ColorInput
          key={yVar}
          label={`Color for ${displayVar(yVar)}`}
          size="xs"
          style={{ flex: '1 1 120px' }}
          value={format.lineColors?.[yVar] ?? '#228be6'}
          onChange={(color) => {
            const lineColors = { ...format.lineColors, [yVar]: color }
            onChange({ ...format, lineColors })
          }}
        />
      ))}
    </Group>
  )
}

function XyTraceStyles({
  spec,
  format,
  onChange,
}: Readonly<{
  spec: PlotSpec
  format: PlotFormat
  onChange: (format: PlotFormat) => void
}>) {
  const yVars = [...spec.xy.yVars, ...(spec.xy.y2Vars ?? [])]
  if (yVars.length === 0) return null

  const LINE_DASH_OPTIONS = [
    { value: 'solid', label: 'Solid' },
    { value: 'dash', label: 'Dashed' },
    { value: 'dot', label: 'Dotted' },
    { value: 'dashdot', label: 'Dash-Dot' },
  ]

  const MARKER_OPTIONS = [
    { value: '', label: 'None' },
    { value: 'circle', label: 'Circle' },
    { value: 'square', label: 'Square' },
    { value: 'diamond', label: 'Diamond' },
    { value: 'triangle-up', label: 'Triangle' },
    { value: 'cross', label: 'Cross' },
    { value: 'x', label: 'X' },
  ]

  return (
    <Stack gap="xs">
      {yVars.map((yVar) => {
        const style = format.traceStyles?.[yVar]
        return (
          <Group key={yVar} grow align="flex-end">
            <Text size="xs" fw={500} style={{ flex: '0 0 110px' }}>
              {displayVar(yVar)}
            </Text>
            <Select
              label="Line dash"
              size="xs"
              data={LINE_DASH_OPTIONS}
              value={style?.dash ?? 'solid'}
              onChange={(v) => {
                const nextStyles = {
                  ...format.traceStyles,
                  [yVar]: {
                    ...style,
                    dash: (v as 'solid' | 'dash' | 'dot' | 'dashdot') || 'solid',
                  },
                }
                onChange({ ...format, traceStyles: nextStyles })
              }}
            />
            <Select
              label="Marker symbol"
              size="xs"
              data={MARKER_OPTIONS}
              value={style?.markerSymbol ?? ''}
              onChange={(v) => {
                const nextStyles = {
                  ...format.traceStyles,
                  [yVar]: {
                    ...style,
                    markerSymbol: v || undefined,
                  },
                }
                onChange({ ...format, traceStyles: nextStyles })
              }}
            />
          </Group>
        )
      })}
    </Stack>
  )
}

function ReferenceAnnotationsSection({
  format,
  onChange,
}: Readonly<{
  format: PlotFormat
  onChange: (format: PlotFormat) => void
}>) {
  const annotations = format.annotations ?? []

  const addAnnotation = (type: 'hline' | 'vline') => {
    const newAnn: import('./types').ReferenceAnnotation = {
      id: `ann-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
      type,
      value: 0,
      text: type === 'hline' ? 'Threshold' : 'Limit',
      color: type === 'hline' ? '#ff6b6b' : '#4dabf7',
      dash: 'dash',
    }
    onChange({ ...format, annotations: [...annotations, newAnn] })
  }

  const updateAnnotation = (id: string, patch: Partial<import('./types').ReferenceAnnotation>) => {
    onChange({
      ...format,
      annotations: annotations.map((a) => (a.id === id ? { ...a, ...patch } : a)),
    })
  }

  const removeAnnotation = (id: string) => {
    onChange({
      ...format,
      annotations: annotations.filter((a) => a.id !== id),
    })
  }

  return (
    <Stack gap="xs">
      <Group justify="space-between" align="center">
        <Text size="xs" c="dimmed">
          Reference lines (thresholds and event limits)
        </Text>
        <Group gap="xs">
          <Button size="compact-xs" variant="default" onClick={() => addAnnotation('hline')}>
            + Horiz (Y)
          </Button>
          <Button size="compact-xs" variant="default" onClick={() => addAnnotation('vline')}>
            + Vert (X)
          </Button>
        </Group>
      </Group>

      {annotations.map((ann) => (
        <Group key={ann.id} grow align="flex-end" gap="xs">
          <Select
            label="Type"
            size="xs"
            style={{ flex: '0 0 90px' }}
            data={[
              { value: 'hline', label: 'Horiz (Y)' },
              { value: 'vline', label: 'Vert (X)' },
            ]}
            value={ann.type}
            onChange={(v) => updateAnnotation(ann.id, { type: (v as 'hline' | 'vline') ?? 'hline' })}
          />
          <NumberInput
            label="Value"
            size="xs"
            value={ann.value}
            onChange={(v) => updateAnnotation(ann.id, { value: typeof v === 'number' ? v : 0 })}
          />
          <TextInput
            label="Label"
            size="xs"
            value={ann.text ?? ''}
            placeholder="Label text"
            onChange={(e) => updateAnnotation(ann.id, { text: e.currentTarget.value })}
          />
          <ColorInput
            label="Color"
            size="xs"
            style={{ flex: '0 0 100px' }}
            value={ann.color ?? (ann.type === 'hline' ? '#ff6b6b' : '#4dabf7')}
            onChange={(c) => updateAnnotation(ann.id, { color: c })}
          />
          <ActionIcon
            color="red"
            variant="subtle"
            size="sm"
            onClick={() => removeAnnotation(ann.id)}
            title="Remove annotation"
            aria-label="Remove annotation"
          >
            <IconTrash size={14} />
          </ActionIcon>
        </Group>
      ))}
    </Stack>
  )
}

export interface FormatErrors {
  xMin?: string
  xMax?: string
  yMin?: string
  yMax?: string
  xUnit?: string
  yUnit?: string
  xLog?: string
  yLog?: string
}

export function validatePlotFormat(spec: PlotSpec): FormatErrors {
  const errors: FormatErrors = {}
  const { format } = spec
  const axes = axisProperties(spec)
  const isPie = spec.kind === 'xy' && spec.xy.chartType === 'pie'
  if (isPie) return errors

  if (format.xLog) {
    if (format.xMin !== null && format.xMin !== undefined && format.xMin <= 0) {
      errors.xMin = 'Log scale requires a value greater than zero.'
    }
    if (format.xMax !== null && format.xMax !== undefined && format.xMax <= 0) {
      errors.xMax = 'Log scale requires a value greater than zero.'
    }
    if (axes && isOffsetUnit(axes.x, format.xUnit, format.celsius)) {
      errors.xUnit = 'Log scale cannot be used with offset units (°C, °F, psig).'
      errors.xLog = 'Log scale cannot be used with offset units.'
    }
  }

  if (format.yLog) {
    if (format.yMin !== null && format.yMin !== undefined && format.yMin <= 0) {
      errors.yMin = 'Log scale requires a value greater than zero.'
    }
    if (format.yMax !== null && format.yMax !== undefined && format.yMax <= 0) {
      errors.yMax = 'Log scale requires a value greater than zero.'
    }
    if (axes && isOffsetUnit(axes.y, format.yUnit, false)) {
      errors.yUnit = 'Log scale cannot be used with offset units (°C, °F, psig).'
      errors.yLog = 'Log scale cannot be used with offset units.'
    }
  }

  if (
    format.xMin !== null &&
    format.xMax !== null &&
    format.xMin !== undefined &&
    format.xMax !== undefined &&
    format.xMin >= format.xMax
  ) {
    errors.xMin = errors.xMin || 'Min must be less than max.'
  }

  if (
    format.yMin !== null &&
    format.yMax !== null &&
    format.yMin !== undefined &&
    format.yMax !== undefined &&
    format.yMin >= format.yMax
  ) {
    errors.yMin = errors.yMin || 'Min must be less than max.'
  }

  return errors
}

function FormatSection({
  spec,
  errors = {},
  onChange,
}: Readonly<{
  spec: PlotSpec
  errors?: FormatErrors
  onChange: (format: PlotFormat) => void
}>) {
  const format = spec.format
  const axes = axisProperties(spec)
  const isPie = spec.kind === 'xy' && spec.xy.chartType === 'pie'
  const is3D = spec.kind === 'xy' && spec.xy.chartType === 'surface3d'
  const isHistogram = spec.kind === 'xy' && spec.xy.chartType === 'histogram'

  return (
    <Stack gap="xs">
      <Group grow>
        <TextInput
          label="Title"
          size="xs"
          value={format.title}
          placeholder="auto"
          onChange={(e) => onChange({ ...format, title: e.currentTarget.value })}
        />
        <NumberInput
          label="Font size"
          size="xs"
          value={format.fontSize}
          min={8}
          max={24}
          onChange={(v) =>
            onChange({ ...format, fontSize: typeof v === 'number' ? v : 13 })
          }
        />
      </Group>
      {!isPie && (
        <Group grow>
          <TextInput
            label="X-axis label"
            size="xs"
            value={format.xLabel}
            placeholder="auto"
            onChange={(e) => onChange({ ...format, xLabel: e.currentTarget.value })}
          />
          <TextInput
            label="Y-axis label"
            size="xs"
            value={format.yLabel}
            placeholder="auto"
            onChange={(e) => onChange({ ...format, yLabel: e.currentTarget.value })}
          />
          {spec.kind === 'xy' && !is3D && (spec.xy.y2Vars?.length ?? 0) > 0 && (
            <TextInput
              label="Right Y-axis label"
              size="xs"
              value={format.y2Label ?? ''}
              placeholder="auto"
              onChange={(e) => onChange({ ...format, y2Label: e.currentTarget.value })}
            />
          )}
        </Group>
      )}
      {!isPie && (
        <Group grow>
          <NumberInput
            label="X min (auto if empty)"
            size="xs"
            error={errors.xMin}
            value={format.xMin ?? ''}
            onChange={(v) =>
              onChange({ ...format, xMin: typeof v === 'number' ? v : null })
            }
          />
          <NumberInput
            label="X max (auto if empty)"
            size="xs"
            error={errors.xMax}
            value={format.xMax ?? ''}
            onChange={(v) =>
              onChange({ ...format, xMax: typeof v === 'number' ? v : null })
            }
          />
          {!is3D && (
            <NumberInput
              label="X tick interval"
              size="xs"
              value={format.xTick ?? ''}
              min={0}
              onChange={(v) =>
                onChange({ ...format, xTick: typeof v === 'number' && v > 0 ? v : null })
              }
            />
          )}
        </Group>
      )}
      {!isPie && (
        <Group grow>
          <NumberInput
            label="Y min (auto if empty)"
            size="xs"
            error={errors.yMin}
            value={format.yMin ?? ''}
            onChange={(v) =>
              onChange({ ...format, yMin: typeof v === 'number' ? v : null })
            }
          />
          <NumberInput
            label="Y max (auto if empty)"
            size="xs"
            error={errors.yMax}
            value={format.yMax ?? ''}
            onChange={(v) =>
              onChange({ ...format, yMax: typeof v === 'number' ? v : null })
            }
          />
          {!is3D && (
            <NumberInput
              label="Y tick interval"
              size="xs"
              value={format.yTick ?? ''}
              min={0}
              onChange={(v) =>
                onChange({ ...format, yTick: typeof v === 'number' && v > 0 ? v : null })
              }
            />
          )}
        </Group>
      )}
      {!isPie && axes && (
        <Group grow>
          <Select
            label={`X unit (${axes.x})`}
            size="xs"
            error={errors.xUnit}
            data={unitIdsFor(axes.x)}
            value={format.xUnit ?? defaultUnitId(axes.x, format.celsius)}
            onChange={(xUnit) => onChange({ ...format, xUnit })}
          />
          <Select
            label={`Y unit (${axes.y})`}
            size="xs"
            error={errors.yUnit}
            data={unitIdsFor(axes.y)}
            value={format.yUnit ?? defaultUnitId(axes.y, false)}
            onChange={(yUnit) => onChange({ ...format, yUnit })}
          />
        </Group>
      )}
      <Group gap="md">
        {!isPie && !is3D && !isHistogram && (
          <Checkbox
            label="Log X"
            size="xs"
            error={errors.xLog}
            checked={format.xLog ?? false}
            indeterminate={format.xLog === null}
            onChange={(e) => onChange({ ...format, xLog: e.currentTarget.checked })}
          />
        )}
        {!isPie && !is3D && !isHistogram && (
          <Checkbox
            label="Log Y"
            size="xs"
            error={errors.yLog}
            checked={format.yLog ?? false}
            indeterminate={format.yLog === null}
            onChange={(e) => onChange({ ...format, yLog: e.currentTarget.checked })}
          />
        )}
        {!isPie && (
          <Checkbox
            label="Grid"
            size="xs"
            checked={format.grid}
            onChange={(e) => onChange({ ...format, grid: e.currentTarget.checked })}
          />
        )}
        <Checkbox
          label="Legend"
          size="xs"
          checked={format.legend}
          onChange={(e) => onChange({ ...format, legend: e.currentTarget.checked })}
        />
        {!isPie && spec.kind === 'xy' && (
          <Checkbox
            label="Show units"
            size="xs"
            checked={format.showUnits !== false}
            onChange={(e) => onChange({ ...format, showUnits: e.currentTarget.checked })}
          />
        )}
        <Select
          label="Legend alignment"
          size="xs"
          w={130}
          data={[
            { value: 'left', label: 'Left' },
            { value: 'center', label: 'Center' },
            { value: 'right', label: 'Right' },
          ]}
          value={format.legendAlign ?? 'center'}
          onChange={(v) =>
            onChange({ ...format, legendAlign: (v as 'left' | 'center' | 'right') ?? 'center' })
          }
          disabled={!format.legend}
          allowDeselect={false}
        />
      </Group>

      {!isPie && !is3D && (
        <>
          <Divider label="Line Colors" labelPosition="left" />
          {spec.kind === 'xy' ? (
            <XyLineColors spec={spec} format={format} onChange={onChange} />
          ) : (
            <Group grow>
              <ColorInput
                label="States Overlay / Cycle path color"
                size="xs"
                value={format.lineColors?.['states'] ?? '#ffa94b'}
                onChange={(color) => {
                  const lineColors = { ...format.lineColors, states: color }
                  onChange({ ...format, lineColors })
                }}
              />
            </Group>
          )}
        </>
      )}

      {!isPie && !is3D && spec.kind === 'xy' && (
        <>
          <Divider label="Trace Styles (Line & Marker)" labelPosition="left" />
          <XyTraceStyles spec={spec} format={format} onChange={onChange} />
        </>
      )}

      {!isPie && !is3D && (
        <>
          <Divider label="Reference Annotations" labelPosition="left" />
          <ReferenceAnnotationsSection format={format} onChange={onChange} />
        </>
      )}
    </Stack>
  )
}

const KIND_OPTIONS = [
  { value: 'property', label: 'Property diagram' },
  { value: 'psychro', label: 'Psychrometric chart' },
  { value: 'xy', label: 'X-Y (parametric table)' },
]

export default function PlotConfigModal({
  spec,
  allowedKinds,
  defaultName,
  fluids,
  tableVars,
  initialXy,
  tables = [],
  occupiedNames = [],
  hasStates,
  stateTables = [],
  onSave,
  onClose,
}: Readonly<Props>) {
  const [draft, setDraft] = useState<PlotSpec>(() => {
    if (spec) return spec
    const base = newPlotSpec(allowedKinds[0], defaultName)
    // Seed a fresh X-Y plot from a table column selection: x = time, y = picks.
    if (initialXy && base.kind === 'xy') {
      const srcTable = tables.find((t) => t.id === initialXy.tableId)
      const data = srcTable?.kind === 'parametric' && srcTable.results.length > 0 ? 'solved' : 'inputs'
      return {
        ...base,
        source: initialXy.tableId ? { kind: 'table', tableId: initialXy.tableId, data } : undefined,
        xy: { ...base.xy, xVar: initialXy.xVar, yVars: initialXy.yVars },
      }
    }
    return base
  })
  const nameError = !draft.name.trim() ? 'A name is required.' : occupiedNames.some((n) => n.toLowerCase() === draft.name.trim().toLowerCase()) ? 'This plot name is already in use.' : undefined
  const creating = spec === null
  const kindOptions = KIND_OPTIONS.filter((o) =>
    allowedKinds.includes(o.value as PlotKind),
  )

  function changeKind(kind: PlotKind) {
    setDraft((d) => ({ ...d, kind, format: { ...d.format, celsius: kind === 'psychro' } }))
  }

  return (
    <Modal
      opened
      onClose={onClose}
      title={creating ? 'New Plot' : `Configure ${spec?.name ?? 'Plot'}`}
      size="lg"
    >
      <Stack gap="sm">
        <Group grow align="flex-start">
          <TextInput
            label="Plot name"
            size="xs"
            value={draft.name}
            error={nameError}
            onChange={(e) => setDraft({ ...draft, name: e.currentTarget.value })}
            autoFocus={creating}
          />
          {kindOptions.length > 1 && (
            <SegmentedControl
              size="xs"
              data={kindOptions}
              value={draft.kind}
              onChange={(kind) => changeKind(kind as PlotKind)}
            />
          )}
        </Group>

        {draft.kind === 'xy' && (
          <>
            <Select
              label="Data source"
              placeholder="Select a source"
              data={[
                { value: 'arrays', label: 'Solved arrays' },
                ...tables.map((t) => ({
                  value: t.id,
                  label: `${t.name} (${t.kind === 'parametric' ? (t.origin === 'ode' ? 'Trajectory' : 'Sweep') : 'Function'})`,
                })),
              ]}
              value={draft.source?.kind === 'arrays' ? 'arrays' : draft.source?.tableId ?? null}
              onChange={(value) => {
                if (value === 'arrays') {
                  setDraft({ ...draft, source: { kind: 'arrays' } })
                } else if (value) {
                  const src = tables.find((t) => t.id === value)
                  const isFn = src?.kind === 'function'
                  setDraft({
                    ...draft,
                    source: {
                      kind: 'table',
                      tableId: value,
                      data: isFn || (src?.kind === 'parametric' && src.results.length === 0) ? 'inputs' : 'solved',
                    },
                  })
                } else {
                  setDraft({ ...draft, source: undefined })
                }
              }}
            />
            {draft.source?.kind === 'table' &&
              tables.find((t) => draft.source?.kind === 'table' && t.id === draft.source.tableId)?.kind === 'parametric' && (
                <SegmentedControl
                  value={draft.source.data}
                  data={[
                    { value: 'inputs', label: 'Raw inputs / trajectory' },
                    { value: 'solved', label: 'Successful solved rows' },
                  ]}
                  onChange={(data) =>
                    setDraft((d) =>
                      d.source?.kind === 'table'
                        ? { ...d, source: { ...d.source, data: data as 'inputs' | 'solved' } }
                        : d
                    )
                  }
                />
              )}
          </>
        )}
        {draft.kind === 'xy' && (
          <XYSection
            config={draft.xy}
            tableVars={(() => {
              if (draft.source?.kind !== 'table') return tableVars
              const targetId = draft.source.tableId
              const target = tables.find((t) => t.id === targetId)
              if (!target) return []
              if (target.kind === 'parametric') return target.vars
              return [target.argName, ...target.columns]
            })()}
            onChange={(xy) => setDraft({ ...draft, xy })}
          />
        )}
        {draft.kind === 'property' && (
          <PropertySection
            config={draft.property}
            fluids={fluids}
            hasStates={hasStates}
            stateTables={stateTables}
            onChange={(property) =>
              setDraft((d) => ({
                ...d,
                property,
                // A new diagram has different axes; drop unit overrides.
                format:
                  property.diagram === d.property.diagram
                    ? d.format
                    : { ...d.format, xUnit: null, yUnit: null },
              }))
            }
          />
        )}
        {draft.kind === 'psychro' && (
          <PsychroSection
            config={draft.psychro}
            stateTables={stateTables}
            onChange={(psychro) => setDraft({ ...draft, psychro })}
          />
        )}

        <Divider label="Format" labelPosition="left" />
        {(() => {
          const formatErrors = validatePlotFormat(draft)
          const hasFormatErrors = Object.values(formatErrors).some(Boolean)
          return (
            <>
              <FormatSection
                spec={draft}
                errors={formatErrors}
                onChange={(format) => setDraft({ ...draft, format })}
              />

              <Group justify="flex-end" mt="xs">
                <Button variant="default" size="xs" onClick={onClose}>
                  Cancel
                </Button>
                <Button
                  size="xs"
                  disabled={!!nameError || hasFormatErrors}
                  onClick={() => onSave({ ...draft, name: draft.name.trim() })}
                >
                  {creating ? 'Add plot' : 'Apply'}
                </Button>
              </Group>
            </>
          )
        })()}
      </Stack>
    </Modal>
  )
}
