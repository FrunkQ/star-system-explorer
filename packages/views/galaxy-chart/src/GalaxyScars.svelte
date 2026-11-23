<svelte:options namespace="svg" />
<!--
  GH9: a galaxy's scars at the chart's time (galaxy histories design 9.4). The kernel's dated marks (GH3) become a
  bent, rippled, ringed and lopsided disc through the renderer's galaxy-marks; this only inks them. `part` "face"
  draws inside the face-on disc's own svg (a layer over it); "edge" draws the disc seen edge-on, with the words.
-->
<script lang="ts">
  import { faceOnPaths, scarsAt } from "../../../renderer/src/index.ts";
  import type { DiscFrame, GalaxyChart } from "../../../ui-contract/src/index.ts";

  let { chart, frame }: { chart: GalaxyChart; frame: DiscFrame } = $props();
  const marks = $derived(chart.assembly?.marks ?? []);
  const face = $derived(faceOnPaths(scarsAt(marks, chart.at_gyr, chart.disc_kpc), frame));
</script>

{#if marks.length}
  <g class="scars" data-scars>
    <path class="scar-edge" d={face.edge} />
    {#each face.rings as r, i (i)}<path class="scar-hole" d={r.hole} style="fill-opacity: {0.6 * r.strength}" />{/each}
    {#each face.rings as r, i (i)}<path class="scar-ring" d={r.d} style="stroke-opacity: {0.3 + 0.7 * r.strength}" />{/each}
    {#each face.knots as k, i (i)}<circle class="scar-knot" cx={k.x} cy={k.y} r={k.r} />{/each}
  </g>
{/if}

<style>
  .scar-edge { fill: none; stroke: var(--sse-ink-muted); stroke-dasharray: 3 3; stroke-width: 1; }
  .scar-hole { fill: var(--sse-chart-ground); stroke: none; }
  .scar-ring { fill: none; stroke: var(--sse-accent); stroke-width: 3; }
  .scar-knot { fill: var(--sse-accent); }
</style>
