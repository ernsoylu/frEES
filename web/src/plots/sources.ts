import type { TableSpec } from '../tables'
import type { VariableResult } from '../api'
import type { PlotSpec, PlotSource } from './types'

function matchChannel(targetVars: Iterable<string>, n: string): boolean {
  const norm = n.toLowerCase().replaceAll('$', '.')
  for (const tv of targetVars) {
    if (tv.toLowerCase().replaceAll('$', '.') === norm) return true
  }
  return false
}

/** Old bindings migrate only when exactly one source contains all requested channels. */
export function resolvePlotSource(spec: PlotSpec, tables: TableSpec[], variables: VariableResult[]): PlotSource | undefined {
  if (spec.source) return spec.source
  if (spec.kind !== 'xy') return undefined
  const names = [...(spec.xy.chartType === 'histogram' ? [] : [spec.xy.xVar]), ...spec.xy.yVars, ...(spec.xy.y2Vars ?? []), spec.xy.zVar, spec.xy.sizeVar].filter((s): s is string => !!s)
  if (!names.length) return undefined
  const candidates: PlotSource[] = tables.flatMap((t) => {
    if (t.kind === 'parametric' && names.every((n) => matchChannel(t.vars, n))) {
      return [{ kind: 'table' as const, tableId: t.id, data: t.origin === 'ode' || t.results.length === 0 ? 'inputs' as const : 'solved' as const }]
    }
    if (t.kind === 'function') {
      const allCols = new Set([t.argName, ...t.columns])
      if (names.every((n) => matchChannel(allCols, n))) {
        return [{ kind: 'table' as const, tableId: t.id, data: 'inputs' as const }]
      }
    }
    return []
  })
  if (names.every((n) => variables.some((v) => matchChannel([v.name], n) || v.name.toLowerCase().startsWith(`${n.toLowerCase()}[`)))) candidates.push({ kind: 'arrays' })
  return candidates.length === 1 ? candidates[0] : undefined
}
