import { useState } from 'react'
import { Button, Checkbox, Divider, Group, Modal, Select, Stack, Switch, Text, TextInput } from '@mantine/core'
import {
  DEFAULT_STOP_CRITERIA,
  StopCriteria,
  UNIT_SYSTEM_OPTIONS,
  UnitSystem,
} from './api'
import { dropPrecache, readDataSaver, writeDataSaver } from './dataSaver'

interface Props {
  criteria: StopCriteria
  unitSystem: UnitSystem
  fillMissing: boolean
  onSave: (criteria: StopCriteria, unitSystem: UnitSystem, fillMissing: boolean) => void
  onClose: () => void
}

type StopCriteriaField = Exclude<keyof StopCriteria, 'complexMode'>

interface Field {
  key: StopCriteriaField
  label: string
  hint: string
  /** Shown but not editable: the browser engine has no such stop rule yet. The
   *  value still round-trips through projects and the request. */
  disabled?: boolean
}

// Mirrors the Options > Preferences > Stop Crit tab.
const FIELDS: Field[] = [
  { key: 'maxIterations', label: 'No. iterations', hint: 'Maximum Newton iterations per block' },
  { key: 'relativeResiduals', label: 'Relative residuals', hint: '|lhs − rhs| / |lhs| convergence tolerance' },
  {
    key: 'changeInVariables',
    label: 'Change in variables',
    // The browser engine's per-block stop rule is the relative residual alone
    // (`engine.rs`, which records the same gap against the Java record). An
    // editable box here advertised a criterion nothing consulted.
    hint: 'Not used by the browser engine — a block stops on the relative residual above. Kept so saved projects round-trip.',
    disabled: true,
  },
  { key: 'elapsedTimeSeconds', label: 'Elapsed time (sec)', hint: 'Abort the solve after this many seconds' },
]

export default function PreferencesModal({ criteria, unitSystem, fillMissing, onSave, onClose }: Readonly<Props>) {
  const [draft, setDraft] = useState<Record<StopCriteriaField, string>>({
    maxIterations: String(criteria.maxIterations),
    relativeResiduals: String(criteria.relativeResiduals),
    changeInVariables: String(criteria.changeInVariables),
    elapsedTimeSeconds: String(criteria.elapsedTimeSeconds),
  })
  const [system, setSystem] = useState<UnitSystem>(unitSystem)
  const [fillMissingState, setFillMissingState] = useState<boolean>(fillMissing)
  const [error, setError] = useState<string | null>(null)
  // Data saver is a *device* setting, not a document one: it never enters a
  // .frees project, so it is read from and written to localStorage here rather
  // than routed through App's onSave (which persists project slices).
  const [dataSaver, setDataSaver] = useState<boolean>(() => readDataSaver())

  function setField(key: StopCriteriaField, value: string) {
    setDraft((d) => ({ ...d, [key]: value }))
    setError(null)
  }

  function restoreDefaults() {
    setDraft({
      maxIterations: String(DEFAULT_STOP_CRITERIA.maxIterations),
      relativeResiduals: String(DEFAULT_STOP_CRITERIA.relativeResiduals),
      changeInVariables: String(DEFAULT_STOP_CRITERIA.changeInVariables),
      elapsedTimeSeconds: String(DEFAULT_STOP_CRITERIA.elapsedTimeSeconds),
    })
    setFillMissingState(false)
    setError(null)
  }

  function save() {
    const parsed: Partial<Record<StopCriteriaField, number>> = {}
    for (const field of FIELDS) {
      const value = Number(draft[field.key])
      if (!Number.isFinite(value) || value <= 0) {
        setError(`${field.label} must be a positive number.`)
        return
      }
      parsed[field.key] = value
    }
    if (!Number.isInteger(parsed.maxIterations!)) {
      setError('No. iterations must be a whole number.')
      return
    }
    if (dataSaver !== readDataSaver()) {
      writeDataSaver(dataSaver)
      if (dataSaver) {
        // Turning it on: stop the *next* precache now rather than at the next
        // boot, and give this browser's storage back. Best-effort and
        // deliberately not awaited — the page keeps working from the still-live
        // worker until it is unloaded.
        void dropPrecache({ serviceWorker: navigator.serviceWorker, caches: globalThis.caches })
      }
    }
    onSave({ ...criteria, ...parsed }, system, fillMissingState)
  }

  return (
    <Modal opened onClose={onClose} title="Preferences" centered size="lg">
      <Text size="sm" c="dimmed" mb="md">
        Calculation stops when any criterion is satisfied. Restore Defaults
        applies the frees defaults (tight tolerances for higher precision).
      </Text>

      <Stack gap="sm">
        <Switch
          label="Data saver"
          description={
            dataSaver
              ? 'On — frees will not download itself for offline use. It arrives piece by piece as you open things, and it will not work without a connection. Takes effect on your next visit; turning it on now also frees the storage this browser already used.'
              : 'Off — frees downloads itself once (about 9 MB) so the whole app, engine included, keeps working with no connection. Turn this on if you are on a metered connection; it takes effect on your next visit.'
          }
          checked={dataSaver}
          onChange={(e) => setDataSaver(e.currentTarget.checked)}
        />
        <Divider />
        <Select
          label="Display unit system"
          description="Calculations always run in SI; results are converted for display"
          data={UNIT_SYSTEM_OPTIONS}
          value={system}
          onChange={(v) => v && setSystem(v as UnitSystem)}
          allowDeselect={false}
        />
        <Checkbox
          label="Fill all missing state variables in background"
          description="Runs thermodynamic queries to compute other properties (like specific volume, quality, enthalpy) for detected state points"
          checked={fillMissingState}
          onChange={(e) => setFillMissingState(e.currentTarget.checked)}
        />
        {FIELDS.map((field) => (
          <TextInput
            key={field.key}
            label={field.label}
            description={field.hint}
            disabled={field.disabled}
            value={draft[field.key]}
            onChange={(e) => setField(field.key, e.currentTarget.value)}
            spellCheck={false}
            styles={{ input: { fontFamily: 'var(--mantine-font-family-monospace)' } }}
          />
        ))}
      </Stack>

      {error && (
        <Text c="red" size="sm" mt="sm">
          {error}
        </Text>
      )}

      <Group justify="space-between" mt="lg">
        <Button variant="subtle" onClick={restoreDefaults}>
          Restore Defaults
        </Button>
        <Group gap="xs">
          <Button variant="default" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={save}>OK</Button>
        </Group>
      </Group>
    </Modal>
  )
}
