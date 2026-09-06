// Story 10.10: Unified project file (`.frees` JSON).
//
// A single document capturing the entire workspace — equation text, Variable
// Information, parametric/function tables, plots, digitizer state, and all
// diagrams — so a model can be saved to and opened from one file, and
// autosaved/restored across reloads. This supersedes the scattered per-feature
// localStorage keys: on save everything is collected into one object written to
// `frees.project`; the legacy keys remain only as a one-time migration source.

import { DEFAULT_STOP_CRITERIA } from './api'
import { writeToHandle } from './saveTarget'
import type { StopCriteria, UnitSystem } from './api'
import { DEFAULT_DRAFT, type VariableDraft } from './VariableInfoModal'
import type { TableSpec } from './tables'
import type { PlotSpec } from './plots/types'
import type { PinnedSlider } from './sliders'
import type { SchematicOffsets } from './schematic/layout'

// v2 (Data Analyzer Phase 2): + `analyzers` slice — layout, signal
// assignments and measurement file REFS only ("template mode", §2.5b in
// todo.md); bulk samples never enter the project file. v1 files migrate by
// defaulting the slice to []. Since D11 the slice is inert: parsed, carried
// and re-serialized, never rendered (see AnalyzerSpec below).
// v3: + `schematic` slice — where the user has dragged each block on the
// rendered schematic. Earlier files migrate by defaulting it to {}, which is
// exactly "nothing dragged yet".
const PROJECT_VERSION = 3
const PROJECT_KEY = 'frees.project'

/**
 * A free-form spreadsheet workbook, as persisted in `.frees` files.
 *
 * The spreadsheet FEATURE is removed (decision D10, Wave H) — no UI renders
 * or edits these any more. The type stays because the data stays: a loaded
 * project's `spreadsheets` array is carried inert through App and written
 * back on save, never destroyed (the `linkedTableId` precedent — D10's
 * compatibility policy). App shows a one-time notice when a loaded project's
 * array is non-empty.
 */
export interface SpreadsheetSpec {
  id: string
  name: string
  /** Sheet data array — opaque JSON. Each entry is `{ name, id, celldata,
   *  styles, … }` in the legacy `{ r, c, v: { v, m, f? } }` cell shape. */
  sheets: unknown[]
  /** Input bindings (variable name → cell ref); inert since D10. */
  bindings?: Record<string, string>
  /** Result bindings (variable name → cell ref); inert since D10. */
  resultBindings?: Record<string, string>
  /** Whether result bindings auto-synced after a solve; inert since D10. */
  autoSync?: boolean
  /** SUPERSEDED even before D10 (the old one-off parametric↔sheet link);
   * kept parsed for downgrade safety. */
  linkedTableId?: string
}

/**
 * One Data Analyzer window, as persisted in `.frees` files.
 *
 * The Data Analyzer FEATURE is removed (decision D11, Wave J) — nothing
 * renders or edits these any more, and the strip/signal shape they described
 * left with `src/analyzer/`. The slice stays because the data stays: a loaded
 * project's `analyzers` array is carried inert through App and written back on
 * save, never destroyed (the D10 `spreadsheets` precedent). App shows a
 * one-time notice when a loaded project's array is non-empty.
 *
 * Only the two fields the notice and the window titles ever needed are typed;
 * the rest of each spec (files, strips, signal colors, offsets) rides along as
 * opaque JSON so a downgrade to a build that still has the Analyzer finds its
 * sessions intact.
 */
export interface AnalyzerSpec {
  id: string
  name: string
  [key: string]: unknown
}

// Child-owned localStorage keys bridged into the project file. These mirror the
// literals used inside DigitizerTab.tsx and WorkspaceDock.tsx; the project
// file is the source of truth, those keys act as local caches.
const DIGITIZER_KEY = 'frees-digitizer'
const DOCK_LAYOUT_KEY = 'frees-dock-layout-v3'

/** The in-memory workspace slices owned by App.tsx that make up a project. */
export interface ProjectSlices {
  text: string
  varDrafts: Record<string, VariableDraft>
  stopCriteria: StopCriteria
  unitSystem: UnitSystem
  fillMissing: boolean
  stateUnitIds: Record<string, string>
  tables: TableSpec[]
  plots: PlotSpec[]
  spreadsheets: SpreadsheetSpec[]
  analyzers: AnalyzerSpec[]
  /** Parameters pinned to the workspace slider strip. */
  sliders?: PinnedSlider[]
  /** Blocks the user has moved on the schematic, as offsets from the
   *  auto-layout. The drawing itself is always derived from the document, so
   *  this is the only part of it worth saving. */
  schematic?: SchematicOffsets
}

export interface FreesProject extends ProjectSlices {
  version: number
  savedAt: string
  // Bridged from child-owned localStorage; opaque to App.
  digitizer: unknown
  dockLayout: unknown
}

function readJson(key: string): unknown {
  try {
    const raw = localStorage.getItem(key)
    return raw ? JSON.parse(raw) : null
  } catch {
    return null
  }
}

/** Assemble a complete project from App's slices plus the bridged child state. */
export function buildProject(slices: ProjectSlices): FreesProject {
  return {
    version: PROJECT_VERSION,
    savedAt: new Date().toISOString(),
    ...slices,
    digitizer: readJson(DIGITIZER_KEY),
    dockLayout: readJson(DOCK_LAYOUT_KEY),
  }
}

/**
 * Write the child-owned slices back to their localStorage caches so that
 * remounting DigitizerTab / DiagramTab restores them from an opened project.
 */
export function writeBridgedKeys(project: FreesProject) {
  try {
    if (project.digitizer != null) {
      localStorage.setItem(DIGITIZER_KEY, JSON.stringify(project.digitizer))
    } else {
      localStorage.removeItem(DIGITIZER_KEY)
    }
    if (project.dockLayout != null) {
      localStorage.setItem(DOCK_LAYOUT_KEY, JSON.stringify(project.dockLayout))
    } else {
      localStorage.removeItem(DOCK_LAYOUT_KEY)
    }
  } catch {
    // Quota or serialization failures are non-fatal; the in-memory state still loads.
  }
}

const ALLOWED_UNIT_SYSTEMS: readonly UnitSystem[] = ['SI', 'ENG_SI', 'ENGLISH']

/** Deep-copy to plain JSON data, dropping anything non-serializable. */
function plainJson<T>(value: T): T {
  try {
    return JSON.parse(JSON.stringify(value ?? null)) as T
  } catch {
    return null as T
  }
}

function finiteNumber(value: unknown, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback
}

/** How far from its auto-layout position a block may be saved, and how many
 *  blocks may carry an offset. A schematic is bounded by the network it draws;
 *  anything past these is a malformed or hostile file, not a real drawing. */
const MAX_OFFSET = 100_000
const MAX_OFFSET_ENTRIES = 5_000

/**
 * Validate the schematic's drag offsets. Coordinates from a project file reach
 * an SVG viewBox and the export's bounding box, so a non-finite or absurd value
 * would render the drawing unusable rather than merely wrong — each is required
 * to be a finite number and clamped to a plausible canvas.
 */
function sanitizeOffsets(value: unknown): SchematicOffsets {
  if (value == null || typeof value !== 'object' || Array.isArray(value)) {
    return {}
  }
  const out: SchematicOffsets = {}
  const clamp = (n: number) => Math.min(MAX_OFFSET, Math.max(-MAX_OFFSET, n))
  for (const [key, raw] of Object.entries(value as Record<string, unknown>)) {
    if (Object.keys(out).length >= MAX_OFFSET_ENTRIES) {
      break
    }
    if (raw == null || typeof raw !== 'object') {
      continue
    }
    const { dx, dy } = raw as { dx?: unknown; dy?: unknown }
    if (typeof dx !== 'number' || typeof dy !== 'number' || !Number.isFinite(dx) || !Number.isFinite(dy)) {
      continue
    }
    out[key] = { dx: clamp(dx), dy: clamp(dy) }
  }
  return out
}

/**
 * Validate and normalize a project into the plain, schema-shaped payload that is
 * safe to persist. Every field is checked against its expected type — and the
 * unit system against an allowlist — before it can reach browser storage, so a
 * project can never poison localStorage with unvalidated, externally influenced
 * values (tssecurity:S8475). Sanitizing here, at write time, keeps the trust
 * boundary independent of whatever code later reads the value back. Returns
 * null for non-object input.
 */
function sanitizeProject(project: FreesProject): FreesProject | null {
  if (project == null || typeof project !== 'object') return null
  const sc = (project.stopCriteria ?? {}) as Partial<StopCriteria>
  return {
    version: PROJECT_VERSION,
    savedAt: typeof project.savedAt === 'string' ? project.savedAt : new Date().toISOString(),
    text: typeof project.text === 'string' ? project.text : '',
    varDrafts: plainJson(project.varDrafts) ?? {},
    stopCriteria: {
      maxIterations: finiteNumber(sc.maxIterations, DEFAULT_STOP_CRITERIA.maxIterations),
      relativeResiduals: finiteNumber(sc.relativeResiduals, DEFAULT_STOP_CRITERIA.relativeResiduals),
      changeInVariables: finiteNumber(sc.changeInVariables, DEFAULT_STOP_CRITERIA.changeInVariables),
      elapsedTimeSeconds: finiteNumber(sc.elapsedTimeSeconds, DEFAULT_STOP_CRITERIA.elapsedTimeSeconds),
      ...(typeof sc.complexMode === 'boolean' ? { complexMode: sc.complexMode } : {}),
    },
    unitSystem: ALLOWED_UNIT_SYSTEMS.includes(project.unitSystem) ? project.unitSystem : 'SI',
    fillMissing: Boolean(project.fillMissing),
    stateUnitIds: plainJson(project.stateUnitIds) ?? {},
    tables: Array.isArray(project.tables) ? plainJson(project.tables) : [],
    plots: Array.isArray(project.plots) ? plainJson(project.plots) : [],
    spreadsheets: Array.isArray(project.spreadsheets) ? plainJson(project.spreadsheets) : [],
    analyzers: Array.isArray(project.analyzers) ? plainJson(project.analyzers) : [],
    sliders: Array.isArray(project.sliders) ? plainJson(project.sliders) : [],
    schematic: sanitizeOffsets(project.schematic),
    digitizer: plainJson(project.digitizer),
    dockLayout: plainJson(project.dockLayout),
  }
}

/**
 * Normalize a project read back from any browser storage (localStorage,
 * IndexedDB) to the current version and schema. Storage is outside the app's
 * trust boundary regardless of which API it hides behind, so reads go through
 * the same migrate-then-sanitize path as writes.
 */
export function normalizeStoredProject(raw: unknown): FreesProject | null {
  if (raw == null || typeof raw !== 'object') return null
  return sanitizeProject(migrate(raw as FreesProject))
}

export function saveProjectLocal(project: FreesProject) {
  const safe = sanitizeProject(project)
  if (safe == null) return
  try {
    localStorage.setItem(PROJECT_KEY, JSON.stringify(safe))
  } catch {
    // Autosave is best-effort; ignore quota errors.
  }
}

export function loadProjectLocal(): FreesProject | null {
  const raw = readJson(PROJECT_KEY)
  return raw ? migrate(raw as FreesProject) : null
}

export function clearProjectLocal() {
  try {
    localStorage.removeItem(PROJECT_KEY)
  } catch {
    // ignore
  }
}

/** Normalize a parsed project to the current version, filling missing slices. */
function migrate(p: FreesProject): FreesProject {
  return {
    version: PROJECT_VERSION,
    savedAt: p.savedAt ?? new Date().toISOString(),
    text: p.text ?? '',
    varDrafts: p.varDrafts ?? {},
    stopCriteria: p.stopCriteria,
    unitSystem: p.unitSystem ?? 'SI',
    fillMissing: Boolean(p.fillMissing),
    stateUnitIds: p.stateUnitIds ?? {},
    tables: p.tables ?? [],
    plots: p.plots ?? [],
    spreadsheets: p.spreadsheets ?? [],
    analyzers: p.analyzers ?? [],
    sliders: p.sliders ?? [],
    // Pre-v3 files predate saved schematic positions; nothing dragged.
    schematic: sanitizeOffsets(p.schematic),
    digitizer: p.digitizer ?? null,
    dockLayout: p.dockLayout ?? null,
  }
}

function sanitizeFilename(name: string): string {
  const base = name.trim().replace(/\.frees$/i, '').replace(/[^\w.-]+/g, '_')
  return `${base || 'untitled'}.frees`
}

function draftIsCustom(draft: VariableDraft): boolean {
  return (
    draft.guess.trim() !== DEFAULT_DRAFT.guess ||
    draft.lower.trim().toLowerCase() !== DEFAULT_DRAFT.lower.toLowerCase() ||
    draft.upper.trim().toLowerCase() !== DEFAULT_DRAFT.upper.toLowerCase() ||
    Boolean(draft.isUnitsUserSet && draft.units.trim()) ||
    draft.uncertainty.trim() !== DEFAULT_DRAFT.uncertainty
  )
}

/** Inputs that live in the project file but not in equation text. */
export function projectOnlyNotes(slices: ProjectSlices & { digitizer?: unknown }): string[] {
  const notes: string[] = []
  const customGuesses = Object.keys(slices.varDrafts ?? {}).filter((name) =>
    draftIsCustom(slices.varDrafts[name] ?? DEFAULT_DRAFT),
  )
  if (customGuesses.length) {
    notes.push(`Variable Information for ${customGuesses.length} variable(s) (not written as GUESS)`)
  }
  if ((slices.tables ?? []).length) {
    notes.push(`${slices.tables.length} GUI table(s) / imported map(s)`)
  }
  if ((slices.plots ?? []).length) {
    notes.push(`${slices.plots.length} plot(s)`)
  }
  if ((slices.sliders ?? []).length) {
    notes.push(`${slices.sliders!.length} slider override(s)`)
  }
  if (slices.schematic && Object.keys(slices.schematic).length) {
    notes.push('schematic layout offsets')
  }
  if ((slices.spreadsheets ?? []).length) {
    notes.push('legacy spreadsheet slice')
  }
  if ((slices.analyzers ?? []).length) {
    notes.push('legacy analyzer slice')
  }
  if (slices.digitizer != null) {
    notes.push('digitizer map data')
  }
  return notes
}

/** Download only the editor document — not the `.frees` project. */
export function downloadEquationText(text: string, filename: string) {
  const blob = new Blob([text], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = sanitizeFilename(filename).replace(/\.frees$/i, '.txt')
  document.body.appendChild(a)
  a.click()
  a.remove()
  URL.revokeObjectURL(url)
}

/** Trigger a browser download of the project as a `.frees` JSON file. */
function downloadProject(project: FreesProject, filename: string) {
  const blob = new Blob([JSON.stringify(project, null, 2)], {
    type: 'application/json',
  })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = sanitizeFilename(filename)
  document.body.appendChild(a)
  a.click()
  a.remove()
  URL.revokeObjectURL(url)
}

export interface SaveViaPickerResult {
  /** True if the project was saved (or a download was triggered). */
  saved: boolean
  /**
   * The picked file's handle when the FS Access API produced one (Chromium) —
   * Wave I keeps it so plain Save can write back without a picker. Null on
   * the download fallback and on cancel.
   */
  handle: FileSystemFileHandle | null
}

/** The picker/handle file-type filter — one definition for save and open. */
export const FREES_FILE_TYPES = [
  { description: 'frees project', accept: { 'application/json': ['.frees'] } },
]

/**
 * Save the project, letting the user choose the destination via the File System
 * Access API (showSaveFilePicker) where supported. Falls back to a plain browser
 * download (fixed Downloads folder) on browsers without the API (e.g. Firefox).
 *
 * `saved` is false only when the user cancelled the picker — so callers can
 * keep the dirty flag set.
 */
export async function saveProject(project: FreesProject, filename: string): Promise<SaveViaPickerResult> {
  const json = JSON.stringify(project, null, 2)
  const suggestedName = sanitizeFilename(filename)
  const picker = (window as unknown as {
    showSaveFilePicker?: (opts: unknown) => Promise<FileSystemFileHandle>
  }).showSaveFilePicker

  if (typeof picker === 'function') {
    try {
      const handle = await picker({ suggestedName, types: FREES_FILE_TYPES })
      const writable = await handle.createWritable()
      await writable.write(json)
      await writable.close()
      return { saved: true, handle }
    } catch (err) {
      // The user dismissed the picker — leave the project unsaved (and dirty).
      if (err instanceof DOMException && err.name === 'AbortError') return { saved: false, handle: null }
      // Any other failure (permissions, unsupported) falls back to a download.
    }
  }

  downloadProject(project, filename)
  return { saved: true, handle: null }
}

/**
 * Wave I: re-save to the file the project came from, no picker. Serializes
 * exactly like `saveProject` and defers the permission dance to
 * `writeToHandle`; the caller falls back to the picker on 'denied'/'failed'.
 */
export async function saveProjectToHandle(
  project: FreesProject,
  handle: FileSystemFileHandle,
): Promise<'saved' | 'denied' | 'failed'> {
  return writeToHandle(handle, JSON.stringify(project, null, 2))
}

/** Read and validate an opened `.frees` file. */
export async function readProjectFile(file: File): Promise<FreesProject> {
  const raw = await file.text()
  const parsed = JSON.parse(raw)
  if (!parsed || typeof parsed !== 'object' || !('version' in parsed)) {
    throw new Error('Not a valid .frees project file.')
  }
  if ((parsed as FreesProject).version > PROJECT_VERSION) {
    throw new Error(
      `This project was saved by a newer version of frees (v${(parsed as FreesProject).version}).`,
    )
  }
  return migrate(parsed as FreesProject)
}
