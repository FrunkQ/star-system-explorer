import { TOKENS } from "./schema.ts";
import { THEMES, type ThemeDoc } from "./themes.ts";

/** Follow `extends` and return the full token set; throws on a loop or a missing base. */
export function resolveTheme(id: string, registry: readonly ThemeDoc[] = THEMES): Record<string, string | number> {
  const seen: string[] = [];
  const chain: ThemeDoc[] = [];
  for (let cur: string | undefined = id; cur; ) {
    if (seen.includes(cur)) throw new Error(`theme inheritance loop: ${[...seen, cur].join(" > ")}`);
    seen.push(cur);
    const doc = registry.find((t) => t.id === cur);
    if (!doc) throw new Error(`no theme ${cur}`);
    chain.unshift(doc);
    cur = doc.extends;
  }
  const out: Record<string, string | number> = {};
  for (const d of chain) Object.assign(out, d.tokens);
  return out;
}

/** CSS custom properties for a theme, e.g. `--sse-ink: #1E2A33;`. Unknown tokens are rejected. */
export function themeCss(id: string, registry: readonly ThemeDoc[] = THEMES): string {
  const t = resolveTheme(id, registry);
  const known = new Set(TOKENS.map((s) => s.name));
  for (const k of Object.keys(t)) if (!known.has(k)) throw new Error(`theme ${id} sets unknown token ${k}`);
  return TOKENS.map((s) => `--sse-${s.name}: ${t[s.name]};`).join(" ");
}
