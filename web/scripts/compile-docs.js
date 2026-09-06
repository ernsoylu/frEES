import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { parseLibrary } from './parse-library.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DOCS_DIR = path.join(__dirname, '../src/docs');
const OUTPUT_FILE = path.join(__dirname, '../src/docsCatalog.ts');

function compileDocs() {
  console.log('Compiling documentation markdown files...');
  if (!fs.existsSync(DOCS_DIR)) {
    console.error(`Docs directory not found: ${DOCS_DIR}`);
    process.exit(1);
  }

  const files = fs.readdirSync(DOCS_DIR).filter(file => file.endsWith('.md'));
  const catalog = {};
  // Where each topic id was first defined, so a duplicate fails loudly instead of
  // silently shadowing the earlier block (the last [Topic:] used to win).
  const topicSource = {};

  for (const file of files) {
    const filepath = path.join(DOCS_DIR, file);
    const content = fs.readFileSync(filepath, 'utf-8');
    const lines = content.split('\n');

    let currentTopic = null;
    let currentContent = [];

    const commit = () => {
      if (!currentTopic) return;
      if (Object.prototype.hasOwnProperty.call(topicSource, currentTopic)) {
        console.error(
          `Duplicate [Topic: ${currentTopic}] — defined in ${topicSource[currentTopic]} and ${file}. ` +
          `Topic ids must be unique; remove or rename one block.`,
        );
        process.exit(1);
      }
      topicSource[currentTopic] = file;
      catalog[currentTopic] = currentContent.join('\n').trim();
    };

    for (const line of lines) {
      const match = line.match(/^\[Topic:\s*([a-zA-Z0-9_-]+)\]/);
      if (match) {
        commit();
        currentTopic = match[1];
        currentContent = [];
      } else {
        if (currentTopic) {
          currentContent.push(line);
        }
      }
    }
    commit();
  }

  // Generate TypeScript code
  let tsCode = `// GENERATED FILE - DO NOT EDIT DIRECTLY.
// Edit the markdown files in src/docs/ and compile using npm run compile-docs.

export const DOCS_CATALOG: Record<string, string> = {
`;

  for (const [key, value] of Object.entries(catalog)) {
    // Escape backticks and backslashes for JS template literals
    const escapedValue = value
      .replace(/\\/g, '\\\\')
      .replace(/`/g, '\\`')
      .replace(/\${/g, '\\${');
    tsCode += `  "${key}": \`${escapedValue}\`,\n`;
  }

  tsCode += '};\n';

  fs.writeFileSync(OUTPUT_FILE, tsCode, 'utf-8');
  console.log(`Successfully compiled ${Object.keys(catalog).length} topics to ${OUTPUT_FILE}`);
  return catalog;
}

// ── Reference pages: src/docs/reference/**/*.md → referenceCatalog.ts ─────────
const REF_DIR = path.join(DOCS_DIR, 'reference');
const REF_OUTPUT = path.join(__dirname, '../src/referenceCatalog.ts');

// Minimal YAML-subset frontmatter parser: scalars, inline [a, b] arrays, and
// block list values (lines beginning with "- "). Sufficient for our schema.
function parseFrontmatter(fm) {
  const out = {};
  const lines = fm.split('\n');
  let listKey = null;
  for (const raw of lines) {
    const line = raw.replace(/\s+$/, '');
    const listItem = line.match(/^\s*-\s+(.*)$/);
    if (listKey && listItem) {
      out[listKey].push(stripQuotes(listItem[1]));
      continue;
    }
    const kv = line.match(/^(\w+):\s*(.*)$/);
    if (!kv) continue;
    const [, key, val] = kv;
    if (val === '') { listKey = key; out[key] = []; continue; }
    listKey = null;
    if (val.startsWith('[') && val.endsWith(']')) {
      out[key] = val.slice(1, -1).split(',').map((s) => stripQuotes(s.trim())).filter(Boolean);
    } else {
      out[key] = stripQuotes(val);
    }
  }
  return out;
}
const stripQuotes = (s) => s.replace(/^["']|["']$/g, '');

// True when `name(` appears in `text` as a call (not as the tail of a longer
// identifier). Both arguments must already be lowercased.
function mentionsCall(text, name) {
  const needle = name + '(';
  let i = -1;
  while ((i = text.indexOf(needle, i + 1)) !== -1) {
    const prev = i === 0 ? '' : text[i - 1];
    if (!/[a-z0-9_$#]/.test(prev)) return true;
  }
  return false;
}

function compileReference(guideCatalog) {
  if (!fs.existsSync(REF_DIR)) {
    fs.writeFileSync(REF_OUTPUT, referenceModule([]), 'utf-8');
    return;
  }
  const pages = [];
  const walk = (dir) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, entry.name);
      if (entry.isDirectory()) { walk(p); continue; }
      if (!entry.name.endsWith('.md') || entry.name.startsWith('_')) continue;
      const raw = fs.readFileSync(p, 'utf-8');
      const m = raw.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/);
      if (!m) continue; // skip files without frontmatter (README, etc.)
      const fm = parseFrontmatter(m[1]);
      if (!fm.name) continue;
      pages.push({
        name: fm.name,
        slug: fm.name.toLowerCase(),
        category: fm.category || 'Uncategorized',
        summary: fm.summary || '',
        related: fm.related || [],
        examples: fm.examples || [],
        tags: fm.tags || [],
        references: fm.references || [],
        // Strip a leading H1 — the renderer (ReferencePageView) already shows the
        // page name as the title, so the body `# name` heading is redundant.
        body: m[2].trim().replace(/^#[ \t]+\S.*(?:\r?\n)+/, ''),
      });
    }
  };
  walk(REF_DIR);
  pages.sort((a, b) => a.category.localeCompare(b.category) || a.name.localeCompare(b.name));
  // Auto backlinks: which guide topics call this symbol (`name(` in the topic
  // markdown, code blocks included)? Computed here so the pages never go stale.
  const topics = Object.entries(guideCatalog || {}).map(([id, md]) => [id, md.toLowerCase()]);
  let backlinks = 0;
  for (const p of pages) {
    const lower = p.name.toLowerCase();
    p.guides = topics.filter(([, md]) => mentionsCall(md, lower)).map(([id]) => id).slice(0, 6);
    backlinks += p.guides.length;
  }
  fs.writeFileSync(REF_OUTPUT, referenceModule(pages), 'utf-8');
  console.log(`Successfully compiled ${pages.length} reference pages to ${REF_OUTPUT} (${backlinks} guide backlinks)`);
  return pages;
}

function referenceModule(pages) {
  const esc = (s) => String(s).replace(/\\/g, '\\\\').replace(/`/g, '\\`').replace(/\$\{/g, '\\${');
  const body = pages.map((p) => {
    const arr = (a) => '[' + a.map((x) => `\`${esc(x)}\``).join(', ') + ']';
    return `  {
    name: \`${esc(p.name)}\`,
    slug: \`${esc(p.slug)}\`,
    category: \`${esc(p.category)}\`,
    summary: \`${esc(p.summary)}\`,
    related: ${arr(p.related)},
    examples: ${arr(p.examples)},
    tags: ${arr(p.tags)},
    references: ${arr(p.references)},
    guides: ${arr(p.guides || [])},
    body: \`${esc(p.body)}\`,
  }`;
  }).join(',\n');
  return `// GENERATED FILE - DO NOT EDIT DIRECTLY.
// Compiled from src/docs/reference/**/*.md by scripts/compile-docs.js (npm run compile-docs).

export interface ReferencePage {
  name: string;
  slug: string;
  category: string;
  summary: string;
  related: string[];
  examples: string[];
  tags: string[];
  references: string[];
  /** Guide topic ids whose text calls this symbol (auto-computed backlinks). */
  guides: string[];
  body: string;
}

export const REFERENCE_PAGES: ReferencePage[] = [
${body}
];
`;
}

// ── Component catalog
// Structural facts (names, ports, PARAM defaults, VARIANT REQUIRE) come from
// this port's parsed library (`scripts/parse-library.mjs` over
// crates/frees-core/src/components/library-data). Authored Markdown keeps
// descriptions, units, summaries, and tags. Regenerate with:
//   npm run compile-docs
// After adding a library component: node scripts/scaffold-reference-pages.mjs
// then npm run compile-docs && npm run check-docs.
const COMP_OUTPUT = path.join(__dirname, '../src/componentCatalog.ts');

// Normalize a unit string from a parameter description (unicode → frees-safe
// ASCII: superscripts, ·, Ω, µ, °). Returns '' if anything unsafe survives, so
// a weird token never breaks the inserted line (and never matters: frees unit
// warnings don't block solving).
function normalizeUnit(u) {
  const s = u
    .replace(/⁰/g, '^0').replace(/¹/g, '^1').replace(/²/g, '^2').replace(/³/g, '^3')
    .replace(/⁴/g, '^4').replace(/⁵/g, '^5').replace(/⁶/g, '^6').replace(/⁷/g, '^7')
    .replace(/⁸/g, '^8').replace(/⁹/g, '^9')
    .replace(/[·⋅]/g, '-').replace(/×/g, '*')
    .replace(/Ω/g, 'ohm').replace(/[µμ]/g, 'u').replace(/°/g, 'deg')
    .trim();
  return /^[A-Za-z0-9/^.\-*() ]+$/.test(s) ? s : '';
}

// Pull the unit out of a parameter description: the last [...] bracket, e.g.
// "Two-phase-zone overall coefficient [W/m²·K]." → "W/m^2-K".
function unitFromDescription(desc) {
  const matches = [...desc.matchAll(/\[([^\]]+)\]/g)];
  if (!matches.length) return '';
  return normalizeUnit(matches[matches.length - 1][1]);
}

// Section helper: lines from a "## Heading" up to the next "## ".
function sectionLines(body, heading) {
  const lines = body.split('\n');
  const start = lines.findIndex((l) => l.trim() === heading);
  if (start < 0) return [];
  const out = [];
  for (let i = start + 1; i < lines.length; i++) {
    if (/^##\s/.test(lines[i])) break;
    out.push(lines[i]);
  }
  return out;
}

// Parse the Usage fenced block → { type, paramOrder }. The first non-empty
// code line looks like `Chiller inst(ref$, cool$, U_tp, ...)`.
function parseUsage(body) {
  const lines = sectionLines(body, '## Usage');
  const code = lines.find((l) => /^\w+\s+\w+\s*\(/.test(l.trim()));
  if (!code) return { type: '', paramOrder: [] };
  const m = code.trim().match(/^(\w+)\s+\w+\s*\(([^)]*)\)/);
  if (!m) return { type: '', paramOrder: [] };
  const inside = m[2].trim();
  const paramOrder = inside && inside !== '...'
    ? inside.split(',').map((s) => s.trim()).filter((s) => s && s !== '...')
    : [];
  return { type: m[1], paramOrder };
}

// Parse the Parameters table → Map<name, { type, description, unit }>.
function parseParamTable(body) {
  const lines = sectionLines(body, '## Parameters');
  const meta = new Map();
  for (const line of lines) {
    const m = line.match(/^\|\s*`([^`]+)`\s*\|\s*([^|]+?)\s*\|\s*([^|]*?)\s*\|/);
    if (!m) continue;
    const name = m[1].trim();
    const type = m[2].trim();
    const description = m[3].trim();
    meta.set(name, {
      type,
      description,
      unit: /string/i.test(type) ? '' : unitFromDescription(description),
    });
  }
  return meta;
}

// Parse the Ports section → ["in", "out", ...] (backtick-quoted, comma-listed).
function parsePorts(body) {
  const lines = sectionLines(body, '## Ports');
  const text = lines.join(' ');
  return [...text.matchAll(/`([^`]+)`/g)].map((m) => m[1].trim());
}

// Parse the optional Model Variants section → [{ name, requires }]. The headings
// look like "### `volumetric` — requires `eta_v`, `disp`, `rpm`" (the requires
// suffix is present only when the variant adds REQUIRE'd params).
function parseVariants(body) {
  const lines = sectionLines(body, '## Model Variants');
  return [...lines.join('\n').matchAll(/^###\s+`([^`]+)`(?:\s+—\s+requires\s+(.+))?$/gm)].map((m) => ({
    name: m[1].trim(),
    requires: m[2] ? [...m[2].matchAll(/`([^`]+)`/g)].map((r) => r[1].trim()) : [],
  }));
}

function markdownComponentPages() {
  const pages = new Map();
  if (!fs.existsSync(REF_DIR)) return pages;
  const walk = (dir) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, entry.name);
      if (entry.isDirectory()) { walk(p); continue; }
      if (!entry.name.endsWith('.md') || entry.name.startsWith('_')) continue;
      const raw = fs.readFileSync(p, 'utf-8');
      const m = raw.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/);
      if (!m) continue;
      const fm = parseFrontmatter(m[1]);
      const libMatch = (fm.category || '').match(/^Component\s*\(([^)]+)\)/);
      if (!libMatch) continue;
      const body = m[2];
      const { type } = parseUsage(body);
      const name = type || (fm.name || '').trim();
      if (!name) continue;
      pages.set(name.toLowerCase(), {
        type: name,
        library: libMatch[1].trim(),
        summary: fm.summary || '',
        tags: fm.tags || [],
        ports: parsePorts(body),
        paramMeta: parseParamTable(body),
        variants: parseVariants(body),
        paramOrder: parseUsage(body).paramOrder,
      });
    }
  };
  walk(REF_DIR);
  return pages;
}

function compileComponents() {
  const library = parseLibrary();
  const pages = markdownComponentPages();
  const specs = library.map((eng) => {
    const md = pages.get(eng.name.toLowerCase());
    const variants = eng.variants;
    const variantNames = variants.map((v) => v.name);
    const byVariant = new Map();
    for (const v of variants) {
      for (const p of v.requires) {
        const key = p.toLowerCase();
        if (!byVariant.has(key)) byVariant.set(key, []);
        byVariant.get(key).push(v.name);
      }
    }
    const paramMeta = md?.paramMeta || new Map();
    const params = eng.params.map((p) => {
      const info = paramMeta.get(p.name) || { description: '', unit: '' };
      const isString = p.name.endsWith('$');
      const isSelector = /model\$$/.test(p.name);
      // Empty = shared/always-shown. Non-empty = shown only for those variants
      // (engine: a REQUIRE-scoped param is optional when another variant is active).
      const requiredByVariants = byVariant.get(p.name.toLowerCase()) || [];
      return {
        name: p.name,
        isString,
        isSelector,
        isMap: isString && /map.*\$$/.test(p.name),
        unit: info.unit || '',
        description: info.description || '',
        required: !isSelector && p.name !== 'domain$',
        values: isSelector ? variantNames : [],
        variants: requiredByVariants,
        defaultValue: p.defaultValue || '',
      };
    });
    return {
      type: eng.name,
      library: eng.domain,
      summary: md?.summary || `Acausal ${eng.domain}-domain component ${eng.name}.`,
      tags: md?.tags?.length ? md.tags : [eng.name.toLowerCase(), 'component', eng.domain, 'acausal'],
      ports: eng.ports.length ? eng.ports : (md?.ports || []),
      params,
      variants,
    };
  });
  specs.sort((a, b) => a.library.localeCompare(b.library) || a.type.localeCompare(b.type));
  fs.writeFileSync(COMP_OUTPUT, componentModule(specs), 'utf-8');
  console.log(`Successfully compiled ${specs.length} component specs to ${COMP_OUTPUT}`);
  fs.writeFileSync(NAMES_OUTPUT, componentNamesModule(specs), 'utf-8');
  console.log(`Successfully compiled ${specs.length} component names to ${NAMES_OUTPUT}`);
}

// Light companion for the editor's autocomplete: just type names + one-line
// summaries, so the editor chunk never imports the full catalog (parameter
// tables and markdown bodies) that only the Component Wizard needs.
const NAMES_OUTPUT = path.join(__dirname, '../src/componentNames.ts');

function componentNamesModule(specs) {
  const esc = (s) => String(s).replace(/\\/g, '\\\\').replace(/`/g, '\\`').replace(/\$\{/g, '\\${');
  // Canonical instantiation signature: the shared (non-variant) params in
  // Usage-line order — what the editor's signature help shows after `Type inst(`.
  const sig = (s) => {
    const shared = (s.params || []).filter((p) => !p.variants || p.variants.length === 0);
    return `${s.type} inst(${shared.map((p) => p.name).join(', ')})`;
  };
  const rows = specs.map((s) =>
    `  { name: \`${esc(s.type)}\`, summary: \`${esc(s.summary)}\`, signature: \`${esc(sig(s))}\` },`).join('\n');
  return `// AUTO-GENERATED by scripts/compile-docs.js — do not edit by hand.

export interface ComponentName {
  name: string
  summary: string
  /** Canonical instantiation line, e.g. \`Resistor inst(R)\`. */
  signature: string
}

export const COMPONENT_NAMES: ComponentName[] = [
${rows}
]
`;
}

function componentModule(specs) {
  const esc = (s) => String(s).replace(/\\/g, '\\\\').replace(/`/g, '\\`').replace(/\$\{/g, '\\${');
  const arr = (a) => '[' + a.map((x) => `\`${esc(x)}\``).join(', ') + ']';
  const paramObj = (p) => `{ name: \`${esc(p.name)}\`, isString: ${p.isString}, isSelector: ${p.isSelector}, isMap: ${p.isMap}, unit: \`${esc(p.unit)}\`, description: \`${esc(p.description)}\`, required: ${p.required}, values: ${arr(p.values)}, variants: ${arr(p.variants)}, defaultValue: \`${esc(p.defaultValue || '')}\` }`;
  const variantObj = (v) => `{ name: \`${esc(v.name)}\`, requires: ${arr(v.requires)} }`;
  const body = specs.map((s) => `  {
    type: \`${esc(s.type)}\`,
    library: \`${esc(s.library)}\`,
    summary: \`${esc(s.summary)}\`,
    tags: ${arr(s.tags)},
    ports: ${arr(s.ports)},
    params: [
${s.params.map((p) => `      ${paramObj(p)}`).join(',\n')}
    ],
    variants: [${s.variants.map(variantObj).join(', ')}],
  }`).join(',\n');
  return `// GENERATED FILE - DO NOT EDIT DIRECTLY.
// Compiled by scripts/compile-docs.js from this port's parsed component library
// plus authored Markdown descriptions (npm run compile-docs).

export interface ComponentParam {
  name: string;          // e.g. "U_tp", "fluid$"
  isString: boolean;     // name ends in "$"
  isSelector: boolean;   // a model$ variant selector
  isMap: boolean;        // a string param naming a TABLE/FUNCTION map (map$, map_eta$)
  unit: string;          // frees-safe unit token, or "" if none/dimensionless
  description: string;
  required: boolean;
  values: string[];      // selector option values (model variants), else []
  variants: string[];    // variants that require this param; [] = shared/always-shown
  defaultValue?: string; // engine-declared default (PARAM model$ = isentropic), else ""
}

export interface ComponentVariant {
  name: string;          // model$ value, e.g. "volumetric"
  requires: string[];    // params this variant requires
}

export interface ComponentSpec {
  type: string;          // "Chiller"
  library: string;       // "ac"
  summary: string;
  tags: string[];
  ports: string[];       // ["ref_in","ref_out","cool_in","cool_out"]
  params: ComponentParam[]; // in Usage (canonical) order
  variants: ComponentVariant[]; // model$ variants, [] if none
}

export const COMPONENT_CATALOG: ComponentSpec[] = [
${body}
];
`;
}

// ── Lightweight topic/slug lists for the MAIN bundle (Spotlight palette, F1
// contextual help). The full catalogs stay code-split with the /help route;
// this module is a few KB of ids and labels only.
const TOPICS_OUTPUT = path.join(__dirname, '../src/docsTopics.ts');

function writeDocsTopics(catalog, refPages) {
  const esc = (x) => String(x).replace(/\\/g, '\\\\').replace(/`/g, '\\`').replace(/\$\{/g, '\\${');
  const topics = [];
  for (const [id, md] of Object.entries(catalog)) {
    const h1 = md.match(/^#[ \t]+(\S.*)$/m);
    if (h1) topics.push({ id, label: h1[1].trim() });
  }
  const slugs = refPages.map((p) => p.slug);
  const body = `// GENERATED FILE - DO NOT EDIT DIRECTLY.
// Compiled by scripts/compile-docs.js (npm run compile-docs). Small id/label
// lists for the main bundle: the Spotlight palette's Documentation group and
// the editor's F1 contextual help. The full content stays in the code-split
// /help route.

export const DOCS_TOPICS: { id: string; label: string }[] = [
${topics.map((t) => `  { id: \`${esc(t.id)}\`, label: \`${esc(t.label)}\` }`).join(',\n')}
];

export const REFERENCE_SLUGS: string[] = [
${slugs.map((sl) => `  \`${esc(sl)}\``).join(',\n')}
];
`;
  fs.writeFileSync(TOPICS_OUTPUT, body, 'utf-8');
  console.log(`Successfully compiled ${topics.length} topic labels + ${slugs.length} slugs to ${TOPICS_OUTPUT}`);
}

const guideCatalog = compileDocs();
const refPages = compileReference(guideCatalog) || [];
compileComponents();
writeDocsTopics(guideCatalog, refPages);
