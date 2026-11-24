<!--
  Range rings on the system chart (RK12, Atomic Rockets plan 13 and question 16): where a weapon's reading against a
  target changes, round the one who holds it. The kernel's `effect_at_range` gives the distances (ui-contract
  `rangeRings`); this draws them. The chart's distances are on a log scale about the primary, so a ring round
  anything else is bent: each point is placed true and projected with the chart's own `project`. An overlay under the
  HUD (layer `range`): the chart mounts it inside its SVG when `hudShows(hud, "range")`.
-->
<script lang="ts">
  import { ringPoints, type RangeRing } from "../../../ui-contract/src/index.ts";

  let { rings, centre_m, project, label = "" }: {
    rings: readonly RangeRing[];
    /** Where the weapon is, in metres from the chart's centre (orbital plane x, y). */
    centre_m: readonly [number, number];
    /** The chart's own placing: metres from its centre (x, y) to SVG px. */
    project: (x_m: number, y_m: number) => readonly [number, number];
    /** Who holds the weapon, for the tooltip. */
    label?: string;
  } = $props();

  const drawn = $derived(
    rings.map((g) => {
      const pts = ringPoints([centre_m[0], centre_m[1], 0], g.radius_m).map(([x, y]) => project(x, y));
      const d = pts.map(([x, y], i) => `${i ? "L" : "M"}${Math.round(x * 10) / 10},${Math.round(y * 10) / 10}`).join(" ") + " Z";
      // The label sits at the ring's top as drawn.
      const top = pts.reduce((a, p) => (p[1] < a[1] ? p : a), pts[0]!);
      return { g, d, top };
    }),
  );
</script>

<g class="range-rings" data-range-rings={rings.length} aria-label={label ? `Range rings: ${label}` : "Range rings"}>
  {#each drawn as r, i (i)}
    <path d={r.d} class="ring" data-range-ring={r.g.kind}><title>{label ? `${label}: ` : ""}{r.g.words}</title></path>
    <text x={r.top[0]} y={r.top[1] - 3} text-anchor="middle" class="lab">{r.g.kind}</text>
  {/each}
</g>

<style>
  .ring {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 1.2;
    vector-effect: non-scaling-stroke;
  }
  .ring[data-range-ring="marginal"] {
    stroke-dasharray: 6 4;
    opacity: 0.8;
  }
  .ring[data-range-ring="even odds"] {
    stroke: var(--sse-ink-muted);
    stroke-dasharray: 1 4;
  }
  .lab {
    font-family: var(--sse-font-mono);
    font-size: 10px;
    fill: var(--sse-ink-muted);
    pointer-events: none;
  }
</style>
