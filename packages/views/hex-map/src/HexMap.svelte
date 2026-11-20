<script lang="ts">
  import type { HexScene } from "./scene.ts";
  import { blockHexes, centre, corners, extent, label, parse, within } from "./grid.ts";
  import { marksFor, SPOTS } from "./glyphs.ts";

  let { scene, params, onpick }: { scene: HexScene; params: Record<string, unknown>; onpick?: (hex: string) => void } = $props();

  const size = $derived(Number(params.hex_size ?? 22));
  const cfg = $derived(scene.config.grid);
  const hexes = $derived.by(() => {
    if (scene.block !== undefined) return blockHexes(cfg, scene.block);
    const { columns, rows } = extent(cfg);
    const out = [];
    for (let c = 1; c <= columns; c++) for (let r = 1; r <= rows; r++) out.push({ col: c, row: r });
    return out;
  });
  const origin = $derived.by(() => {
    const c0 = Math.min(...hexes.map((h) => h.col));
    const r0 = Math.min(...hexes.map((h) => h.row));
    const p = centre(cfg, { col: c0, row: r0 }, size);
    return { x: p.x - size, y: p.y - size };
  });
  const at = (h: { col: number; row: number }) => {
    const p = centre(cfg, h, size);
    return { x: p.x - origin.x, y: p.y - origin.y };
  };
  const byHex = $derived(new Map(scene.worlds.map((w) => [w.hex, w])));
  const inRange = $derived.by(() => {
    const s = scene.selected ? parse(cfg, scene.selected) : null;
    if (!s || !scene.range) return new Set<string>();
    return new Set(within(cfg, s, scene.range).map((h) => label(cfg, h)));
  });
  const box = $derived.by(() => {
    const ps = hexes.map(at);
    return { w: Math.max(...ps.map((p) => p.x)) + size * 1.2, h: Math.max(...ps.map((p) => p.y)) + size * 1.2 };
  });
  const tier = $derived(String(params.tier ?? "canvas"));
</script>

{#if tier === "text"}
  <table class="hex-table">
    <thead><tr><th>Hex</th><th>Name</th>{#each Object.keys(scene.worlds[0]?.fields ?? {}) as f}<th>{f}</th>{/each}</tr></thead>
    <tbody>
      {#each scene.worlds as w}
        <tr><td>{w.hex}</td><td>{w.name}</td>{#each Object.values(w.fields) as v}<td>{Array.isArray(v) ? v.join(" ") : String(v)}</td>{/each}</tr>
      {/each}
    </tbody>
  </table>
{:else}
  <svg class="hex-map" viewBox="0 0 {box.w} {box.h}" role="img" aria-label={scene.title ?? "Hex map"}>
    {#each hexes as h}
      {@const p = at(h)}
      {@const l = label(cfg, h)}
      {@const w = byHex.get(l)}
      <g class="hex" class:range={inRange.has(l)} class:selected={scene.selected === l}>
        <polygon points={corners(p, size)} role="button" tabindex="-1" aria-label="hex {l}" onclick={() => onpick?.(l)} onkeydown={(e) => { if (e.key === "Enter") onpick?.(l); }} />
        <text class="num" x={p.x} y={p.y - size * 0.62}>{l}</text>
        {#if w}
          {#each marksFor(w, scene.config.glyphs) as m}
            {@const [dx, dy] = SPOTS[m.at] ?? [0, 0]}
            {#if m.at === "ring"}
              <circle class="ring tone-{m.tone}" cx={p.x} cy={p.y} r={size * 0.72} />
            {:else if m.text !== undefined}
              <text class="mark tone-{m.tone}" x={p.x + dx * size} y={p.y + dy * size + 3}>{m.text}</text>
            {:else}
              <use class="mark tone-{m.tone}" href="#hm-{m.mark}" x={p.x + dx * size} y={p.y + dy * size} />
            {/if}
          {/each}
          <text class="name" x={p.x} y={p.y + size * 0.78}>{String(w.fields[scene.config.glyphs.label_field] ?? w.name)}</text>
        {/if}
      </g>
    {/each}
    <defs>
      <symbol id="hm-world" overflow="visible"><circle r="4" /></symbol>
      <symbol id="hm-world-wet" overflow="visible"><circle r="4" class="fill" /></symbol>
      <symbol id="hm-gas-giant" overflow="visible"><circle r="2" class="fill" /></symbol>
      <symbol id="hm-star" overflow="visible"><path d="M0,-4 L1.2,-1.2 4,-1.2 1.8,0.6 2.6,3.6 0,1.8 -2.6,3.6 -1.8,0.6 -4,-1.2 -1.2,-1.2Z" class="fill" /></symbol>
      <symbol id="hm-triangle" overflow="visible"><path d="M0,-3.5 L3.2,2.5 -3.2,2.5Z" class="fill" /></symbol>
      <symbol id="hm-square" overflow="visible"><rect x="-2.6" y="-2.6" width="5.2" height="5.2" class="fill" /></symbol>
      <symbol id="hm-belt" overflow="visible"><circle r="1" cx="-3" class="fill" /><circle r="1" class="fill" /><circle r="1" cx="3" class="fill" /></symbol>
    </defs>
  </svg>
{/if}

<style>
  .hex-map { width: 100%; height: auto; font-family: var(--sse-font-mono); background: var(--sse-chart-ground, var(--sse-ground)); }
  polygon { fill: none; stroke: var(--sse-rule); stroke-width: 0.6; cursor: pointer; }
  .range polygon { fill: color-mix(in srgb, var(--sse-accent) 14%, transparent); }
  .selected polygon { stroke: var(--sse-accent); stroke-width: 1.6; }
  .num { font-size: 5px; fill: var(--sse-ink-muted); text-anchor: middle; }
  .name { font-size: 6px; fill: var(--sse-ink); text-anchor: middle; }
  .mark { font-size: 7px; text-anchor: middle; fill: var(--sse-ink); stroke: var(--sse-ink); }
  .mark :global(.fill), :global(symbol .fill) { fill: currentColor; }
  .ring { fill: none; stroke-width: 1.2; stroke-dasharray: 3 2; }
  .tone-ink { color: var(--sse-ink); }
  .tone-muted { color: var(--sse-ink-muted); fill: var(--sse-ink-muted); }
  .tone-accent { color: var(--sse-accent); stroke: var(--sse-accent); }
  .tone-caution { color: var(--sse-caution, var(--sse-accent)); stroke: var(--sse-caution, var(--sse-accent)); stroke-dasharray: 4 3; }
  .tone-danger { color: var(--sse-alert, var(--sse-accent)); stroke: var(--sse-alert, var(--sse-accent)); }
  .hex-table { font-family: var(--sse-font-mono); color: var(--sse-ink); border-collapse: collapse; }
  .hex-table td, .hex-table th { padding: 2px 8px; border-bottom: 1px solid var(--sse-rule); text-align: left; }
</style>
