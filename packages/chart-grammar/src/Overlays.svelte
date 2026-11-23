<!-- Pack overlays over a map (overlay.ts): lines, hexes and labels already placed in the map's frame, turned into
     pixels by the view's own `at`. Drawn in theme ink, thin and quiet, under the map's marks; the view decides when
     (under its HUD switch) and which (its Layers tool). -->
<script lang="ts">
  import { hexCorners, type ViewOverlay } from "../../ui-contract/src/index.ts";

  let { overlays, at, scale }: { overlays: readonly ViewOverlay[]; at: (p: readonly number[]) => { x: number; y: number }; scale: number } = $props();
  const pts = (ps: readonly [number, number][]) => ps.map((p) => at(p)).map((q) => `${q.x.toFixed(1)},${q.y.toFixed(1)}`).join(" ");
</script>

<g class="overlays" data-overlays={overlays.length} aria-hidden="true">
  {#each overlays as o (o.id)}
    <g data-overlay={o.id}>
      {#each o.layers as l, li (li)}
        {#if l.kind === "lines"}
          {#each l.items as it, i (i)}
            {#if it.closed}<polygon class={l.ink} points={pts(it.points)} />{:else}<polyline class={l.ink} points={pts(it.points)} />{/if}
          {/each}
        {:else if l.kind === "hexes"}
          {#each l.items as it, i (i)}
            {@const c = at(it.centre)}
            <polygon class="hex {l.ink}" points={pts(hexCorners(it.centre, it.size, it.orient, l.rotation_deg ?? 0))} />
            {#if it.label}<text class="hexlabel {l.ink}" x={c.x} y={c.y - it.size * scale * 0.55} font-size={Math.min(11, it.size * scale * 0.42)}>{it.label}</text>{/if}
          {/each}
        {:else}
          {#each l.items as it, i (i)}
            {@const c = at(it.at)}
            <text class="label {l.ink}" x={c.x} y={c.y}>{it.text}</text>
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
