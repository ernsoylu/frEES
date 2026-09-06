// Project-file round-trip + migration tests.
//
// v2 `analyzers` slice: written by the Data Analyzer in "template mode" —
// layout + signal assignments + file signatures, refs only, never samples.
// The FEATURE is removed (decision D11) and the slice is now **inert**: it is
// parsed, carried and re-serialized untouched so an old session survives a
// round-trip through a build that can no longer show it. That is what the
// first group asserts, and it is the compatibility policy in test form.
//
// v3 `schematic` slice: where the user has dragged each block on the rendered
// schematic. The drawing itself is always regenerated from the document, so
// only the offsets are saved.

import { describe, expect, it, beforeEach } from 'vitest'
import { DEFAULT_STOP_CRITERIA } from './api'
import {
  buildProject,
  loadProjectLocal,
  projectOnlyNotes,
  readProjectFile,
  saveProjectLocal,
  type AnalyzerSpec,
  type ProjectSlices,
} from './project'
import { DEFAULT_DRAFT } from './VariableInfoModal'

// A real Phase-2 analyzer session, in the shape the removed feature wrote it.
const analyzer: AnalyzerSpec = {
  id: 'a1',
  name: 'Analyzer 1',
  files: [
    { measurementId: 'm1', signature: { name: 'run1.csv', size: 12345, headerHash: 'ff00aa11' } },
  ],
  strips: [
    { id: 's1', signals: [{ measurementId: 'm1', channel: 'speed', color: '#4dabf7' }] },
    { id: 's2', signals: [{ measurementId: 'm1', channel: 'valve', color: '#ffa94d' }] },
  ],
}

const slices: ProjectSlices = {
  text: 'x = 1',
  varDrafts: {},
  stopCriteria: DEFAULT_STOP_CRITERIA,
  unitSystem: 'SI',
  fillMissing: false,
  stateUnitIds: {},
  tables: [],
  plots: [],
  spreadsheets: [],
  analyzers: [analyzer],
}

function asFile(payload: unknown): File {
  // jsdom's File lacks .text(); readProjectFile only needs that one method.
  return { text: async () => JSON.stringify(payload) } as unknown as File
}

beforeEach(() => localStorage.clear())

describe('project v2 analyzers slice (inert since D11)', () => {
  it('buildProject carries the analyzers slice at the current version', () => {
    const p = buildProject(slices)
    expect(p.version).toBe(3)
    expect(p.analyzers).toEqual([analyzer])
  })

  it('round-trips through the local autosave whole and unchanged', () => {
    saveProjectLocal(buildProject(slices))
    // Deep equality, not a spot check: nothing renders these any more, so the
    // only guarantee worth having is that every field comes back untouched.
    expect(loadProjectLocal()?.analyzers).toEqual([analyzer])
  })

  it('reads a v2 file and preserves analyzer refs (never bulk data)', async () => {
    const p = await readProjectFile(asFile(buildProject(slices)))
    expect(p.analyzers).toEqual([analyzer])
    expect(JSON.stringify(p.analyzers)).not.toContain('samples')
  })

  it('migrates a v1 file (no analyzers slice) to an empty array', async () => {
    const v1 = { ...buildProject({ ...slices, analyzers: [] }), version: 1 } as Record<string, unknown>
    delete v1.analyzers
    const p = await readProjectFile(asFile(v1))
    expect(p.version).toBe(3)
    expect(p.analyzers).toEqual([])
  })

  it('rejects files written by a newer version', async () => {
    const future = { ...buildProject(slices), version: 99 }
    await expect(readProjectFile(asFile(future))).rejects.toThrow(/newer version/)
  })
})

describe('project v3 schematic slice', () => {
  const dragged = { bcp: { dx: 60, dy: 240 }, batt: { dx: -12.5, dy: 0 } }

  it('round-trips dragged block positions through the autosave', () => {
    saveProjectLocal(buildProject({ ...slices, schematic: dragged }))
    expect(loadProjectLocal()?.schematic).toEqual(dragged)
  })

  it('round-trips through a saved file', async () => {
    const p = await readProjectFile(asFile(buildProject({ ...slices, schematic: dragged })))
    expect(p.schematic).toEqual(dragged)
  })

  it('migrates a pre-v3 file to an empty arrangement', async () => {
    const v2 = { ...buildProject(slices), version: 2 } as Record<string, unknown>
    delete v2.schematic
    const p = await readProjectFile(asFile(v2))
    expect(p.schematic).toEqual({})
  })

  it('drops entries that are not a finite pair of numbers', async () => {
    // Coordinates from a project file reach an SVG viewBox; a NaN or a string
    // there breaks the whole drawing rather than one block.
    const p = await readProjectFile(
      asFile({
        ...buildProject(slices),
        schematic: {
          good: { dx: 1, dy: 2 },
          nan: { dx: Number.NaN, dy: 2 },
          text: { dx: '10', dy: 2 },
          missing: { dx: 3 },
          nul: null,
        },
      }),
    )
    expect(p.schematic).toEqual({ good: { dx: 1, dy: 2 } })
  })

  it('clamps an absurd offset instead of letting it blow up the canvas', async () => {
    const p = await readProjectFile(
      asFile({ ...buildProject(slices), schematic: { far: { dx: 1e12, dy: -1e12 } } }),
    )
    expect(p.schematic?.far).toEqual({ dx: 100_000, dy: -100_000 })
  })

  it('ignores a non-object schematic slice', async () => {
    for (const bad of [[], 'nope', 42]) {
      const p = await readProjectFile(asFile({ ...buildProject(slices), schematic: bad }))
      expect(p.schematic).toEqual({})
    }
  })
})

describe('projectOnlyNotes', () => {
  it('is empty when only equation text would travel', () => {
    expect(projectOnlyNotes({ ...slices, analyzers: [] })).toEqual([])
  })

  it('names GUI tables, guesses, and layout that Export equation text drops', () => {
    const notes = projectOnlyNotes({
      ...slices,
      varDrafts: { x: { ...DEFAULT_DRAFT, guess: '2' } },
      tables: [{ id: 't1', name: 'map', kind: 'function', columns: [], rows: [] } as never],
      schematic: { p1: { dx: 1, dy: 0 } },
    })
    expect(notes.join(' ')).toMatch(/Variable Information/)
    expect(notes.join(' ')).toMatch(/GUI table/)
    expect(notes.join(' ')).toMatch(/schematic layout/)
    expect(notes.join(' ')).toMatch(/legacy analyzer/)
  })
})
