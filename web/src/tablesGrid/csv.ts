// tablesGrid/csv.ts
//
// CSV in and out for the Tables workbook.
//
// OUT — `downloadValuesAsCsv`: the active table as a .csv download.
//
// IN — `parseCsvTable`: the small reader that took over from papaparse when
// the Data Analyzer left (decision D11). Import CSV… needs one thing the
// analyzer's streaming ingest never got to be: simple. There is no time base
// to detect, no strictly-monotonic contract to enforce and no worker — a
// function table is a header row plus numeric columns, so that is all this
// parses. Quoted fields (with embedded delimiters, quotes and newlines), CRLF,
// a UTF-8 BOM and `;`/tab/`|` delimiters are handled because real exports have
// them. Unterminated quotes fail the parse. Blank and non-numeric cells become
// NaN and are skipped downstream by `functionSpecFromXY`; ragged records and
// numeric-looking failures are listed on `rejectedRows` so the dialog can show
// them. Delimiter/header/decimal/unit-row overrides come from the import dialog.

/** Quote-double `"` and wrap any cell containing a comma, quote, CR or LF. */
export function csvCell(value: unknown): string {
  let val = String(value ?? '').replaceAll('"', '""')
  if (/[",\r\n]/.test(val)) val = `"${val}"`
  return val
}

export function serializeCsv(values: unknown[][], comments: readonly string[] = []): string {
  const lines = comments.map((comment) => `# ${comment.replace(/[\r\n]+/g, ' ')}`)
  for (const row of values) lines.push(row.map(csvCell).join(','))
  return lines.join('\n')
}

/** CSV download for the Tables workbook. */
export function downloadValuesAsCsv(
  values: unknown[][],
  filename: string,
  comments: readonly string[] = [],
): void {
  const blob = new Blob([serializeCsv(values, comments)], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  a.remove()
  URL.revokeObjectURL(url)
}

// ---------------------------------------------------------------------------
// Reading

/** Delimiters sniffed from the header line, in no particular order — the one
 *  that splits it into the most fields wins, comma on a tie. */
const DELIMITERS = [',', ';', '\t', '|'] as const

/** One parsed column: the numbers are positional (one entry per data row) so
 *  any two columns pair up by row index without re-walking the file. */
export interface CsvColumn {
  /** Header text, de-duplicated and never blank (`col3` when unnamed). */
  name: string
  /** Zero-based position in the file. */
  index: number
  /** One entry per data row; blank and non-numeric cells are NaN. */
  values: Float64Array
  /** How many of `values` are finite — 0 marks a text column. */
  numericCount: number
  unit?: string
}

export interface CsvTable {
  columns: CsvColumn[]
  /** Data rows (the header row, if any, excluded). */
  rowCount: number
  /** True when the file opened straight into data: columns are named colN. */
  headerless: boolean
  /** The delimiter actually used. */
  delimiter: string
  rejectedRows?: { record: number; reason: string }[]
}

/** Decimal-comma cells (`1,5` or `1.234,56`) become JavaScript numbers. */
export function applyDecimalConvention(cell: string, decimal: '.' | ',' = '.'): string {
  if (decimal !== ',') return cell
  const trimmed = cell.trim()
  const comma = trimmed.lastIndexOf(',')
  if (comma < 0) return cell
  return trimmed.slice(0, comma).replaceAll('.', '') + '.' + trimmed.slice(comma + 1)
}

/** A cell's numeric value; blank, text and `NaN` all read as NaN. */
export function cellToNumber(cell: string, decimal: '.' | ',' = '.'): number {
  const trimmed = applyDecimalConvention(cell, decimal).trim()
  if (trimmed === '') return Number.NaN
  return Number(trimmed)
}

function isNumericCell(cell: string, decimal: '.' | ',' = '.'): boolean {
  const trimmed = applyDecimalConvention(cell, decimal).trim()
  return trimmed !== '' && !Number.isNaN(Number(trimmed))
}

/** True when a non-numeric cell looks like a failed number, not a label. */
function looksLikeNumber(cell: string): boolean {
  const trimmed = cell.trim()
  if (trimmed === '') return false
  if (/^(nan|[+-]?inf(?:inity)?)$/i.test(trimmed)) return true
  if (/\s/.test(trimmed)) return false
  return /[0-9]/.test(trimmed) && /^[+-]?[0-9.,eE+]+$/.test(trimmed)
}

/**
 * Splits CSV text into raw cells, RFC 4180 style: `"` opens a quoted field at
 * its start (leading spaces tolerated), `""` is a literal quote inside one,
 * and CR / LF / CRLF all end a row. A UTF-8 BOM is stripped. Rows are ragged
 * exactly as the file is — squaring them up is `parseCsvTable`'s job.
 * Optional `maxRows` stops parsing after accumulating that many data rows.
 */
function consumeQuoted(
  text: string,
  start: number,
  len: number,
  recordNum: number,
): { nextIndex: number; content: string } {
  let i = start
  let chunkStart = start
  let content = ''
  while (i < len) {
    if (text[i] === '"') {
      if (text[i + 1] === '"') {
        content += text.slice(chunkStart, i) + '"'
        i += 2
        chunkStart = i
      } else {
        content += text.slice(chunkStart, i)
        return { nextIndex: i + 1, content }
      }
    } else {
      i++
    }
  }
  throw new Error(`Unclosed quoted field in record ${recordNum}`)
}

export function splitCsvRows(text: string, delimiter: string, maxRows?: number): string[][] {
  const rows: string[][] = []
  let row: string[] = []
  let field = ''
  let closedQuote = false
  let started = false // this row has content (guards the trailing newline)
  let i = text.charCodeAt(0) === 0xfeff ? 1 : 0
  const len = text.length

  const endField = () => {
    row.push(field)
    field = ''
    closedQuote = false
    started = true
  }
  const endRow = () => {
    endField()
    rows.push(row)
    row = []
    started = false
  }

  let chunkStart = i

  while (i < len) {
    const ch = text[i]

    if (closedQuote && ch !== delimiter && ch !== '\r' && ch !== '\n') {
      if (!ch.trim()) {
        i++
        chunkStart = i
        continue
      }
      throw new Error(`Unexpected text after closing quote in record ${rows.length + 1}`)
    }

    if (ch === '"' && field === '' && text.slice(chunkStart, i).trim() === '') {
      const quoted = consumeQuoted(text, i + 1, len, rows.length + 1)
      field = quoted.content
      i = quoted.nextIndex
      chunkStart = i
      closedQuote = true
      continue
    }

    if (ch === delimiter) {
      field += text.slice(chunkStart, i)
      endField()
      i++
      chunkStart = i
      continue
    }

    if (ch === '\r' || ch === '\n') {
      field += text.slice(chunkStart, i)
      if (ch === '\r' && text[i + 1] === '\n') i++
      endRow()
      i++
      chunkStart = i
      if (maxRows !== undefined && rows.length >= maxRows) {
        return rows
      }
      continue
    }

    i++
  }

  if (chunkStart < len) {
    field += text.slice(chunkStart, len)
  }
  // A file ending in a newline must not produce a phantom last row.
  if (field !== '' || started) endRow()
  return rows
}

/** Counts `delimiter` outside quoted fields. */
function countOutsideQuotes(line: string, delimiter: string): number {
  let n = 0
  let quoted = false
  for (let i = 0; i < line.length; i++) {
    const ch = line[i]
    if (ch === '"') {
      if (quoted && line[i + 1] === '"') i++
      else quoted = !quoted
    } else if (ch === delimiter && !quoted) n++
  }
  return n
}

/**
 * Sniffs the delimiter from the first line carrying content. Comma wins ties
 * and an undelimited file (one column), which is the honest answer: a
 * single-column CSV has no delimiter to find.
 */
export function detectDelimiter(text: string): string {
  const head = text.slice(0, 1 << 16)
  let line = ''
  for (const candidate of head.split(/\r\n|\r|\n/)) {
    if (candidate.trim() !== '') {
      line = candidate
      break
    }
  }
  let best = ','
  let bestCount = 0
  for (const d of DELIMITERS) {
    const n = countOutsideQuotes(line, d)
    if (n > bestCount) {
      bestCount = n
      best = d
    }
  }
  return best
}

/** Makes every name non-blank and unique (case-insensitively), preserving the
 *  first occurrence and suffixing the rest `name (2)`, `name (3)`, … */
function uniqueNames(raw: string[]): string[] {
  const bases = raw.map((name, i) => name.trim() || `col${i + 1}`)
  const occupied = new Set(bases.map((name) => name.toLowerCase()))
  const seen = new Set<string>()
  return bases.map((base) => {
    let name = base
    for (let suffix = 2; seen.has(name.toLowerCase()); suffix++) {
      name = `${base} (${suffix})`
      while (occupied.has(name.toLowerCase())) name = `${base} (${++suffix})`
    }
    seen.add(name.toLowerCase())
    return name
  })
}

/**
 * Parses CSV/TSV text into named numeric columns.
 *
 * The first content row is the header unless every cell is a number, in which
 * case the file is treated as headerless and the columns are named `col1…colN`.
 * A mixed row such as the curve-family header `x,1000,2000` therefore stays a
 * header. Ragged rows are squared up: the column count is the widest row, short
 * rows pad with NaN, and columns past the header are named positionally. Blank
 * lines are dropped.
 *
 * `delimiter` overrides the sniffer. `options` is the dialog's header /
 * decimal / unit-row interpretation.
 */
export interface CsvOptions {
  header?: 'auto' | 'yes' | 'no'
  decimal?: '.' | ','
  unitRow?: boolean
}

function populateCsvColumns(
  dataRows: string[][],
  columnCount: number,
  offset: number,
  decimal: '.' | ',',
  columns: CsvColumn[],
  rejectedRows: { record: number; reason: string }[],
) {
  for (let r = 0; r < dataRows.length; r++) {
    const row = dataRows[r]
    if (row.length !== columnCount) {
      rejectedRows.push({
        record: r + offset + 1,
        reason: 'Ragged record; missing cells remain blank',
      })
    }
    for (let c = 0; c < columnCount; c++) {
      const cell = row[c] ?? ''
      const value = cellToNumber(cell, decimal)
      if (looksLikeNumber(cell) && !Number.isFinite(value)) {
        rejectedRows.push({
          record: r + offset + 1,
          reason: `Column ${c + 1}: invalid number`,
        })
      }
      columns[c].values[r] = value
      if (Number.isFinite(value)) columns[c].numericCount++
    }
  }
}

export function parseCsvTable(
  text: string,
  delimiter?: string,
  options: CsvOptions = {},
  maxRows?: number,
): CsvTable {
  const delim = delimiter ?? detectDelimiter(text)
  const rows = splitCsvRows(text, delim, maxRows).filter((r) => {
    if (!r.some((c) => c.trim() !== '')) return false
    return !(r[0]?.trim().startsWith('#') && r.every((c, i) => i === 0 || c.trim() === ''))
  })
  if (rows.length === 0) {
    return { columns: [], rowCount: 0, headerless: false, delimiter: delim }
  }

  const decimal = options.decimal ?? '.'
  // Auto: a first row of only numbers is data (`0,10`). A mixed row such as
  // the curve-family header `x,1000,2000` stays a header.
  const headerless =
    options.header === 'no' ||
    (options.header !== 'yes' && rows[0].every((c) => isNumericCell(c, decimal)))
  const offset = (headerless ? 0 : 1) + (options.unitRow ? 1 : 0)
  const unitCells = options.unitRow ? rows[headerless ? 0 : 1] : undefined
  const dataRows = rows.slice(offset)
  const columnCount = rows.reduce((max, r) => Math.max(max, r.length), 0)
  const headerCells = headerless ? [] : rows[0]
  const names = uniqueNames(
    Array.from({ length: columnCount }, (_, c) => headerCells[c] ?? ''),
  )

  const columns: CsvColumn[] = names.map((name, index) => ({
    name,
    index,
    values: new Float64Array(dataRows.length),
    numericCount: 0,
    unit: unitCells?.[index]?.trim() || undefined,
  }))
  const rejectedRows: { record: number; reason: string }[] = []
  populateCsvColumns(dataRows, columnCount, offset, decimal, columns, rejectedRows)

  return { columns, rowCount: dataRows.length, headerless, delimiter: delim, rejectedRows }
}

/**
 * Fast preview of the first `maxRows` data rows of a CSV file.
 * Avoids full file parsing on large (10–64 MiB) files for instant modal UI preview.
 */
export function parseCsvPreview(
  text: string,
  maxRows = 50,
  delimiter?: string,
  options: CsvOptions = {},
): CsvTable {
  return parseCsvTable(text, delimiter, options, maxRows)
}

