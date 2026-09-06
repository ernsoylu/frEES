// Singleton client for the frees WASM engine worker.
//
// Lazily spawns engine.worker.ts on first use, correlates request/response
// pairs by id, and exposes typed async calls that JSON.parse the worker's
// result strings into the REST wire shapes api.ts already declares. A worker
// that dies (script load failure, fatal WASM trap, OOM) rejects everything in flight
// and is retired so the next call spawns a fresh one. Non-fatal errors leave the worker usable.

import type {
  CheckResponse,
  DiagramResponse,
  LanguageReference,
  PsychartResponse,
  ReplResponse,
  SolveResponse,
} from '../api'
import type { EngineRequest, EngineResponse } from './engine.worker'

/** Called with the engine's overall completion (0…1) while a solve runs. */
export type ProgressListener = (fraction: number) => void

interface Pending {
  resolve: (result: string) => void
  reject: (reason: Error) => void
  onProgress?: ProgressListener
}

let worker: Worker | null = null
let nextId = 0
const pending = new Map<number, Pending>()

/** Rejects everything in flight and drops the worker so the next call respawns. */
function fail(reason: Error): void {
  const inFlight = [...pending.values()]
  pending.clear()
  worker?.terminate()
  worker = null
  for (const entry of inFlight) entry.reject(reason)
}

/** Terminates the active worker, rejecting any in-flight requests and resetting worker state. */
export function wasmStop(): void {
  fail(new Error('Operation stopped'))
}

function spawn(): Worker {
  const w = new Worker(new URL('./engine.worker.ts', import.meta.url), {
    type: 'module',
  })
  w.onmessage = (event: MessageEvent<EngineResponse>) => {
    const response = event.data
    const entry = pending.get(response.id)
    if (!entry) return
    if ('progress' in response) {
      try {
        entry.onProgress?.(response.progress)
      } catch {
        /* a progress listener is decoration; its failure is not the solve's */
      }
      return
    }
    pending.delete(response.id)
    if (response.ok) {
      entry.resolve(response.result)
    } else {
      if ('fatal' in response && response.fatal) {
        fail(new Error(response.error))
      }
      entry.reject(new Error(response.error))
    }
  }
  w.onerror = (event: ErrorEvent) => {
    fail(new Error(event.message || 'The engine worker failed'))
  }
  w.onmessageerror = () => {
    fail(new Error('The engine worker sent an unreadable message'))
  }
  return w
}

/** Posts one request and resolves with the worker's raw JSON-string reply. */
function call(
  method: EngineRequest['method'],
  args: string[],
  onProgress?: ProgressListener,
): Promise<string> {
  worker ??= spawn()
  const id = nextId++
  return new Promise<string>((resolve, reject) => {
    pending.set(id, { resolve, reject, onProgress })
    worker?.postMessage({ id, method, args } satisfies EngineRequest)
  })
}

/** Runs a solve in the engine worker; resolves to the parsed SolveResponse. */
export async function wasmSolve(
  source: string,
  requestJson: string,
  onProgress?: ProgressListener,
): Promise<SolveResponse> {
  return JSON.parse(
    await call('solve', [source, requestJson], onProgress),
  ) as SolveResponse
}

/** Runs a Tables-workbook sweep in the engine worker; resolves to the raw JSON string. */
export async function wasmSolveTable(
  source: string,
  requestJson: string,
  onProgress?: ProgressListener,
): Promise<string> {
  return call('solveTable', [source, requestJson], onProgress)
}

/** Runs a Monte Carlo propagation in the engine worker; resolves to the raw JSON string. */
export async function wasmMonteCarlo(
  source: string,
  requestJson: string,
): Promise<string> {
  return call('monteCarlo', [source, requestJson])
}

/** The four OptimizeController surfaces; raw JSON strings out. */
export async function wasmOptimize(source: string, requestJson: string): Promise<string> {
  return call('optimize', [source, requestJson])
}
export async function wasmOptimizeMulti(source: string, requestJson: string): Promise<string> {
  return call('optimizeMulti', [source, requestJson])
}
export async function wasmCurveFit(requestJson: string): Promise<string> {
  return call('curveFit', [requestJson])
}
export async function wasmParameterFit(requestJson: string): Promise<string> {
  return call('parameterFit', [requestJson])
}

/** The two ControlController surfaces; raw JSON strings out. */
export async function wasmPidTune(requestJson: string): Promise<string> {
  return call('pidTune', [requestJson])
}
export async function wasmExtractPlant(requestJson: string): Promise<string> {
  return call('extractPlant', [requestJson])
}

/** Runs a check in the engine worker; resolves to the parsed CheckResponse. */
export async function wasmCheck(
  source: string,
  requestJson: string,
): Promise<CheckResponse> {
  return JSON.parse(await call('check', [source, requestJson])) as CheckResponse
}

/** POST /api/repl/evaluate. */
export async function wasmReplEvaluate(
  expression: string,
  unitSystem: string,
): Promise<ReplResponse> {
  return JSON.parse(
    await call('replEvaluate', [JSON.stringify({ expression, unitSystem })]),
  ) as ReplResponse
}

/** POST /api/repl/clear. */
export async function wasmReplClear(name?: string): Promise<void> {
  await call('replClear', [JSON.stringify(name ?? null)])
}

/** The engine's language reference. */
export async function wasmReference(): Promise<LanguageReference> {
  return JSON.parse(await call('reference', [])) as LanguageReference
}

/** The engine crate's semver. */
export function wasmVersion(): Promise<string> {
  return call('version', [])
}

/** GET /api/plot/fluids. */
export async function wasmFluids(): Promise<{
  available: boolean
  fluids: string[]
  backend: string
}> {
  return JSON.parse(await call('fluids', [])) as {
    available: boolean
    fluids: string[]
    backend: string
  }
}

function unwrapPlot<T>(payload: string): T {
  const parsed = JSON.parse(payload) as T & { error?: string }
  if (typeof parsed.error === 'string') throw new Error(parsed.error)
  return parsed
}

/** POST /api/plot/propplot — saturation dome, isolines and markers. */
export async function wasmPropertyDiagram(
  fluid: string,
  kind: string,
): Promise<DiagramResponse> {
  return unwrapPlot<DiagramResponse>(
    await call('propertyDiagram', [fluid, kind]),
  )
}

/** POST /api/plot/psychart — the psychrometric chart. */
export async function wasmPsychrometricChart(
  pressure: number,
  tMin: number,
  tMax: number,
): Promise<PsychartResponse> {
  return unwrapPlot<PsychartResponse>(
    await call('psychrometricChart', [JSON.stringify({ pressure, tMin, tMax })]),
  )
}
