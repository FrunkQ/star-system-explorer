<!--
  Cutaway view (V6, handoff 8.2): a wedge cut through the selected world.
  Global shells fill the wedge; shells that lie only under a region (the
  south pole's ice and brine) fill a sector at its edge, named for the
  region. Features sit at their heights. The ladder beside it lists the
  same things top to bottom in words. It draws what the world stack says
  and nothing else: no stack, no picture.
-->
<script lang="ts">
  import { cutRadius, formatKm, kindPhrase, ladder, regionCounts, type Json, type SystemScene, type ViewContext, type WorldShell, type WorldStack } from "../../../ui-contract/src/index.ts";
  import { Card } from "../../../chart-grammar/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  const ws = $derived(ctx.workspace);
  const subject = $derived(scene.subjects.find((s) => s.address === ws.selection));
  const withStacks = $derived(scene.subjects.filter((s) => s.world));
  const w = $derived<WorldStack | undefined>(subject?.world);
  const scale = $derived(params.scale === "true" ? "true" : "log");
  const wedge = $derived(Math.min(360, Math.max(60, Number(params.wedge_deg ?? 140))));
  const lines = $derived(w ? ladder(w) : []);
  let picked = $state<string | null>(null);
  $effect(() => {
    void ws.selection;
    picked = null;
  });

  // Geometry: the wedge points down at the centre of the world; stations sit in the margin above it.
  const Rpx = 200;
  const SIDE = 64;
  const half = $derived(wedge >= 180 ? Rpx : Rpx * Math.sin(((wedge / 2) * Math.PI) / 180));
  const W = $derived(Math.round(2 * (half + SIDE)));
  const cx = $derived(W / 2);
  const rad = (d: number) => (d * Math.PI) / 180;
  const pt = (r: number, a: number) => `${(cx + r * Math.sin(rad(a))).toFixed(2)} ${(cy - r * Math.cos(rad(a))).toFixed(2)}`;
  function sector(r0: number, r1: number, a0: number, a1: number): string {
    const big = a1 - a0 > 180 ? 1 : 0;
    if (a1 - a0 >= 359.9) {
      // A full ring: two half arcs each way.
      const ring = (r: number, sweep: 0 | 1) => (r <= 0 ? "" : `M ${pt(r, 0)} A ${r} ${r} 0 1 ${sweep} ${pt(r, 180)} A ${r} ${r} 0 1 ${sweep} ${pt(r, 0)} Z`);
      return `${ring(r1, 1)} ${ring(r0, 0)}`;
    }
    const outer = `M ${pt(r1, a0)} A ${r1} ${r1} 0 ${big} 1 ${pt(r1, a1)}`;
    const inner = r0 <= 0 ? `L ${pt(0, 0)}` : `L ${pt(r0, a1)} A ${r0} ${r0} 0 ${big} 0 ${pt(r0, a0)}`;
    return `${outer} ${inner} Z`;
  }
  // Regional shells take the last quarter of the wedge (at least 30°).
  const regionArc = $derived(Math.max(30, wedge / 4));
  const a0 = $derived(-wedge / 2);
  const a1 = $derived(wedge / 2);
  const global = $derived(w ? w.shells.filter((s) => !s.within) : []);
  const regional = $derived(w ? w.shells.filter((s) => s.within) : []);
  const regionName = (k: string) => w?.regions.find((r) => r.key === k)?.name ?? k;
  const r = (m: number) => (w ? cutRadius(w, m, scale) * Rpx : 0);
  // Features above every shell (stations) sit in the margin above the cut, in order.
  const shellTop = $derived(w ? Math.max(0, ...w.shells.map((s) => s.top_m)) : 0);
  const outside = $derived(w ? w.features.filter((f) => f.altitude_m > shellTop).sort((a, b) => a.altitude_m - b.altitude_m) : []);
  const outsideCount = $derived(outside.length);
  const cy = $derived(22 + 14 * outsideCount + Rpx);
  const H = $derived(Math.round(cy + (wedge > 180 ? -Rpx * Math.cos(((wedge / 2) * Math.PI) / 180) : 0) + 22));
  const fr = (f: { key: string; altitude_m: number }) => (f.altitude_m > shellTop ? Rpx + 12 + 14 * outside.findIndex((o) => o.key === f.key) : r(f.altitude_m));
  // Marks: one per feature, except the surface, which gets one mark worded like its ladder line.
  // Labels alternate sides top to bottom so neighbours do not overprint.
  const marks = $derived.by(() => {
    if (!w) return [];
    const surfaceLine = lines.find((l) => l.where === "surface" && !l.key);
    const own = w.features.filter((f) => f.altitude_m !== 0).map((f) => ({ key: f.key, label: f.name ?? kindPhrase(f.kind), at: f.altitude_m, y: fr(f) }));
    const all = surfaceLine ? [...own, { key: "surface", label: surfaceLine.text, at: 0, y: r(0) }] : own;
    return all.sort((a, b) => b.at - a.at).map((m, i) => ({ ...m, left: i % 2 === 1 }));
  });
  const FILL: Record<string, string> = {
    atmosphere: "var(--sse-zone, color-mix(in srgb, var(--sse-accent) 12%, transparent))",
    crust: "color-mix(in srgb, var(--sse-ink) 30%, var(--sse-ground))",
    mantle: "color-mix(in srgb, var(--sse-ink) 18%, var(--sse-ground))",
    core: "color-mix(in srgb, var(--sse-ink) 45%, var(--sse-ground))",
    ice_shell: "color-mix(in srgb, var(--sse-accent) 10%, var(--sse-ground))",
    sub_ice_ocean: "color-mix(in srgb, var(--sse-accent) 40%, var(--sse-ground))",
  };
  const fill = (s: WorldShell) => FILL[s.kind] ?? "color-mix(in srgb, var(--sse-ink) 10%, var(--sse-ground))";
  // Ticks on the depth axis: the shell boundaries, labelled on the wedge's left edge.
  // Boundaries closer than 9 px to the last labelled one keep their tick but lose the label.
  const ticks = $derived.by(() => {
    if (!w) return [];
    let last = Infinity;
    return [...new Set(global.flatMap((s) => [s.top_m, s.bottom_m]))]
      .filter((m) => m > -w.radius_m)
      .sort((a, b) => b - a)
      .map((m) => {
        const show = last - r(m) >= 9;
        if (show) last = r(m);
        return { m, show };
      });
  });
</script>

<div class="wrap" data-cutaway={w ? w.body : "none"} data-scale={scale}>
  {#if w && subject}
    <Card title="{subject.label}: cutaway" subtitle={regionCounts(w) ? `Regions: ${regionCounts(w)}` : undefined}>
      <div class="body" class:noladder={params.ladder === false}>
        <figure>
          <svg viewBox="0 0 {W} {H}" role="img" aria-label="Cut through {subject.label}, {scale === 'log' ? 'depth on a log scale' : 'to scale'}. The ladder lists every layer in words.">
            <!-- Global shells fill the whole wedge; regional ones lie over them in the region's sector. -->
            {#each global as s (s.key)}
              <path d={sector(r(s.bottom_m), r(s.top_m), a0, a1)} fill={fill(s)} stroke="var(--sse-ink)" stroke-width={picked === s.key ? 2 : 0.6} data-shell={s.key} />
            {/each}
            {#if regional.length}
              <line x1={cx} y1={cy} x2={cx + r(0) * Math.sin(rad(a1 - regionArc))} y2={cy - r(0) * Math.cos(rad(a1 - regionArc))} stroke="var(--sse-ink)" stroke-dasharray="3 3" />
              {#each regional as s (s.key)}
                <path d={sector(r(s.bottom_m), r(s.top_m), a1 - regionArc, a1)} fill={fill(s)} stroke="var(--sse-ink)" stroke-width={picked === s.key ? 2 : 0.6} data-shell={s.key} data-within={s.within} />
              {/each}
              <text x={cx + (Rpx + 10) * Math.sin(rad(a1 - regionArc / 2))} y={cy - (Rpx + 10) * Math.cos(rad(a1 - regionArc / 2))} class="lbl" text-anchor="start">{regionName(regional[0]!.within!)}</text>
            {/if}
            {#each ticks as { m, show } (m)}
              <line x1={cx + r(m) * Math.sin(rad(a0))} y1={cy - r(m) * Math.cos(rad(a0))} x2={cx + (r(m) + 6) * Math.sin(rad(a0))} y2={cy - (r(m) + 6) * Math.cos(rad(a0))} stroke="var(--sse-ink)" />
              {#if show}<text x={cx + (r(m) + 9) * Math.sin(rad(a0))} y={cy - (r(m) + 9) * Math.cos(rad(a0)) + 3} class="tick" text-anchor="end">{formatKm(m)}</text>{/if}
            {/each}
            {#each marks as m (m.key)}
              <g data-feature={m.key}>
                <circle cx={cx} cy={cy - m.y} r={picked === m.key ? 5 : 3.5} fill="var(--sse-accent)" stroke="var(--sse-ground)" />
                <text x={m.left ? cx - 8 : cx + 8} y={cy - m.y + 3.5} class="lbl" text-anchor={m.left ? "end" : "start"}>{m.label}</text>
              </g>
            {/each}
          </svg>
          <figcaption>Depth in km{scale === "log" ? ", on a log scale so thin layers show" : ", to scale"}. Radius {Math.round(w.radius_m / 1000).toLocaleString("en-GB")} km.</figcaption>
        </figure>
        {#if params.ladder !== false}
          <ol class="ladder" aria-label="Above, on and within {subject.label}">
            {#each lines as l, i (i)}
              <li data-ladder={l.key || "surface"} class:picked={picked === l.key}>
                <button type="button" class="row" aria-pressed={!!l.key && picked === l.key} onclick={() => (picked = !l.key || picked === l.key ? null : l.key)}>
                  <span class="depth">{l.depth}</span>
                  <span class="what">{l.text}</span>
                  <span class="where">{l.where}</span>
                </button>
              </li>
            {/each}
          </ol>
        {/if}
      </div>
      {#if (scene as { fixture?: boolean }).fixture || w.name.includes("fixture")}<p class="note" role="note">This stack is a fixture until the kernel builds world stacks from each world's history.</p>{/if}
    </Card>
  {:else}
    <Card title={subject ? `${subject.label}: no cutaway yet` : "Nothing selected"}>
      <p class="note">
        {subject ? "The kernel has no world stack for this body yet, so there is nothing true to draw." : "Pick a world on the chart to cut it open."}
        {#if withStacks.length}Worlds with a stack:{/if}
      </p>
      {#if withStacks.length}
        <ul class="pick">
          {#each withStacks as s (s.address)}<li><button type="button" class="linkbtn" onclick={() => ws.select(s.address)}>{s.label}</button></li>{/each}
        </ul>
      {/if}
    </Card>
  {/if}
</div>

<style>
  .wrap {
    min-width: 0;
  }
  .body {
    padding: 8px 0 0;
    display: grid;
    grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);
    gap: 12px 16px;
    align-items: start;
  }
  .body.noladder {
    grid-template-columns: minmax(0, 1fr);
  }
  @media (max-width: 640px) {
    .body {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  figure {
    margin: 0;
    min-width: 0;
    max-width: 560px;
  }
  svg {
    width: 100%;
    height: auto;
    display: block;
  }
  .lbl,
  .tick {
    font-family: var(--sse-font-body, inherit);
    fill: var(--sse-ink);
  }
  .lbl {
    font-size: 11px;
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 3px;
    stroke-linejoin: round;
  }
  .tick {
    font-size: 10px;
    fill: var(--sse-ink-muted);
  }
  figcaption,
  .note {
    margin: 4px 0 0;
    padding: 0 12px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .ladder {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 2px;
    font-size: 12px;
  }
  .row {
    font: inherit;
    color: inherit;
    background: none;
    border: 0;
    border-bottom: 1px solid var(--sse-rule);
    padding: 3px 12px;
    width: 100%;
    box-sizing: border-box;
    text-align: left;
    display: grid;
    grid-template-columns: 8.5em minmax(0, 1fr);
    gap: 0 8px;
    cursor: pointer;
  }
  .picked .row {
    background: color-mix(in srgb, var(--sse-accent) 12%, transparent);
  }
  .depth {
    font-variant-numeric: tabular-nums;
    color: var(--sse-ink-muted);
  }
  .where {
    grid-column: 2;
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .pick {
    margin: 6px 0 0;
    padding-left: 18px;
  }
  .linkbtn {
    font: inherit;
    color: var(--sse-accent);
    background: none;
    border: 0;
    padding: 0;
    text-decoration: underline;
    cursor: pointer;
  }
</style>
