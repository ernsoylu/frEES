// Reading and writing in-text GUESS directives, so the Variable Information
// window and the document stop being two sources of truth for the same thing.
//
// The backend already treats the text as authoritative (its parser merges
// GUESS over the window's values, text winning). These helpers are the other
// half: show the user what the document says, and let them push what they
// typed back into it — after which the values travel with the file, diff, and
// survive copy-paste, which values living only in a modal never do.

export interface GuessDirective {
  /** Name as written in the document. */
  name: string
  guess: number | null
  lower: number | null
  upper: number | null
  /** 0-based line index, so a rewrite can replace it in place. */
  line: number
}

const NUM = String.raw`[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?`
const GUESS_LINE = new RegExp(
  String.raw`^\s*GUESS\s+([A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*)\s*(?:=\s*(` + NUM + String.raw`))?` +
    String.raw`\s*(?:\[\s*(` + NUM + String.raw`)\s*,\s*(` + NUM + String.raw`)\s*\])?\s*$`,
  'i',
)

/** Every GUESS directive in the document, in source order. */
export function readGuessDirectives(text: string): GuessDirective[] {
  const out: GuessDirective[] = []
  if (!text) {
    return out
  }
  text.split('\n').forEach((line, index) => {
    const m = GUESS_LINE.exec(line)
    if (!m) {
      return
    }
    const [, name, guess, lower, upper] = m
    // A bare `GUESS x` declares nothing; the parser rejects it, so it is not
    // reported as a directive here either.
    if (guess === undefined && lower === undefined) {
      return
    }
    out.push({
      name,
      guess: guess === undefined ? null : Number(guess),
      lower: lower === undefined ? null : Number(lower),
      upper: upper === undefined ? null : Number(upper),
      line: index,
    })
  })
  return out
}

/** The directive text for one variable, or null when there is nothing to say. */
export function formatGuessDirective(
  name: string,
  guess: number | null,
  lower: number | null,
  upper: number | null,
): string | null {
  const hasBounds = lower !== null && upper !== null && Number.isFinite(lower) && Number.isFinite(upper)
  const hasGuess = guess !== null && Number.isFinite(guess)
  if (!hasGuess && !hasBounds) {
    return null
  }
  const head = hasGuess ? `GUESS ${name} = ${guess}` : `GUESS ${name}`
  return hasBounds ? `${head} [${lower}, ${upper}]` : head
}

/**
 * Writes the given directives into the document: an existing GUESS line for a
 * variable is replaced in place (so the user's ordering and surrounding
 * comments survive), a new one is appended, and a variable whose directive is
 * now empty has its line removed. Everything else in the text is untouched.
 */
export function writeGuessDirectives(
  text: string,
  entries: readonly { name: string; guess: number | null; lower: number | null; upper: number | null }[],
): string {
  const lines = text.split('\n')
  const existing = new Map<string, GuessDirective>()
  for (const d of readGuessDirectives(text)) {
    existing.set(d.name.toLowerCase(), d)
  }

  const dropped = new Set<number>()
  const appended: string[] = []
  for (const entry of entries) {
    const directive = formatGuessDirective(entry.name, entry.guess, entry.lower, entry.upper)
    const prior = existing.get(entry.name.toLowerCase())
    if (prior && directive) {
      lines[prior.line] = directive
    } else if (prior) {
      dropped.add(prior.line)
    } else if (directive) {
      appended.push(directive)
    }
  }

  const kept = lines.filter((_, i) => !dropped.has(i))
  if (appended.length === 0) {
    return kept.join('\n')
  }
  // Appended directives go at the end, after a blank separator line, so they
  // read as a block rather than colliding with the last equation.
  const body = kept.join('\n').replace(/\s+$/, '')
  return `${body}\n\n${appended.join('\n')}\n`
}
