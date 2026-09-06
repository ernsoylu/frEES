#!/usr/bin/env node
// Golden corpus parity replay through the compiled WASM engine in Node.
// Replays fixtures in fixtures/golden/ against the web/src/wasm/pkg artifact.

import { readFileSync, readdirSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { initSync, solve } from '../src/wasm/pkg/frees.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
const ROOT = resolve(__dirname, '../..')
const FIXTURES_DIR = resolve(ROOT, 'fixtures')
const GOLDEN_DIR = resolve(FIXTURES_DIR, 'golden')
const WASM_PATH = resolve(ROOT, 'web/src/wasm/pkg/frees_bg.wasm')

const REL_TOL = 1e-9
const ABS_TOL = 1e-12

function close(actual, expected, tol) {
  if (Number.isNaN(actual) && Number.isNaN(expected)) return true
  if (actual === expected) return true
  const diff = Math.abs(actual - expected)
  return diff <= ABS_TOL || diff <= tol * Math.max(Math.abs(expected), Math.abs(actual))
}

function relDiff(actual, expected) {
  if (actual === expected) return 0
  const diff = Math.abs(actual - expected)
  return diff / Math.max(Math.abs(expected), Math.abs(actual), 1e-12)
}

function compareOdeTables(expectedTables, actualTables, relTol, fail, perturbTrajectory = false) {
  if (!expectedTables) {
    if (actualTables && actualTables.length > 0) {
      fail(`WASM produced ${actualTables.length} ODE table(s) but golden has no ode_tables section`)
    }
    return
  }
  if (!Array.isArray(actualTables)) {
    fail(`WASM odeTables missing or invalid`)
    return
  }
  if (expectedTables.length !== actualTables.length) {
    fail(`Java recorded ${expectedTables.length} ODE table(s), WASM produced ${actualTables.length}`)
    return
  }
  for (let i = 0; i < expectedTables.length; i++) {
    const want = expectedTables[i]
    const got = actualTables[i]
    if (want.name !== got.name) fail(`ode_tables[${i}] name = ${got.name} but Java got ${want.name}`)
    if (want.method !== got.method) fail(`ode_tables[${i}] method = ${got.method} but Java got ${want.method}`)
    const wantCols = want.columns || []
    const gotCols = got.vars || []
    if (JSON.stringify(wantCols) !== JSON.stringify(gotCols)) {
      fail(`ode_tables[${i}] columns = ${JSON.stringify(gotCols)} but Java got ${JSON.stringify(wantCols)}`)
      continue
    }
    if ((want.stopped || false) !== got.stopped) {
      fail(`ode_tables[${i}] stopped = ${got.stopped} but Java got ${want.stopped}`)
    }
    if (!close(got.endTime, want.end_time, relTol)) {
      fail(`ode_tables[${i}] end_time = ${got.endTime} but Java got ${want.end_time}`)
    }
    const wantRows = want.rows || []
    const gotRows = got.rows || []
    if (wantRows.length !== gotRows.length) {
      fail(`ode_tables[${i}] rows count = ${gotRows.length} but Java sampled ${wantRows.length}`)
      continue
    }
    const scales = wantCols.map((col, c) => {
      if (col === 'time') return 0
      let max = 0
      for (const r of wantRows) {
        if (r[c] !== undefined && !isNaN(r[c])) {
          max = Math.max(max, Math.abs(r[c]))
        }
      }
      return max
    })

    for (let r = 0; r < wantRows.length; r++) {
      const wantRow = wantRows[r]
      const gotRow = gotRows[r]
      for (let c = 0; c < wantRow.length; c++) {
        let wantVal = wantRow[c]
        if (perturbTrajectory && r === 1 && c === 1) {
          wantVal += 1.0 // deliberate trajectory perturbation
        }
        const gotVal = gotRow[c]
        const scale = scales[c]
        const diff = Math.abs(gotVal - wantVal)
        const anchored = scale > 0 && diff <= relTol * scale
        if (!close(gotVal, wantVal, relTol) && !anchored) {
          const col = gotCols[c] || `col_${c}`
          fail(`ode_tables[${i}] row ${r} col ${col} = ${gotVal} but Java got ${wantVal} (diff ${diff}, scale ${scale})`)
        }
      }
    }

    const wantEvents = want.events || []
    const gotEvents = got.events || []
    if (wantEvents.length !== gotEvents.length) {
      fail(`ode_tables[${i}] events count = ${gotEvents.length} but Java recorded ${wantEvents.length}`)
    }
  }
}

function parseArgs() {
  const args = process.argv.slice(2)
  let shardIndex = 0
  let shardCount = 1
  let filterName = null
  let verifyPerturbation = false

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--shard' && i + 1 < args.length) {
      const parts = args[++i].split('/')
      shardIndex = parseInt(parts[0], 10)
      shardCount = parseInt(parts[1], 10)
    } else if (args[i] === '--fixture' && i + 1 < args.length) {
      filterName = args[++i]
    } else if (args[i] === '--verify-perturbation') {
      verifyPerturbation = true
    }
  }
  return { shardIndex, shardCount, filterName, verifyPerturbation }
}

async function main() {
  const { shardIndex, shardCount, filterName, verifyPerturbation } = parseArgs()

  initSync({ module: readFileSync(WASM_PATH) })

  const tolDoc = JSON.parse(readFileSync(resolve(FIXTURES_DIR, 'tolerances-rustprop.json'), 'utf8'))
  const declaredTols = new Map()
  for (const [k, v] of Object.entries(tolDoc.fixtures || {})) {
    declaredTols.set(k, v.relative)
  }
  const absolutes = new Map()
  for (const [k, v] of Object.entries(tolDoc.absolute || {})) {
    const vars = new Map()
    for (const [varName, varEntry] of Object.entries(v.variables || {})) {
      vars.set(varName.toLowerCase(), varEntry.absolute)
    }
    absolutes.set(k, vars)
  }
  const solverFloors = new Map(Object.entries(tolDoc.solver_floor || {}))

  let browserDivergencesDoc = { fixtures: {} }
  try {
    browserDivergencesDoc = JSON.parse(readFileSync(resolve(FIXTURES_DIR, 'browser-divergences.json'), 'utf8'))
  } catch (e) {}
  const browserDivergences = new Map(
    Object.entries(browserDivergencesDoc.fixtures || {}).map(([k, v]) => [k, v.relative])
  )

  let allFiles = readdirSync(GOLDEN_DIR)
    .filter(f => f.endsWith('.json'))
    .sort()

  if (filterName) {
    allFiles = allFiles.filter(f => f.replace(/\.json$/, '').includes(filterName))
  }

  // Partition by i % shardCount == shardIndex
  const myFiles = allFiles.filter((_, i) => i % shardCount === shardIndex)
  console.log(`wasm-parity: replaying shard ${shardIndex}/${shardCount} (${myFiles.length} of ${allFiles.length} golden fixtures)`)

  const failures = []
  let passed = 0

  const t0 = performance.now()

  for (const file of myFiles) {
    const stem = file.replace(/\.json$/, '')
    const fixture = JSON.parse(readFileSync(resolve(GOLDEN_DIR, file), 'utf8'))
    const req = {}
    if (fixture.request?.variableInfo) req.variableInfo = fixture.request.variableInfo
    if (fixture.request?.stopCriteria) req.stopCriteria = fixture.request.stopCriteria
    if (solverFloors.has(stem)) {
      req.stopCriteria = { ...(req.stopCriteria || {}), relativeResiduals: solverFloors.get(stem) }
    }
    if (fixture.function_tables) req.functionTables = fixture.function_tables

    let out
    try {
      out = JSON.parse(solve(fixture.source, JSON.stringify(req)))
    } catch (e) {
      failures.push({ fixture: stem, detail: `WASM trap/panic: ${e.message}` })
      continue
    }

    const expectError = fixture.expect.error
    if (expectError != null) {
      if (out.success) {
        failures.push({ fixture: stem, detail: `Java failed with ${expectError.type || 'error'} but WASM solved` })
      } else {
        passed++
      }
      continue
    }

    if (!out.success) {
      failures.push({ fixture: stem, detail: `Solve failed: ${out.error?.message || JSON.stringify(out.error)}` })
      continue
    }

    let fixtureFailed = false
    const fail = (detail) => {
      fixtureFailed = true
      failures.push({ fixture: stem, detail })
    }

    // Verify non-finite values
    for (const v of out.variables || []) {
      if (!Number.isFinite(v.value)) {
        fail(`variable ${v.name} is non-finite: ${v.value}`)
        break
      }
    }
    if (fixtureFailed) continue

    let relTol = declaredTols.get(stem) ?? REL_TOL
    if (browserDivergences.has(stem)) {
      relTol = Math.max(relTol, browserDivergences.get(stem))
    }
    const coveredAbs = absolutes.get(stem)
    const actualVars = new Map((out.variables || []).map(v => [v.name.toLowerCase(), v.value]))

    // Check variables
    for (const [k, expected] of Object.entries(fixture.expect.variables || {})) {
      let exp = expected
      if (verifyPerturbation && stem === 'canonical' && k === 'x') {
        exp = expected + 0.5 // deliberate perturbation
      }

      const act = actualVars.get(k.toLowerCase())
      if (act === undefined) {
        fail(`missing variable \`${k}\``)
        break
      }
      const absTol = coveredAbs?.get(k.toLowerCase())
      if (absTol !== undefined) {
        if (Math.abs(act - exp) > absTol) {
          fail(`\`${k}\` = ${act} but Java got ${exp} (abs diff ${Math.abs(act - exp)}, abs tol ${absTol})`)
          break
        }
      } else {
        if (!close(act, exp, relTol)) {
          fail(`\`${k}\` = ${act} but Java got ${exp} (rel ${relDiff(act, exp).toExponential()}, tol ${relTol.toExponential()})`)
          break
        }
      }
    }

    if (fixtureFailed) continue

    // Check ODE tables if recorded in golden
    if (fixture.expect.ode_tables !== undefined && fixture.expect.ode_tables !== null) {
      compareOdeTables(
        fixture.expect.ode_tables,
        out.odeTables || [],
        relTol,
        fail,
        verifyPerturbation && stem === 'dyn_plain_ode'
      )
    }

    if (!fixtureFailed) {
      passed++
    }
  }

  const elapsed = ((performance.now() - t0) / 1000).toFixed(2)
  console.log(`parity-shard: shard ${shardIndex}/${shardCount} — replayed ${myFiles.length} of corpus ${allFiles.length}, passed ${passed}, failed ${failures.length} in ${elapsed}s`)

  if (failures.length > 0) {
    console.error(`\nFailures (${failures.length}):`)
    for (const f of failures.slice(0, 20)) {
      console.error(`  [${f.fixture}] ${f.detail}`)
    }
    process.exit(1)
  }
}

main().catch(err => {
  console.error('Fatal error:', err)
  process.exit(1)
})
