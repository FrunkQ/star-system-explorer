// Generates tests/fixtures/classify/v3-oracle.json: bodies scattered around
// every SSE 3 fingerprint, classified by SSE 3's own code, so the Rust port
// of the classic scheme (crates/sse-sift/src/classic.rs) can be checked
// against the original, not against a re-reading of it.
//
//   node --experimental-strip-types scripts/v3-classify-oracle.mjs <v3-checkout>
//
// <v3-checkout> holds src/lib/system/classification.ts and
// static/rulepacks/starter-sf/classification.json (SSE 3.1). Deterministic:
// a fixed-seed PRNG, so re-running gives the same file.
import { readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const v3 = resolve(process.argv[2] ?? ".");
const { classifyByFingerprint, explainClassification } = await import(
  pathToFileURL(join(v3, "src/lib/system/classification.ts")).href
);
const pack = JSON.parse(readFileSync(join(v3, "static/rulepacks/starter-sf/classification.json"), "utf8"));
const fps = pack.classifier.fingerprints;
const max = pack.classifier.maxClasses || 4;

let s = 0x5e4c1a55; // mulberry32
const rnd = () => {
  s = (s + 0x6d2b79f5) | 0;
  let t = Math.imul(s ^ (s >>> 15), 1 | s);
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};
const cats = ["water", "methane", "ammonia", "none", "None", "H2", "He", "N2", "CO2", "SO2", "S", "PH3", "NH3", "Cl2", "F2", "O2", ""];
const isCat = (b) => typeof b === "string" || (Array.isArray(b) && (typeof b[0] === "string" || typeof b[1] === "string"));

function fill(f, fp) {
  for (const [k, band] of Object.entries({ ...(fp.gate ?? {}), ...fp.match })) {
    if (k in f) continue;
    const r = rnd();
    if (r < 0.04) continue; // missing feature
    if (isCat(band)) {
      const ok = typeof band === "string" ? [band] : band;
      f[k] = rnd() < 0.8 ? ok[Math.floor(rnd() * ok.length)] : cats[Math.floor(rnd() * cats.length)];
      continue;
    }
    const [lo, hi] = band;
    if (r < 0.12) { f[k] = rnd() < 0.5 ? lo : hi; continue; } // exactly on an edge
    let x = lo + (hi - lo) * rnd();
    if (rnd() < 0.35) { // outside, up to 25% relative, so soft edges are exercised
      const below = rnd() < 0.5;
      const ref = below ? (lo !== 0 ? Math.abs(lo) : hi !== 0 ? Math.abs(hi) : 1) : (hi !== 0 ? Math.abs(hi) : 1);
      x = below ? lo - ref * 0.25 * rnd() : hi + ref * 0.25 * rnd();
    }
    f[k] = Number(x.toPrecision(1 + Math.floor(rnd() * 6)));
  }
}

const bodies = [];
for (let i = 0; i < 3000; i++) {
  const f = {};
  fill(f, fps[i % fps.length]);
  const extra = Math.floor(rnd() * 3);
  for (let j = 0; j < extra; j++) fill(f, fps[Math.floor(rnd() * fps.length)]);
  const classes = classifyByFingerprint({ ...f }, fps, max);
  const e = explainClassification({ ...f }, fps);
  bodies.push({
    features: f,
    classes,
    base: e.base,
    fallback: e.fallback,
    borderline: e.borderline,
    candidates: e.candidates.map((c) => [c.class, c.score]),
    modifiers: e.modifiers.map((m) => [m.class, m.score]),
  });
}
const out = {
  about: "SSE 3.1 classifier output (classification.ts, starter-sf classification.json) on generated bodies; scores are SSE 3's own toFixed(2) values. Regenerate with scripts/v3-classify-oracle.mjs.",
  max_classes: max,
  bodies,
};
writeFileSync(new URL("../tests/fixtures/classify/v3-oracle.json", import.meta.url), JSON.stringify(out) + "\n");
console.log(`${bodies.length} bodies, ${new Set(bodies.map((b) => b.classes[0])).size} distinct first classes`);

// The Solar System fixture: SSE4 facts -> SSE 3 features through the classic
// pack's axis registry (field, divide_by, missing: the same arithmetic as
// sse_sift::Axis::value), then SSE 3's classifier. Writes v3_classes.
const classic = JSON.parse(readFileSync(new URL("../packs/sse3-classic/pack.json", import.meta.url), "utf8"));
const solarUrl = new URL("../tests/fixtures/classify/solar-system.json", import.meta.url);
const solar = JSON.parse(readFileSync(solarUrl, "utf8"));
for (const b of solar.bodies) {
  const f = {};
  for (const [id, a] of classic.provides.other.axes) {
    let v = b.facts[a.field] ?? a.missing;
    if (v === undefined) continue;
    if (a.divide_by && typeof v === "number") v = v / a.divide_by;
    f[id] = v;
  }
  b.v3_classes = classifyByFingerprint(f, fps, max);
}
writeFileSync(solarUrl, JSON.stringify(solar, null, 1) + "\n");
console.log(solar.bodies.map((b) => `${b.name}: ${b.v3_classes.join(", ")}`).join("\n"));
