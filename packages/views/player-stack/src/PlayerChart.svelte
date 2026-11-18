<!--
  Chart: the 2D orbital chart a player is allowed to see. Bodies whose orbit
  the audience has are drawn on a log-radial scale; bodies known only to
  exist are listed beside the chart as "position not known", never placed by
  guess. Selecting a body selects it for the other views of the screen.
-->
<script lang="ts">
  import { fact, subjectName, wordsFor, type PlayerContext, type PlayerScene } from "../../../ui-contract/src/index.ts";
  import { fitScale, radius, polar } from "../../../chart-grammar/src/scale.ts";
  import { topLevel } from "./pick.ts";

  let { ctx, scene }: { ctx: PlayerContext; scene: PlayerScene } = $props();
  const AU = 1.495978707e11;
  let w = $state(600);
  let h = $state(400);
  const star = $derived(scene.subjects.find((s) => s.address === scene.scope));
  const field = $derived(String(ctx.params.distance_field ?? "orbit.semi_major_axis_m"));
  const bodies = $derived(
    topLevel(scene).map((s) => {
      const f = fact(s, field);
      const a = typeof f?.value === "number" ? f.value / AU : f?.band ? (f.band[0] + f.band[1]) / 2 / AU : null;
      return { s, a, name: subjectName(s) };
    }),
  );
  const placed = $derived(bodies.filter((b) => b.a !== null));
  const unplaced = $derived(bodies.filter((b) => b.a === null));
  const cx = $derived(Math.min(w, h) / 2 + 8);
  const cy = $derived(h / 2);
  const scale = $derived(fitScale(placed.map((b) => b.a!), 26, Math.max(60, Math.min(w, h) / 2 - 18)));
  // Phase angles are not player data; bodies are spread evenly by order so labels never collide.
  const angle = (i: number) => 20 + i * (300 / Math.max(1, placed.length));
  const sel = $derived(ctx.selection());

  function key(e: KeyboardEvent, address: string) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      ctx.select(address);
    }
  }
</script>

<div class="pchart" bind:clientWidth={w} bind:clientHeight={h}>
  <svg width={w} height={h} role="group" aria-label="Chart of {star ? subjectName(star) : 'the system'}">
    {#each placed as b, i (b.s.address)}
      {@const r = radius(b.a!, scale)}
      {@const [x, y] = polar(cx, cy, r, angle(i))}
      <circle class="orbit" {cx} {cy} {r} />
      <g
        class="body"
        class:sel={sel === b.s.address}
        role="button"
        tabindex="0"
        aria-pressed={sel === b.s.address}
        aria-label="{b.name}, {b.a!.toPrecision(2)} AU"
        onclick={() => ctx.select(b.s.address)}
        onkeydown={(e) => key(e, b.s.address)}
      >
        <circle cx={x} cy={y} r="14" class="hit" />
        <circle cx={x} cy={y} r="6" class="dot" />
        <text x={x + 10} y={y - 9}>{b.name}</text>
      </g>
    {/each}
    {#if star}
      <g class="body star" class:sel={sel === star.address} role="button" tabindex="0" aria-pressed={sel === star.address} aria-label={subjectName(star)} onclick={() => ctx.select(star.address)} onkeydown={(e) => key(e, star.address)}>
        <circle {cx} {cy} r="16" class="hit" />
        <circle {cx} {cy} r="9" class="sun" />
        <text x={cx + 13} y={cy + 22}>{subjectName(star)}</text>
      </g>
    {/if}
    {#if placed.length}
      <text class="scale" x="10" y={h - 10}>log scale · {scale.inner_au.toPrecision(2)} to {scale.outer_au.toPrecision(2)} AU</text>
    {/if}
  </svg>
  {#if unplaced.length}
    <div class="unplaced" aria-label="Position not known">
      <span class="k">{wordsFor(ctx.vocab, field).label}: not known</span>
      {#each unplaced as b (b.s.address)}
        <button type="button" aria-pressed={sel === b.s.address} onclick={() => ctx.select(b.s.address)}>{b.name}</button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .pchart {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 240px;
    background: var(--sse-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  svg {
    position: absolute;
    inset: 0;
    display: block;
  }
  .orbit {
    fill: none;
    stroke: var(--sse-ink);
    stroke-width: 0.8;
    opacity: 0.55;
  }
  .hit {
    fill: transparent;
  }
  .dot {
    fill: var(--sse-ink);
  }
  .sun {
    fill: var(--sse-accent);
  }
  .body {
    cursor: pointer;
    outline: none;
  }
  .body text {
    fill: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-size: 15px;
  }
  .body.sel .dot {
    fill: var(--sse-accent);
  }
  .body.sel text {
    font-weight: 600;
  }
  .body:focus-visible .hit {
    stroke: var(--sse-focus);
    stroke-width: 2;
  }
  .scale {
    fill: var(--sse-ink-muted);
    font-family: var(--sse-font-mono);
    font-size: 11px;
  }
  .unplaced {
    position: absolute;
    right: 10px;
    top: 10px;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
  }
  .k {
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .unplaced button {
    min-height: 32px;
    border: var(--sse-line) dashed var(--sse-ink-muted);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-size: 15px;
    padding: 0 10px;
    cursor: pointer;
  }
  .unplaced button[aria-pressed="true"] {
    border-color: var(--sse-accent);
    color: var(--sse-accent-ink);
  }
</style>
