<!--
  GH9: the galaxy seen edge-on at the chart's time, with the scars its history left (galaxy histories design 9.4):
  the warped and rippled mid-plane, the thin disc round it and the thick disc older mergers heated up. Heights are
  drawn stretched so a ripple a few hundred parsecs high shows; the caption says by how much. Presentation only.
-->
<script lang="ts">
  import { edgeOnPaths, scarsAt, scarWords } from "../../../renderer/src/index.ts";
  import type { GalaxyChart } from "../../../ui-contract/src/index.ts";

  let { chart, width = 520 }: { chart: GalaxyChart; width?: number } = $props();
  const H = 110;
  const marks = $derived(chart.assembly?.marks ?? []);
  const outer = $derived(chart.rings.at(-1)?.outer_kpc ?? chart.disc_kpc * 5);
  const sc = $derived(scarsAt(marks, chart.at_gyr, chart.disc_kpc));
  const edge = $derived(edgeOnPaths(sc, [width, H], outer));
  const words = $derived(scarWords(sc));
</script>

{#if marks.length}
  <figure class="edgeon" data-edge-on>
    <svg viewBox="0 0 {width} {H}" width="100%" role="img" aria-label="The disc edge-on: {words.length ? words.join('; ') : 'an undisturbed disc'}">
      <path class="thick" d={edge.thick} style="fill-opacity: {0.15 + 0.35 * sc.thick}" />
      <path class="thin" d={edge.thin} />
      <path class="mid" d={edge.mid} />
    </svg>
    <figcaption>
      Edge-on, heights {edge.stretch} times. {#if words.length}Scars now: {words.join("; ")}.{:else}No collision has left a mark that still shows.{/if}
    </figcaption>
  </figure>
{/if}

<style>
  figure { margin: 0.5em 0 0; }
  svg { display: block; height: auto; background: var(--sse-chart-ground); }
  .thick { fill: var(--sse-ink-faint); stroke: none; }
  .thin { fill: var(--sse-ink-muted); fill-opacity: 0.5; stroke: none; }
  .mid { fill: none; stroke: var(--sse-ink); stroke-width: 1; }
  figcaption { color: var(--sse-ink-muted); font-size: 0.8em; line-height: 1.4; }
</style>
