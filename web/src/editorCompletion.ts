// Contextual completions for component types, ports, public members, and
// named parameters. Inserts only the missing path segment after a trailing
// dot, or `name=` for a parameter that is not already in the call.

import { COMPONENT_CATALOG } from './componentCatalog'
import { declaredInstances } from './schematic/declaration'
import { previewDomain } from './schematic/wiring'

const FLUID_MEMBERS = ['P', 'h', 'mdot', 'T']
const HEAT_MEMBERS = ['T', 'Qdot']
const ELEC_MEMBERS = ['V', 'I']
const MECH_MEMBERS = ['w', 'tau']
const TRANS_MEMBERS = ['vel', 'F']
const SIGNAL_MEMBERS = ['sig']

export interface CompletionItem {
  label: string
  type: string
  apply: string
  info?: string
}

export interface LocalComponent {
  name: string
  ports: string[]
  params: string[]
}

export function membersForDomain(domain: string | null): string[] {
  switch (domain) {
    case 'heat':
      return HEAT_MEMBERS
    case 'electrical':
      return ELEC_MEMBERS
    case 'mechanical':
      return MECH_MEMBERS
    case 'translational':
      return TRANS_MEMBERS
    case 'signal':
      return SIGNAL_MEMBERS
    case 'fluid':
      return FLUID_MEMBERS
    default:
      return [...FLUID_MEMBERS, ...HEAT_MEMBERS, ...ELEC_MEMBERS]
  }
}

function paramNames(list: string): string[] {
  return list
    .split(',')
    .map((part) => part.trim().split(/\s*=/)[0]?.trim() ?? '')
    .filter((name) => /^[A-Za-z_][\w$]*$/.test(name))
}

/** User `COMPONENT` / `SUBSYSTEM` blocks: ports from the header, params from PARAM. */
export function localComponentNames(text: string): LocalComponent[] {
  const out: LocalComponent[] = []
  const re = /^\s*(?:COMPONENT|SUBSYSTEM)\s+(\w+)\s*\(([^)]*)\)/gim
  let m: RegExpExecArray | null
  while ((m = re.exec(text)) !== null) {
    const name = m[1]
    const ports = paramNames(m[2])
    const after = text.slice(m.index + m[0].length)
    const end = after.search(/^\s*END\b/im)
    const body = end >= 0 ? after.slice(0, end) : after
    const params: string[] = []
    for (const line of body.split('\n')) {
      const param = /^\s*PARAM\s+(.+)$/i.exec(line)
      if (param) params.push(...paramNames(param[1]))
    }
    out.push({ name, ports, params })
  }
  return out
}

export function localSignature(
  text: string,
  typeName: string,
): { usage: string; detail: string } | null {
  const local = localComponentNames(text).find((c) => c.name.toLowerCase() === typeName.toLowerCase())
  if (!local) return null
  const args = [...local.ports, ...local.params.map((p) => `${p}=`)]
  return {
    usage: `${local.name} Instance(${args.join(', ')})`,
    detail: 'Local component definition',
  }
}

function catalogArgs(typeName: string): { ports: string[]; params: string[] } | null {
  const spec = COMPONENT_CATALOG.find((c) => c.type.toLowerCase() === typeName.toLowerCase())
  if (!spec) return null
  return { ports: spec.ports, params: spec.params.map((p) => p.name) }
}

/** Completions for the identifier (or dotted path) ending at `prefix`. */
export function completionsForPrefix(text: string, prefix: string): CompletionItem[] | null {
  const dotted = /^(.*)\.([A-Za-z_][\w]*)?$/.exec(prefix)
  if (!dotted) return null
  const head = dotted[1]
  const parts = head.split('.')
  const instances = declaredInstances(text)
  const locals = localComponentNames(text)

  if (parts.length === 1) {
    const inst = instances.get(parts[0].toLowerCase())
    if (!inst) return null
    const spec = catalogArgs(inst.type)
    const local = locals.find((l) => l.name.toLowerCase() === inst.type.toLowerCase())
    const ports = spec?.ports ?? local?.ports ?? []
    return ports.map((p) => ({
      label: p,
      type: 'property',
      apply: p,
      info: `${inst.type} port`,
    }))
  }
  if (parts.length === 2) {
    const inst = instances.get(parts[0].toLowerCase())
    if (!inst) return null
    const domain = previewDomain(inst.type, parts[1])
    return membersForDomain(domain).map((m) => ({
      label: m,
      type: 'property',
      apply: m,
      info: `${inst.label}.${parts[1]} member`,
    }))
  }
  return null
}

export function localTypeCompletions(text: string): CompletionItem[] {
  return localComponentNames(text).map((c) => ({
    label: c.name,
    type: 'class',
    apply: `${c.name} `,
    info: 'Local component definition',
  }))
}

/** Named parameters of `Type Instance(` that are not already present. */
export function parameterCompletions(
  text: string,
  typeName: string,
  already: ReadonlySet<string>,
): CompletionItem[] {
  const spec = catalogArgs(typeName)
  const local = localComponentNames(text).find((c) => c.name.toLowerCase() === typeName.toLowerCase())
  const names = local?.params ?? spec?.params ?? []
  const localNote = local ? ' (local)' : ''
  return names
    .filter((name) => !already.has(name.toLowerCase()))
    .map((name) => ({
      label: name,
      type: 'property',
      apply: `${name}=`,
      info: `${typeName} parameter${localNote}`,
    }))
}

export function namedArgsAlreadyPresent(callText: string): Set<string> {
  const used = new Set<string>()
  for (const m of callText.matchAll(/([A-Za-z_][\w$]*)\s*=/g)) {
    used.add(m[1].toLowerCase())
  }
  return used
}
