<!-- Pack overlays over a map (overlay.ts): lines, hexes and labels already placed in the map's frame, turned into
     pixels by the view's own `at`. Drawn in theme ink, thin and quiet, under the map's marks; the view decides when
     (under its HUD switch) and which (its Layers tool). -->
<script lang="ts">
  import { hexCorners, mapSlotToken, type OverlayInk, type ViewOverlay } from "../../ui-contract/src/index.ts";

  let { overlays, at, scale }: { overlays: readonly ViewOverlay[]; at: (p: readonly number[]) => { x: number; y: number }; scale: number } = $props();
  // A plain ink is a class; a map palette slot (OV2) is the theme's slot colour, plain ink where a theme has none yet.
  const inkClass = (k: OverlayInk) => (typeof k === "string" ? k : "slot");
  const inkColour = (k: OverlayInk) => (typeof k === "string" ? undefined : `var(${mapSlotToken(k.slot)}, var(--sse-ink))`);
  const ring = (ps: readonly [number, number][]) => ps.map((p) => at(p)).map((q, i) => `${i ? "L" : "M"}${q.x.toFixed(1)},${q.y.toFixed(1)}`).join("") + "Z";
  const pts = (ps: readonly [number, number][]) => ps.map((p) => at(p)).map((q) => `${q.x.toFixed(1)},${q.y.toFixed(1)}`).join(" ");
</script>

<g class="overlays" data-overlays={overlays.length} aria-hidden="true">
  {#each overlays as o (o.id)}
    <g data-overlay={o.id}>
      {#each o.layers as l, li (li)}
        {#if l.kind === "lines"}
          {#each l.items as it, i (i)}
            {#if it.closed}<polygon class={inkClass(l.ink)} style:stroke={inkColour(l.ink)} points={pts(it.points)} />{:else}<polyline class={inkClass(l.ink)} style:stroke={inkColour(l.ink)} points={pts(it.points)} />{/if}
          {/each}
        {:else if l.kind === "hexes"}
          {#each l.items as it, i (i)}
            {@const c = at(it.centre)}
            <polygon class="hex {inkClass(l.ink)}" style:stroke={inkColour(l.ink)} points={pts(hexCorners(it.centre, it.size, it.orient, l.rotation_deg ?? 0))} />
            {#if it.label}<text class="hexlabel {inkClass(l.ink)}" x={c.x} y={c.y - it.size * scale * 0.55} font-size={Math.min(11, it.size * scale * 0.42)}>{it.label}</text>{/if}
          {/each}
        {:else if l.kind === "areas"}
          <!-- Areas (OV2): outlined for now, solid where held, dashed where claimed, dotted where influenced; the
               Atlas look (tint along the inside of the border, lettered names) is OV4's. -->
          {#each l.items as it, i (i)}
            <path class="area {it.status} {l.as}" d={it.rings.map(ring).join("")} style:stroke={inkColour(it.ink)} data-area={it.key} data-status={it.status} />
          {/each}
        {:else}
          {#each l.items as it, i (i)}
            {@const c = at(it.at)}
            <text class="label {inkClass(l.ink)}" x={c.x} y={c.y}>{it.text}</text>
          {/each}
        {/if}
      {/each}
    </g>
  {/each}
</g>

<style>
  polygon,
  polyline {
    fill: none;
    stroke-width: var(--sse-line, 1px);
    vector-effect: non-scaling-stroke;
  }
  .ink {
    stroke: var(--sse-ink);
  }
  .faint {
    stroke: var(--sse-ink-faint, var(--sse-ink-muted));
  }
  .accent {
    stroke: var(--sse-accent);
  }
  .area {
    fill: none;
    fill-rule: evenodd;
    stroke: var(--sse-ink);
    stroke-width: var(--sse-line, 1px);
    vector-effect: non-scaling-stroke;
  }
  .area.claims {
    stroke-dasharray: 6 4;
  }
  .area.influences {
    stroke-dasharray: 1 3;
  }
  .hex {
    stroke-width: 0.5px;
  }
  text {
    stroke: none;
    text-anchor: middle;
    dominant-baseline: middle;
    font-family: var(--sse-font-mono, monospace);
    font-size: 11px;
  }
  text.ink {
    fill: var(--sse-ink);
  }
  text.faint {
    fill: var(--sse-ink-muted);
  }
  text.accent {
    fill: var(--sse-accent);
  }
</style>
