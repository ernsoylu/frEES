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
//   response {id, ok: true, result: string} | {id, ok: false, fatal?: boolean, error: string}
//
// `result` is the raw JSON string the wasm boundary emits (a REST-shaped
// SolveResponse/CheckResponse/LanguageReference; a bare semver string for
// 'version'). Bulk numeric tables travel separately in transferable buffers;
// envelope parsing happens on the client side.
//
// Failure discipline:
// Document problems return {ok: true} with error data payloads from the wasm boundary.
// Fatal failures (wasm traps, memory corruption, init failures) are marked {fatal: true}
// so engineClient retires the worker; ordinary non-fatal dispatch problems leave it usable.

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
  solve_zerocopy,
  solve_table_zerocopy,
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
  | {
      id: number
      ok: true
      result: string
      matrix?: Float64Array | null
      odeBuffers?: Float64Array[] | null
    }
  | { id: number; ok: false; fatal?: boolean; error: string }
  /** An in-flight solve's overall completion, 0…1. Never terminal: the
   *  request still settles with an `ok` message afterwards. */
  | { id: number; progress: number }

// The tsconfig compiles against the DOM lib (the worker file shares the app's
// program), where `self` is a Window; narrow it to the two members a dedicated
// worker actually uses instead of dragging in the conflicting webworker lib.
const ctx = self as unknown as {
  onmessage: ((event: MessageEvent<EngineRequest>) => void) | null
  postMessage(message: EngineResponse, transfer?: Transferable[]): void
}

let inFlightId: number | null = null

;(
  globalThis as unknown as { __freesOnProgress?: (fraction: number) => void }
).__freesOnProgress = (fraction: number) => {
  if (inFlightId === null) return
  if (typeof fraction !== 'number' || !Number.isFinite(fraction)) return
  ctx.postMessage({
    id: inFlightId,
    progress: Math.min(1, Math.max(0, fraction)),
  })
}

let readyFailed = false
const ready = init({
  module_or_path: new URL('./pkg/frees_bg.wasm', import.meta.url),
}).catch((err: unknown) => {
  readyFailed = true
  throw err
})

ctx.onmessage = (event: MessageEvent<EngineRequest>) => {
  void handle(event)
}

const handle = async (event: MessageEvent<EngineRequest>) => {
  const { id, method, args } = event.data
  try {
    await ready
    inFlightId = method === 'solve' || method === 'solveTable' ? id : null
    let result: string
    let matrix: Float64Array | null = null
    let odeBuffers: Float64Array[] | null = null
    const transferables: Transferable[] = []

    switch (method) {
      case 'solve': {
        const out = solve_zerocopy(args[0] ?? '', args[1] ?? '')
        result = (out?.envelope as string) ?? ''
        const rawOde = out?.odeBuffers as Float64Array[] | undefined
        if (Array.isArray(rawOde) && rawOde.length > 0) {
          odeBuffers = rawOde
          for (const b of rawOde) {
            if (b && b.buffer) {
              transferables.push(b.buffer)
            }
          }
        }
        break
      }
      case 'solveTable': {
        const out = solve_table_zerocopy(args[0] ?? '', args[1] ?? '')
        result = (out?.envelope as string) ?? ''
        const rawMat = out?.matrix as Float64Array | null | undefined
        if (rawMat && rawMat.buffer) {
          matrix = rawMat
          transferables.push(rawMat.buffer)
        }
        break
      }
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
    ctx.postMessage(
      {
        id,
        ok: true,
        result,
        ...(matrix ? { matrix } : {}),
        ...(odeBuffers ? { odeBuffers } : {}),
      },
      transferables,
    )
  } catch (e) {
    const isFatal =
      readyFailed ||
      e instanceof WebAssembly.RuntimeError ||
      e instanceof WebAssembly.LinkError ||
      e instanceof WebAssembly.CompileError ||
      (e instanceof Error &&
        /unreachable|out of bounds|memory|panic|trap|corrupt/i.test(e.message))
    ctx.postMessage({
      id,
      ok: false,
      fatal: isFatal,
      error: e instanceof Error ? e.message : String(e),
    })
  } finally {
    inFlightId = null
  }
}
