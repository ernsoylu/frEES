// Locate the call the caret sits in and which argument is active.
//
// Component instantiations are `Type Instance(...)`. The previous scanner
// treated the instance name as the callee, so `Compressor CMP(eta=` showed no
// compressor help. Argument counting now walks the whole prefix, skips
// strings/comments, and prefers a named `foo=` argument over comma position.

export interface ActiveCall {
  /** Name used to look up a signature: the type of `Type Instance(`, else the callee. */
  name: string
  /** Instance identifier when the head is `Type Instance(`. */
  instanceName?: string
  /** 0-based top-level comma index, used when the current arg is positional. */
  argIndex: number
  /** Current argument's `name` when written `name=...`. */
  namedArg?: string
}

type Kind = 'code' | 'brace' | 'line' | 'sq' | 'dq'

function scanKind(src: string): Kind[] {
  const kind: Kind[] = new Array(src.length)
  let mode: Kind = 'code'
  for (let i = 0; i < src.length; i++) {
    const ch = src[i]
    if (mode === 'brace') {
      kind[i] = 'brace'
      if (ch === '}') mode = 'code'
      continue
    }
    if (mode === 'line') {
      kind[i] = 'line'
      if (ch === '\n') mode = 'code'
      continue
    }
    if (mode === 'sq') {
      kind[i] = 'sq'
      if (ch === "'") mode = 'code'
      continue
    }
    if (mode === 'dq') {
      // Double-quoted comments, matching the lexer: `"` opens a comment.
      kind[i] = 'dq'
      if (ch === '"') mode = 'code'
      continue
    }
    if (ch === '{') {
      kind[i] = 'brace'
      mode = 'brace'
    } else if (ch === '"') {
      kind[i] = 'dq'
      mode = 'dq'
    } else if (ch === '/' && src[i + 1] === '/') {
      kind[i] = 'line'
      mode = 'line'
    } else if (ch === "'") {
      kind[i] = 'sq'
      mode = 'sq'
    } else {
      kind[i] = 'code'
    }
  }
  return kind
}

function identAt(src: string, end: number): { start: number; name: string } | null {
  let i = end
  while (i > 0 && /[A-Za-z0-9_$]/.test(src[i - 1])) i -= 1
  if (i === end) return null
  const name = src.slice(i, end)
  if (!/^[A-Za-z_]/.test(name)) return null
  return { start: i, name }
}

function skipSpace(src: string, kind: Kind[], from: number): number {
  let i = from
  while (i > 0) {
    const k = kind[i - 1]
    if (k !== 'code') {
      i -= 1
      continue
    }
    if (/\s/.test(src[i - 1])) {
      i -= 1
      continue
    }
    break
  }
  return i
}

/** The call the caret sits inside: type/callee name + active argument. */
export function activeCallAt(doc: string, caret: number): ActiveCall | null {
  if (caret < 0) return null
  const src = doc.slice(0, Math.min(caret, doc.length))
  if (!src) return null
  const kind = scanKind(src)
  if (kind[src.length - 1] !== 'code' && kind[src.length - 1] !== undefined) {
    // Inside a comment or string: stay quiet.
    const k = kind[src.length - 1]
    if (k !== 'code') return null
  }

  let depth = 0
  let argIndex = 0
  let open = -1
  for (let i = src.length - 1; i >= 0; i--) {
    if (kind[i] !== 'code') continue
    const ch = src[i]
    if (ch === ')') depth += 1
    else if (ch === '(') {
      if (depth === 0) {
        open = i
        break
      }
      depth -= 1
    } else if (ch === ',' && depth === 0) argIndex += 1
  }
  if (open < 0) return null

  const headEnd = skipSpace(src, kind, open)
  const inst = identAt(src, headEnd)
  if (!inst) return null

  const beforeInst = skipSpace(src, kind, inst.start)
  const typeTok = identAt(src, beforeInst)

  const currentArg = src.slice(open + 1)
  const lastComma = (() => {
    let d = 0
    let at = -1
    for (let i = 0; i < currentArg.length; i++) {
      const abs = open + 1 + i
      if (kind[abs] !== 'code') continue
      const ch = currentArg[i]
      if (ch === '(' || ch === '[') d += 1
      else if (ch === ')' || ch === ']') d -= 1
      else if (ch === ',' && d === 0) at = i
    }
    return at
  })()
  const argStart = open + 1 + lastComma + 1
  let namedArg: string | undefined
  {
    let i = argStart
    while (i < src.length && (kind[i] !== 'code' || /\s/.test(src[i]))) i += 1
    const rest = src.slice(i)
    const named = /^([A-Za-z_][A-Za-z0-9_]*\$?)\s*=/.exec(rest)
    if (named) namedArg = named[1]
  }

  if (typeTok) {
    return {
      name: typeTok.name,
      instanceName: inst.name,
      argIndex,
      namedArg,
    }
  }
  return {
    name: inst.name,
    argIndex,
    namedArg,
  }
}

/** Split a usage line's parenthesized argument list on top-level commas. */
export function usageArgParts(usage: string): string[] {
  const open = usage.indexOf('(')
  const close = usage.lastIndexOf(')')
  if (open < 0 || close <= open) return []
  const inner = usage.slice(open + 1, close)
  const parts: string[] = []
  let depth = 0
  let start = 0
  for (let i = 0; i < inner.length; i++) {
    const ch = inner[i]
    if (ch === '(' || ch === '[') depth += 1
    else if (ch === ')' || ch === ']') depth -= 1
    else if (ch === ',' && depth === 0) {
      parts.push(inner.slice(start, i))
      start = i + 1
    }
  }
  parts.push(inner.slice(start))
  return parts
}

/** Parameter name of a usage part (` eta` → `eta`, ` fluid$=Water` → `fluid$`). */
export function usageArgName(part: string): string {
  const trimmed = part.trim()
  const eq = trimmed.indexOf('=')
  const head = (eq >= 0 ? trimmed.slice(0, eq) : trimmed).trim()
  return head.replace(/[^A-Za-z0-9_$].*$/, '')
}

/** Index of the argument to bold, preferring a named match over comma position. */
export function highlightArgIndex(usage: string, call: ActiveCall): number {
  if (call.namedArg) {
    const names = usageArgParts(usage).map(usageArgName)
    const i = names.findIndex((n) => n.toLowerCase() === call.namedArg!.toLowerCase())
    if (i >= 0) return i
  }
  return call.argIndex
}
