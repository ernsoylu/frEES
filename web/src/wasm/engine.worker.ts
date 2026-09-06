// Module worker hosting the frees WASM engine off the UI thread.
//
// Protocol (see engineClient.ts, the only sender):
// Progress: 'solve' and 'solveTable' are synchronous wasm calls that can run
// for minutes, so the engine reports how far along it is *from inside* the
// call. It does that by calling `globalThis.__freesOnProgress`, which this file
// defines (the boundary declares it `catch`, so a host that does not — the
// parity harness, a test — costs one swallowed TypeError and then nothing).
// Posting from inside a blocking call is exactly what makes it useful: the
// worker thread is busy, but the *main* thread is not, so the message lands and
// the bar paints while the solve is still running.
//
//   request  {id, method: 'solve' | 'solveTable' | 'check' | 'reference' |
//                     'version' | 'fluids' | 'propertyDiagram' |
//                     'psychrometricChart' | 'replEvaluate' | 'replClear' |
//                     'monteCarlo' | 'optimize' | 'optimizeMulti' |
//                     'curveFit' | 'parameterFit' | 'pidTune' |
//                     'extractPlant',
//             args: string[]}
//   response {id, ok: true, result: string} | {id, ok: false, error: string}
//
// `result` is the raw JSON string the wasm boundary emits (a REST-shaped
// SolveResponse/CheckResponse/LanguageReference; a bare semver string for
// 'version') — parsing happens on the client side, so the worker only ever
// posts strings.
//
// The measured-data line is gone from this protocol entirely. D6 removed
// MDF4 reading — which is why the protocol is strings-only rather than
// carrying transferable byte buffers — and D11 removed the Data Analyzer and
// the engine's measurement stack behind it, taking `measurementCalc` with
// them. Measured data now reaches a document as a CSV-imported function
// table, which travels inside an ordinary `solve` request.
//
// Failure discipline: nothing may kill the worker. The wasm boundary already
// returns every *document* problem as data; this dispatch wraps the rest
// (init failure, unknown method, an unexpected trap) in {ok: false}.

import init, {
  check,
  curve_fit,
  extract_plant,
  fluids,
  property_diagram,
  psychrometric_chart,
  reference,
  repl_clear,
  repl_evaluate,
  monte_carlo,
  optimize,
  optimize_multi,
  parameter_fit,
  pid_tune,
  solve,
  solve_table,
  version,
} from './pkg/frees.js'

export interface EngineRequest {
  id: number
  method:
    | 'solve'
    | 'solveTable'
    | 'monteCarlo'
    | 'optimize'
    | 'optimizeMulti'
    | 'curveFit'
    | 'parameterFit'
    | 'pidTune'
    | 'extractPlant'
    | 'check'
    | 'reference'
    | 'version'
    | 'fluids'
    | 'propertyDiagram'
    | 'psychrometricChart'
    | 'replEvaluate'
    | 'replClear'
  args: string[]
}

export type EngineResponse =
  | { id: number; ok: true; result: string }
  | { id: number; ok: false; error: string }
  /** An in-flight solve's overall completion, 0…1. Never terminal: the
   *  request still settles with an `ok` message afterwards. */
  | { id: number; progress: number }

// The tsconfig compiles against the DOM lib (the worker file shares the app's
// program), where `self` is a Window; narrow it to the two members a dedicated
// worker actually uses instead of dragging in the conflicting webworker lib.
const ctx = self as unknown as {
  onmessage: ((event: MessageEvent<EngineRequest>) => void) | null
  postMessage(message: EngineResponse): void
}

// The request the engine is inside right now, so the progress hook — which the
// engine calls with a bare fraction — can address its message. The worker
// handles exactly one request at a time (the wasm calls are synchronous), so a
// single slot is the whole correlation story. `null` between requests, which is
// what makes a stray late call from a torn-down solve harmless.
let inFlightId: number | null = null

// The engine's progress sink. Declared on `globalThis` because the wasm
// boundary imports it as a plain global rather than taking a callback
// argument — that keeps the exported `solve(source, request)` signature the one
// api.ts already sends.
;(
  globalThis as unknown as { __freesOnProgress?: (fraction: number) => void }
).__freesOnProgress = (fraction: number) => {
  if (inFlightId === null) return
  // Anything the engine sends that is not a usable fraction is dropped here
  // rather than becoming a NaN width on a DOM node.
  if (typeof fraction !== 'number' || !Number.isFinite(fraction)) return
  ctx.postMessage({
    id: inFlightId,
    progress: Math.min(1, Math.max(0, fraction)),
  })
}

// Kick off wasm instantiation immediately so it overlaps the first request.
// `new URL(..., import.meta.url)` lets Vite emit the .wasm as a hashed asset
// and rewrite the URL in both dev and build.
const ready = init({
  module_or_path: new URL('./pkg/frees_bg.wasm', import.meta.url),
})

// `onmessage` is typed as returning void, so the async body is wrapped and
// its promise explicitly discarded: every failure is already turned into an
// error response inside `handle`, so there is nothing left to await on.
ctx.onmessage = (event: MessageEvent<EngineRequest>) => {
  void handle(event)
}

const handle = async (event: MessageEvent<EngineRequest>) => {
  const { id, method, args } = event.data
  try {
    await ready
    // Only the two solving methods report; everything else here is fast enough
    // that a bar would flicker rather than inform.
    inFlightId = method === 'solve' || method === 'solveTable' ? id : null
    let result: string
    switch (method) {
      case 'solve':
        result = solve(args[0] ?? '', args[1] ?? '')
        break
      case 'solveTable':
        result = solve_table(args[0] ?? '', args[1] ?? '')
        break
      case 'monteCarlo':
        result = monte_carlo(args[0] ?? '', args[1] ?? '')
        break
      case 'optimize':
        result = optimize(args[0] ?? '', args[1] ?? '')
        break
      case 'optimizeMulti':
        result = optimize_multi(args[0] ?? '', args[1] ?? '')
        break
      case 'curveFit':
        result = curve_fit(args[0] ?? '')
        break
      case 'parameterFit':
        result = parameter_fit(args[0] ?? '')
        break
      case 'pidTune':
        result = pid_tune(args[0] ?? '')
        break
      case 'extractPlant':
        result = extract_plant(args[0] ?? '')
        break
      case 'check':
        result = check(args[0] ?? '', args[1] ?? '')
        break
      case 'reference':
        result = reference()
        break
      case 'version':
        result = version()
        break
      case 'fluids':
        result = fluids()
        break
      case 'propertyDiagram':
        result = property_diagram(args[0] ?? '', args[1] ?? '')
        break
      case 'psychrometricChart':
        result = psychrometric_chart(args[0] ?? '')
        break
      // The REPL evaluates against the workspace the last successful `solve`
      // left in this module, so both calls must reach the *same* worker
      // instance as the solve did. They do: engineClient keeps one.
      case 'replEvaluate':
        result = repl_evaluate(args[0] ?? '')
        break
      case 'replClear':
        repl_clear(args[0] ?? 'null')
        result = ''
        break
      default:
        throw new Error(`Unknown engine method: ${String(method)}`)
    }
    ctx.postMessage({ id, ok: true, result })
  } catch (e) {
    ctx.postMessage({
      id,
      ok: false,
      error: e instanceof Error ? e.message : String(e),
    })
  } finally {
    // Whatever happened, this request is no longer the one to report against.
    inFlightId = null
  }
}
