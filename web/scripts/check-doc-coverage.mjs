// Documentation coverage gate.
//
// Enforces the correctness invariants that must hold at all times:
//   1. Every authored reference page names a symbol that actually exists in the
//      backend (no orphan pages drifting from the implementation).
//   2. Every example id a page binds to (frontmatter `examples:` / inline
//      [Run: id]) exists in the verified examples library.
//   3. Every "See also" cross-link (frontmatter `related:`) resolves to a page
//      that actually exists (no dangling badges to deleted/renamed/absent pages).
//   4. Every function-call mention in the guide prose (src/docs/*.md inline code,
//      `name(`) names a real backend callable — so a guide can't teach a built-in
//      that doesn't exist (e.g. the cbrt/log2/Dipole class of drift).
//
// Also reports documented-vs-total coverage (informational until Phase 2 fills
// the reference set). Exits non-zero on any invariant violation.
//
// Run: node scripts/build-doc-manifest.mjs && node scripts/check-doc-coverage.mjs

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { parseLibrary } from './parse-library.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SRC = path.join(__dirname, '../src');
const REF_DIR = path.join(SRC, 'docs/reference');
const read = (p) => fs.readFileSync(p, 'utf-8');

// Known symbol slugs from the generated manifest (every documentable family).
const manifest = JSON.parse(read(path.join(REF_DIR, 'function-manifest.json')));
const symbols = new Set();
for (const f of manifest.functions) symbols.add(f.name.toLowerCase());
for (const f of manifest.matrixFunctions) symbols.add(f.name.toLowerCase());
for (const p of manifest.callProcedures) symbols.add(p.name.toLowerCase());
for (const p of manifest.propertyFunctions) symbols.add(p.name.toLowerCase());
for (const c of manifest.components) symbols.add(c.name.toLowerCase());
for (const f of manifest.materials.functions) symbols.add(f.toLowerCase());
for (const r of manifest.replCasOps) symbols.add(r.toLowerCase());

// Complete callable surface for the guide-prose linter: the manifest families
// above, plus the function aliases and dispatch-only arms the manifest records.
const callable = new Set(symbols);
for (const f of manifest.functions) for (const a of f.aliases || []) callable.add(a.toLowerCase());
for (const d of manifest.dispatchOnly || []) {
  callable.add(d.name.toLowerCase());
  for (const a of d.aliases || []) callable.add(a.toLowerCase());
}
// Real backend callables the manifest does not yet enumerate: unit helpers,
// the der() dynamic operator, property-output functions (P_sat/T_sat/MolarMass/…),
// chemistry/EOS outputs, and the low-level BLAS primitives. All verified present
// in the backend. TODO: fold these into build-doc-manifest.mjs so this list shrinks.
const EXTRA_CALLABLES = [
  'convert', 'converttemp', 'der', 'identity',
  'p_sat', 't_sat', 'molarmass', 'heatingvalue', 'stoichafr', 'surfacetension',
  'isidealgas', 'phase$', 'stagnationtemp', 'stagnationpres',
  'scal', 'asum', 'nrm2', 'copy', 'ger', 'gemv', 'gemm', 'axpy',
  // Data Analyzer calculated-signal time operators: handled by
  // TimeSeriesEvaluator (core measurement package), not the function
  // registry — real, documented, callable in calc formulas only.
  'delta', 'movavg', 'delay',
];
for (const n of EXTRA_CALLABLES) callable.add(n);
// Bare math notation that reads like a call in inline code (`num(s)/den(s)`, a
// generic `Tname(...)` placeholder) — not references to a built-in. `name` is
// the same shape: `programming_logic.md` writes `[ … ] = name( … )` to show the
// destructuring FORM, and a metasyntactic placeholder is not a missing
// function. It is listed here rather than fixed in the prose because the prose
// is right.
const NOTATION = new Set(['num', 'den', 'tname', 'name']);

// Example ids in the verified library.
const exampleIds = new Set(
  [...read(path.join(SRC, 'examples.ts')).matchAll(/id:\s*'([^']+)'/g)].map((m) => m[1]),
);

// Walk authored reference pages.
const pages = [];
const walk = (dir) => {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p);
    else if (e.name.endsWith('.md') && !e.name.startsWith('_')) {
      const raw = read(p);
      const fm = raw.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/);
      if (!fm) continue;
      const name = fm[1].match(/^name:\s*(.+)$/m)?.[1].trim();
      if (!name) continue;
      const exFm = fm[1].match(/^examples:\s*\[([^\]]*)\]/m);
      const exFrontmatter = exFm ? exFm[1].split(',').map((s) => s.trim().replace(/^["']|["']$/g, '')).filter(Boolean) : [];
      const exInline = [...fm[2].matchAll(/\[Run:\s*([a-zA-Z0-9_-]+)\s*\]/g)].map((m) => m[1]);
      const relFm = fm[1].match(/^related:\s*\[([^\]]*)\]/m);
      const related = relFm ? relFm[1].split(',').map((s) => s.trim().replace(/^["']|["']$/g, '')).filter(Boolean) : [];
      const generated = /^generated:\s*true\s*$/m.test(fm[1]);
      // Cookbook/guide pages document a task, not a backend symbol — exempt from
      // the symbol-existence check (their example bindings are still validated).
      const guide = /^guide:\s*true\s*$/m.test(fm[1]);

      // Substance tier (non-guide pages). The Syntax block is always a fenced
      // code sample, so it is not a signal; a *worked* example is one in the
      // Examples section or a [Run:] binding.
      const body = fm[2];
      const exSection = body.split(/^##\s+Examples\s*$/m)[1];
      const hasWorkedExample = exInline.length > 0
        || (exSection && /```|\[Run:/.test(exSection.split(/^##\s/m)[0]));
      const refsNonEmpty = /^references:\s*\n\s*-\s+/m.test(fm[1]);
      const bodyLen = body.replace(/\s+/g, ' ').trim().length;
      const tier = hasWorkedExample ? 'rich'
        : (refsNonEmpty || bodyLen >= 500) ? 'reference'
        : 'stub';

      pages.push({ file: path.relative(SRC, p), name, generated, guide, related, tier, examples: [...new Set([...exFrontmatter, ...exInline])] });
    }
  }
};
if (fs.existsSync(REF_DIR)) walk(REF_DIR);

// Every page is addressable by its name (lower-cased) — the slug a `related:`
// cross-link must resolve to.
const pageSlugs = new Set(pages.map((p) => p.name.toLowerCase()));

const errors = [];
for (const pg of pages) {
  if (!pg.guide && !symbols.has(pg.name.toLowerCase())) {
    errors.push(`${pg.file}: documents "${pg.name}" which is not a known backend symbol`);
  }
  for (const id of pg.examples) {
    if (!exampleIds.has(id)) errors.push(`${pg.file}: binds example "${id}" which is not in examples.ts`);
  }
  for (const rel of pg.related) {
    if (!pageSlugs.has(rel.toLowerCase())) {
      errors.push(`${pg.file}: "See also" links to "${rel}" which has no reference page`);
    }
  }
}
// Invariant 4: guide-prose function mentions must name a real callable. Scan the
// inline-code spans of each guide markdown file for a `name(` shape; skip 1–2 char
// identifiers (loop/user variables) and known math notation.
const GUIDE_DIR = path.join(SRC, 'docs');
for (const file of fs.readdirSync(GUIDE_DIR).filter((f) => f.endsWith('.md'))) {
  const text = read(path.join(GUIDE_DIR, file));
  const flagged = new Set();
  for (const m of text.matchAll(/`([A-Za-z_][A-Za-z0-9_]*\$?)\s*\(/g)) {
    const name = m[1].toLowerCase();
    if (name.replace(/\$$/, '').length <= 2 || NOTATION.has(name) || callable.has(name) || flagged.has(name)) {
      continue;
    }
    flagged.add(name);
    errors.push(`docs/${file}: guide documents call "${m[1]}(…)" which is not a known backend function`);
  }
}

const guides = pages.filter((p) => p.guide).length;

// Coverage is presence (every symbol has a page); substance is depth. Report both
// so a 100% presence number doesn't hide signature-only stubs. Tiers (non-guide):
//   rich      — has a worked example (runnable [Run:] or an Examples-section sample)
//   reference — substantive (citations, or a non-trivial body) but no worked example
//   stub      — signature-only: no worked example, no references, short body
const symbolPages = pages.filter((p) => !p.guide);
const rich = symbolPages.filter((p) => p.tier === 'rich').length;
const reference = symbolPages.filter((p) => p.tier === 'reference').length;
const stubs = symbolPages.filter((p) => p.tier === 'stub');

const total = manifest.coverage.documentableSurfaceTotal;
console.log(
  `doc-coverage: ${symbolPages.length}/${total} symbols have a page (${(100 * symbolPages.length / total).toFixed(1)}%) — `
  + `${rich} rich (worked example), ${reference} reference, ${stubs.length} stub (signature-only); ${guides} cookbook guide(s).`,
);
if (stubs.length) {
  // Informational, not a failure: surface the thinnest pages as enrichment work.
  console.log(`\n  ${stubs.length} stub page(s) to enrich (no example, no references):`);
  console.log('    ' + stubs.map((p) => p.name).sort().join(', '));
}

// Exact engine ↔ catalog ↔ page name-set for components. Do not hard-code a count.
const library = parseLibrary();
const libNames = new Set(library.map((c) => c.name.toLowerCase()));
const componentPages = pages.filter((p) => p.file.includes(`${path.sep}components${path.sep}`) || p.file.includes('/components/'));
const pageCompNames = new Set(componentPages.map((p) => p.name.toLowerCase()));
const catalogPath = path.join(SRC, 'componentCatalog.ts');
if (fs.existsSync(catalogPath)) {
  const catalogSrc = read(catalogPath);
  const catalogNames = new Set([...catalogSrc.matchAll(/type:\s*`([^`]+)`/g)].map((m) => m[1].toLowerCase()));
  for (const name of libNames) {
    if (!catalogNames.has(name)) errors.push(`component catalog missing engine component "${name}" (run npm run compile-docs)`);
    if (!pageCompNames.has(name)) errors.push(`no reference page for engine component "${name}" (run node scripts/scaffold-reference-pages.mjs)`);
  }
  for (const name of catalogNames) {
    if (!libNames.has(name)) errors.push(`component catalog has "${name}" which is not in the engine library`);
  }
  for (const name of pageCompNames) {
    if (!libNames.has(name)) errors.push(`component page "${name}" is not in the engine library`);
  }
}

if (errors.length) {
  console.error(`\n✗ ${errors.length} coverage error(s):`);
  for (const e of errors) console.error('  - ' + e);
  process.exit(1);
}
console.log(
  `\n✓ all reference pages name real symbols, bind real examples, cross-link real pages, and guides cite real functions. ` +
    `Component inventory: ${library.length} engine = catalog = pages.`,
);
