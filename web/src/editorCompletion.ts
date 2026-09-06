// Contextual completions for component types, ports, public members, and
// named parameters. Inserts only the missing path segment after a trailing
// dot, or `name=` for a parameter that is not already in the call.

import { COMPONENT_CATALOG } from './componentCatalog'
import { declaredInstances } from './schematic/declaration'
import { previewDomain } from './schematic/wiring'

const DOMAIN_MEMBERS: Record<string, string[]> = {
  heat: ['T', 'Qdot'],
  electrical: ['V', 'I'],
  mechanical: ['w', 'tau'],
  translational: ['vel', 'F'],
  signal: ['sig'],
  fluid: ['P', 'h', 'mdot', 'T'],
}

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
  return DOMAIN_MEMBERS[domain ?? ''] ?? [...DOMAIN_MEMBERS.fluid, ...DOMAIN_MEMBERS.heat, ...DOMAIN_MEMBERS.electrical]
}

function paramNames(list: string): string[] {
  return list
    .split(',')
    .map((part) => part.trim().split(/\s*=/)[0]?.trim() ?? '')
    .filter((name) => /^[A-Za-z_][\w$]*$/.test(name))
}

function items(labels: string[], apply: (name: string) => string, info: string): CompletionItem[] {
  return labels.map((label) => ({ label, type: 'property', apply: apply(label), info }))
}

function findLocal(text: string, typeName: string): LocalComponent | undefined {
  return localComponentNames(text).find((c) => c.name.toLowerCase() === typeName.toLowerCase())
}

/** User `COMPONENT` / `SUBSYSTEM` blocks: ports from the header, params from PARAM. */
export function localComponentNames(text: string): LocalComponent[] {
  const out: LocalComponent[] = []
  const re = /^\s*(?:COMPONENT|SUBSYSTEM)\s+(\w+)\s*\(([^)]*)\)/gim
  let m: RegExpExecArray | null
  while ((m = re.exec(text)) !== null) {
    const after = text.slice(m.index + m[0].length)
    const end = after.search(/^\s*END\b/im)
    const params: string[] = []
    for (const line of (end >= 0 ? after.slice(0, end) : after).split('\n')) {
      const param = /^\s*PARAM\s+(.+)$/i.exec(line)
      if (param) params.push(...paramNames(param[1]))
    }
    out.push({ name: m[1], ports: paramNames(m[2]), params })
  }
  return out
}

export function localSignature(
  text: string,
  typeName: string,
): { usage: string; detail: string } | null {
  const local = findLocal(text, typeName)
  if (!local) return null
  return {
    usage: `${local.name} Instance(${[...local.ports, ...local.params.map((p) => `${p}=`)].join(', ')})`,
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
  const parts = dotted[1].split('.')
  const inst = declaredInstances(text).get(parts[0].toLowerCase())
  if (!inst) return null
  if (parts.length === 1) {
    const ports = catalogArgs(inst.type)?.ports ?? findLocal(text, inst.type)?.ports ?? []
    return items(ports, (p) => p, `${inst.type} port`)
  }
  if (parts.length === 2) {
    return items(membersForDomain(previewDomain(inst.type, parts[1])), (m) => m, `${inst.label}.${parts[1]} member`)
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
  const local = findLocal(text, typeName)
  const names = (local?.params ?? catalogArgs(typeName)?.params ?? []).filter(
    (name) => !already.has(name.toLowerCase()),
  )
  return items(names, (name) => `${name}=`, `${typeName} parameter${local ? ' (local)' : ''}`)
}

export function namedArgsAlreadyPresent(callText: string): Set<string> {
  const used = new Set<string>()
  for (const m of callText.matchAll(/([A-Za-z_][\w$]*)\s*=/g)) {
    used.add(m[1].toLowerCase())
  }
  return used
}
