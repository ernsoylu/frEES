import { helpUrl } from './helpUrl'
import { forwardRef, useEffect, useImperativeHandle, useMemo, useRef, useState } from 'react'
import { useComputedColorScheme } from '@mantine/core'
import CodeMirror, { ReactCodeMirrorRef } from '@uiw/react-codemirror'
import { Decoration, DecorationSet, EditorView, keymap, showTooltip, Tooltip } from '@codemirror/view'
import { Diagnostic, lintGutter, setDiagnostics } from '@codemirror/lint'
import { REFERENCE_SLUGS } from './docsTopics'
import { Extension, StateEffect, StateField } from '@codemirror/state'
import { HighlightStyle, StreamLanguage, StringStream, syntaxHighlighting } from '@codemirror/language'
import { CompletionContext, CompletionResult } from '@codemirror/autocomplete'
import { tags } from '@lezer/highlight'
import { catalogFunctionNames, FUNCTION_CATEGORIES } from './functionCatalog'
import { COMPONENT_NAMES } from './componentNames'
import { activeCallAt, highlightArgIndex } from './signatureHelp'

// Imperative handle the parent uses to drive the editor (insert at caret, jump
// to a line) without reaching into the DOM, mirroring the old textareaRef ops.
export interface EquationEditorHandle {
  insertSnippet: (snippet: string) => void
  /** Append `text` as its own line at the end of the document, guaranteeing a
   *  line break before and after so repeated calls each land on a fresh line. */
  insertStatement: (text: string, opts?: { focus?: boolean }) => void
  /** Replace the whole document (project load, examples, generated equations).
   *  Does NOT fire onChange — the caller already holds the new text. */
  setDoc: (text: string) => void
  goToLine: (line: number) => void
  focus: () => void
}

// frees keywords (block/control-flow) highlighted distinctly from functions.
const KEYWORDS = new Set([
  'FOR', 'TO', 'STEP', 'WHILE', 'DO', 'REPEAT', 'UNTIL', 'IF', 'THEN', 'ELSE',
  'END', 'FUNCTION', 'PROCEDURE', 'MODULE', 'CALL', 'PARAMETRIC', 'TABLE',
  'PLOT', 'DUPLICATE', 'AND', 'OR', 'NOT', 'DYNAMIC', 'STATE', 'EVENT',
  'SYMBOLIC',
  'COMPONENT', 'PARAM', 'VARIANT', 'REQUIRE', 'CONNECT', 'LINEARIZE', 'GUESS',
  'INPUT', 'OUTPUT',
])

// Built-in function names from the Functions-menu catalog (callee of each CALL
// snippet, e.g. `CALL lqr(...)` -> `lqr`, otherwise the leading identifier,
// minus block scaffolds). Used for syntax highlighting and autocomplete — so
// typing `CALL lq` completes `lqr`.
const FUNCTION_NAMES = [...catalogFunctionNames()]
const FUNCTION_SET = new Set(FUNCTION_NAMES.map((n) => n.toLowerCase()))

// Component types from the generated names companion (never the full catalog —
// that would drag every parameter table and markdown body into this chunk).
// Completing `Resis` yields `Resistor ` ready for the instance name, with the
// one-line summary as the popup info.
const COMPONENT_COMPLETIONS = COMPONENT_NAMES.map((c) => ({
  label: c.name,
  type: 'class',
  apply: `${c.name} `,
  info: c.summary || undefined,
}))

// ---------------------------------------------------------------------------
// Signature help: a tooltip above the caret showing the call's usage line with
// the active argument bold, for every catalog function and component type.

interface SignatureInfo {
  usage: string
  detail?: string
}

/** name (lowercased) → usage line, from the function catalog (keyed the same
 *  way catalogFunctionNames derives names) and the component names companion. */
const SIGNATURES: Map<string, SignatureInfo> = (() => {
  const map = new Map<string, SignatureInfo>()
  for (const category of FUNCTION_CATEGORIES) {
    for (const item of category.items) {
      const call = /^CALL\s+([A-Za-z_][A-Za-z0-9_]*)/.exec(item.snippet)
      const name = call ? call[1] : /^([A-Za-z_][A-Za-z0-9_]*\$?)/.exec(item.snippet)?.[1]
      if (name && item.usage && !map.has(name.toLowerCase())) {
        map.set(name.toLowerCase(), { usage: item.usage, detail: item.description })
      }
    }
  }
  for (const c of COMPONENT_NAMES) {
    map.set(c.name.toLowerCase(), { usage: c.signature, detail: c.summary })
  }
  return map
})()



/** DOM for the tooltip: usage line with the active argument bold (when the
 *  usage's parenthesis list parses), plus a dimmed one-line detail. */
function renderSignature(sig: SignatureInfo, argIndex: number): HTMLElement {
  const root = document.createElement('div')
  root.className = 'cm-signature-help'
  const line = document.createElement('div')
  const open = sig.usage.indexOf('(')
  const close = sig.usage.lastIndexOf(')')
  if (open >= 0 && close > open) {
    line.appendChild(document.createTextNode(sig.usage.slice(0, open + 1)))
    // Split the argument list on top-level commas only (CALL usages carry
    // an output section after ':' — bolding stays within the input args).
    const inner = sig.usage.slice(open + 1, close)
    const parts: string[] = []
    let depth = 0
    let start = 0
    for (let i = 0; i < inner.length; i++) {
      const ch = inner[i]
      if (ch === '(' || ch === '[') depth++
      else if (ch === ')' || ch === ']') depth--
      else if (ch === ',' && depth === 0) {
        parts.push(inner.slice(start, i))
        start = i + 1
      }
    }
    parts.push(inner.slice(start))
    parts.forEach((part, i) => {
      if (i > 0) line.appendChild(document.createTextNode(','))
      const span = document.createElement(i === argIndex ? 'b' : 'span')
      span.textContent = part
      line.appendChild(span)
    })
    line.appendChild(document.createTextNode(sig.usage.slice(close)))
  } else {
    line.textContent = sig.usage
  }
  root.appendChild(line)
  if (sig.detail) {
    const detail = document.createElement('div')
    detail.className = 'cm-signature-detail'
    detail.textContent = sig.detail
    root.appendChild(detail)
  }
  return root
}

const signatureField = StateField.define<Tooltip | null>({
  create: () => null,
  update(value, tr) {
    if (!tr.docChanged && !tr.selection) return value
    const state = tr.state
    const caret = state.selection.main.head
    if (!state.selection.main.empty) return null
    const call = activeCallAt(state.sliceDoc(0, caret), caret)
    if (!call) return null
    const sig = SIGNATURES.get(call.name.toLowerCase())
    if (!sig) return null
    return {
      pos: caret,
      above: true,
      create: () => ({ dom: renderSignature(sig, highlightArgIndex(sig.usage, call)) }),
    }
  },
  provide: (field) => showTooltip.from(field),
})

interface StreamState {
  /** Closer of an unclosed `{…}` or `"…"` comment; null when in code. */
  commentClose: '}' | '"' | null
}

/** Consumes the remainder of an open comment, clearing the flag at its closer. */
function continueComment(stream: StringStream, state: StreamState): string {
  const close = state.commentClose
  while (!stream.eol()) {
    if (stream.next() === close) {
      state.commentClose = null
      break
    }
  }
  return 'comment'
}

/** Starts a `{…}` or `"…"` comment; sets the multi-line flag if it does not close. */
function startComment(stream: StringStream, state: StreamState, close: '}' | '"'): string {
  stream.next()
  while (!stream.eol()) {
    if (stream.next() === close) return 'comment'
  }
  state.commentClose = close
  return 'comment'
}

/** Consumes a quoted string literal up to its closing quote {@code ch}. */
function scanString(stream: StringStream, ch: string): string {
  stream.next()
  while (!stream.eol()) {
    if (stream.next() === ch) break
  }
  return 'string'
}

/** Classifies an identifier word as a keyword, known function, or unstyled variable. */
function scanWord(stream: StringStream): string | null {
  stream.match(/^[A-Za-z_][A-Za-z0-9_]*\$?/)
  const word = stream.current()
  if (KEYWORDS.has(word.toUpperCase())) return 'keyword'
  if (FUNCTION_SET.has(word.toLowerCase())) return 'function'
  return null
}

// A small line-oriented tokenizer for the frees language: {comments}, string
// literals, numbers, keywords, and known built-in functions. Unknown
// identifiers (user variables) are left unstyled.
// Exported for other frees-DSL inputs (the REPL terminal, the component
// wizard) — anywhere a field takes frees source rather than a bare number.
export const freesLanguage = StreamLanguage.define<StreamState>({
  startState: () => ({ commentClose: null }),
  token(stream, state) {
    if (state.commentClose) return continueComment(stream, state)
    if (stream.eatSpace()) return null
    if (stream.eol()) return null

    const ch = stream.peek() ?? ''
    if (ch === '{') return startComment(stream, state, '}')
    // Double quotes delimit comments in frees, not strings. Strings are `'…'`.
    if (ch === '"') return startComment(stream, state, '"')
    if (ch === "'") return scanString(stream, ch)
    if (/\d/.test(ch) || (ch === '.' && /\d/.test(stream.string.charAt(stream.pos + 1)))) {
      if (!stream.match(/^\d*\.?\d+([eE][+-]?\d+)?[ij]?/)) stream.next()
      return 'number'
    }
    if (/[A-Za-z_]/.test(ch)) return scanWord(stream)
    if (/[+\-*/^=<>:|,~]/.test(ch)) {
      stream.next()
      return 'operator'
    }
    stream.next()
    return null
  },
  tokenTable: {
    comment: tags.comment,
    string: tags.string,
    number: tags.number,
    keyword: tags.keyword,
    function: tags.function(tags.variableName),
    operator: tags.operator,
  },
})

// Syntax palette for dark mode (bright tokens on a dark background).
export const freesHighlightDark = HighlightStyle.define([
  { tag: tags.comment, color: '#7d8590', fontStyle: 'italic' },
  { tag: tags.string, color: '#38d9a9' },
  { tag: tags.number, color: '#ffa94d' },
  { tag: tags.keyword, color: '#da77f2', fontWeight: 'bold' },
  { tag: tags.function(tags.variableName), color: '#74c0fc' },
  { tag: tags.operator, color: '#ced4da' },
])

// Light-mode counterpart: darker, higher-contrast tokens that stay legible on a
// white background (the bright dark-mode colours wash out on light).
export const freesHighlightLight = HighlightStyle.define([
  { tag: tags.comment, color: '#6e7781', fontStyle: 'italic' },
  { tag: tags.string, color: '#0a7c5a' },
  { tag: tags.number, color: '#b35900' },
  { tag: tags.keyword, color: '#9c36b5', fontWeight: 'bold' },
  { tag: tags.function(tags.variableName), color: '#1971c2' },
  { tag: tags.operator, color: '#495057' },
])

// Build an editor theme for the active colour scheme so the editor blends with
// the surrounding Mantine surface in both light and dark mode.
function makeFreesTheme(dark: boolean) {
  return EditorView.theme(
    {
      '&': {
        backgroundColor: dark
          ? 'var(--mantine-color-dark-7)'
          : 'var(--mantine-color-white)',
        color: dark ? 'var(--mantine-color-dark-0)' : 'var(--mantine-color-gray-9)',
        fontSize: 'var(--mantine-font-size-sm)',
        height: '100%',
      },
      '.cm-content': {
        fontFamily: 'var(--mantine-font-family-monospace)',
        caretColor: dark ? 'var(--mantine-color-dark-0)' : 'var(--mantine-color-gray-9)',
      },
      '.cm-scroller': {
        fontFamily: 'var(--mantine-font-family-monospace)',
        lineHeight: '1.6',
      },
      '.cm-gutters': {
        backgroundColor: dark
          ? 'var(--mantine-color-dark-8)'
          : 'var(--mantine-color-gray-0)',
        color: dark ? 'var(--mantine-color-dark-3)' : 'var(--mantine-color-gray-6)',
        border: 'none',
        borderRight: '1px solid var(--mantine-color-default-border)',
      },
      '.cm-activeLine': {
        backgroundColor: dark ? 'rgba(255, 255, 255, 0.03)' : 'rgba(0, 0, 0, 0.04)',
      },
      '.cm-activeLineGutter': {
        backgroundColor: dark
          ? 'var(--mantine-color-dark-7)'
          : 'var(--mantine-color-gray-1)',
      },
      '.cm-errorLine': {
        backgroundColor: 'rgba(250, 82, 82, 0.13)',
        boxShadow: 'inset 2px 0 0 var(--mantine-color-red-6)',
      },
      // Lint hover tooltip (diagnostic message) styled to the Mantine surface.
      '.cm-tooltip': {
        backgroundColor: dark ? 'var(--mantine-color-dark-6)' : 'var(--mantine-color-white)',
        color: dark ? 'var(--mantine-color-dark-0)' : 'var(--mantine-color-gray-9)',
        border: '1px solid var(--mantine-color-default-border)',
        borderRadius: 'var(--mantine-radius-sm)',
      },
      '.cm-tooltip-lint': {
        fontFamily: 'var(--mantine-font-family)',
        fontSize: 'var(--mantine-font-size-xs)',
      },
      '.cm-diagnostic-error': { borderLeft: '3px solid var(--mantine-color-red-6)' },
      '.cm-signature-help': {
        padding: '4px 8px',
        maxWidth: '480px',
        fontFamily: 'var(--mantine-font-family-monospace)',
        fontSize: 'var(--mantine-font-size-xs)',
      },
      '.cm-signature-detail': {
        marginTop: '2px',
        fontFamily: 'var(--mantine-font-family)',
        color: dark ? 'var(--mantine-color-dark-2)' : 'var(--mantine-color-gray-6)',
        maxWidth: '460px',
        whiteSpace: 'normal',
      },
      '&.cm-focused': { outline: 'none' },
    },
    { dark },
  )
}

/** One syntax error the check surfaced, with its 1-based editor position. */
export interface EditorSyntaxError {
  line: number
  column: number
  message: string
}

// State-managed decorations that tint every line a syntax error points at.
// The parent pushes the line numbers via the setErrorLines effect.
const setErrorLines = StateEffect.define<number[]>()
const errorLineDecoration = Decoration.line({ class: 'cm-errorLine' })

const errorLineField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    value = value.map(tr.changes)
    for (const effect of tr.effects) {
      if (effect.is(setErrorLines)) {
        const lines = [...new Set(effect.value)]
          .filter((line) => line >= 1 && line <= tr.state.doc.lines)
          .sort((a, b) => a - b)
        value = Decoration.set(lines.map((line) => errorLineDecoration.range(tr.state.doc.line(line).from)))
      }
    }
    return value
  },
  provide: (field) => EditorView.decorations.from(field),
})

// Paint every reported error as a full-line tint (errorLineField) plus a lint
// diagnostic — squiggle, gutter marker, and a hover tooltip carrying each
// error's own message. The single errorLine/errorMessage pair remains the
// fallback for callers (Solve failures) that only know one position.
function applyErrorMarks(view: EditorView, errorList: readonly EditorSyntaxError[] | null | undefined,
                         errorLine: number | null, errorMessage?: string | null) {
  const entries: EditorSyntaxError[] = errorList?.length
    ? [...errorList]
    : errorLine != null
      ? [{ line: errorLine, column: 0, message: errorMessage?.trim() || `Syntax error on line ${errorLine}` }]
      : []
  const valid = entries.filter((e) => e.line >= 1 && e.line <= view.state.doc.lines)
  view.dispatch({ effects: setErrorLines.of(valid.map((e) => e.line)) })
  const diagnostics: Diagnostic[] = valid
    .sort((a, b) => a.line - b.line || a.column - b.column)
    .map((e) => {
      const target = view.state.doc.line(e.line)
      // Start the squiggle at the reported column when we have one (clamped
      // inside the line); the tint still covers the whole line.
      const from = e.column > 0 ? Math.min(target.from + e.column - 1, Math.max(target.to - 1, target.from)) : target.from
      return {
        from,
        to: target.to,
        severity: 'error' as const,
        message: e.message?.trim() || `Syntax error on line ${e.line}`,
      }
    })
  view.dispatch(setDiagnostics(view.state, diagnostics))
}

function makeCompletionSource(
  namesRef: React.MutableRefObject<{ functions: string[]; variables: string[] }>,
) {
  return (context: CompletionContext): CompletionResult | null => {
    const word = context.matchBefore(/[A-Za-z_](?=([A-Za-z0-9_]*))\1$/)
    if (!word || (word.from === word.to && !context.explicit)) return null
    const { functions, variables } = namesRef.current
    const options = [
      ...functions.map((name) => ({ label: name, type: 'function', apply: `${name}(` })),
      ...variables.map((name) => ({ label: name, type: 'variable' })),
      ...COMPONENT_COMPLETIONS,
    ]
    return { from: word.from, options }
  }
}

interface Props {
  /** Document at mount time (lazy — called once). The editor owns the text
   *  afterwards (uncontrolled); programmatic replacements go through the
   *  setDoc() handle, so parent re-renders can never reset the doc mid-typing. */
  initialDoc: () => string
  onChange: (value: string) => void
  variables: string[]
  errorLine: number | null
  /** Message for the error on `errorLine`, surfaced as the lint hover tooltip. */
  errorMessage?: string | null
  /** Every syntax error from the last Check — when present, all of them are
   *  marked (tint + squiggle + per-error tooltip) and errorLine/errorMessage
   *  serve only as the fallback for single-position failures. */
  errorList?: readonly EditorSyntaxError[] | null
  placeholder?: string
}

// F1 = contextual help: open the reference page for the documented symbol
// under the cursor, or the portal itself when the cursor isn't on one. The
// slug set is the compact generated list (docsTopics.ts), not the catalogs.
const REFERENCE_SLUG_SET = new Set(REFERENCE_SLUGS)

const f1ContextualHelp = keymap.of([
  {
    key: 'F1',
    run: (view) => {
      const pos = view.state.selection.main.head
      const line = view.state.doc.lineAt(pos)
      const text = line.text
      const col = pos - line.from
      const isWordChar = (ch: string) => /[A-Za-z0-9_$#]/.test(ch)
      let start = col
      while (start > 0 && isWordChar(text[start - 1])) start--
      let end = col
      while (end < text.length && isWordChar(text[end])) end++
      const word = text.slice(start, end).toLowerCase()
      globalThis.open(REFERENCE_SLUG_SET.has(word) ? helpUrl(`#refpage:${word}`) : helpUrl(), '_blank')
      return true
    },
  },
])

function EquationEditorInner(
  { initialDoc, onChange, variables, errorLine, errorMessage, errorList, placeholder }: Readonly<Props>,
  ref: React.Ref<EquationEditorHandle>,
) {
  const cmRef = useRef<ReactCodeMirrorRef>(null)
  const viewRef = useRef<EditorView | null>(null)
  // Frozen at mount (initialDoc doubles as the lazy initializer); never updated,
  // so the CodeMirror wrapper's value-sync effect can never clobber user typing.
  const [mountDoc] = useState(initialDoc)
  // Set while setDoc() replaces the document so the change doesn't echo back
  // through onChange as if the user had typed it.
  const programmaticRef = useRef(false)
  // Read by the (stable) completion source so suggestions always reflect the
  // latest variable list without reconfiguring the editor.
  const namesRef = useRef({ functions: FUNCTION_NAMES, variables })
  namesRef.current.variables = variables

  // Rebuild the theme/highlight when the Mantine colour scheme changes so the
  // editor follows light/dark like the rest of the workspace.
  const colorScheme = useComputedColorScheme('dark')
  const isDark = colorScheme === 'dark'

  const extensions = useMemo<Extension[]>(
    () => [
      freesLanguage,
      freesLanguage.data.of({ autocomplete: makeCompletionSource(namesRef) }),
      syntaxHighlighting(isDark ? freesHighlightDark : freesHighlightLight),
      makeFreesTheme(isDark),
      errorLineField,
      signatureField,
      lintGutter(),
      f1ContextualHelp,
    ],
    [isDark],
  )

  // Push the error decoration + diagnostic whenever the reported error changes
  // (and once on mount, after onCreateEditor has captured the view).
  useEffect(() => {
    const view = viewRef.current
    if (view) applyErrorMarks(view, errorList, errorLine, errorMessage)
  }, [errorLine, errorMessage, errorList])

  useImperativeHandle(
    ref,
    () => ({
      insertSnippet(snippet: string) {
        const view = viewRef.current
        if (!view) return
        const caretMark = snippet.indexOf('$0')
        const clean = snippet.replace('$0', '')
        const { from, to } = view.state.selection.main
        const caret = from + (caretMark >= 0 ? caretMark : clean.length)
        view.dispatch({
          changes: { from, to, insert: clean },
          selection: { anchor: caret },
        })
        view.focus()
      },
      insertStatement(text: string, opts?: { focus?: boolean }) {
        const view = viewRef.current
        if (!view) return
        const doc = view.state.doc
        const end = doc.length
        // Prefix a newline unless the doc is empty or already ends with one, so
        // the statement never glues onto the previous line; suffix one so the
        // next insert (or the user's typing) starts on its own line too.
        const needsLeadingNL = end > 0 && doc.sliceString(end - 1, end) !== '\n'
        const insert = `${needsLeadingNL ? '\n' : ''}${text}\n`
        const caret = end + insert.length - 1
        view.dispatch({
          changes: { from: end, to: end, insert },
          selection: { anchor: caret },
        })
        if (opts?.focus !== false) {
          view.focus()
        }
      },
      setDoc(text: string) {
        const view = viewRef.current
        if (!view) return
        programmaticRef.current = true
        try {
          view.dispatch({
            changes: { from: 0, to: view.state.doc.length, insert: text },
            selection: { anchor: 0 },
          })
        } finally {
          programmaticRef.current = false
        }
      },
      goToLine(line: number) {
        const view = viewRef.current
        if (!view) return
        const n = Math.min(Math.max(line, 1), view.state.doc.lines)
        const target = view.state.doc.line(n)
        view.dispatch({
          selection: { anchor: target.from, head: target.to },
          scrollIntoView: true,
        })
        view.focus()
      },
      focus() {
        viewRef.current?.focus()
      },
    }),
    [],
  )

  return (
    <CodeMirror
      ref={cmRef}
      value={mountDoc}
      onChange={(v) => {
        if (!programmaticRef.current) onChange(v)
      }}
      extensions={extensions}
      placeholder={placeholder}
      theme="none"
      height="100%"
      style={{ flex: 1, minHeight: 260, overflow: 'hidden' }}
      basicSetup={{ foldGutter: false, highlightActiveLine: true, bracketMatching: true }}
      onCreateEditor={(view) => {
        viewRef.current = view
        applyErrorMarks(view, errorList, errorLine, errorMessage)
      }}
    />
  )
}

const EquationEditor = forwardRef(EquationEditorInner)
export default EquationEditor
