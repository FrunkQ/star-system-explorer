<!--
  The Flow view (RK13, Atomic Rockets plan 16): any ledger's rows (from, to, amount, kind) as bands between nodes in
  columns, band width by amount. Left to right; top to bottom when the room is narrow (a phone). A band under one part
  in a hundred is a hairline; a dashed band is one the books cannot account for. It lays out only what it is given
  (ui-contract flow.ts) and decides no physics. Curious depth shows the names; Science adds each figure.
-->
<script lang="ts">
  import { layoutFlow, sayNumber, type FlowBand, type FlowRow } from "../../../ui-contract/src/index.ts";

  let {
    rows,
    titles = [],
    column,
    rank,
    amountWords = (v: number) => sayNumber(v, 3),
    figures = true,
    label = "Flow",
  }: {
    rows: readonly FlowRow[];
    /** One title per column, read in the reading direction. */
    titles?: readonly string[];
    column?: (id: string) => number | undefined;
    rank?: (id: string) => number;
    /** A figure in the ledger's own unit. */
    amountWords?: (v: number) => string;
    /** Figures on each node and band (Science depth and deeper). */
    figures?: boolean;
    label?: string;
  } = $props();

  let width = $state(640);
  const vertical = $derived(width < 520);
  const layout = $derived(
    layoutFlow(rows, {
      width: Math.max(280, width),
      height: vertical ? Math.max(420, 140 * Math.max(2, titles.length || 3)) : 320,
      vertical,
      ...(column ? { column } : {}),
      ...(rank ? { rank } : {}),
      ...(vertical ? { margin: [30, 18] as [number, number] } : {}),
    }),
  );
  const last = $derived(Math.max(0, layout.columns.length - 1));
  // Heat bands shade by temperature, hotter more solid, from the same accent.
  const temps = $derived([...new Set(rows.filter((r) => r.k !== undefined).map((r) => r.k!))].sort((a, b) => a - b));
  const shade = (b: FlowBand) => (b.row.k === undefined || temps.length < 2 ? 0.72 : 0.42 + (0.45 * temps.indexOf(b.row.k)) / (temps.length - 1));
  const tip = (b: FlowBand) => `${b.row.from} to ${b.row.to}: ${amountWords(b.row.amount)}`;
</script>

<figure class="flow" bind:clientWidth={width} aria-label={label} data-flow data-vertical={vertical}>
  <svg viewBox="0 0 {layout.width} {layout.height}" width="100%" role="img" aria-label={label}>
    {#each titles as t, i (i)}
      {#if layout.columns[i] !== undefined}
        {#if vertical}
          <text class="col" x="4" y={layout.columns[i]! - 8}>{t}</text>
        {:else}
          <text class="col" x={layout.columns[i]! + 5} y="12" text-anchor="middle">{t}</text>
        {/if}
      {/if}
    {/each}
    {#each layout.bands as b, i (i)}
      <path
        d={b.d}
        class="band k-{b.row.kind}"
        class:dashed={b.row.dashed}
        class:hair={b.hairline}
        stroke-width={b.width}
        style:opacity={shade(b)}
        data-band={b.row.kind}
        data-from={b.row.from}
        data-to={b.row.to}
      ><title>{tip(b)}</title></path>
    {/each}
    {#each layout.nodes as n (n.id)}
      <rect x={n.x} y={n.y} width={n.w} height={n.h} class="node" class:nowhere={n.dashed} data-node={n.id} />
      {#if vertical}
        <text class="n" x={n.x} y={n.column === last ? n.y + n.h + 13 : n.y - 4}>{n.id}{#if figures}<tspan class="v" dx="4">{amountWords(n.amount)}</tspan>{/if}</text>
      {:else if n.column === 0}
        <text class="n" x={n.x - 8} y={n.y + n.h / 2 + 4} text-anchor="end">{n.id}{#if figures}<tspan class="v" x={n.x - 8} dy="14">{amountWords(n.amount)}</tspan>{/if}</text>
      {:else if n.column === last}
        <text class="n" x={n.x + n.w + 8} y={n.y + n.h / 2 + 4}>{n.id}{#if figures}<tspan class="v" x={n.x + n.w + 8} dy="14">{amountWords(n.amount)}</tspan>{/if}</text>
      {:else}
        <text class="n" x={n.x + n.w / 2} y={n.y - 5} text-anchor="middle">{n.id}{#if figures}<tspan class="v" dx="4">{amountWords(n.amount)}</tspan>{/if}</text>
      {/if}
    {/each}
  </svg>
</figure>

<style>
  .flow {
    margin: 0;
    min-width: 0;
  }
  svg {
    display: block;
    overflow: visible;
  }
  text {
    fill: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  .col {
    font-family: var(--sse-font-mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    fill: var(--sse-ink-muted);
  }
  .n {
    font-size: 12px;
    font-weight: 500;
    /* A halo of the ground so a label stays readable where a band passes under it. */
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 3px;
    stroke-linejoin: round;
  }
  .v {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    font-weight: 400;
    fill: var(--sse-ink-muted);
  }
  .node {
    fill: var(--sse-ink);
  }
  .node.nowhere {
    fill: var(--sse-alert, var(--sse-accent));
  }
  .band {
    fill: none;
    stroke: var(--sse-ink-muted);
    transition: opacity var(--sse-motion-quick, 140ms) var(--sse-ease, ease-out);
  }
  .band:hover {
    opacity: 1 !important;
  }
  .band.k-electric {
    stroke: var(--sse-focus, var(--sse-accent));
  }
  .band.k-heat {
    stroke: var(--sse-accent);
  }
  .band.k-jet {
    stroke: var(--sse-ink);
  }
  .band.k-fuel {
    stroke: var(--sse-ink-muted);
  }
  .band.dashed {
    stroke: var(--sse-alert, var(--sse-accent));
    stroke-dasharray: 6 4;
  }
  .band.hair {
    opacity: 0.9 !important;
  }
  @media (prefers-reduced-motion: reduce) {
    .band {
      transition: none;
    }
  }
</style>
