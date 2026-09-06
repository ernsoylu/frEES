// tablesGrid/csv.test.ts
//
// The Import CSV… reader (decision D11). These are the edge cases that made
// papaparse look necessary and turned out not to be: quoting, CRLF, a BOM,
// alternative delimiters, ragged rows and headers that are missing, blank or
// duplicated. The last two cases walk the whole Import CSV… path — parse, then
// functionSpecFromXY — because that pairing is the feature.

import { describe, expect, it } from 'vitest'
import { detectDelimiter, parseCsvTable, serializeCsv, splitCsvRows } from './csv'
import { functionSpecFromXY } from './composeTables'

const numbers = (col: { values: Float64Array }) => Array.from(col.values)

describe('splitCsvRows', () => {
  it('splits plain rows and drops the trailing newline', () => {
    expect(splitCsvRows('a,b\n1,2\n', ',')).toEqual([
      ['a', 'b'],
      ['1', '2'],
    ])
  })

  it('keeps delimiters, quotes and newlines inside quoted fields', () => {
    const text = 'name,note\n"Smith, J.","said ""hi""\nthen left"\n'
    expect(splitCsvRows(text, ',')).toEqual([
      ['name', 'note'],
      ['Smith, J.', 'said "hi"\nthen left'],
    ])
  })

  it('treats a quote in the middle of a field as literal text', () => {
    expect(splitCsvRows('a,3" pipe\n', ',')).toEqual([['a', '3" pipe']])
  })

  it('handles CRLF, bare CR and a UTF-8 BOM', () => {
    expect(splitCsvRows('﻿a,b\r\n1,2\r3,4', ',')).toEqual([
      ['a', 'b'],
      ['1', '2'],
      ['3', '4'],
    ])
  })

  it('preserves empty fields, including a trailing one', () => {
    expect(splitCsvRows('1,,3,\n', ',')).toEqual([['1', '', '3', '']])
  })
})

describe('detectDelimiter', () => {
  it.each([
    [',', 'x,y\n1,2\n'],
    [';', 'x;y\n1;2\n'],
    ['\t', 'x\ty\n1\t2\n'],
    ['|', 'x|y\n1|2\n'],
  ])('finds %j', (delimiter, text) => {
    expect(detectDelimiter(text)).toBe(delimiter)
  })

  it('ignores delimiters inside quoted header cells', () => {
    expect(detectDelimiter('"a;b;c";"d;e"\n1;2\n')).toBe(';')
  })

  it('skips leading blank lines to find the header', () => {
    expect(detectDelimiter('\n\n a;b \n1;2\n')).toBe(';')
  })

  it('falls back to comma for a single-column file', () => {
    expect(detectDelimiter('value\n1\n2\n')).toBe(',')
  })
})

describe('parseCsvTable', () => {
  it('reads a header row and numeric columns', () => {
    const t = parseCsvTable('time,speed\n0,10\n1,20.5\n2,-3e2\n')
    expect(t.headerless).toBe(false)
    expect(t.rowCount).toBe(3)
    expect(t.columns.map((c) => c.name)).toEqual(['time', 'speed'])
    expect(numbers(t.columns[1])).toEqual([10, 20.5, -300])
    expect(t.columns[1].numericCount).toBe(3)
  })

  it('names columns positionally when the file opens on data', () => {
    const t = parseCsvTable('0,10\n1,20\n')
    expect(t.headerless).toBe(true)
    expect(t.columns.map((c) => c.name)).toEqual(['col1', 'col2'])
    expect(t.rowCount).toBe(2)
  })

  it('fills a blank header cell with its position', () => {
    const t = parseCsvTable('time,,speed\n0,x,10\n')
    expect(t.columns.map((c) => c.name)).toEqual(['time', 'col2', 'speed'])
  })

  it('de-duplicates repeated header names case-insensitively', () => {
    const t = parseCsvTable('T,T,t\n1,2,3\n')
    expect(t.columns.map((c) => c.name)).toEqual(['T', 'T (2)', 't (3)'])
  })

  it('squares up ragged rows, padding short ones with NaN', () => {
    const t = parseCsvTable('a,b\n1,2,3\n4\n')
    expect(t.columns).toHaveLength(3)
    expect(t.columns[2].name).toBe('col3')
    expect(numbers(t.columns[1])).toEqual([2, Number.NaN])
    expect(numbers(t.columns[2])).toEqual([3, Number.NaN])
  })

  it('reads blank and non-numeric cells as NaN and counts what is numeric', () => {
    const t = parseCsvTable('x,y\n1,2\n2,\n3,n/a\n4,NaN\n5,6\n')
    expect(numbers(t.columns[1])).toEqual([2, Number.NaN, Number.NaN, Number.NaN, 6])
    expect(t.columns[1].numericCount).toBe(2)
    expect(t.columns[0].numericCount).toBe(5)
  })

  it('drops blank lines without shifting rows', () => {
    const t = parseCsvTable('x,y\n\n1,2\n\n\n3,4\n')
    expect(t.rowCount).toBe(2)
    expect(numbers(t.columns[0])).toEqual([1, 3])
  })

  it('reads a semicolon export with a unit row as skippable NaN', () => {
    const t = parseCsvTable('Time;Torque\n[s];[Nm]\n0;1.5\n1;2.5\n')
    expect(t.delimiter).toBe(';')
    expect(t.rowCount).toBe(3)
    expect(numbers(t.columns[1])).toEqual([Number.NaN, 1.5, 2.5])
  })

  it('returns an empty table for empty or whitespace-only text', () => {
    for (const text of ['', '\n\n', '   \n \r\n']) {
      const t = parseCsvTable(text)
      expect(t.columns).toEqual([])
      expect(t.rowCount).toBe(0)
    }
  })

  it('honours an explicit delimiter over the sniffer', () => {
    const t = parseCsvTable('a;b\n1;2\n', ',')
    expect(t.columns.map((c) => c.name)).toEqual(['a;b'])
  })
})

describe('parse → function table (the Import CSV… path)', () => {
  it('turns two columns into a 1-D function table, skipping unusable pairs', () => {
    const t = parseCsvTable('time,speed,label\n0,10,a\n1,,b\n2,30,c\n3,x,d\n4,50,e\n')
    const x = t.columns[0]
    const y = t.columns[1]
    const { spec, usedRows, skippedRows, decimated } = functionSpecFromXY({
      name: 'speed',
      argName: 'time',
      xs: x.values,
      ys: y.values,
    })
    expect(usedRows).toBe(3)
    expect(skippedRows).toBe(2)
    expect(decimated).toBe(false)
    expect(spec.rows).toEqual([
      { x: '0', ys: ['10'] },
      { x: '2', ys: ['30'] },
      { x: '4', ys: ['50'] },
    ])
    expect(t.columns[2].numericCount).toBe(0)
  })

  it('decimates a long recording to the table row cap', () => {
    const lines = ['t,v']
    for (let i = 0; i < 12_000; i++) lines.push(`${i * 0.01},${Math.sin(i)}`)
    const t = parseCsvTable(lines.join('\r\n'))
    expect(t.rowCount).toBe(12_000)
    const { spec, decimated } = functionSpecFromXY({
      name: 'v',
      argName: 't',
      xs: t.columns[0].values,
      ys: t.columns[1].values,
      reduction: 'decimate',
    })
    expect(decimated).toBe(true)
    expect(spec.rows).toHaveLength(5000)
    expect(spec.rows[0].x).toBe('0')
    expect(Number(spec.rows[4999].x)).toBeCloseTo(119.99, 6)
  })
})

describe('CSV interpretation escape hatches (T8)', () => {
  it('reserves already-suffixed headers against the occupied namespace', () => {
    expect(parseCsvTable('a,a,a (2)\n1,2,3').columns.map((c) => c.name)).toEqual([
      'a',
      'a (3)',
      'a (2)',
    ])
  })

  it('keeps a mixed numeric family header as a header, not data', () => {
    const t = parseCsvTable('x,1000,2000\n1,2,3')
    expect(t.headerless).toBe(false)
    expect(t.columns.map((c) => c.name)).toEqual(['x', '1000', '2000'])
    expect(t.rowCount).toBe(1)
  })

  it('reports malformed quoting instead of silently concatenating fields', () => {
    expect(() => splitCsvRows('x,"open', ',')).toThrow('Unclosed')
    expect(() => splitCsvRows('"x"oops,y', ',')).toThrow('closing quote')
  })

  it('keeps newlines inside quoted fields as one record', () => {
    const t = parseCsvTable('x,y,note\n1,2,"line 1\nline 2"\n3,4,ok\n')
    expect(t.rowCount).toBe(2)
    expect(numbers(t.columns[0])).toEqual([1, 3])
    expect(numbers(t.columns[1])).toEqual([2, 4])
    expect(t.columns[2].values).toHaveLength(2)
    expect(t.rejectedRows ?? []).toEqual([])
  })

  it('reports ragged records and numeric-looking failures, not label columns', () => {
    const t = parseCsvTable('x,y,note\n1,2,ok\n3,4\n5,n/a,bad\n6,1.2.3,x\n')
    expect(t.rejectedRows).toEqual([
      { record: 3, reason: 'Ragged record; missing cells remain blank' },
      { record: 5, reason: 'Column 2: invalid number' },
    ])
  })

  it('quotes CR as well as LF and skips comment lines on re-import', () => {
    const text = serializeCsv([['a', 'b\rc'], ['1', '2']], ['name=demo', 'values=exact'])
    expect(text).toContain('"b\rc"')
    expect(text.startsWith('# name=demo')).toBe(true)
    const parsed = parseCsvTable(text)
    expect(parsed.columns.map((c) => c.name)).toEqual(['a', 'b\rc'])
    expect(parsed.rowCount).toBe(1)
  })

  it('honours explicit delimiter, header, decimal-comma and unit-row choices', () => {
    const csv = parseCsvTable('p;t\nkPa;C\n1,5;20,25\n1.234,56;7,5', ';', {
      header: 'yes',
      decimal: ',',
      unitRow: true,
    })
    expect(numbers(csv.columns[0])).toEqual([1.5, 1234.56])
    expect(numbers(csv.columns[1])).toEqual([20.25, 7.5])
    expect(csv.columns[0].unit).toBe('kPa')
    expect(csv.columns[1].unit).toBe('C')
    expect(csv.rowCount).toBe(2)
  })
})
