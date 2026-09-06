// Generate content-complete reference pages for every documentable symbol that
// does not have a hand-authored page. Every part is drawn from an authoritative
// source — never fabricated:
//   - functions/procedures: manifest signature (parsed into argument tables),
//     the registry description, an example binding found by scanning examples.ts,
//     and a per-CATEGORY standard reference;
//   - components: the real ports, parameters, and CONSTITUTIVE EQUATIONS parsed
//     verbatim from the .frees std-lib body.
//
// Pages are flagged `generated: true` (machine-generated, not hand-verified prose)
// and hand-authored rich pages are never overwritten.
//
// Run: node scripts/build-doc-manifest.mjs && node scripts/scaffold-reference-pages.mjs

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { parseLibrary } from './parse-library.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SRC = path.join(__dirname, '../src');
const REF = path.join(SRC, 'docs/reference');
const REPO = path.resolve(__dirname, '../..');
const read = (p) => fs.readFileSync(p, 'utf-8');
const manifest = JSON.parse(read(path.join(REF, 'function-manifest.json')));

// Per-category standard references (used where a page has no hand-picked citation).
const CATEGORY_REFS = {
  'Special Functions': ['NIST Digital Library of Mathematical Functions (dlmf.nist.gov)'],
  'Stats': ['Standard engineering-statistics references'],
  'Matrix': ['Standard numerical linear-algebra references'],
  'Control Systems': ['Standard control-systems engineering references'],
  'Control': ['Standard control-systems engineering references'],
  'Compressible Flow': ['Standard compressible-flow references'],
  'Heat Transfer': ['Standard heat-transfer and heat-exchanger design references'],
  'Flow Networks': ['Standard fluid-mechanics and hydraulic-resistance references'],
  'Pneumatics': ['ISO 6358 — Pneumatic fluid power: determination of flow-rate characteristics'],
  'Two-Phase Flow': ['Standard two-phase flow and boiling/condensation references'],
  'Atmosphere': ['U.S. Standard Atmosphere, 1976 (NOAA/NASA/USAF)'],
  'Properties (EOS)': ['Standard gas- and liquid-properties references'],
  'Combustion': ['Standard combustion and internal-combustion-engine references'],
  'Calculus': ['Standard numerical-methods references'],
  'Interpolation': ['Standard numerical-methods references'],
  'Fluid Properties': ['CoolProp open-source thermophysical property library'],
  'Solid Materials': ['Standard engineering material-property references'],
  'CAS (REPL)': ['Symja / matheclipse computer-algebra system'],
};
const refsFor = (cat) => CATEGORY_REFS[cat] || [];

// ── Existing authored slugs ──────────────────────────────────────────────────
const existing = new Set();
const walkExisting = (d) => fs.readdirSync(d, { withFileTypes: true }).forEach((e) => {
  const p = path.join(d, e.name);
  if (e.isDirectory()) walkExisting(p);
  else if (e.name.endsWith('.md') && !e.name.startsWith('_')) {
    const nm = read(p).match(/^---\n[\s\S]*?\bname:\s*(.+)$/m);
    if (nm) existing.add(nm[1].trim().toLowerCase());
  }
});
walkExisting(REF);

// ── Example bindings ─────────────────────────────────────────────────────────
const exSrc = read(path.join(SRC, 'examples.ts'));
const exBlocks = [];
{ const re = /id:\s*'([^']+)'([\s\S]*?)(?=\n  \},)/g; let m;
  while ((m = re.exec(exSrc)) !== null) exBlocks.push([m[1], m[2]]); }
const boundExamples = (name) => {
  const re = new RegExp('\\b' + name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&') + '\\s*\\(', 'i');
  return exBlocks.filter(([, t]) => re.test(t)).map(([id]) => id);
};

// ── Helpers ──────────────────────────────────────────────────────────────────
const folderFor = (cat) => cat.toLowerCase().replace(/[()]/g, '').replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
const esc = (s) => String(s).replace(/`/g, "'");
const tagsFrom = (name, cat) => [...new Set(name.toLowerCase().split(/[_\s]+/).concat(folderFor(cat).split('-')))].filter((t) => t.length > 1);

function parseSig(sig) {
  const inside = (sig.match(/\(([^)]*)\)/) || [, ''])[1];
  const [inPart, outPart] = inside.split(':');
  const toArgs = (s) => (s || '').split(',').map((a) => a.trim()).filter(Boolean);
  return { inputs: toArgs(inPart), outputs: toArgs(outPart) };
}
const argRow = (a) => `| \`${a}\` | ${a.endsWith('$') ? 'String' : 'Number'} | Yes | ${a.endsWith('$') ? 'String argument.' : 'Numeric argument.'} |`;

function pageBody({ name, summary, syntax, description, inputs, outputs, references, extra }) {
  const inRows = inputs.length
    ? ['## Input Arguments', '', '| Argument | Type | Required | Description |', '| --- | --- | --- | --- |', ...inputs.map(argRow), '']
    : [];
  const outRows = outputs && outputs.length
    ? ['## Output Arguments', '', '| Argument | Type | Description |', '| --- | --- | --- |', ...outputs.map((o) => `| \`${o}\` | Number/Array | Output value. |`), '']
    : [];
  const refRows = references && references.length
    ? ['## References', '', ...references.map((r, i) => `${i + 1}. ${r}.`), '']
    : [];
  return [
    `# ${name}`, '',
    summary, '',
    '> **Auto-generated** from the function registry. The syntax, description, and arguments are taken directly from the implementation; a worked example and an expanded mathematical derivation are added as the page is curated.', '',
    '## Syntax', '', '```', syntax, '```', '',
    '## Description', '', description, '',
    ...(extra || []),
    ...inRows, ...outRows, ...refRows,
  ].join('\n');
}

function emit(folder, { name, category, summary, related, examples, tags, body, generated = true }) {
  const dir = path.join(REF, folder);
  fs.mkdirSync(dir, { recursive: true });
  const fm = [
    '---',
    `name: ${name}`,
    `category: ${category}`,
    `summary: ${summary.replace(/\n/g, ' ')}`,
    `related: [${(related || []).join(', ')}]`,
    `examples: [${(examples || []).join(', ')}]`,
    `tags: [${(tags || []).join(', ')}]`,
    `references: []`,
    ...(generated ? ['generated: true'] : []),
    '---', '', body, '',
  ].join('\n');
  fs.writeFileSync(path.join(dir, name.replace(/[^A-Za-z0-9_]/g, '_') + '.md'), fm);
  generatedSlugs.add(name.toLowerCase());
}

let made = 0;
const generatedSlugs = new Set();
const done = (name) => existing.has(name.toLowerCase()) || generatedSlugs.has(name.toLowerCase());

// 1. Functions + matrix functions
for (const f of [...manifest.functions, ...manifest.matrixFunctions]) {
  if (done(f.name)) continue;
  const cat = f.category || 'Math';
  const { inputs, outputs } = parseSig(f.signature || `${f.name}()`);
  emit(folderFor(cat), {
    name: f.name, category: cat, summary: esc(f.description || f.name),
    examples: boundExamples(f.name), tags: tagsFrom(f.name, cat),
    body: pageBody({ name: f.name, summary: esc(f.description || ''), syntax: f.signature, description: esc(f.description || ''), inputs, outputs, references: refsFor(cat) }),
  });
  made++;
}

// 2. CALL procedures
for (const p of manifest.callProcedures) {
  if (done(p.name)) continue;
  const { inputs, outputs } = parseSig(p.signature);
  emit('control', {
    name: p.name, category: 'Control Systems', summary: esc(p.description),
    examples: boundExamples(p.name), tags: tagsFrom(p.name, 'control'),
    body: pageBody({ name: p.name, summary: esc(p.description), syntax: p.signature, description: esc(p.description) + ' Invoked as a `CALL` with the listed inputs and outputs.', inputs, outputs, references: refsFor('Control Systems') }),
  });
  made++;
}

// 3. Property functions
for (const p of manifest.propertyFunctions) {
  if (done(p.name)) continue;
  const ha = p.kind === 'humid-air';
  emit('properties', {
    name: p.name, category: 'Fluid Properties',
    summary: `${ha ? 'Humid-air' : 'Fluid'} property: ${p.name} from a real-fluid (CoolProp) backend.`,
    examples: boundExamples(p.name), tags: [p.name.toLowerCase(), 'property', ha ? 'humid-air' : 'fluid', 'coolprop'],
    body: pageBody({ name: p.name,
      summary: `Returns the **${p.name}** of a ${ha ? 'humid-air (AirH2O)' : 'real fluid'} from any valid pair of independent state properties (CoolProp backend).`,
      syntax: ha ? `${p.name}(AirH2O, T=, P=, R=)` : `${p.name}(Fluid, P=, T=)`,
      description: `${ha ? 'A humid-air property; supply the dry-bulb T, total pressure P, and one humidity coordinate (R, W, B, or D).' : 'Supply the fluid name and any two independent state properties (T, P, h, s, x, …).'} Property names are case-insensitive.`,
      inputs: [], references: refsFor('Fluid Properties') }),
  });
  made++;
}

// 4. Solid-material functions
for (const f of manifest.materials.functions) {
  if (done(f)) continue;
  emit('materials', {
    name: f, category: 'Solid Materials', summary: `Solid-material property accessor ${f}(Material[, T]).`,
    examples: boundExamples(f), tags: [f.toLowerCase().replace('_', ''), 'material', 'solid', 'property'],
    body: pageBody({ name: f,
      summary: `Returns a solid-material property via \`${f}(Material[, T])\` from the built-in material database.`,
      syntax: `${f}(Material[, T])`, description: 'Looks up a thermophysical/mechanical property of a named solid (e.g. Aluminum, Copper, Steel). Some properties accept an optional temperature.',
      inputs: [], references: refsFor('Solid Materials') }),
  });
  made++;
}

// 5. Components — real ports, params, and constitutive equations from THIS
// port's embedded library (crates/frees-core/src/components/library-data).
const compInfo = {};
for (const c of parseLibrary()) {
  compInfo[c.name] = {
    domain: c.domain,
    ports: c.ports,
    params: c.params.filter((p) => !p.fromRequire).map((p) => (p.defaultValue ? `${p.name} = ${p.defaultValue}` : p.name)),
    shared: c.shared || [],
    variants: c.variants.map((v) => ({ name: v.name, require: v.requires, eqs: v.eqs || [] })),
  };
}
for (const c of manifest.components) {
  if (done(c.name)) continue;
  const info = compInfo[c.name] || { domain: c.domain, ports: [], params: [], shared: [], variants: [] };
  const portRow = info.ports.length ? ['## Ports', '', '`' + info.ports.join('`, `') + '`', ''] : [];
  const paramRows = info.params.length
    ? ['## Parameters', '', '| Parameter | Type |', '| --- | --- |', ...info.params.map((p) => { const nm = p.split('=')[0].trim(); return `| \`${nm}\` | ${nm.endsWith('$') ? 'String' : 'Number'} |`; }), '']
    : [];
  const eqRows = info.shared.length ? ['## Constitutive Equations', '', 'The acausal equations this component expands into (over its port members and parameters):', '', '```', ...info.shared, '```', ''] : [];
  const variantRows = info.variants.length
    ? ['## Model Variants', '', 'Selected via the `model$` parameter; each adds its own equations (and `REQUIRE`d parameters):', '',
        ...info.variants.flatMap((v) => [
          `### \`${v.name}\`${v.require.length ? ' — requires `' + v.require.join('`, `') + '`' : ''}`, '',
          ...(v.eqs.length ? ['```', ...v.eqs, '```'] : ['_No additional equations (uses the shared body)._']), '']),
      ]
    : [];
  emit(`components/${c.domain}`, {
    name: c.name, category: `Component (${c.domain})`,
    summary: `Acausal ${c.domain}-domain component ${c.name}${info.ports.length ? ' with ports ' + info.ports.join(', ') : ''}.`,
    examples: boundExamples(c.name), tags: [c.name.toLowerCase(), 'component', c.domain, 'acausal'],
    body: [
      `# ${c.name}`, '',
      `Reusable acausal **${c.domain}-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.`, '',
      '> **Auto-generated** from this port\'s component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.', '',
      '## Usage', '', '```', `${c.name} inst(${info.params.map((p) => p.split('=')[0].trim()).join(', ') || 'param = value, ...'})`, '```', '',
      ...portRow, ...paramRows, ...eqRows, ...variantRows,
    ].join('\n'),
  });
  made++;
}

// 6. REPL-only CAS operations
for (const op of manifest.replCasOps) {
  if (done(op)) continue;
  emit('cas', {
    name: op, category: 'CAS (REPL)', summary: `Symbolic ${op} (REPL-only Symja CAS operation).`,
    examples: [], tags: [op.toLowerCase(), 'cas', 'symbolic', 'repl'],
    body: pageBody({ name: op,
      summary: `Symbolic computer-algebra operation **${op}**, available in the REPL terminal (Symja backend).`,
      syntax: `${op}(expr)`,
      description: 'A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.',
      inputs: ['expr'], references: refsFor('CAS (REPL)') }),
  });
  made++;
}

console.log(`scaffold: generated ${made} pages (skipped ${existing.size} authored).`);
