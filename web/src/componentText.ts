// Converts a Component Browser/Wizard selection into the frees component
// instantiation line that gets injected into the equation editor, e.g.
//   Chiller CHLR1(ref$=R1234yf, cool$=EG50, U_tp=3000 [W/m^2-K], ...)
//
// Units are appended only to a plain numeric literal. An explicit annotation,
// variable, or expression is preserved as source text so `UA=conductance` does
// not become `UA=conductance [W/K]` (which the parser reads as extra unknowns
// `w` and `k` through array syntax).
import type { ComponentSpec, ComponentParam } from './componentCatalog'

export type ParamValues = Record<string, string>

/** Signed/scientific numeric literal the lexer accepts as a NUMBER token. */
const NUMERIC_LITERAL = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$/

/** `10 [W/K]`, `-2.5e-3[degC]` — a numeric literal that already carries units. */
const UNIT_ANNOTATED_LITERAL = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?\s*\[[^\]]+]$/

/** Bare identifier, including a trailing `$` string name. */
const IDENT = /^[A-Za-z_][A-Za-z0-9_]*\$?$/

/** Dotted member path (`HX.in.P`, `conductance`). */
const MEMBER_PATH = /^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*\$?$/

/** The model$ variant currently in effect: the chosen value, else the engine's
 *  declared default, else the first documented variant. */
export function selectedVariant(spec: ComponentSpec, values: ParamValues): string | null {
  if (spec.variants.length === 0) return null
  const chosen = (values['model$'] ?? '').trim()
  if (chosen) return chosen
  const selector = spec.params.find((p) => p.isSelector)
  const declared = (selector?.defaultValue ?? '').trim()
  if (declared) return declared
  return spec.variants[0].name
}

/** Params relevant to the current selection: shared params (variants: []) plus
 *  the variant-specific params required by the active model$. Mirrors the backend
 *  rule (ComponentExpander.resolve): a param required only by other variants is
 *  inactive (optional / hidden). */
export function activeParams(spec: ComponentSpec, values: ParamValues): ComponentParam[] {
  const variant = selectedVariant(spec, values)
  return spec.params.filter((p) => p.variants.length === 0 || (variant !== null && p.variants.includes(variant)))
}

/** Inactive params that still have a form value — kept as drafts, omitted from
 *  newly generated source. */
export function inactiveDraftParams(spec: ComponentSpec, values: ParamValues): string[] {
  const active = new Set(activeParams(spec, values).map((p) => p.name))
  return spec.params
    .filter((p) => !active.has(p.name) && (values[p.name] ?? '').trim() !== '')
    .map((p) => p.name)
}

/** True when `raw` is a plain numeric literal the parser would accept. */
export function isPlainNumericLiteral(raw: string): boolean {
  return NUMERIC_LITERAL.test(raw.trim())
}

/** True when `raw` is a numeric literal that already has a `[unit]` suffix. */
export function isUnitAnnotatedLiteral(raw: string): boolean {
  return UNIT_ANNOTATED_LITERAL.test(raw.trim())
}

/** Quote a string parameter when the value is not a bare identifier. Already
 *  single-quoted values are left alone. Double quotes are comments in frees. */
export function formatStringValue(raw: string): string {
  const v = raw.trim()
  if (v === '') return v
  if (/^'[^']*'$/.test(v)) return v
  if (IDENT.test(v)) return v
  return `'${v.replace(/'/g, "\\'")}'`
}

function formatNumericValue(raw: string, unit: string): string {
  const v = raw.trim()
  if (unit && isPlainNumericLiteral(v)) return `${v} [${unit}]`
  return v
}

/** Build the single-line component instantiation text. Only active params are
 *  emitted (so a stale value from an unselected variant is dropped); empty values
 *  are skipped (an unset optional/selector param uses the std-library default). */
export function generateComponentText(
  spec: ComponentSpec,
  instanceName: string,
  values: ParamValues,
): string {
  const parts: string[] = []
  for (const p of activeParams(spec, values)) {
    const raw = (values[p.name] ?? '').trim()
    if (raw === '') continue
    if (p.isString) {
      parts.push(`${p.name}=${formatStringValue(raw)}`)
    } else {
      parts.push(`${p.name}=${formatNumericValue(raw, p.unit)}`)
    }
  }
  const name = instanceName.trim() || suggestInstanceName(spec.type)
  return `${spec.type} ${name}(${parts.join(', ')})`
}

/** Assemble the full editor block: any preamble lines (correlation helpers, a
 *  TABLE block) followed by the component line, ready for insertStatement. */
export function assembleBlock(preamble: string[], componentLine: string): string {
  return [...preamble, componentLine].filter((l) => l.trim() !== '').join('\n')
}

/** Legal frees identifier for an instance name (letters/digits/underscore,
 *  not starting with a digit). */
export function isValidInstanceName(name: string): boolean {
  return /^[A-Za-z_]\w*$/.test(name.trim())
}

export function instanceNameError(name: string, taken: Iterable<string> = []): string | null {
  const trimmed = name.trim()
  if (!trimmed) return 'An instance name is required.'
  if (!/^[A-Za-z_]\w*$/.test(trimmed)) {
    return 'Must be a valid identifier (letters, digits, underscore; not starting with a digit).'
  }
  const lower = trimmed.toLowerCase()
  for (const existing of taken) {
    if (existing.trim().toLowerCase() === lower) {
      return `'${trimmed}' collides with existing instance '${existing.trim()}' (names are case-insensitive).`
    }
  }
  return null
}

/** A short, editable default instance name derived from the component type:
 *  prefer its capital letters (MovingBoundaryEvaporator → MBE), else the first
 *  four characters uppercased (Chiller → CHIL). When `taken` is given, a
 *  numeric suffix is added until the name is free under case-insensitive match. */
export function suggestInstanceName(type: string, taken: Iterable<string> = []): string {
  const caps = type.replace(/[^A-Z]/g, '')
  const base = (caps.length >= 2 ? caps : type.slice(0, 4)).toUpperCase().replace(/[^A-Z0-9]/g, '') || 'C'
  const used = new Set([...taken].map((n) => n.trim().toLowerCase()).filter(Boolean))
  if (!used.has(base.toLowerCase())) return base
  let n = 2
  while (used.has(`${base}${n}`.toLowerCase())) n += 1
  return `${base}${n}`
}

function bracketsBalanced(s: string): boolean {
  let paren = 0
  let square = 0
  for (const ch of s) {
    if (ch === '(') paren += 1
    else if (ch === ')') paren -= 1
    else if (ch === '[') square += 1
    else if (ch === ']') square -= 1
    if (paren < 0 || square < 0) return false
  }
  return paren === 0 && square === 0
}

/** A field-level syntax check. Does not ask whether the whole network solves:
 *  an unwired component is a valid insertion. */
export function paramValueError(param: ComponentParam, raw: string): string | null {
  const v = raw.trim()
  if (v === '') return param.required ? 'Required.' : null
  if (param.isSelector) {
    if (param.values.length === 0) return null
    const hit = param.values.some((opt) => opt.toLowerCase() === v.toLowerCase())
    return hit ? null : `Unknown variant '${v}'.`
  }
  if (param.isString) {
    if (/^'[^']*'$/.test(v)) return null
    if (IDENT.test(v)) return null
    // Custom fluid/table names (`INCOMP::MEG[0.50]`) are quoted on emit.
    if (/^[A-Za-z0-9_:.[\]%-]+$/.test(v) && bracketsBalanced(v)) return null
    return 'Not a valid string. Use a name or a single-quoted literal.'
  }
  if (isPlainNumericLiteral(v) || isUnitAnnotatedLiteral(v)) return null
  if (MEMBER_PATH.test(v)) return null
  // Arithmetic / call expression: accept a conservative character set with
  // balanced brackets so `2*UA` and `ua_hx(1,2,3,4,5)` insert, but `10 [W/K] [`
  // and a trailing operator do not look valid.
  if (/^[A-Za-z0-9_.$+\-*/^(), [\]]+$/.test(v) && bracketsBalanced(v) && !/[+\-*/^,(]$/.test(v)) {
    return null
  }
  return 'Not a valid value or expression.'
}

/** Which required params are still empty — used to gate the Add button. Only
 *  considers params active for the current variant. */
export function missingRequiredParams(spec: ComponentSpec, values: ParamValues): string[] {
  return activeParams(spec, values)
    .filter((p) => p.required && (values[p.name] ?? '').trim() === '')
    .map((p) => p.name)
}

/** Per-field errors for the current variant. Empty map → the form is insertable
 *  as far as field syntax goes (instance name is checked separately). */
export function paramValueErrors(spec: ComponentSpec, values: ParamValues): Record<string, string> {
  const out: Record<string, string> = {}
  for (const p of activeParams(spec, values)) {
    const err = paramValueError(p, values[p.name] ?? '')
    if (err) out[p.name] = err
  }
  return out
}

/** Known fluids the wizard offers; custom strings remain allowed. */
export const KNOWN_FLUIDS = [
  'Water',
  'Air',
  'CO2',
  'R134a',
  'R1234yf',
  "INCOMP::MEG[0.50]",
  "INCOMP::MPG[0.50]",
]
