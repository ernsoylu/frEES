import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { ActionIcon, Badge, Button, Group, Paper, Select, Stack, Text, Tooltip } from '@mantine/core'
import {
  IconArrowsMaximize,
  IconDownload,
  IconLayoutGrid,
  IconZoomIn,
  IconZoomOut,
} from '@tabler/icons-react'
import type { CheckResponse, ComponentResult, VariableResult } from '../api'
import {
  declarationLine,
  declarationLineFromCheck,
  declaredComponentTypes,
  declaredInstances,
} from './declaration'
import { buildLineStyles, lineId, lineLabel, type LineStyle } from './palette'
import {
  layoutSchematic,
  routeEdge,
  type SchematicEdge,
  type SchematicNode,
  type SchematicOffsets,
} from './layout'
import { glyphFilled, glyphPath, SHAPE_LABELS } from './symbols'
import { badgeFor, formatCompact, indexVariables, readoutFor, type NodeReadout } from './readouts'
import { COMPONENT_CATALOG } from '../componentCatalog'
import {
  alreadyConnected,
  connectStatement,
  endpointRef,
  previewWire,
  type WireEndpoint,
} from './wiring'

const MIN_ZOOM = 0.25
const MAX_ZOOM = 3

interface Props {
  /** The check response holding the connection topology; it outlives the
   *  solve result, so the schematic tracks the text without needing a solve. */
  checkResult: CheckResponse | null
  /** Solved component instances, when a solve has run — supplies the written
   *  spelling, the component type and the parameter bindings for each node. */
  components?: ComponentResult[]
  /** Solved variables, so each block can show its own results. */
  variables?: VariableResult[]
  /** The document text, for locating an instance's declaration on click. */
  text: string
  /** Reveal a 1-based line in the editor. */
  onRevealLine: (line: number) => void
  /** Append a statement to the document (wiring emits `connect(...)` lines).
   *  Absent = the canvas stays read-only. */
  onEmitStatement?: (statement: string) => void
  /** Where the user has dragged each block, owned by the workspace so it rides
   *  the project file — the drawing is regenerated from the document on every
   *  check, so these offsets are the only part of it worth saving. */
  offsets?: SchematicOffsets
  onOffsetsChange?: (next: SchematicOffsets) => void
  /** Instance ids to emphasize from a Check diagnosis (free quantities). */
  highlightIds?: readonly string[]
}

type Offsets = SchematicOffsets

/**
 * Rendered schematic of the component network, drawn as a circuit: each
 * working fluid gets its own line color and its own framed band, blocks carry
 * the symbol of what they are, and every block reports its own solved state on
 * hover. Nodes can be dragged; the canvas pans and zooms. Everything is drawn
 * from the check payload, so the drawing follows the text as it is checked.
 */
export default function SchematicTab({
  checkResult,
  components,
  variables,
  text,
  onRevealLine,
  onEmitStatement,
  offsets: savedOffsets,
  onOffsetsChange,
  highlightIds = [],
}: Readonly<Props>) {
  const svgRef = useRef<SVGSVGElement>(null)
  const viewportRef = useRef<HTMLDivElement>(null)
  const [zoom, setZoom] = useState(1)
  const [pan, setPan] = useState({ x: 0, y: 0 })
  // Dragging updates many times a second; routing every pointermove through
  // the workspace would re-render the app and restart its autosave debounce
  // once per frame. The canvas therefore owns the live value — seeded from the
  // workspace — and publishes upward once, when the drag ends. The ref mirrors
  // the state so the commit can read the final value without waiting for a
  // render.
  const offsetsRef = useRef<Offsets>(savedOffsets ?? {})
  const [offsets, setOffsets] = useState<Offsets>(savedOffsets ?? {})
  const applyOffsets = useCallback((next: Offsets) => {
    offsetsRef.current = next
    setOffsets(next)
  }, [])
  const [hovered, setHovered] = useState<string | null>(null)
  const [pinned, setPinned] = useState<string | null>(null)
  const [pendingPort, setPendingPort] = useState<{ instance: string; port: string } | null>(null)
  const [wireNote, setWireNote] = useState<string | null>(null)
  const [selectFrom, setSelectFrom] = useState<string | null>(null)
  const [selectTo, setSelectTo] = useState<string | null>(null)
  const lastEmitted = useRef<{ a: string; b: string } | null>(null)

  const connections = useMemo(() => checkResult?.connections ?? [], [checkResult])

  const labels = useMemo(() => {
    const map = new Map<string, { label: string; type?: string }>()
    // The document covers the un-solved case; Check identity then a solve
    // refine nested scopes the text scan cannot distinguish.
    for (const [instance, hit] of documentInstances(text)) {
      map.set(instance, hit)
    }
    for (const i of checkResult?.instances ?? []) {
      map.set(i.name.toLowerCase(), { label: i.label, type: i.type })
    }
    for (const c of components ?? []) {
      map.set(c.name.toLowerCase(), { label: c.name, type: c.type })
    }
    return map
  }, [checkResult, components, text])

  const declaredIds = useMemo(() => [...documentInstances(text).keys()], [text])

  // The document wires components but the payload carries no topology: the
  // check did not get far enough to expand the network. Say so, rather than
  // showing a silently edgeless canvas right after the user wired something.
  const topologyStale = useMemo(
    () => connections.length === 0 && /^\s*connect\s*\(/im.test(text),
    [connections, text],
  )

  /** Each instance's declared ports, by component type, so unwired ports show. */
  const portsByInstance = useMemo(() => {
    const byType = new Map(COMPONENT_CATALOG.map((c) => [c.type.toLowerCase(), c.ports]))
    const out = new Map<string, readonly string[]>()
    for (const [instance, hit] of documentInstances(text)) {
      const ports = byType.get(hit.type.toLowerCase())
      if (ports && ports.length > 0) {
        out.set(instance, ports)
      }
    }
    return out
  }, [text])

  /** Written spelling of each fluid (`EG50`), for the legend — the payload
   *  canonicalises to lowercase but the document said it properly. */
  const fluidSpelling = useMemo(() => {
    const out = new Map<string, string>()
    for (const c of components ?? []) {
      for (const p of c.params) {
        if (p.name.endsWith('$') && p.ref) {
          out.set(p.ref.toLowerCase(), p.ref)
        }
      }
    }
    return out
  }, [components])

  const layout = useMemo(
    () =>
      layoutSchematic(connections, labels, declaredIds, portsByInstance, (key, index) => {
        const base = lineLabel(key, fluidSpelling)
        return index > 0 ? `${base} (${index + 1})` : base
      }),
    [connections, labels, declaredIds, portsByInstance, fluidSpelling],
  )

  const styles = useMemo(() => buildLineStyles(layout.lines), [layout.lines])
  const styleOf = useCallback(
    (id: string): LineStyle => styles.get(id) ?? { color: '#adb5bd', width: 1.8 },
    [styles],
  )

  // A drag moves a node; the layout stays the authority for where it started.
  const positioned = useMemo(() => {
    const map = new Map<string, SchematicNode>()
    for (const n of layout.nodes) {
      const o = offsets[n.id]
      map.set(n.id, o ? { ...n, x: n.x + o.dx, y: n.y + o.dy } : n)
    }
    return map
  }, [layout.nodes, offsets])

  const values = useMemo(
    () => indexVariables(variables, checkResult?.inferredUnits),
    [variables, checkResult?.inferredUnits],
  )
  const readouts = useMemo(() => {
    const out = new Map<string, NodeReadout>()
    for (const n of layout.nodes) {
      if (n.kind === 'instance') {
        out.set(n.id, readoutFor(n, layout.edges, values, components))
      }
    }
    return out
  }, [layout.nodes, layout.edges, values, components])

  // The viewBox is expressed in VIEWPORT pixels divided by zoom, so `zoom = 1`
  // is a true 1:1 rendering and panning moves by real pixels. Deriving it from
  // the drawing's own size instead would make the SVG silently scale-to-fit,
  // and every zoom step would then mean something different per document.
  // Zero until the element has actually been measured — a guessed size here
  // would let the one-time fit below compute a zoom for a viewport that never
  // existed, leaving the drawing framed for the wrong window.
  const [viewport, setViewport] = useState({ w: 0, h: 0 })
  useEffect(() => {
    const el = viewportRef.current
    if (!el) {
      return
    }
    const observer = new ResizeObserver(([entry]) => {
      const box = entry.contentRect
      if (box.width > 0 && box.height > 0) {
        setViewport({ w: box.width, h: box.height })
      }
    })
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  const fitTo = useCallback(
    (w: number, h: number) => {
      if (layout.width === 0 || layout.height === 0 || w === 0) {
        return
      }
      const next = Math.min(w / layout.width, h / layout.height, MAX_ZOOM)
      setZoom(Math.max(MIN_ZOOM, next))
      setPan({ x: 0, y: 0 })
    },
    [layout.width, layout.height],
  )
  // Auto-framing stays on until the user takes control of the view, then never
  // fights them again. A one-shot "fit on first render" is not enough: the
  // panel is still settling when the first measurement arrives, so the drawing
  // would be framed for a window that existed for one frame.
  const userAdjusted = useRef(false)
  const fit = useCallback(() => {
    userAdjusted.current = false
    fitTo(viewport.w, viewport.h)
  }, [fitTo, viewport.w, viewport.h])
  const takeControl = useCallback(() => {
    userAdjusted.current = true
  }, [])

  const nodeKey = layout.nodes.map((n) => n.id).join(',')

  /**
   * Hands the current arrangement to the workspace, so it rides the project
   * file. Offsets for blocks the document no longer declares are dropped here
   * rather than as the network changes: a stale entry is harmless to draw (no
   * node looks it up), and pruning at save time means editing a document never
   * discards a position the user set on a block that survives the edit.
   */
  const commitOffsets = useCallback(() => {
    if (!onOffsetsChange) {
      return
    }
    const alive = new Set(nodeKey.split(','))
    const live: Offsets = {}
    for (const [id, offset] of Object.entries(offsetsRef.current)) {
      if (alive.has(id)) {
        live[id] = offset
      }
    }
    onOffsetsChange(live)
  }, [nodeKey, onOffsetsChange])

  // Keep the whole network framed as the panel settles and the document
  // changes, so the user opens onto the drawing rather than its top-left
  // corner — but stop the moment they zoom, pan or drag anything.
  useEffect(() => {
    if (userAdjusted.current || layout.nodes.length === 0 || viewport.w === 0) {
      return
    }
    fitTo(viewport.w, viewport.h)
  }, [layout.nodes.length, layout.width, layout.height, viewport.w, viewport.h, fitTo])

  const revealInstance = (node: SchematicNode) => {
    if (node.kind !== 'instance') {
      return
    }
    const line =
      declarationLineFromCheck(checkResult?.instances, node.id) ??
      declarationLineFromCheck(checkResult?.instances, node.label) ??
      declarationLine(text, node.label)
    if (line !== null) {
      onRevealLine(line)
    }
  }

  const portChoices = useMemo(() => {
    const out: { value: string; label: string }[] = []
    for (const n of layout.nodes) {
      if (n.kind !== 'instance') continue
      for (const p of n.ports) {
        out.push({ value: `${n.id}::${p.port}`, label: `${n.label}.${p.port}` })
      }
    }
    return out
  }, [layout.nodes])

  const endpointOf = (instance: string, port: string): WireEndpoint => {
    const node = layout.nodes.find((n) => n.id === instance)
    return {
      instance,
      port,
      label: node?.label ?? instance,
      type: node?.type,
    }
  }

  const tryConnect = (from: WireEndpoint, to: WireEndpoint) => {
    if (!onEmitStatement) {
      return
    }
    const preview = previewWire(from, to, connections)
    if (!preview.ok) {
      setWireNote(preview.reason)
      setPendingPort(null)
      return
    }
    const statement = connectStatement(from, to)
    lastEmitted.current = { a: endpointRef(from), b: endpointRef(to) }
    onEmitStatement(statement)
    setPendingPort(null)
    setSelectFrom(null)
    setSelectTo(null)
    setWireNote(`Connection added; checking…  Undo (Ctrl+Z) removes the statement.`)
  }

  const clickPort = (instance: string, port: string) => {
    if (!onEmitStatement) {
      return
    }
    setWireNote(null)
    if (!pendingPort) {
      setPendingPort({ instance, port })
      return
    }
    tryConnect(endpointOf(pendingPort.instance, pendingPort.port), endpointOf(instance, port))
  }

  useEffect(() => {
    const pending = lastEmitted.current
    if (!pending || !checkResult) {
      return
    }
    if (alreadyConnected(connections, pending.a, pending.b)) {
      setWireNote(`Connected ${pending.a} — ${pending.b}`)
      lastEmitted.current = null
      return
    }
    const msg = checkResult.message ?? ''
    if (/domain|fluid family|connect\(/i.test(msg) && !checkResult.solvable) {
      setWireNote(msg)
      lastEmitted.current = null
    }
  }, [checkResult, connections])

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setPendingPort(null)
        setWireNote(null)
        return
      }
      if (!pinned) {
        return
      }
      const focusInSchematic = viewportRef.current?.contains(document.activeElement)
      if (!focusInSchematic) {
        return
      }
      const step = e.shiftKey ? 20 : 8
      let dx = 0
      let dy = 0
      if (e.key === 'ArrowLeft') dx = -step
      else if (e.key === 'ArrowRight') dx = step
      else if (e.key === 'ArrowUp') dy = -step
      else if (e.key === 'ArrowDown') dy = step
      else return
      e.preventDefault()
      takeControl()
      const cur = offsetsRef.current[pinned] ?? { dx: 0, dy: 0 }
      const next = { ...offsetsRef.current, [pinned]: { dx: cur.dx + dx, dy: cur.dy + dy } }
      applyOffsets(next)
      onOffsetsChange?.(next)
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [pinned, applyOffsets, onOffsetsChange, takeControl])

  const exportSvg = () => {
    const svg = svgRef.current
    if (!svg) {
      return
    }
    // The live viewBox frames what is on screen; an export has to frame the
    // whole drawing, including anything the user has dragged out of view.
    let minX = 0
    let minY = 0
    let maxX = layout.width
    let maxY = layout.height
    for (const n of positioned.values()) {
      minX = Math.min(minX, n.x - 20)
      minY = Math.min(minY, n.y - 20)
      maxX = Math.max(maxX, n.x + n.w + 20)
      maxY = Math.max(maxY, n.y + n.h + 20)
    }
    const clone = svg.cloneNode(true) as SVGSVGElement
    clone.setAttribute('viewBox', `${minX} ${minY} ${maxX - minX} ${maxY - minY}`)
    clone.setAttribute('width', String(Math.round(maxX - minX)))
    clone.setAttribute('height', String(Math.round(maxY - minY)))
    const xml = new XMLSerializer().serializeToString(clone)
    const blob = new Blob([xml], { type: 'image/svg+xml' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = 'schematic.svg'
    document.body.appendChild(link)
    link.click()
    link.remove()
    URL.revokeObjectURL(url)
  }

  const drag = useDragging(zoom, offsetsRef, applyOffsets, setPan, commitOffsets)

  const active = pinned ?? hovered
  const activeNode = active ? positioned.get(active) : undefined
  const activeReadout = active ? readouts.get(active) : undefined

  if (connections.length === 0 && declaredIds.length === 0) {
    return (
      <Stack gap="xs" p="md" align="center" justify="center" h="100%">
        <Text size="sm" c="dimmed" ta="center" maw={430}>
          No component network to draw. Instantiate components and wire them — with
          <Text span ff="monospace" size="sm">{' connect(a.out, b.in) '}</Text>
          or by sharing a stream name — then press Check.
        </Text>
      </Stack>
    )
  }

  return (
    <Stack gap={4} h="100%" style={{ minHeight: 0, position: 'relative' }}>
      <Group gap="xs" px="xs" pt={4} wrap="wrap">
        {layout.lines.map((key) => {
          const id = lineId(key)
          const style = styleOf(id)
          return (
            <Group key={id} gap={4}>
              <span
                style={{
                  width: 16,
                  height: 3,
                  borderRadius: 2,
                  background: style.dash ? 'none' : style.color,
                  borderTop: style.dash ? `3px dashed ${style.color}` : undefined,
                  display: 'inline-block',
                }}
              />
              <Text size="xs" c="dimmed">
                {lineLabel(key, fluidSpelling)}
              </Text>
            </Group>
          )
        })}
        <Badge size="xs" variant="light" color="gray">
          {layout.nodes.filter((n) => n.kind === 'instance').length} components
        </Badge>
        {onEmitStatement && pendingPort && (
          <Badge size="xs" variant="filled" color="teal">
            wiring from {pendingPort.instance}.{pendingPort.port} — pick a second port (Esc cancels)
          </Badge>
        )}
        {onEmitStatement && !pendingPort && wireNote && (
          <Text size="xs" c="dimmed" maw={520}>
            {wireNote}
          </Text>
        )}
        {topologyStale && (
          <Text size="xs" c="orange">
            connections not shown — the document has errors; fix them and Check
          </Text>
        )}
        {onEmitStatement && (
          <Group gap={6} wrap="nowrap">
            <Select
              size="xs"
              placeholder="From port"
              data={portChoices}
              value={selectFrom}
              onChange={setSelectFrom}
              searchable
              w={160}
              aria-label="Connect from port"
            />
            <Select
              size="xs"
              placeholder="To port"
              data={portChoices}
              value={selectTo}
              onChange={setSelectTo}
              searchable
              w={160}
              aria-label="Connect to port"
            />
            <Button
              size="compact-xs"
              disabled={!selectFrom || !selectTo}
              onClick={() => {
                const a = parseChoice(selectFrom)
                const b = parseChoice(selectTo)
                if (a && b) tryConnect(endpointOf(a.instance, a.port), endpointOf(b.instance, b.port))
              }}
            >
              Connect
            </Button>
          </Group>
        )}
        <Group gap={2} ml="auto">
          <Tooltip label="Zoom out">
            <ActionIcon size="sm" variant="subtle" aria-label="Zoom out" onClick={() => {
                takeControl()
                setZoom((z) => Math.max(MIN_ZOOM, z / 1.25))
              }}>
              <IconZoomOut size={15} />
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Zoom in">
            <ActionIcon size="sm" variant="subtle" aria-label="Zoom in" onClick={() => {
                takeControl()
                setZoom((z) => Math.min(MAX_ZOOM, z * 1.25))
              }}>
              <IconZoomIn size={15} />
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Fit to window">
            <ActionIcon size="sm" variant="subtle" aria-label="Fit to window" onClick={fit}>
              <IconArrowsMaximize size={15} />
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Reset layout">
            <ActionIcon
              size="sm"
              variant="subtle"
              aria-label="Reset layout"
              onClick={() => {
                applyOffsets({})
                onOffsetsChange?.({})
                setPan({ x: 0, y: 0 })
                fit()
              }}
            >
              <IconLayoutGrid size={15} />
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Export SVG">
            <ActionIcon size="sm" variant="subtle" aria-label="Export SVG" onClick={exportSvg}>
              <IconDownload size={15} />
            </ActionIcon>
          </Tooltip>
        </Group>
      </Group>

      <div
        ref={viewportRef}
        style={{ flex: 1, minHeight: 0, overflow: 'hidden', cursor: drag.panning ? 'grabbing' : 'grab' }}
        onPointerDown={(e) => {
          takeControl()
          drag.startPan(e)
        }}
        onWheel={(e) => {
          if (!e.ctrlKey && !e.metaKey) {
            return
          }
          e.preventDefault()
          takeControl()
          setZoom((z) => Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z * (e.deltaY < 0 ? 1.1 : 1 / 1.1))))
        }}
      >
        <svg
          ref={svgRef}
          width="100%"
          height="100%"
          viewBox={`${-pan.x / zoom} ${-pan.y / zoom} ${(viewport.w || layout.width) / zoom} ${(viewport.h || layout.height) / zoom}`}
          xmlns="http://www.w3.org/2000/svg"
          role="application"
          aria-label="Component network schematic. Tab to nodes and ports. Enter connects or inspects. Escape cancels wiring. Arrow keys nudge a focused node."
        >
          {/* Circuit bands: one frame per fluid loop / coupling network, so two
              circuits that share a bond-graph domain still read apart. */}
          <g>
            {layout.groups.map((g) => {
              const style = styleOf(g.lineId)
              return (
                <g key={g.id}>
                  <rect
                    x={g.x}
                    y={g.y}
                    width={g.w}
                    height={g.h}
                    rx={10}
                    fill={`${style.color}0d`}
                    stroke={`${style.color}44`}
                    strokeWidth={1}
                  />
                  <text x={g.x + 12} y={g.y + 15} fontSize={11} fontWeight={600} fill={style.color}>
                    {g.label.toUpperCase()}
                  </text>
                </g>
              )
            })}
          </g>

          <g>
            {layout.edges.map((e) => {
              const a = positioned.get(e.from)
              const b = positioned.get(e.to)
              if (!a || !b) {
                return null
              }
              const style = styleOf(e.lineId)
              const lit = active !== null && (e.from === active || e.to === active)
              return (
                <path
                  key={e.id}
                  d={routeEdge(a, b, e.fromPort, e.toPort)}
                  fill="none"
                  stroke={style.color}
                  strokeWidth={lit ? style.width + 1.2 : style.width}
                  strokeDasharray={style.dash}
                  strokeLinejoin="round"
                  strokeOpacity={active !== null && !lit ? 0.28 : 0.95}
                >
                  <title>{edgeTitle(e)}</title>
                </path>
              )
            })}
          </g>

          <g>
            {layout.nodes.map((base) => {
              const n = positioned.get(base.id) ?? base
              return n.kind === 'junction' ? (
                <circle
                  key={n.id}
                  cx={n.x + n.w / 2}
                  cy={n.y + n.h / 2}
                  r={n.w / 2}
                  fill={styleOf(layout.edges.find((e) => e.from === n.id || e.to === n.id)?.lineId ?? '').color}
                  stroke="#1a1b1e"
                  strokeWidth={1.5}
                />
              ) : (
                <NodeBlock
                  key={n.id}
                  node={n}
                  badge={badgeFor(n, readouts.get(n.id) ?? { ports: [], outputs: [], params: [] })}
                  active={active === n.id}
                  dimmed={active !== null && active !== n.id}
                  styleOf={styleOf}
                  wiring={Boolean(onEmitStatement)}
                  pendingPort={pendingPort}
                  highlighted={highlightIds.includes(n.id)}
                  onPointerDown={(e) => {
                    takeControl()
                    drag.startNode(e, n.id)
                  }}
                  onClick={() => {
                    if (drag.moved()) {
                      return
                    }
                    setPinned((p) => (p === n.id ? null : n.id))
                    revealInstance(n)
                  }}
                  onEnter={() => setHovered(n.id)}
                  onLeave={() => setHovered(null)}
                  onPort={(port) => clickPort(n.id, port)}
                />
              )
            })}
          </g>
        </svg>
      </div>

      {activeNode && activeReadout && (
        <ReadoutCard
          node={activeNode}
          readout={activeReadout}
          pinned={pinned === activeNode.id}
          localType={
            checkResult?.instances?.find((i) => i.name.toLowerCase() === activeNode.id)?.localType ??
            false
          }
          onReveal={() => revealInstance(activeNode)}
          onNudge={
            pinned === activeNode.id
              ? (dx, dy) => {
                  takeControl()
                  const cur = offsetsRef.current[activeNode.id] ?? { dx: 0, dy: 0 }
                  const next = {
                    ...offsetsRef.current,
                    [activeNode.id]: { dx: cur.dx + dx, dy: cur.dy + dy },
                  }
                  applyOffsets(next)
                  onOffsetsChange?.(next)
                }
              : undefined
          }
        />
      )}
    </Stack>
  )
}

/** Instances the document declares — types filtered against the component
 *  catalog and the document's own COMPONENT blocks, so prose in a comment can
 *  never become a node. */
function documentInstances(text: string): Map<string, { label: string; type: string }> {
  const known = new Set<string>(COMPONENT_CATALOG.map((c) => c.type.toLowerCase()))
  for (const local of declaredComponentTypes(text)) {
    known.add(local)
  }
  return declaredInstances(text, known)
}

function parseChoice(value: string | null): { instance: string; port: string } | null {
  if (!value) return null
  const sep = value.indexOf('::')
  if (sep < 0) return null
  return { instance: value.slice(0, sep), port: value.slice(sep + 2) }
}

interface BlockProps {
  node: SchematicNode
  badge: { label: string; value: number; units: string } | null
  active: boolean
  dimmed: boolean
  styleOf: (id: string) => LineStyle
  wiring: boolean
  pendingPort: { instance: string; port: string } | null
  highlighted?: boolean
  onPointerDown: (e: React.PointerEvent) => void
  onClick: () => void
  onEnter: () => void
  onLeave: () => void
  onPort: (port: string) => void
}

/** One component, drawn as its circuit symbol with its identity and its most
 *  telling number. */
function NodeBlock({
  node,
  badge,
  active,
  dimmed,
  styleOf,
  wiring,
  pendingPort,
  highlighted = false,
  onPointerDown,
  onClick,
  onEnter,
  onLeave,
  onPort,
}: Readonly<BlockProps>) {
  const glyph = glyphPath(node.shape)
  const GLYPH = 20
  const activate = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      onClick()
    }
  }
  return (
    <g
      transform={`translate(${node.x}, ${node.y})`}
      opacity={dimmed && !highlighted ? 0.45 : 1}
      onPointerDown={onPointerDown}
      onClick={onClick}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
      onKeyDown={activate}
      tabIndex={0}
      role="button"
      aria-label={`${node.type ?? 'component'} ${node.label}`}
      style={{ cursor: 'move' }}
    >
      <title>{`${node.type ?? ''} ${node.label} — ${SHAPE_LABELS[node.shape]}`}</title>
      <rect
        width={node.w}
        height={node.h}
        rx={node.terminal ? 3 : 7}
        fill={active ? '#2b3138' : '#25292e'}
        stroke={highlighted ? '#fab005' : active ? '#12b886' : borderOf(node)}
        strokeWidth={highlighted || active ? 2.2 : 1.2}
        strokeDasharray={node.terminal ? '4 2' : undefined}
      />
      {glyph && (
        <g transform={`translate(6, ${(node.h - GLYPH) / 2}) scale(${GLYPH})`}>
          <path
            d={glyph}
            fill={glyphFilled(node.shape) ? '#909296' : 'none'}
            stroke="#909296"
            strokeWidth={0.07}
            vectorEffect="non-scaling-stroke"
          />
        </g>
      )}
      <text x={GLYPH + 12} y={badge ? 19 : 22} fontSize={12.5} fontWeight={600} fill="#e9ecef">
        {node.label}
      </text>
      <text x={GLYPH + 12} y={badge ? 31 : 36} fontSize={10} fill="#909296">
        {node.type}
      </text>
      {badge && (
        <text x={GLYPH + 12} y={44} fontSize={10.5} fontWeight={600} fill="#63e6be">
          {badgeText(badge)}
        </text>
      )}
      {node.ports.map((p) => {
        const armed = pendingPort?.instance === node.id && pendingPort?.port === p.port
        const color = p.lineId ? styleOf(p.lineId).color : '#495057'
        return (
          <g
            key={p.port}
            tabIndex={wiring ? 0 : undefined}
            role={wiring ? 'button' : undefined}
            aria-label={`${node.label}.${p.port} port`}
            onClick={(e) => {
              if (wiring) {
                e.stopPropagation()
                onPort(p.port)
              }
            }}
            onKeyDown={(e) => {
              if (!wiring) return
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault()
                e.stopPropagation()
                onPort(p.port)
              }
            }}
            style={{ cursor: wiring ? 'crosshair' : 'move' }}
          >
            <title>{`${node.label}.${p.port}`}</title>
            {wiring && (
              <circle cx={p.dx} cy={p.dy} r={10} fill="transparent" />
            )}
            <circle
              cx={p.dx}
              cy={p.dy}
              r={armed ? 5 : 3.2}
              fill={armed ? '#12b886' : color}
              stroke="#1a1b1e"
              strokeWidth={1}
            />
          </g>
        )
      })}
    </g>
  )
}

/** The hover/pin card: what this block is, what its ports are doing, what it
 *  computed, and what it was built from. */
function ReadoutCard({
  node,
  readout,
  pinned,
  localType = false,
  onNudge,
  onReveal,
}: Readonly<{
  node: SchematicNode
  readout: NodeReadout
  pinned: boolean
  localType?: boolean
  onNudge?: (dx: number, dy: number) => void
  onReveal?: () => void
}>) {
  const empty = readout.ports.length === 0 && readout.outputs.length === 0 && readout.params.length === 0
  return (
    <Paper
      withBorder
      shadow="md"
      p="xs"
      radius="sm"
      style={{
        position: 'absolute',
        right: 8,
        bottom: 8,
        maxWidth: 320,
        maxHeight: '70%',
        overflow: 'auto',
        pointerEvents: pinned ? 'auto' : 'none',
        zIndex: 5,
      }}
    >
      <Group gap={6} mb={4} wrap="nowrap">
        <Text size="sm" fw={700}>
          {node.label}
        </Text>
        <Text size="xs" c="dimmed">
          {node.type}
        </Text>
        {localType && (
          <Badge size="xs" variant="light" color="violet">
            local definition
          </Badge>
        )}
        {pinned && (
          <Badge size="xs" variant="light" color="teal" ml="auto">
            pinned
          </Badge>
        )}
        {onReveal && (
          <Button size="compact-xs" variant="subtle" onClick={onReveal}>
            Source
          </Button>
        )}
      </Group>
      {onNudge && (
        <Group gap={4} mb={4}>
          <Button size="compact-xs" variant="default" aria-label="Nudge up" onClick={() => onNudge(0, -8)}>
            ↑
          </Button>
          <Button size="compact-xs" variant="default" aria-label="Nudge down" onClick={() => onNudge(0, 8)}>
            ↓
          </Button>
          <Button size="compact-xs" variant="default" aria-label="Nudge left" onClick={() => onNudge(-8, 0)}>
            ←
          </Button>
          <Button size="compact-xs" variant="default" aria-label="Nudge right" onClick={() => onNudge(8, 0)}>
            →
          </Button>
          <Text size="xs" c="dimmed">
            or arrow keys
          </Text>
        </Group>
      )}

      {empty && (
        <Text size="xs" c="dimmed">
          {SHAPE_LABELS[node.shape]} — no solved values yet. Press Solve.
        </Text>
      )}

      {readout.ports.map((p) => (
        <div key={p.port}>
          <Text size="xs" fw={600} c="dimmed" mt={4}>
            {p.port}
          </Text>
          {p.readings.map((r) => (
            <Group key={r.label} gap={6} justify="space-between" wrap="nowrap">
              <Text size="xs" c="dimmed">
                {r.label}
              </Text>
              <Text size="xs" ff="monospace">
                {formatCompact(r.value)} {r.units}
              </Text>
            </Group>
          ))}
        </div>
      ))}

      {readout.outputs.length > 0 && (
        <>
          <Text size="xs" fw={600} c="teal" mt={6}>
            results
          </Text>
          {readout.outputs.map((r) => (
            <Group key={r.label} gap={6} justify="space-between" wrap="nowrap">
              <Text size="xs" c="dimmed">
                {r.label}
              </Text>
              <Text size="xs" ff="monospace">
                {formatCompact(r.value)} {r.units}
              </Text>
            </Group>
          ))}
        </>
      )}

      {readout.params.length > 0 && (
        <>
          <Text size="xs" fw={600} c="dimmed" mt={6}>
            parameters
          </Text>
          {readout.params.map((p) => (
            <Group key={p.name} gap={6} justify="space-between" wrap="nowrap" align="start">
              <Text size="xs" c="dimmed">
                {p.name}
              </Text>
              <Text size="xs" ff="monospace" ta="right">
                {p.text}
              </Text>
            </Group>
          ))}
        </>
      )}
    </Paper>
  )
}

/**
 * Pointer dragging for both the canvas (pan) and individual blocks (move).
 * Kept in one hook because they share the same pointer-capture bookkeeping and
 * the same "did this actually move?" test that stops a drag from firing the
 * click handler underneath it.
 */
function useDragging(
  zoom: number,
  offsetsRef: React.RefObject<Offsets>,
  applyOffsets: (next: Offsets) => void,
  setPan: React.Dispatch<React.SetStateAction<{ x: number; y: number }>>,
  onDragEnd: () => void,
) {
  const state = useRef<{ id: string | null; x: number; y: number; moved: boolean } | null>(null)
  const [panning, setPanning] = useState(false)

  useEffect(() => {
    const move = (e: PointerEvent) => {
      const s = state.current
      if (!s) {
        return
      }
      const dx = e.clientX - s.x
      const dy = e.clientY - s.y
      if (Math.abs(dx) + Math.abs(dy) > 3) {
        s.moved = true
      }
      s.x = e.clientX
      s.y = e.clientY
      if (s.id === null) {
        setPan((p) => ({ x: p.x + dx, y: p.y + dy }))
        return
      }
      const id = s.id
      const current = offsetsRef.current
      applyOffsets({
        ...current,
        [id]: { dx: (current[id]?.dx ?? 0) + dx / zoom, dy: (current[id]?.dy ?? 0) + dy / zoom },
      })
    }
    const up = () => {
      const s = state.current
      if (s?.id === null) {
        setPanning(false)
      } else if (s?.moved) {
        // One workspace update per drag, not per frame.
        onDragEnd()
      }
      // Keep `moved` readable for the click that fires right after pointerup.
      setTimeout(() => {
        state.current = null
      }, 0)
    }
    window.addEventListener('pointermove', move)
    window.addEventListener('pointerup', up)
    return () => {
      window.removeEventListener('pointermove', move)
      window.removeEventListener('pointerup', up)
    }
  }, [zoom, offsetsRef, applyOffsets, setPan, onDragEnd])

  return {
    panning,
    moved: () => state.current?.moved === true,
    startPan: (e: React.PointerEvent) => {
      if (e.button !== 0 || state.current) {
        return
      }
      state.current = { id: null, x: e.clientX, y: e.clientY, moved: false }
      setPanning(true)
    },
    startNode: (e: React.PointerEvent, id: string) => {
      if (e.button !== 0) {
        return
      }
      e.stopPropagation()
      state.current = { id, x: e.clientX, y: e.clientY, moved: false }
    },
  }
}

/** "a.out → b.in  (EG50 · liquid)" — the hover description of one connection. */
function edgeTitle(e: SchematicEdge): string {
  const from = endpointName(e.from, e.fromPort)
  const to = endpointName(e.to, e.toPort)
  return `${from} → ${to}  (${lineDescription(e)})`
}

function endpointName(instance: string, port?: string): string {
  return port ? `${instance}.${port}` : instance
}

/** What flows in a line: its fluid qualified by connector type, or — outside
 *  the fluid domain, where neither exists — the domain itself. */
function lineDescription(e: SchematicEdge): string {
  if (!e.fluid) {
    return e.domain
  }
  return e.connector ? `${e.fluid} · ${e.connector}` : e.fluid
}

/** Terminals (sources, sinks, grounds) get a lighter outline to go with their
 *  dashed border — they bound the model rather than being part of it. */
function borderOf(node: SchematicNode): string {
  return node.terminal ? '#6c757d' : '#4a4f55'
}

/** "q = 6444.3 W" — the one number printed on a block. */
function badgeText(badge: { label: string; value: number; units: string }): string {
  const unit = badge.units ? ` ${badge.units}` : ''
  return `${badge.label} = ${formatCompact(badge.value)}${unit}`
}

