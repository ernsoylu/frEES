// Structural inventory of this port's embedded component library.
//
// Parses crates/frees-core/src/components/library-data/*.frees for names, ports,
// PARAM defaults, and VARIANT REQUIRE lists. Descriptions and units stay in
// Markdown; this file is the engine's structural facts.
//
// Used by compile-docs, build-doc-manifest, scaffold-reference-pages, and the
// catalog name-set check. A standalone checkout is enough — no Java sibling.

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
export const LIBRARY_DIR = path.resolve(
  __dirname,
  '../../crates/frees-core/src/components/library-data',
);

function parseParamPiece(piece) {
  const s = piece.trim();
  if (!s) return null;
  const eq = s.indexOf('=');
  if (eq < 0) return { name: s, defaultValue: '', fromRequire: false };
  return { name: s.slice(0, eq).trim(), defaultValue: s.slice(eq + 1).trim(), fromRequire: false };
}

function parseParamLine(line) {
  const body = line.replace(/^PARAM\s+/i, '');
  const out = [];
  for (const piece of body.split(',')) {
    const p = parseParamPiece(piece);
    if (p) out.push(p);
  }
  return out;
}

/** Parse one domain file into component records (original spelling preserved). */
export function parseLibraryFile(text, domain) {
  const lines = text.split('\n');
  const components = [];
  for (let i = 0; i < lines.length; i++) {
    const head = lines[i].match(/^\s*COMPONENT\s+(\w+)\s*(?:\(([^)]*)\))?/);
    if (!head) continue;
    const name = head[1];
    const ports = (head[2] || '').split(',').map((s) => s.trim()).filter(Boolean);
    const params = [];
    const variants = [];
    const shared = [];
    let cur = null;
    let depth = 0;
    for (i += 1; i < lines.length; i++) {
      const raw = lines[i].replace(/\s+$/, '');
      const l = raw.trim();
      if (/^END\b/i.test(l)) {
        if (depth === 0) break;
        depth -= 1;
        cur = null;
        continue;
      }
      const vm = l.match(/^VARIANT\s+(\w+)(?:\s+REQUIRE\s+(.+))?/i);
      if (vm) {
        depth += 1;
        cur = {
          name: vm[1],
          requires: (vm[2] || '').split(',').map((s) => s.trim()).filter(Boolean),
          eqs: [],
        };
        variants.push(cur);
        continue;
      }
      if (/^PARAM\s+/i.test(l) && depth === 0) {
        params.push(...parseParamLine(l));
        continue;
      }
      if (!l || /^(REQUIRE|OUTPUT|MODEL|\{|\/\/)/.test(l)) continue;
      (cur ? cur.eqs : shared).push(raw.replace(/^\s{0,4}/, ''));
    }
    const known = new Set(params.map((p) => p.name.toLowerCase()));
    for (const v of variants) {
      for (const req of v.requires) {
        if (!known.has(req.toLowerCase())) {
          params.push({ name: req, defaultValue: '', fromRequire: true });
          known.add(req.toLowerCase());
        }
      }
    }
    components.push({ name, domain, ports, params, variants, shared });
  }
  return components;
}

/** Every shipped component, in domain-file then declaration order. */
export function parseLibrary() {
  if (!fs.existsSync(LIBRARY_DIR)) {
    throw new Error(`component library not found: ${LIBRARY_DIR}`);
  }
  const files = fs.readdirSync(LIBRARY_DIR).filter((f) => f.endsWith('.frees')).sort();
  const out = [];
  for (const file of files) {
    const domain = file.replace(/\.frees$/, '');
    const text = fs.readFileSync(path.join(LIBRARY_DIR, file), 'utf-8');
    out.push(...parseLibraryFile(text, domain));
  }
  return out;
}

export function libraryNameSet() {
  return new Set(parseLibrary().map((c) => c.name.toLowerCase()));
}
