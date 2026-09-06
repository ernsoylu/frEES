// Locating a component instance's declaration in the document text, so
// clicking a node in the schematic can reveal the line that created it.
// Text-scanning (the same approach the PID loop analysis uses) rather than a
// backend line number: the AST carries no source positions, and the text is
// right here.

/**
 * Blanks every `{ ... }` free-text comment, keeping the line structure intact
 * (line count and column positions are preserved, so a 1-based line number
 * computed on the result still points at the right line of the original).
 *
 * frees comments SPAN LINES, and the schematic used to strip them with a
 * per-line regex — so prose inside a block comment was scanned as code. Two
 * ordinary English words followed by a parenthesis look exactly like a
 * component instantiation, which is how a document that describes its own
 * "Coolant line (EG50)" and its "radiator pipe (three wall-HX cells)" put
 * phantom `line` and `pipe` components on the canvas.
 */
export function stripComments(text: string): string {
  let out = ''
  let inBlock = false
  for (const rawLine of text.split('\n')) {
    let line = ''
    for (let i = 0; i < rawLine.length; i++) {
      const ch = rawLine[i]
      if (inBlock) {
        // Blank the comment body, but keep the character count.
        line += ch === '}' ? ((inBlock = false), ' ') : ' '
      } else if (ch === '{') {
        inBlock = true
        line += ' '
      } else if (ch === '/' && rawLine[i + 1] === '/') {
        break // `//` comments run to end of line
      } else {
        line += ch
      }
    }
    out += line + '\n'
  }
  return out.slice(0, -1)
}

/**
 * The 1-based line declaring `instance`, or null when it isn't found.
 * Component instantiation is `Type Instance(params…)`, so the instance name
 * is the second identifier on its line. Matching is case-insensitive (frees
 * names are), skips comments — including a name mentioned inside a multi-line
 * `{ … }` block — and ignores a match inside a `connect(…)` or an equation,
 * where the name appears as a reference rather than a declaration.
 */
/** Prefer Check identity lines when present; nested names are scoped (`a.inner`). */
export function declarationLineFromCheck(
  instances: readonly { name: string; label: string; line: number }[] | undefined,
  instance: string,
): number | null {
  if (!instances || !instance) {
    return null
  }
  const key = instance.toLowerCase()
  const hit = instances.find(
    (i) => i.name.toLowerCase() === key || i.label.toLowerCase() === key,
  )
  return hit && hit.line > 0 ? hit.line : null
}

export function declarationLine(text: string, instance: string): number | null {
  if (!text || !instance) {
    return null
  }
  const escaped = instance.replace(/[.*+?^${}()|[\]\\]/g, String.raw`\$&`)
  const declaration = new RegExp(String.raw`^\s*[A-Za-z_][\w$]*\s+` + escaped + String.raw`\s*\(`, 'i')
  // Comment blanking preserves line structure, so indices still line up with
  // the original document the caller will scroll to.
  const lines = stripComments(text).split('\n')
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    if (/^\s*connect\s*\(/i.test(line)) {
      continue
    }
    if (declaration.test(line)) {
      return i + 1
    }
  }
  return null
}

/** Statement keywords that open a block or bind a name — never a component. */
const BLOCK_KEYWORDS =
  /^(connect|component|function|procedure|module|table|parametric|dynamic|linearize|plot|state|subsystem|variant|param|end|if|repeat|duplicate|guess|limits)\b/i

/**
 * Instance name → component type, read straight from the document.
 *
 * The solve response also carries this, but only after a network solves —
 * and a network being wired is exactly one that does not yet solve. The text
 * is the source of truth and is always available, so wiring reads from it.
 *
 * `knownTypes` (lowercased component type names — the catalog plus any
 * `COMPONENT` blocks the document declares) filters the result down to things
 * that really are components. Two identifiers and a paren is a shape English
 * prose hits by accident, so without this filter the canvas grows nodes for
 * phrases. Omit it to accept any well-formed declaration.
 */
export function instanceTypes(text: string, knownTypes?: ReadonlySet<string>): Map<string, string> {
  const out = new Map<string, string>()
  for (const [id, hit] of declaredInstances(text, knownTypes)) {
    out.set(id, hit.type)
  }
  return out
}

/**
 * Instances the document declares, keyed by canonical (lowercase) name, each
 * with the spelling the document actually used.
 *
 * The written spelling matters: a solve reports it, but a document that has
 * only been checked has not produced one yet — and a drawing that labels the
 * pump `pump` until you solve looks like it lost information it never had.
 * See {@link instanceTypes} for the `knownTypes` filter.
 */
export function declaredInstances(
  text: string,
  knownTypes?: ReadonlySet<string>,
): Map<string, { label: string; type: string }> {
  const out = new Map<string, { label: string; type: string }>()
  if (!text) {
    return out
  }
  for (const rawLine of stripComments(text).split('\n')) {
    const line = rawLine.trim()
    if (BLOCK_KEYWORDS.test(line)) {
      continue
    }
    // `Type Name(...)` — two identifiers then an open paren.
    const m = /^([A-Za-z_]\w*)\s+([A-Za-z_]\w*)\s*\(/.exec(line)
    if (m && (!knownTypes || knownTypes.has(m[1].toLowerCase()))) {
      out.set(m[2].toLowerCase(), { label: m[2], type: m[1] })
    }
  }
  return out
}

/** Component types the document defines itself with `COMPONENT Name(...)`,
 *  lowercased — these are as real as the catalog's, just local. */
export function declaredComponentTypes(text: string): Set<string> {
  const out = new Set<string>()
  for (const line of stripComments(text).split('\n')) {
    const m = /^\s*(?:component|subsystem)\s+([a-z_]\w*)/i.exec(line)
    if (m) {
      out.add(m[1].toLowerCase())
    }
  }
  return out
}
