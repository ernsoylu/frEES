import type { SolveResponse } from './api'

/**
 * Printable calculation report: a fully self-contained HTML document (inline
 * styles, no external assets, so it renders in a bare new window and prints
 * cleanly to PDF from the browser). Sections: header with run stats, the
 * document source, the solved variables with units and uncertainties, and any
 * unit-consistency warnings — the deliverable an engineering review actually
 * wants from a calc sheet.
 */

function esc(s: string): string {
  return s
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
}

function fmt(v: number): string {
  if (!Number.isFinite(v)) return String(v)
  const a = Math.abs(v)
  if (a !== 0 && (a < 1e-4 || a >= 1e7)) return v.toExponential(6)
  return String(Math.round(v * 1e9) / 1e9)
}

export function buildReportHtml(
  projectName: string,
  documentText: string,
  response: SolveResponse,
  generatedAt: Date = new Date(),
): string {
  const stats = response.stats
  const statsLine = stats
    ? `${stats.equations} equations · ${stats.unknowns} unknowns · ${stats.blocks} blocks · ` +
      `${stats.iterations} iterations · max residual ${fmt(stats.maxResidual)}`
    : ''
  const rows = (response.variables ?? [])
    .map((v) => {
      const unc = v.uncertainty && v.uncertainty > 0 ? `± ${fmt(v.uncertainty)}` : ''
      return `<tr><td class="mono">${esc(v.name)}</td><td class="num mono">${fmt(v.value)}</td>` +
        `<td class="num mono">${esc(unc)}</td><td class="mono unit">${esc(v.units || '')}</td></tr>`
    })
    .join('\n')
  const warnings = (response.unitWarnings ?? [])
    .map((w) => `<li>${esc(w)}</li>`)
    .join('\n')
  const commit = (globalThis as { __BUILD_COMMIT__?: string }).__BUILD_COMMIT__

  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>${esc(projectName)} — frees report</title>
<style>
  :root { color-scheme: light; }
  body { font-family: Georgia, 'Times New Roman', serif; color: #1a1a1a; margin: 2.2rem auto; max-width: 52rem; padding: 0 1.5rem; }
  h1 { font-size: 1.5rem; margin: 0 0 0.15rem; }
  h2 { font-size: 1.05rem; border-bottom: 1px solid #999; padding-bottom: 0.2rem; margin-top: 1.6rem; }
  .meta { color: #555; font-size: 0.85rem; margin-bottom: 0.4rem; }
  .mono { font-family: 'Cascadia Code', Consolas, Menlo, monospace; font-size: 0.82rem; }
  pre.doc { background: #f6f6f4; border: 1px solid #ddd; border-radius: 4px; padding: 0.8rem; white-space: pre-wrap; overflow-wrap: anywhere; }
  table { border-collapse: collapse; width: 100%; }
  th, td { border-bottom: 1px solid #e2e2e2; text-align: left; padding: 0.22rem 0.6rem 0.22rem 0; vertical-align: baseline; }
  th { font-size: 0.78rem; text-transform: uppercase; letter-spacing: 0.04em; color: #666; }
  td.num { text-align: right; }
  td.unit { color: #555; }
  ul.warnings { color: #7a5b00; }
  footer { margin-top: 2rem; color: #888; font-size: 0.75rem; border-top: 1px solid #ddd; padding-top: 0.4rem; }
  @media print {
    body { margin: 0 auto; }
    pre.doc { break-inside: auto; }
    tr { break-inside: avoid; }
  }
  @page { margin: 18mm 16mm; }
</style>
</head>
<body>
<h1>${esc(projectName)}</h1>
<div class="meta">frees calculation report — generated ${esc(generatedAt.toISOString().replace('T', ' ').slice(0, 16))} UTC</div>
${statsLine ? `<div class="meta">${esc(statsLine)}</div>` : ''}

<h2>Document</h2>
<pre class="doc mono">${esc(documentText)}</pre>

<h2>Solution</h2>
<table>
<thead><tr><th>Variable</th><th style="text-align:right">Value</th><th style="text-align:right">Uncertainty</th><th>Units</th></tr></thead>
<tbody>
${rows}
</tbody>
</table>
${warnings ? `\n<h2>${(response.unitWarnings ?? []).some((w) => /unknown unit/i.test(w)) ? 'Warnings (not verified SI)' : 'Warnings'}</h2>\n<p class="meta">Unknown units are left unconverted and must not be read as verified SI values.</p>\n<ul class="warnings">\n${warnings}\n</ul>` : ''}

<footer>Produced by frees${commit ? ` (build ${esc(commit)})` : ''} — values solved in SI, displayed per the session's unit system.</footer>
</body>
</html>`
}

/** Opens the report in a new window (Blob URL — same pattern as the CSV
 *  exports; every interpolated value is HTML-escaped in buildReportHtml) and
 *  hands it to the browser's print flow, where print-to-PDF is the export
 *  path. Returns false when the popup was blocked so the caller can say so. */
export function openPrintReport(projectName: string, documentText: string,
                                response: SolveResponse): boolean {
  const html = buildReportHtml(projectName, documentText, response)
  const url = URL.createObjectURL(new Blob([html], { type: 'text/html' }))
  const w = globalThis.open(url, '_blank')
  if (!w) {
    URL.revokeObjectURL(url)
    return false
  }
  // Print once the new document has laid out; poll readyState instead of a
  // load listener so an already-loaded race can't strand the dialog.
  const tryPrint = () => {
    if (w.closed) return
    if (w.document?.readyState === 'complete') {
      w.focus()
      w.print()
    } else {
      w.setTimeout(tryPrint, 100)
    }
  }
  w.setTimeout(tryPrint, 150)
  // The window keeps its document alive after revocation; this just frees the blob.
  globalThis.setTimeout(() => URL.revokeObjectURL(url), 60_000)
  return true
}
