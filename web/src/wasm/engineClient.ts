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

export interface WorkerResult {
  result: string
  matrix?: Float64Array | null
  odeBuffers?: Float64Array[] | null
}

interface Pending {
  worker: Worker
  resolve: (result: WorkerResult) => void
  reject: (reason: Error) => void
  onProgress?: ProgressListener
}

const pool: Worker[] = []
let nextId = 0
const pending = new Map<number, Pending>()

/** Maximum number of workers in the pool to bound memory and CPU usage. */
export const MAX_WORKER_POOL_SIZE = 4

function detectDefaultConcurrency(): number {
  if (typeof navigator !== 'undefined') {
    const isMobileOrLowMemory =
      (typeof (navigator as unknown as { deviceMemory?: number }).deviceMemory === 'number' &&
        (navigator as unknown as { deviceMemory: number }).deviceMemory <= 4) ||
      /Mobi|Android/i.test(navigator.userAgent)
    if (isMobileOrLowMemory) {
      return Math.min(2, Math.max(1, navigator.hardwareConcurrency || 2))
    }
    if (typeof navigator.hardwareConcurrency === 'number' && navigator.hardwareConcurrency > 0) {
      return Math.min(MAX_WORKER_POOL_SIZE, Math.max(1, navigator.hardwareConcurrency))
    }
  }
  return 2
}

let configuredConcurrency: number | null = null

/** Sets the worker pool concurrency limit (clamped between 1 and MAX_WORKER_POOL_SIZE). */
export function setWorkerPoolConcurrency(count: number): void {
  configuredConcurrency = Number.isNaN(count) ? 1 : Math.min(MAX_WORKER_POOL_SIZE, Math.max(1, Math.floor(count)))
}

/** Returns the effective worker pool concurrency limit. */
export function getWorkerPoolConcurrency(): number {
  return configuredConcurrency ?? detectDefaultConcurrency()
}

/** Resets the worker pool concurrency limit back to hardware-detected default. */
export function resetWorkerPoolConcurrency(): void {
  configuredConcurrency = null
}

/** Retires additional pool workers (1..N-1), keeping primary worker 0 alive to conserve memory. */
export function retireExtraWorkers(): void {
  for (let i = 1; i < pool.length; i++) {
    if (pool[i]) {
      for (const [id, entry] of pending) {
        if (entry.worker === pool[i]) {
          pending.delete(id)
          entry.reject(new Error('Operation stopped'))
        }
      }
      try {
        pool[i].terminate()
      } catch {
        /* ignore */
      }
    }
  }
  if (pool.length > 1) {
    pool.length = 1
  }
}

/** Rejects everything in flight and drops all workers so the next call respawns. */
function fail(reason: Error): void {
  const inFlight = [...pending.values()]
  pending.clear()
  for (const w of pool) {
    if (w) {
      try {
        w.terminate()
      } catch {
        /* ignore */
      }
    }
  }
  pool.length = 0
  for (const entry of inFlight) entry.reject(reason)
}

/** Terminates all active workers in the pool, rejecting any in-flight requests and resetting worker state. */
export function wasmStop(): void {
  fail(new Error('Operation stopped'))
}

function spawn(): Worker {
  const w = new Worker(new URL('./engine.worker.ts', import.meta.url), {
    type: 'module',
  })
  w.onmessage = (event: MessageEvent<EngineResponse>) => {
    if (!pool.includes(w)) return
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
      entry.resolve({
        result: response.result,
        matrix: 'matrix' in response ? response.matrix : null,
        odeBuffers: 'odeBuffers' in response ? response.odeBuffers : null,
      })
    } else {
      if ('fatal' in response && response.fatal) {
        fail(new Error(response.error))
      }
      entry.reject(new Error(response.error))
    }
  }
  w.onerror = (event: ErrorEvent) => {
    if (!pool.includes(w)) return
    fail(new Error(event.message || 'The engine worker failed'))
  }
  w.onmessageerror = () => {
    if (!pool.includes(w)) return
    fail(new Error('The engine worker sent an unreadable message'))
  }
  return w
}

function getWorker(index: number): Worker {
  if (!pool[index]) {
    pool[index] = spawn()
  }
  return pool[index]
}

/** Posts one request and resolves with the worker's result payload and transferred buffers. */
function call(
  method: EngineRequest['method'],
  args: string[],
  onProgress?: ProgressListener,
  workerIndex = 0,
): Promise<WorkerResult> {
  const id = nextId++
  return new Promise<WorkerResult>((resolve, reject) => {
    const w = getWorker(workerIndex)
    pending.set(id, { worker: w, resolve, reject, onProgress })
    try {
      w.postMessage({ id, method, args } satisfies EngineRequest)
    } catch (error) {
      pending.delete(id)
      reject(error)
    }
  })
}

/** Regular expression detecting parametric accessor calls in document source. */
export const ACCESSOR_PATTERN =
  /(?:^|[^a-zA-Z0-9_])(?:TableRun#|TableRun|NParametricRuns|TableValue|TableSum|TableAvg|TableMin|TableMax|TableStdDev|IntegralValue)\s*\(/i

/** Returns true if the source document mentions any parametric table accessor. */
export function mentionsParametricAccessor(source: string): boolean {
  return ACCESSOR_PATTERN.test(source)
}

export interface TableRowResultDto {
  success: boolean
  values: Record<string, number>
  error: string | null
  status?: string
}

export interface TableStatsDto {
  converged: boolean
  passes: number
  termination: string
  accessor: boolean
  runs: number
  solved: number
  failed: number
  notRun: number
  equations: number
  unknowns: number
  iterations: number
  elapsedMillis: number
  maxResidual: number
}

export interface SolveTableResponseDto {
  results?: TableRowResultDto[]
  stats?: TableStatsDto | null
  variables?: unknown[]
  error?: string
  matrix?: Float64Array | null
  varNames?: string[]
}

export function hydrateRowValues(dto: SolveTableResponseDto): void {
  if (!dto.results || !dto.matrix || !dto.varNames || dto.varNames.length === 0) {
    return
  }
  const matrix = dto.matrix
  const varNames = dto.varNames
  const numCols = varNames.length
  for (let r = 0; r < dto.results.length; r++) {
    const row = dto.results[r]
    if (row && row.success && (!row.values || Object.keys(row.values).length === 0)) {
      const vals: Record<string, number> = {}
      const offset = r * numCols
      for (let c = 0; c < numCols; c++) {
        const val = matrix[offset + c]
        if (Number.isFinite(val)) {
          vals[varNames[c]] = val
        }
      }
      row.values = vals
    }
  }
}

function parseSolveTableResult({ result, matrix }: WorkerResult): SolveTableResponseDto {
  const parsed = JSON.parse(result) as SolveTableResponseDto
  if (matrix) {
    parsed.matrix = matrix
    hydrateRowValues(parsed)
  }
  return parsed
}

/** Merges chunked solveTable responses into a single combined response matching the Rust engine's contract. */
export function mergeSolveTableResponses(
  parsedChunks: SolveTableResponseDto[],
  totalRows: number,
  elapsedMillis: number,
): SolveTableResponseDto {
  const allResults: TableRowResultDto[] = []
  let totalSolved = 0
  let totalFailed = 0
  let totalNotRun = 0
  let totalIterations = 0
  let maxResidual = 0
  let hasDeadline = false
  let allCompleted = true
  let allConverged = true

  let lastEquations = 0
  let lastUnknowns = 0
  let lastVariables: unknown[] = []

  for (const chunk of parsedChunks) {
    if (chunk.results) {
      allResults.push(...chunk.results)
    }
    const stats = chunk.stats
    if (stats) {
      totalSolved += stats.solved ?? 0
      totalFailed += stats.failed ?? 0
      totalNotRun += stats.notRun ?? 0
      totalIterations += stats.iterations ?? 0
      if (typeof stats.maxResidual === 'number' && Number.isFinite(stats.maxResidual)) {
        maxResidual = Math.max(maxResidual, stats.maxResidual)
      }
      if (stats.termination === 'deadline') {
        hasDeadline = true
      }
      if (stats.termination !== 'completed') {
        allCompleted = false
      }
      if (!stats.converged) {
        allConverged = false
      }
    }
  }

  // Find the last chunk with successful rows to pull equations, unknowns, and variables (matching Rust's last-wins rule)
  for (let i = parsedChunks.length - 1; i >= 0; i--) {
    const chunk = parsedChunks[i]
    if (chunk.stats && chunk.stats.solved > 0) {
      lastEquations = chunk.stats.equations ?? 0
      lastUnknowns = chunk.stats.unknowns ?? 0
      if (Array.isArray(chunk.variables) && chunk.variables.length > 0) {
        lastVariables = chunk.variables
      }
      break
    }
  }

  const termination = hasDeadline
    ? 'deadline'
    : allCompleted && allConverged
      ? 'completed'
      : 'pass-limit'

  // Failed chunks can omit computed columns; align by name and preserve missing cells.
  const varNames = [...new Set(parsedChunks.flatMap((chunk) => chunk.varNames ?? []))]
  const mergedMatrix = varNames.length
    ? new Float64Array(totalRows * varNames.length).fill(NaN)
    : null
  let rowOffset = 0
  for (const chunk of parsedChunks) {
    if (mergedMatrix && chunk.matrix && chunk.varNames) {
      const columns = chunk.varNames.map((name) => varNames.indexOf(name))
      for (let r = 0; r < (chunk.results?.length ?? 0); r++) {
        for (let c = 0; c < columns.length; c++) {
          mergedMatrix[(rowOffset + r) * varNames.length + columns[c]] =
            chunk.matrix[r * columns.length + c]
        }
      }
    }
    rowOffset += chunk.results?.length ?? 0
  }

  const mergedDto: SolveTableResponseDto = {
    results: allResults,
    stats: {
      converged: allConverged && !hasDeadline,
      passes: 1,
      termination,
      accessor: false,
      runs: totalRows,
      solved: totalSolved,
      failed: totalFailed,
      notRun: totalNotRun,
      equations: lastEquations,
      unknowns: lastUnknowns,
      iterations: totalIterations,
      elapsedMillis,
      maxResidual,
    },
    variables: lastVariables,
    ...(varNames.length ? { varNames } : {}),
  }

  if (mergedMatrix) mergedDto.matrix = mergedMatrix

  return mergedDto
}

/** Runs a solve in the engine worker; resolves to the parsed SolveResponse. */
export async function wasmSolve(
  source: string,
  requestJson: string,
  onProgress?: ProgressListener,
): Promise<SolveResponse> {
  const { result, odeBuffers } = await call('solve', [source, requestJson], onProgress)
  const parsed = JSON.parse(result) as SolveResponse
  if (parsed.odeTables && odeBuffers && odeBuffers.length > 0) {
    for (let i = 0; i < parsed.odeTables.length; i++) {
      const table = parsed.odeTables[i]
      const buf = odeBuffers[i]
      if (table && buf) {
        table.matrix = buf
        // Reconstruct rows for backward compatibility if rows is empty and buf is present
        const numVars = table.vars?.length ?? 0
        if (numVars > 0 && (!table.rows || table.rows.length === 0)) {
          const numRows = Math.floor(buf.length / numVars)
          const rows: (number | null)[][] = new Array(numRows)
          for (let r = 0; r < numRows; r++) {
            const offset = r * numVars
            const row: (number | null)[] = new Array(numVars)
            for (let c = 0; c < numVars; c++) {
              const val = buf[offset + c]
              row[c] = Number.isFinite(val) ? val : null
            }
            rows[r] = row
          }
          table.rows = rows
        }
      }
    }
  }
  return parsed
}

/** Runs a Tables-workbook sweep in the engine worker(s); resolves to the parsed response with transferred data. */
export async function wasmSolveTable(
  source: string,
  requestJson: string,
  onProgress?: ProgressListener,
): Promise<SolveTableResponseDto> {
  const concurrency = getWorkerPoolConcurrency()
  if (concurrency <= 1 || mentionsParametricAccessor(source)) {
    const workerRes = await call('solveTable', [source, requestJson], onProgress, 0)
    return parseSolveTableResult(workerRes)
  }

  let request: {
    table?: {
      variables?: string[]
      rows?: Record<string, number>[]
    }
    [key: string]: unknown
  } | null = null

  try {
    request = JSON.parse(requestJson)
  } catch {
    const workerRes = await call('solveTable', [source, requestJson], onProgress, 0)
    return parseSolveTableResult(workerRes)
  }

  const rows = request?.table?.rows
  const variables = request?.table?.variables
  // Keep serial if invalid table, single row, cap exceeded (> 5000), or missing variables
  if (!rows || !Array.isArray(rows) || rows.length <= 1 || rows.length > 5000 || !variables) {
    const workerRes = await call('solveTable', [source, requestJson], onProgress, 0)
    return parseSolveTableResult(workerRes)
  }

  const totalRows = rows.length
  const workerCount = Math.min(concurrency, totalRows)
  if (workerCount <= 1) {
    const workerRes = await call('solveTable', [source, requestJson], onProgress, 0)
    return parseSolveTableResult(workerRes)
  }

  // Partition rows across workerCount chunks
  const chunkSize = Math.ceil(totalRows / workerCount)
  const chunks: Record<string, number>[][] = []
  for (let i = 0; i < totalRows; i += chunkSize) {
    chunks.push(rows.slice(i, i + chunkSize))
  }

  const chunkProgress = new Array<number>(chunks.length).fill(0)
  const handleChunkProgress = (chunkIndex: number, fraction: number) => {
    if (!onProgress) return
    chunkProgress[chunkIndex] = fraction
    let totalFraction = 0
    for (let i = 0; i < chunks.length; i++) {
      totalFraction += (chunks[i].length / totalRows) * chunkProgress[i]
    }
    try {
      onProgress(Math.min(1, Math.max(0, totalFraction)))
    } catch {
      /* ignore */
    }
  }

  const startTime = Date.now()
  const chunkPromises = chunks.map((chunkRows, chunkIndex) => {
    const chunkRequest = JSON.stringify({
      ...request,
      table: {
        variables,
        rows: chunkRows,
      },
    })
    return call(
      'solveTable',
      [source, chunkRequest],
      onProgress ? (f) => handleChunkProgress(chunkIndex, f) : undefined,
      chunkIndex,
    )
  })

  const chunkOutputs = await Promise.all(chunkPromises)
  const elapsedMillis = Math.max(0, Date.now() - startTime)

  const parsedChunks = chunkOutputs.map(parseSolveTableResult)

  const firstError = parsedChunks.find((c) => c.error)
  if (firstError) {
    return firstError
  }

  return mergeSolveTableResponses(parsedChunks, totalRows, elapsedMillis)
}

/** Runs a Monte Carlo propagation in the engine worker; resolves to the raw JSON string. */
export async function wasmMonteCarlo(
  source: string,
  requestJson: string,
): Promise<string> {
  return (await call('monteCarlo', [source, requestJson])).result
}

/** The four OptimizeController surfaces; raw JSON strings out. */
export async function wasmOptimize(source: string, requestJson: string): Promise<string> {
  return (await call('optimize', [source, requestJson])).result
}
export async function wasmOptimizeMulti(source: string, requestJson: string): Promise<string> {
  return (await call('optimizeMulti', [source, requestJson])).result
}
export async function wasmCurveFit(requestJson: string): Promise<string> {
  return (await call('curveFit', [requestJson])).result
}
export async function wasmParameterFit(requestJson: string): Promise<string> {
  return (await call('parameterFit', [requestJson])).result
}

/** The two ControlController surfaces; raw JSON strings out. */
export async function wasmPidTune(requestJson: string): Promise<string> {
  return (await call('pidTune', [requestJson])).result
}
export async function wasmExtractPlant(requestJson: string): Promise<string> {
  return (await call('extractPlant', [requestJson])).result
}

/** Runs a check in the engine worker; resolves to the parsed CheckResponse. */
export async function wasmCheck(
  source: string,
  requestJson: string,
): Promise<CheckResponse> {
  return JSON.parse((await call('check', [source, requestJson])).result) as CheckResponse
}

/** POST /api/repl/evaluate. */
export async function wasmReplEvaluate(
  expression: string,
  unitSystem: string,
): Promise<ReplResponse> {
  return JSON.parse(
    (await call('replEvaluate', [JSON.stringify({ expression, unitSystem })])).result,
  ) as ReplResponse
}

/** POST /api/repl/clear. */
export async function wasmReplClear(name?: string): Promise<void> {
  await call('replClear', [JSON.stringify(name ?? null)])
}

/** The engine's language reference. */
export async function wasmReference(): Promise<LanguageReference> {
  return JSON.parse((await call('reference', [])).result) as LanguageReference
}

/** The engine crate's semver. */
export async function wasmVersion(): Promise<string> {
  return (await call('version', [])).result
}

/** GET /api/plot/fluids. */
export async function wasmFluids(): Promise<{
  available: boolean
  fluids: string[]
  backend: string
}> {
  return JSON.parse((await call('fluids', [])).result) as {
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
    (await call('propertyDiagram', [fluid, kind])).result,
  )
}

/** POST /api/plot/psychart — the psychrometric chart. */
export async function wasmPsychrometricChart(
  pressure: number,
  tMin: number,
  tMax: number,
): Promise<PsychartResponse> {
  return unwrapPlot<PsychartResponse>(
    (await call('psychrometricChart', [JSON.stringify({ pressure, tMin, tMax })])).result,
  )
}
