<!-- One inked line: an orbit (circle), a path, or a belt (dotted band). Tone and weight are tokens. -->
<script lang="ts">
  type Tone = "ink" | "faint" | "muted" | "accent" | "zone";
  type Dash = "solid" | "dashed" | "dotted" | "belt";
  let {
    circle,
    d,
    tone = "faint",
    weight = "line",
    dash = "solid",
    width,
  }: {
    circle?: { cx: number; cy: number; r: number };
    d?: string;
    tone?: Tone;
    weight?: "hair" | "line" | "heavy";
    dash?: Dash;
    /** Stroke width in px, for bands (belts, zones) that have real width on the chart. */
    width?: number;
  } = $props();
  const style = $derived(width === undefined ? undefined : `stroke-width: ${width}px`);
</script>

{#if circle}
  <circle class="ink t-{tone} {weight} {dash}" cx={circle.cx} cy={circle.cy} r={circle.r} {style} />
{:else if d}
  <path class="ink t-{tone} {weight} {dash}" {d} {style} />
{/if}

<style>
  .ink {
    fill: none;
    vector-effect: non-scaling-stroke;
  }
  .t-ink { stroke: var(--sse-ink); }
  .t-faint { stroke: var(--sse-ink-faint); }
  .t-muted { stroke: var(--sse-ink-muted); }
  .t-accent { stroke: var(--sse-accent); }
  .t-zone { stroke: var(--sse-zone); }
  .hair { stroke-width: var(--sse-line-hair); }
  .line { stroke-width: var(--sse-line); }
  .heavy { stroke-width: var(--sse-line-heavy); }
  .dashed { stroke-dasharray: 6 4; }
  .dotted { stroke-dasharray: 1 5; stroke-linecap: round; }
  .belt { stroke-dasharray: 1 6; stroke-linecap: round; stroke-width: 5px; }
</style>
