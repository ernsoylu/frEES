import type { TableSpec } from './tables'

export const MAX_TABLE_CELLS = 500_000
export const MAX_TABLE_BYTES = 64 * 1024 * 1024

export function tableShape(value: unknown): value is TableSpec {
  if (!value || typeof value !== 'object') return false
  const t = value as Record<string, unknown>
  if (typeof t.id !== 'string' || !t.id || typeof t.name !== 'string' || !Array.isArray(t.rows)) return false
  if (t.kind === 'parametric') return Array.isArray(t.vars) && t.vars.every((v) => typeof v === 'string')
    && t.rows.every((r) => r && typeof r.id === 'string' && r.values && typeof r.values === 'object' && Object.values(r.values).every((v) => typeof v === 'string'))
  return t.kind === 'function' && typeof t.argName === 'string' && typeof t.paramName === 'string'
    && Array.isArray(t.columns) && t.columns.every((c) => typeof c === 'string')
    && t.rows.every((r) => r && typeof r.x === 'string' && Array.isArray(r.ys) && r.ys.length === (t.columns as unknown[]).length && r.ys.every((y: unknown) => typeof y === 'string'))
}

/** Drafts remain editable. Consumers must refuse invalid numerical inputs with locations. */
export function tableInputIssues(t: TableSpec, checkCapacity = true): string[] {
  const issues: string[] = []
  const names = t.kind === 'parametric' ? t.vars : [t.argName, ...(t.paramName ? [t.paramName] : [])]
  if (!t.name.trim() || (t.kind === 'function' && !/^[A-Za-z]\w*$/.test(t.name))) issues.push('Invalid table name')
  if (new Set(names.map((n) => n.toLowerCase())).size !== names.length || names.some((n) => !n.trim())) issues.push('Column names must be nonempty and unique')
  const width = t.kind === 'parametric' ? t.vars.length : t.columns.length + 1
  if (checkCapacity && t.source !== 'code' && (t.rows.length > 5000 || t.rows.length * width > MAX_TABLE_CELLS)) issues.push(`Table exceeds 5,000 rows or ${MAX_TABLE_CELLS} cells`)
  if (t.kind === 'parametric' && new Set(t.rows.map((r) => r.id)).size !== t.rows.length) issues.push('Duplicate row IDs')
  t.rows.forEach((row, i) => {
    const values = t.kind === 'parametric'
      ? t.vars.map((v) => [v, (row as typeof t.rows[number]).values[v] ?? ''])
      : [[t.argName, (row as { x: string }).x], ...(row as { ys: string[] }).ys.map((y, j) => [`curve ${j + 1}`, y])]
    for (const [name, raw] of values) if (raw.trim() && !Number.isFinite(Number(raw))) issues.push(`Row ${i + 1}, ${name}: invalid number “${raw}”`)
  })
  if (t.kind === 'function' && !t.is1D) t.columns.forEach((c, i) => {
    if (!c.trim() || !Number.isFinite(Number(c))) issues.push(`Curve ${i + 1}: invalid family value`)
  })
  if (JSON.stringify(t).length * 2 > MAX_TABLE_BYTES) issues.push('Table exceeds the 64 MiB storage budget')
  if (t.recoveredContent !== undefined) issues.push('Malformed imported content is preserved in this table’s recoveredContent field; repair the original file before solving')
  return issues
}

export function requireValidTable(t: TableSpec, checkCapacity = true): void {
  const issues = tableInputIssues(t, checkCapacity)
  if (issues.length) throw new Error(`${t.name}: ${issues.slice(0, 12).join('; ')}`)
}
