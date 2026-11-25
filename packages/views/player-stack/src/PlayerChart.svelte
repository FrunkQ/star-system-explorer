<!--
  Chart: the 2D orbital chart a player is allowed to see. Bodies whose orbit
  the audience has are drawn on a log-radial scale; bodies known only to
  exist are listed beside the chart as "position not known", never placed by
  guess. Selecting a body selects it for the other views of the screen.
-->
<script lang="ts">
  import { constant, controlOn, fact, subjectName, wordsFor, type PlayerContext, type PlayerScene } from "../../../ui-contract/src/index.ts";
  import { fitScale, radius, polar } from "../../../chart-grammar/src/scale.ts";
  import { topLevel } from "./pick.ts";
  import HatchDefs from "../../../chart-grammar/src/HatchDefs.svelte";
  import Mark from "../../../chart-grammar/src/Mark.svelte";

  let { ctx, scene }: { ctx: PlayerContext; scene: PlayerScene } = $props();
  const AU = constant("au");
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
  // With `animate` on, bodies drift round their orbits (presentation only; still under reduced motion).
  let drift = $state(0);
  $effect(() => {
    if (ctx.params.animate !== true || (typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches)) return;
    let raf = 0;
    const tick = () => {
      drift = (drift + 0.15) % 360;
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  });
  const angle = (i: number) => 20 + i * (300 / Math.max(1, placed.length)) + drift / (1 + i);
  const showNames = $derived(ctx.params.labels !== false);
  const fontPx = $derived(Number(ctx.params.label_size ?? 15));
  const markScale = $derived(Number(ctx.params.mark_scale ?? 1));
  const sel = $derived(ctx.selection());
  // AT13: with "select" off the bodies are drawn, not pressed: no button role, no tab stop, no click.
  const pick = $derived(controlOn(ctx.controls, "select"));

  function key(e: KeyboardEvent, address: string) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      ctx.select(address);
    }
  }
</script>

<div class="pchart" bind:clientWidth={w} bind:clientHeight={h} style="--label-px: {fontPx}px" data-animate={ctx.params.animate === true}>
  <svg width={w} height={h} role="group" aria-label="Chart of {star ? subjectName(star) : 'the system'}">
    {#if ctx.mark}<HatchDefs prefix="pc" />{/if}
    {#each placed as b, i (b.s.address)}
      {@const r = radius(b.a!, scale)}
      {@const [x, y] = polar(cx, cy, r, angle(i))}
      <circle class="orbit" {cx} {cy} {r} />
      <!-- svelte-ignore a11y_no_noninteractive_tabindex (a button while "select" is on; then it has a tab stop) -->
      <g
        class="body"
        class:sel={sel === b.s.address}
        role={pick ? "button" : "img"}
        tabindex={pick ? 0 : undefined}
        aria-pressed={pick ? sel === b.s.address : undefined}
        aria-label="{b.name}, {b.a!.toPrecision(2)} AU"
        onclick={() => pick && ctx.select(b.s.address)}
        onkeydown={(e) => pick && key(e, b.s.address)}
      >
        <circle cx={x} cy={y} r="14" class="hit" />
        {#if ctx.mark}
          {@const m = ctx.mark(b.s)}
          <Mark mark={{ ...m, size: Math.max(4, Math.min(8, m.size)) * markScale }} cx={x} cy={y} prefix="pc" />
          {#if showNames}<text x={x + 12 * markScale} y={y - 10 * markScale}>{b.name}</text>{/if}
        {:else}
          <circle cx={x} cy={y} r={6 * markScale} class="dot" />
          {#if showNames}<text x={x + 10 * markScale} y={y - 9 * markScale}>{b.name}</text>{/if}
        {/if}
      </g>
    {/each}
    {#if star}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex (as above) -->
      <g class="body star" class:sel={sel === star.address} role={pick ? "button" : "img"} tabindex={pick ? 0 : undefined} aria-pressed={pick ? sel === star.address : undefined} aria-label={subjectName(star)} onclick={() => pick && ctx.select(star.address)} onkeydown={(e) => pick && key(e, star.address)}>
        <circle {cx} {cy} r="16" class="hit" />
        <circle {cx} {cy} r="9" class="sun" />
        {#if showNames}<text x={cx + 13} y={cy + 22}>{subjectName(star)}</text>{/if}
      </g>
    {/if}
    {#if placed.length && ctx.params.show_scale !== false}
      <text class="scale" x="10" y={h - 10}>log scale · {scale.inner_au.toPrecision(2)} to {scale.outer_au.toPrecision(2)} AU</text>
    {/if}
  </svg>
  {#if unplaced.length && ctx.params.show_unplaced !== false}
    <div class="unplaced" aria-label="Position not known">
      <span class="k">{wordsFor(ctx.vocab, field).label}: not known</span>
      {#each unplaced as b (b.s.address)}
        {#if pick}<button type="button" aria-pressed={sel === b.s.address} onclick={() => ctx.select(b.s.address)}>{b.name}</button>{:else}<span class="name">{b.name}</span>{/if}
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
    background: var(--sse-chart-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    filter: drop-shadow(var(--sse-text-glow));
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
  .body[role="img"] {
    cursor: default;
  }
  .body text {
    fill: var(--sse-ink);
    font-family: var(--sse-title-font);
    font-style: var(--sse-title-style);
    font-size: var(--label-px, 15px);
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
    letter-spacing: var(--sse-label-tracking);
    text-transform: var(--sse-label-case);
    color: var(--sse-ink-muted);
  }
  .unplaced .name {
    color: var(--sse-ink);
    font-family: var(--sse-title-font);
    font-size: 15px;
  }
  .unplaced button {
    min-height: 32px;
    border: var(--sse-line) dashed var(--sse-ink-muted);
    background: var(--sse-chart-ground);
    color: var(--sse-ink);
    font-family: var(--sse-title-font);
    font-size: 15px;
    padding: 0 10px;
    cursor: pointer;
  }
  .unplaced button[aria-pressed="true"] {
    border-color: var(--sse-accent);
    color: var(--sse-accent-ink);
  }
</style>
