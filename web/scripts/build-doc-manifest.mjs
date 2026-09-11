// Build the documentation manifest — the machine-readable inventory of every
// documentable symbol in frees, reconciled directly against the backend so the
// reference docs cannot silently drift from the implementation.
//
// Sources of truth (read live, never hand-copied):
//   1. parser/FunctionRegistry.java   — structured {name, signature, desc, category}
//   2. ast/Evaluator.java             — scalar built-in dispatch (`case "..."`)
//   3. ast/ControlSystemsEvaluator.java — control-systems dispatch
//   4. api/ReplEvaluator.java         — REPL-only CAS ops
//   5. src/docs/reference/**/*.md     — authored pages (frontmatter `name:`)
//
// Output: src/docs/reference/function-manifest.json
//   - `functions`: every registered function with its info + whether a page exists
//   - `dispatchOnly`: names dispatched by the backend but absent from the registry
//     (these need a FunctionRegistry entry before they can be documented cleanly)
//   - `coverage`: counts to drive the Phase-0 coverage gate
//
// Run: node scripts/build-doc-manifest.mjs   (also wired into compile-docs later)

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { parseLibrary } from './parse-library.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const WASM_REPO = path.resolve(__dirname, '../..');

// The Java sources this script reads live in the READ-ONLY REFERENCE REPO, a
// sibling of this one — not inside it. This file was vendored from the frEES
// frontend, where `backend/` sat beside `frontend/`, and it kept resolving
// `../..`; in this repo that is a `backend/` directory that has never existed,
// so `npm run check-docs` died on ENOENT before it could check anything.
//
// Resolution mirrors `tools/frees-home.sh`, which is the one place that
// decision is made for the Rust-side oracle tools: `$FREES_HOME` wins, then the
// sibling under either spelling the directory has worn.
function findReferenceRepo() {
  const candidates = process.env.FREES_HOME
    ? [process.env.FREES_HOME]
    : [path.join(WASM_REPO, '../frees'), path.join(WASM_REPO, '../frEES')];
  return candidates.find((c) =>
    fs.existsSync(path.join(c, 'backend/core/src/main/java/com/frees/backend')),
  );
}

const REF_DIR = path.join(__dirname, '../src/docs/reference');
const OUT = path.join(REF_DIR, 'function-manifest.json');

const REFERENCE = findReferenceRepo();

// Post core/web split: pure computation (parser/ast/props/...) lives in core,
// the Spring web layer (controllers, ReplEvaluator — which needs the Redis-backed
// session cache) lives in web.
const BK = REFERENCE
  ? path.join(REFERENCE, 'backend/core/src/main/java/com/frees/backend')
  : '';
const BK_WEB = REFERENCE
  ? path.join(REFERENCE, 'backend/web/src/main/java/com/frees/backend')
  : '';

const read = (p) => fs.readFileSync(p, 'utf-8');

// ── 1. Parse FunctionRegistry.java (the structured registry) ─────────────────
function parseRegistry() {
  const src = read(path.join(BK, 'parser/FunctionRegistry.java'));
  const re = /new FunctionInfo\(\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*\)/g;
  const out = [];
  let m;
  while ((m = re.exec(src)) !== null) {
    out.push({
      name: m[1],
      signature: m[2].replace(/\\"/g, '"'),
      description: m[3].replace(/\\"/g, '"'),
      category: m[4],
    });
  }
  return out;
}

// ── 2. Dispatch arms from an evaluator's switch ──────────────────────────────
// Returns one entry per `case ... ->` arm: the list of labels that share it.
// Labels sharing an arm are ALIASES of one function (e.g. case "t0_t","isen_t0_t").
function dispatchArms(rel, base = BK) {
  const src = read(path.join(base, rel));
  // Case labels may be string literals or named `static final String` constants
  // (Sonar S1192 pushes repeated dispatch names into constants); resolve the
  // constants to their values. Identifiers that don't resolve (e.g. enum
  // labels in unrelated switches) are dropped.
  const consts = new Map();
  const constRe = /static\s+final\s+String\s+([A-Z][A-Z0-9_]*)\s*=\s*"((?:[^"\\]|\\.)*)"/g;
  let c;
  while ((c = constRe.exec(src)) !== null) consts.set(c[1], c[2]);
  const arms = [];
  const label = '(?:"(?:[^"\\\\]|\\\\.)*"|[A-Z][A-Z0-9_]*)';
  const caseRe = new RegExp(`case\\s+(${label}(?:\\s*,\\s*${label})*)\\s*->`, 'g');
  let m;
  while ((m = caseRe.exec(src)) !== null) {
    const labels = (m[1].match(/"(?:[^"\\]|\\.)*"|[A-Z][A-Z0-9_]*/g) || [])
      .map((l) => (l.startsWith('"') ? l.slice(1, -1) : consts.get(l)))
      .filter(Boolean)
      .map((l) => l.toLowerCase());
    if (labels.length) arms.push(labels);
  }
  return arms;
}

// Operators / relational / logical tokens that share the switch but are not
// user-facing functions — excluded from the documentable surface.
const NON_FUNCTION_TOKENS = new Set([
  '<', '<=', '<>', '=', '>', '>=', '+', '-', '*', '/', '^',
  'and', 'or', 'not', 'xor',
]);

// ── 3. Name-set-routed families the Evaluator switch does NOT carry as cases ──

// Components: every `COMPONENT <Name>` in THIS port's embedded library.
function parseComponents() {
  return parseLibrary()
    .map((c) => ({ name: c.name, domain: c.domain }))
    .sort((a, b) => a.name.localeCompare(b.name));
}

// Fluid + humid-air property functions: the OUTPUTS / HA_OUTPUTS map keys.
function parsePropertyFunctions() {
  const src = read(path.join(BK, 'props/PropertyFunctions.java'));
  const block = (startKey) => {
    const i = src.indexOf(startKey);
    const seg = src.slice(i, src.indexOf(');', i));
    const keys = new Set();
    const re = /Map\.entry\(\s*(?:"([^"]+)"|VOLUME|DMASS)\s*,/g;
    let m;
    while ((m = re.exec(seg)) !== null) keys.add(m[1] || 'volume'); // VOLUME constant = "volume"
    return [...keys];
  };
  return {
    fluid: block('OUTPUTS = Map.ofEntries').sort(),
    humidAir: block('HA_OUTPUTS = Map.ofEntries').sort(),
  };
}

// Solid-material functions and the supported material database keys.
function parseMaterials() {
  const src = read(path.join(BK, 'props/SolidProperties.java'));
  const seg = src.slice(0, src.indexOf(');'));
  const mats = [...seg.matchAll(/Map\.entry\("([^"]+)"\s*,\s*new Material/g)].map((m) => m[1]);
  return {
    functions: ['k_', 'c_', 'rho_', 'E_', 'nu_'], // conductivity, cp, density, Young's modulus, Poisson
    materials: [...new Set(mats)].sort(),
  };
}

// CALL procedures: no clean backend name-set; sourced from the curated frontend
// inventory helpReference.CALL_PROCEDURES (name, signature, desc, category).
// Pull a quoted field (', ", or `) from one object-literal body.
function field(body, key) {
  const m = body.match(new RegExp(key + '\\s*:\\s*(["\'`])((?:(?!\\1).|\\\\.)*)\\1'));
  return m ? m[2].replace(/\\(['"`])/g, '$1') : '';
}

function parseCallProcedures() {
  const src = read(path.join(__dirname, '../src/helpReference.ts'));
  const i = src.indexOf('CALL_PROCEDURES');
  const seg = src.slice(i, src.indexOf('];', i));
  return [...seg.matchAll(/\{([^{}]*)\}/g)].map((m) => ({
    name: field(m[1], 'name'),
    category: field(m[1], 'category'),
    description: field(m[1], 'desc'),
    signature: field(m[1], 'signature'),
  })).filter((p) => p.name);
}

// Matrix / vector functions (matrix-routed, not in the scalar Evaluator switch);
// sourced from the curated helpReference.MATRIX_FUNCTIONS.
function parseMatrixFunctions() {
  const src = read(path.join(__dirname, '../src/helpReference.ts'));
  const i = src.indexOf('MATRIX_FUNCTIONS: FuncEntry[]');
  const seg = src.slice(i, src.indexOf('];', i));
  const re = /\{\s*name:\s*'([^']+)',\s*desc:\s*'((?:[^'\\]|\\.)*)'/g;
  const out = [];
  let m;
  while ((m = re.exec(seg)) !== null) {
    // name field embeds the signature, e.g. "SolveLinear(A, b)" — take the head as the name.
    const sig = m[1];
    const name = sig.replace(/\(.*$/, '').split(/\s*\/\s*/)[0].trim();
    out.push({ name, signature: sig, description: m[2].replace(/\\'/g, "'") });
  }
  return out;
}

// ── 4. Authored reference pages (frontmatter name:) ──────────────────────────
function authoredPages() {
  const names = new Set();
  if (!fs.existsSync(REF_DIR)) return names;
  const walk = (dir) => {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, e.name);
      if (e.isDirectory()) walk(p);
      else if (e.name.endsWith('.md') && !e.name.startsWith('_')) {
        const fm = read(p).match(/^---\n([\s\S]*?)\n---/);
        const nm = fm && fm[1].match(/^name:\s*(.+)$/m);
        if (nm) names.add(nm[1].trim().toLowerCase());
      }
    }
  };
  walk(REF_DIR);
  return names;
}

// ── The Rust registries (this port's own truth) ──────────────────────────────
//
// The Java sources above are the *reference* implementation. They are also, on
// a normal checkout, absent — and until Phase 4.7 this script silently reused
// whatever function list happened to be committed when someone last had the
// sibling repo. That is how 73 live intrinsics ended up undocumented while
// `npm run check-docs` reported 655/655.
//
// So the documentable surface is now reconciled against the **shipping engine**
// too, whether or not the Java repo is present: `eval::INTRINSICS` is a static
// table of `strict!("name", …)` / `lazy!("name", …)` rows, and the CALL targets
// are a static list in `procedures.rs`. Both are read straight out of the Rust
// source, because a build step that needs the engine compiled would not run in
// the same place this one does.

const EVAL_RS = path.join(WASM_REPO, 'crates/frees-core/src/eval.rs');
const PROCEDURES_RS = path.join(WASM_REPO, 'crates/frees-core/src/procedures.rs');

/** Every name in `eval::INTRINSICS`, lowercase, sorted. */
function rustIntrinsics() {
  if (!fs.existsSync(EVAL_RS)) return [];
  const src = read(EVAL_RS);
  const names = new Set();
  for (const m of src.matchAll(/(?:strict|lazy)!\(\s*"([^"]+)"/g)) {
    names.add(m[1].toLowerCase());
  }
  return [...names].sort();
}

/** Every name in `procedures::EXPANDED_CALL_TARGETS`, lowercase, sorted. */
function rustCallTargets() {
  if (!fs.existsSync(PROCEDURES_RS)) return [];
  const src = read(PROCEDURES_RS);
  const block = src.match(/const EXPANDED_CALL_TARGETS: &\[&str\] = &\[([\s\S]*?)\n\];/);
  if (!block) return [];
  const names = new Set();
  for (const m of block[1].matchAll(/"([^"]+)"/g)) names.add(m[1].toLowerCase());
  return [...names].sort();
}

/**
 * Fold the Rust registries into a manifest built from (or cached from) the Java
 * side, and report what only one of them knows about.
 *
 * A **union**, deliberately, not a replacement. Dropping a name the Java
 * registry has would orphan its authored page and fail the coverage gate for a
 * reason that has nothing to do with the engine; adding a name the Rust table
 * has is exactly the drift this exists to surface. Rust-only entries carry
 * `source: "rust"` so a reader can see which half they came from.
 */
function mergeRustRegistries(manifest) {
  const pages = authoredPages();
  const report = { functionsAdded: [], callsAdded: [] };

  const known = new Set((manifest.functions || []).map((f) => f.name.toLowerCase()));
  for (const f of manifest.functions || []) {
    for (const a of f.aliases || []) known.add(a.toLowerCase());
  }
  for (const d of manifest.dispatchOnly || []) known.add(String(d.name || d).toLowerCase());

  const intrinsics = rustIntrinsics();
  for (const name of intrinsics) {
    if (known.has(name)) continue;
    report.functionsAdded.push(name);
    manifest.functions.push({
      name,
      signature: `${name}(…)`,
      description: '',
      category: 'Built-in',
      aliases: [],
      source: 'rust',
      documented: pages.has(name),
    });
  }
  // The reverse check — "a documented function the engine no longer dispatches"
  // — is deliberately NOT made here. `eval::INTRINSICS` is only one of the
  // port's dispatch paths: the dense linear algebra goes through
  // `linalg::eval_intrinsic`, the control suite through `control::eval`, and
  // both are reached by synthetic `$` names this regex cannot see. Comparing
  // against the intrinsic table alone reported 45 live functions as dead.

  const callNames = new Set((manifest.callProcedures || []).map((p) => p.name.toLowerCase()));
  for (const name of rustCallTargets()) {
    if (callNames.has(name)) continue;
    report.callsAdded.push(name);
    manifest.callProcedures.push({
      name,
      signature: `CALL ${name}(…)`,
      description: '',
      source: 'rust',
      documented: pages.has(name),
    });
  }

  manifest.functions.sort((a, b) => a.name.localeCompare(b.name));
  manifest.callProcedures.sort((a, b) => a.name.localeCompare(b.name));
  return report;
}

function reportMerge(report, reference) {
  if (report.functionsAdded.length) {
    console.warn(
      `build-doc-manifest: ${report.functionsAdded.length} intrinsic(s) live in ` +
        `eval::INTRINSICS but not in the ${reference ? 'Java registry' : 'committed manifest'}: ` +
        report.functionsAdded.join(', '),
    );
  }
  if (report.callsAdded.length) {
    console.warn(
      `build-doc-manifest: ${report.callsAdded.length} CALL target(s) added from ` +
        `procedures::EXPANDED_CALL_TARGETS: ${report.callsAdded.join(', ')}`,
    );
  }
}

function writeManifest(manifest) {
  // The Java repo is present, so every family above is live — but the port has
// its own dispatch table, and this is the one place the two can be compared.
const mergeReport = mergeRustRegistries(manifest);
manifest.derivedFrom = 'java+rust';
recountCoverage(manifest);

fs.mkdirSync(REF_DIR, { recursive: true });
  const rendered = JSON.stringify(manifest, null, 2) + '\n';
  const stripDate = (s) => s.replace(/^\s*"generatedAt":.*$/m, '');
  const previous = fs.existsSync(OUT) ? read(OUT) : null;
  if (previous !== null && stripDate(previous) === stripDate(rendered)) {
    console.log('doc-manifest: unchanged against the backend registries — not rewritten.');
  } else {
    fs.writeFileSync(OUT, rendered);
  }
}

function recountCoverage(manifest) {
  const pages = authoredPages();
  const all = new Map();
  const note = (name, documented) => {
    const k = String(name).toLowerCase();
    all.set(k, (all.get(k) || false) || documented);
  };
  for (const f of manifest.functions || []) note(f.name, pages.has(f.name.toLowerCase()));
  for (const f of manifest.matrixFunctions || []) note(f.name, pages.has(f.name.toLowerCase()));
  for (const p of manifest.callProcedures || []) note(p.name, pages.has(p.name.toLowerCase()));
  for (const p of manifest.propertyFunctions || []) note(p.name, pages.has(String(p.name).toLowerCase()));
  for (const c of manifest.components || []) note(c.name, pages.has(c.name.toLowerCase()));
  for (const f of manifest.materials?.functions || []) note(f, pages.has(String(f).toLowerCase()));
  for (const r of manifest.replCasOps || []) note(r, pages.has(String(r).toLowerCase()));
  manifest.coverage.documentableSurfaceTotal = all.size;
  manifest.coverage.components = (manifest.components || []).length;
  manifest.coverage.documented = [...all.values()].filter(Boolean).length;
}

/** No Java sibling: keep committed function families, refresh components from this library. */
function refreshComponentsOnly() {
  if (!fs.existsSync(OUT)) {
    console.warn(
      'build-doc-manifest: reference repo not found and no committed function-manifest.json.',
    );
    process.exit(0);
  }
  const manifest = JSON.parse(read(OUT));
  const pages = authoredPages();
  manifest.components = parseComponents().map((c) => ({
    ...c,
    documented: pages.has(c.name.toLowerCase()),
  }));
  // Without the Java repo the Rust registries are the ONLY live source of truth
  // for the function surface. If they cannot be read there is nothing left
  // reconciling the manifest against the shipping engine, and a coverage number
  // computed from a cached list is a claim this script has no basis for — so it
  // fails rather than printing one.
  if (!rustIntrinsics().length) {
    console.error(
      `build-doc-manifest: no Java reference repo AND eval::INTRINSICS could not be read ` +
        `from ${path.relative(WASM_REPO, EVAL_RS)}. Refusing to report coverage from a ` +
        `cached list — there would be nothing checking it against the engine.`,
    );
    process.exit(1);
  }
  const report = mergeRustRegistries(manifest);
  manifest.note =
    "GENERATED by scripts/build-doc-manifest.mjs. Function and CALL families are " +
    "reconciled against the Rust registries (eval::INTRINSICS, " +
    "procedures::EXPANDED_CALL_TARGETS); the remaining families are the last " +
    "generation from the Java reference repo. Do not edit by hand.";
  manifest.derivedFrom = 'rust';
  // Named so a reader knows exactly which counts are live and which are cached.
  manifest.staleFamilies = ['matrixFunctions', 'propertyFunctions', 'materials', 'replCasOps'];
  recountCoverage(manifest);
  writeManifest(manifest);
  reportMerge(report, false);
  const cov = manifest.coverage;
  console.log(
    `doc-manifest: no Java reference repo — functions and CALL targets reconciled against ` +
      `the Rust registries, ${cov.components} components refreshed from this port's library ` +
      `(${cov.documentableSurfaceTotal} documentable, ${cov.documented} documented). ` +
      `Cached from the last Java generation: ${manifest.staleFamilies.join(', ')} → ` +
      `${path.relative(WASM_REPO, OUT)}`,
  );
}

if (!REFERENCE) {
  console.warn(
    'build-doc-manifest: reference repo not found (set $FREES_HOME, or put it ' +
      'beside this one as ../frees). Refreshing component inventory from this ' +
      'port\'s library; other families stay as last generated.',
  );
  refreshComponentsOnly();
  process.exit(0);
}

// ── Build ────────────────────────────────────────────────────────────────────
const registry = parseRegistry();
const registered = new Set(registry.map((f) => f.name.toLowerCase()));

// Only Evaluator.java carries the scalar built-in dispatch. ControlSystemsEvaluator's
// `case` labels are output-member selectors (gm/pm/tr/ts/Kp…) that pick an element of a
// multi-output result array — not functions — so it is deliberately NOT a dispatch source.
const arms = dispatchArms('ast/Evaluator.java')
  .map((labels) => labels.filter((l) => !NON_FUNCTION_TOKENS.has(l)))
  .filter((labels) => labels.length);

// Map each registered function to the aliases it picks up from its dispatch arm.
const aliasesFor = {};
for (const labels of arms) {
  const canon = labels.find((l) => registered.has(l));
  if (canon) {
    const al = labels.filter((l) => l !== canon);
    if (al.length) aliasesFor[canon] = [...new Set([...(aliasesFor[canon] || []), ...al])];
  }
}

const pages = authoredPages();
const functions = registry.map((f) => ({
  ...f,
  aliases: aliasesFor[f.name.toLowerCase()] || [],
  documented: pages.has(f.name.toLowerCase()),
}));

// Genuinely missing functions: dispatch arms where NO label is in the registry.
// One entry per arm (canonical = first label, plus its aliases).
const dispatchOnly = arms
  .filter((labels) => !labels.some((l) => registered.has(l)))
  .map((labels) => ({ name: labels[0], aliases: labels.slice(1) }))
  .sort((a, b) => a.name.localeCompare(b.name));

const repl = [...new Set(dispatchArms('api/ReplEvaluator.java', BK_WEB).flat())]
  .filter((n) => !NON_FUNCTION_TOKENS.has(n)).sort();

// Name-set-routed families (not in the Evaluator switch).
const components = parseComponents().map((c) => ({ ...c, documented: pages.has(c.name.toLowerCase()) }));
const props = parsePropertyFunctions();
const propertyFunctions = [
  ...props.fluid.map((n) => ({ name: n, kind: 'fluid', documented: pages.has(n) })),
  ...props.humidAir.map((n) => ({ name: n, kind: 'humid-air', documented: pages.has(n) })),
];
const materials = parseMaterials();
const callProcedures = parseCallProcedures().map((p) => ({ ...p, documented: pages.has(p.name.toLowerCase()) }));
const matrixFunctions = parseMatrixFunctions().map((p) => ({ ...p, documented: pages.has(p.name.toLowerCase()) }));

// Unique documentable symbols across all families (a few control names appear in
// both FunctionRegistry's Control category and callProcedures — count them once).
const allSymbols = new Map(); // slug -> documented
const note1 = (name, documented) => {
  const k = name.toLowerCase();
  allSymbols.set(k, (allSymbols.get(k) || false) || documented);
};
functions.forEach((f) => note1(f.name, f.documented));
matrixFunctions.forEach((f) => note1(f.name, f.documented));
callProcedures.forEach((p) => note1(p.name, p.documented));
propertyFunctions.forEach((p) => note1(p.name, p.documented));
components.forEach((c) => note1(c.name, c.documented));
materials.functions.forEach((f) => note1(f, pages.has(f.toLowerCase())));
repl.forEach((r) => note1(r, pages.has(r.toLowerCase())));
const surfaceTotal = allSymbols.size;
const documentedTotal = [...allSymbols.values()].filter(Boolean).length;

const manifest = {
  generatedAt: new Date().toISOString().slice(0, 10),
  note: 'GENERATED by scripts/build-doc-manifest.mjs from the backend registries + std-lib. Do not edit by hand.',
  coverage: {
    documentableSurfaceTotal: surfaceTotal,
    registeredFunctions: functions.length,
    matrixFunctions: matrixFunctions.length,
    components: components.length,
    propertyFunctions: propertyFunctions.length,
    callProcedures: callProcedures.length,
    materialFunctions: materials.functions.length,
    replCasOps: repl.length,
    documented: documentedTotal,
    dispatchOnlyNeedingRegistry: dispatchOnly.length,
  },
  functions,
  dispatchOnly,
  matrixFunctions,
  callProcedures,
  propertyFunctions,
  materials,
  components,
  replCasOps: repl,
};

fs.mkdirSync(REF_DIR, { recursive: true });

// Write only when something OTHER than the date changed. `npm run check-docs`
// regenerates this file every time it runs, and `generatedAt` alone would dirty
// a committed 106 KB artifact on every run — noise that trains people to
// `git checkout` the manifest, which is how a real drift would get discarded
// with it. A date bump is not news; a registry change is.
const rendered = JSON.stringify(manifest, null, 2) + '\n';
const stripDate = (s) => s.replace(/^\s*"generatedAt":.*$/m, '');
const previous = fs.existsSync(OUT) ? read(OUT) : null;
if (previous !== null && stripDate(previous) === stripDate(rendered)) {
  console.log('doc-manifest: unchanged against the backend registries — not rewritten.');
} else {
  fs.writeFileSync(OUT, rendered);
}

reportMerge(mergeReport, true);
const cov = manifest.coverage;
console.log(`doc-manifest: ${cov.documentableSurfaceTotal} documentable symbols ` +
  `(${cov.documented} documented) — ${functions.length} functions, ${matrixFunctions.length} matrix fns, ` +
  `${components.length} components, ${propertyFunctions.length} property fns, ${callProcedures.length} CALL procs, ` +
  `${materials.functions.length} material fns, ${repl.length} CAS ops; ` +
  `${dispatchOnly.length} dispatch-only gaps → ${path.relative(WASM_REPO, OUT)}`);
