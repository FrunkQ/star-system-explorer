<!--
  The formation chart (formation design section 7): distance from the star across, time down the page, both on log
  scales. Each body is an inked track that thickens as it grows and bends as it migrates; merging tracks join, thrown
  out ones run off the right edge. The gas disc is a tint that ends when the gas leaves, the condensation lines lean
  inward as it cools, belts are hatched, events are diamonds and the upheaval is a marked span. The time slice is the
  rule across: drag it, or use the arrow keys on it. Drawn from the chart model (ui-contract formation-chart.ts).
-->
<script lang="ts">
  import { chartAu, chartTime, chartXY, massWidth, type FormationChart } from "../../../ui-contract/src/index.ts";

  let {
    chart,
    slice,
    selected = null,
    onslice,
    onslicedone,
    onpick,
  }: {
    chart: FormationChart;
    slice: number;
    /** The body whose story is shown, by its id on the chart. */
    selected?: string | null;
    onslice: (t: number) => void;
    onslicedone: (t: number) => void;
    onpick: (id: string) => void;
  } = $props();

  let w = $state(760);
  let h = $state(520);
  const L = 58;
  const T = 22;
  const R = 28;
  const B = 24;
  const pw = $derived(Math.max(120, w - L - R));
  const ph = $derived(Math.max(160, h - T - B));
  const s = $derived(chartXY(chart, pw, ph));
  const X = (a: number) => L + s.x(a);
  const Y = (t: number) => T + s.y(t);

  const nowNear = (t: number) => Math.abs(Math.log10(t) - Math.log10(chart.now)) < 0.01;
  const tracks = $derived(chart.tracks);
  // Names beside the tracks that last, in the lower part of the chart, staggered where tracks run close.
  const names = $derived.by(() => {
    // Low on the chart, or, when the chart runs on to the star's end, just above now.
    const tl = chart.fate ? chart.now / 12 : Math.pow(10, Math.log10(chart.t_top) + 0.72 * (Math.log10(chart.t_bottom) - Math.log10(chart.t_top)));
    const out: { id: string; x: number; y: number; text: string }[] = [];
    const aAt = (k: (typeof tracks)[number]) => k.segs.find((g) => g.t1 >= tl)?.a1 ?? k.end.a;
    const lasting = tracks.filter((k) => k.end.how === "lasts" || k.end.how === "engulfed").sort((a, b) => aAt(a) - aAt(b));
    // Greedy rows: each name takes the first row where it clears the name before it.
    const rowsEnd: number[] = [];
    for (const k of lasting) {
      const x = X(aAt(k)) + massWidth(k.m_end) / 2 + 4;
      const width = 8 * k.label.length + 6;
      let row = rowsEnd.findIndex((end) => end < x);
      if (row < 0) row = rowsEnd.length;
      rowsEnd[row] = x + width;
      out.push({ id: k.id, x, y: Y(tl) + row * 15, text: k.label });
    }
    return out;
  });
  // Event words, kept apart so they never sit on each other; the diamond keeps the full text as its title.
  const eventLabels = $derived.by(() => {
    const used: { x: number; y: number }[] = [];
    return chart.events.map((e) => {
      const x = X(e.a);
      const y = Y(e.t);
      const clash = used.some((u) => Math.abs(u.y - y) < 13 && Math.abs(u.x - x) < 170);
      const show = !!e.short && !clash;
      if (show) used.push({ x, y });
      const right = x < L + pw - 170;
      return { e, x, y, show, right };
    });
  });

  // The slice: drag anywhere on the chart, or the arrow keys on its handle (a tenth of a decade a press).
  let dragging = $state(false);
  const tAt = (clientY: number, el: Element) => s.t(clientY - el.getBoundingClientRect().top - T);
  function down(e: PointerEvent) {
    if ((e.target as Element).closest("[data-track],[data-event]")) return;
    dragging = true;
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
    onslice(tAt(e.clientY, e.currentTarget as Element));
  }
  function move(e: PointerEvent) {
    if (dragging) onslice(tAt(e.clientY, e.currentTarget as Element));
  }
  function up(e: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    onslicedone(tAt(e.clientY, e.currentTarget as Element));
  }
  function key(e: KeyboardEvent) {
    const step = e.shiftKey ? 0.5 : 0.1;
    const lt = Math.log10(slice);
    let t: number | null = null;
    if (e.key === "ArrowUp") t = Math.pow(10, lt - step);
    else if (e.key === "ArrowDown") t = Math.pow(10, lt + step);
    else if (e.key === "Home") t = chart.t_top;
    else if (e.key === "End") t = chart.now;
    if (t === null) return;
    e.preventDefault();
    t = Math.max(chart.t_top, Math.min(chart.t_bottom, t));
    onslice(t);
    onslicedone(t);
  }
  const uid = $props.id();
  const clip = `fc-${uid}`;
</script>

<div class="fchart" bind:clientWidth={w} bind:clientHeight={h}>
  <!-- svelte-ignore a11y_no_static_element_interactions (pointer drag moves the slice; its handle takes the keys) -->
  <svg width={w} height={h} viewBox="0 0 {w} {h}" role="img" aria-label="How this system formed: distance from the star across, time down the page. {tracks.filter((k) => k.end.how === 'lasts' || k.end.how === 'engulfed').length} worlds last to now." onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up} class:dragging>
    <defs>
      <clipPath id={clip}><rect x={L} y={T} width={pw} height={ph} /></clipPath>
      <pattern id="{clip}-gas" width="7" height="7" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><line x1="0" y1="0" x2="0" y2="7" class="hatch-gas" /></pattern>
      <pattern id="{clip}-belt" width="4" height="4" patternUnits="userSpaceOnUse"><line x1="0" y1="0" x2="0" y2="4" class="hatch-belt" /></pattern>
      <marker id="{clip}-arrow" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0L8 4L0 8Z" class="arrow" /></marker>
    </defs>

    <!-- Axes: time down the left, distance along the foot. -->
    <rect class="frame" x={L} y={T} width={pw} height={ph} />
    {#each chart.ticks_t as k (k.t)}
      <line class="tick" x1={L - 4} x2={L} y1={Y(k.t)} y2={Y(k.t)} />
      <line class="grid" x1={L} x2={L + pw} y1={Y(k.t)} y2={Y(k.t)} />
      <text class="axis" x={L - 7} y={Y(k.t) + 3} text-anchor="end">{k.label}</text>
    {/each}
    {#each chart.ticks_a as k (k.a)}
      <line class="tick" x1={X(k.a)} x2={X(k.a)} y1={T + ph} y2={T + ph + 4} />
      <line class="grid" x1={X(k.a)} x2={X(k.a)} y1={T} y2={T + ph} />
      <text class="axis" x={X(k.a)} y={T + ph + 15} text-anchor="middle">{k.label}</text>
    {/each}
    <text class="kicker" x={L} y={T - 8}>{pw < 460 ? "Time ↓" : "Time since the star lit ↓"}</text>
    <text class="kicker" x={L + pw} y={T - 8} text-anchor="end">{pw < 460 ? "Distance →" : "Distance from the star →"}</text>

    <g clip-path="url(#{clip})">
      <!-- The gas disc, while it lasts. -->
      <rect x={L} y={T} width={pw} height={Math.max(0, Y(chart.gas_until) - T)} fill="url(#{clip}-gas)" class="gas" data-gas />
      <line class="gas-end" x1={L} x2={L + pw} y1={Y(chart.gas_until)} y2={Y(chart.gas_until)} />
      <text class="note gas-t" x={L + 6} y={T + 12}>Gas disc, gone by {chartTime(chart.gas_until)}</text>

      <!-- The upheaval: the violent phase from the break to the settling. -->
      {#if chart.upheaval}
        <rect class="upheaval" x={L} y={Y(chart.upheaval.from)} width={pw} height={Math.max(2, Y(chart.upheaval.to) - Y(chart.upheaval.from))} data-upheaval />
        <text class="note up-t" x={L + 6} y={Math.max(Y(chart.upheaval.from) + 11, Y(chart.upheaval.to) - 5)}>Upheaval · {chartTime(chart.upheaval.from)} to {chartTime(chart.upheaval.to)}</text>
      {/if}

      <!-- The star's end, once the look-ahead has answered: what comes after now is one plausible future. -->
      {#if chart.fate}
        <rect class="future" x={L} y={Y(chart.now)} width={pw} height={Math.max(0, T + ph - Y(chart.now))} data-future />
        <line class="now" x1={L} x2={L + pw} y1={Y(chart.now)} y2={Y(chart.now)} />
        <text class="note now-t" x={L + 6} y={Y(chart.now) + 12}>below now: one plausible future</text>
        <rect class="dies" x={L} y={Y(chart.fate.ms_end)} width={pw} height={Math.max(3, Y(chart.fate.end) - Y(chart.fate.ms_end))} data-star-end><title>{chart.fate.text}</title></rect>
        <text class="note dies-t" x={L + 6} y={Math.min(T + ph - 4, Y(chart.fate.end) + 12)}>The star swells and dies, {chartTime(chart.fate.ms_end)} to {chartTime(chart.fate.end)}: then a {chart.fate.remnant}</text>
      {/if}

      <!-- Belts and clouds left over. -->
      {#each chart.belts as b, i (i)}
        {#if !b.sphere}
          <rect class="belt" x={X(b.a0)} y={Y(b.from)} width={Math.max(3, X(b.a1) - X(b.a0))} height={Math.max(0, T + ph - Y(b.from))} fill="url(#{clip}-belt)" data-belt={b.kind}><title>{b.words}: {chartAu(b.a0)} to {chartAu(b.a1)}, from {chartTime(b.from)}</title></rect>
          <text class="note belt-t" x={X(b.a0) + 3} y={T + ph - 6}>{b.words}</text>
        {:else}
          <rect class="cloud" x={Math.min(L + pw - 6, X(b.a0))} y={Y(b.from)} width={Math.max(6, L + pw - X(b.a0))} height={Math.max(0, T + ph - Y(b.from))} data-belt={b.kind}><title>{b.words}: a sphere round the whole system, {chartAu(b.a0)} and out</title></rect>
        {/if}
      {/each}

      <!-- The condensation lines, leaning inward as the disc cools. -->
      {#each chart.lines as l (l.i)}
        <polyline class="cline" class:water={l.i === 2} points={l.pts.map((p) => `${X(p.a)},${Y(p.t)}`).join(" ")} data-line={l.words}><title>{l.words}</title></polyline>
        {#if l.i === 2 && l.pts.length}
          <text class="note water-t" x={X(l.pts[l.pts.length - 1]!.a) + 5} y={Math.min(Y(l.pts[l.pts.length - 1]!.t) - 4, T + 26)}>water ice line</text>
        {/if}
      {/each}

      <!-- The tracks. -->
      {#each tracks as k (k.id)}
        <g class="track" class:giant={k.giant} class:selected={k.id === selected} class:lost={k.end.how !== "lasts" && k.end.how !== "engulfed"} data-track={k.id} data-track-end={k.end.how}>
          <title>{k.label}: born {chartTime(k.born.t)} at {chartAu(k.born.a)}; {k.end.how === "lasts" ? `now ${chartAu(k.end.a)}` : k.end.how === "merged" ? `merged at ${chartTime(k.end.t)}` : k.end.how === "ejected" ? `thrown out at ${chartTime(k.end.t)}` : k.end.how === "engulfed" ? `now ${chartAu(k.segs.find((g) => g.t1 >= chart.now * (1 - 1e-9))?.a1 ?? k.end.a)}, swallowed by the star at ${chartTime(k.end.t)}` : `shattered at ${chartTime(k.end.t)}`}</title>
          {#each k.segs as g, i (i)}
            <line x1={X(g.a0)} y1={Y(g.t0)} x2={X(g.a1)} y2={Y(g.t1)} stroke-width={massWidth(g.m)} />
          {/each}
          {#if k.end.how === "merged"}
            <line class="join" x1={X(k.end.a)} y1={Y(k.end.t)} x2={X(k.end.into_a)} y2={Y(k.end.t)} />
          {:else if k.end.how === "ejected"}
            <line class="thrown" x1={X(k.end.a)} y1={Y(k.end.t)} x2={L + pw - 2} y2={Y(k.end.t)} marker-end="url(#{clip}-arrow)" />
          {:else if k.end.how === "engulfed"}
            <path class="engulfed" d="M{X(k.end.a) - 4} {Y(k.end.t) - 4}l8 8m0 -8l-8 8" />
          {:else if k.end.how === "shattered"}
            <path class="shatter" d="M{X(k.end.a) - 4} {Y(k.end.t) - 4}l8 8m0 -8l-8 8" />
          {/if}
          <!-- A wide, clear stroke over the track so a thin one is easy to press. -->
          <polyline class="hit" points={[k.born, ...k.segs.map((g) => ({ t: g.t1, a: g.a1 }))].map((p) => `${X(p.a)},${Y(p.t)}`).join(" ")} role="button" tabindex={k.end.how === "lasts" || k.end.how === "engulfed" ? 0 : -1} aria-label={k.label} onclick={() => onpick(k.id)} onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), onpick(k.id))} />
        </g>
      {/each}
      {#each names as n (n.id)}
        <text class="name" class:selected={n.id === selected} x={n.x} y={n.y}>{n.text}</text>
      {/each}

      <!-- Events. -->
      {#each eventLabels as v, i (i)}
        <g class="event" class:violent={/^(impact|scatter|ejected)/.test(v.e.kind)} data-event={v.e.kind}>
          <rect x={v.x - 4} y={v.y - 4} width="8" height="8" transform="rotate(45 {v.x} {v.y})"><title>{chartTime(v.e.t)}: {v.e.text}</title></rect>
          {#if v.show}<text class="ev-t" x={v.right ? v.x + 9 : v.x - 9} y={v.y + 3} text-anchor={v.right ? "start" : "end"}>{v.e.short}</text>{/if}
        </g>
      {/each}
    </g>

    <!-- The time slice. -->
    <g class="slice" data-slice={slice}>
      <line x1={L - 6} x2={L + pw + 6} y1={Y(slice)} y2={Y(slice)} />
      <text class="slice-t" x={L + 6} y={Y(slice) - 5}>time slice · {chartTime(slice)}{nowNear(slice) ? " · now" : ""}</text>
      <rect class="handle" x={L + pw - 1} y={Y(slice) - 7} width="13" height="13" role="slider" tabindex="0" aria-label="Time slice" aria-valuemin={chart.t_top} aria-valuemax={chart.t_bottom} aria-valuenow={slice} aria-valuetext="{chartTime(slice)} since the star lit" aria-orientation="vertical" onkeydown={key} data-slice-handle />
    </g>
  </svg>
</div>

<style>
  .fchart {
    position: relative;
    width: 100%;
    min-width: 0;
    height: 100%;
    min-height: 360px;
    overflow: hidden;
  }
  svg {
    display: block;
    touch-action: pan-x;
    cursor: ns-resize;
    user-select: none;
  }
  .frame {
    fill: none;
    stroke: var(--sse-ink-muted);
    stroke-width: var(--sse-line);
  }
  .grid {
    stroke: var(--sse-ink-faint);
    stroke-width: var(--sse-line);
    stroke-dasharray: 1 4;
    opacity: 0.6;
  }
  .tick {
    stroke: var(--sse-ink-muted);
    stroke-width: var(--sse-line);
  }
  .axis,
  .note,
  .ev-t,
  .slice-t,
  .kicker {
    font-family: var(--sse-font-mono);
    font-size: 10.5px;
  }
  .axis {
    fill: var(--sse-ink-muted);
  }
  .kicker {
    fill: var(--sse-ink-muted);
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
  .note {
    fill: var(--sse-ink-muted);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 3px;
  }
  .gas {
    opacity: 0.55;
  }
  :global(.hatch-gas) {
    stroke: var(--sse-ink-faint);
    stroke-width: 1;
  }
  :global(.hatch-belt) {
    stroke: var(--sse-ink-muted);
    stroke-width: 1;
  }
  .gas-end {
    stroke: var(--sse-zone-ink);
    stroke-dasharray: 1 3;
    stroke-width: calc(var(--sse-line) * 1.5);
  }
  .gas-t {
    fill: var(--sse-zone-ink);
  }
  .future {
    fill: color-mix(in srgb, var(--sse-ink) 4%, transparent);
  }
  .now {
    stroke: var(--sse-ink);
    stroke-width: 1;
    stroke-dasharray: 2 3;
  }
  .dies {
    fill: color-mix(in srgb, var(--sse-accent) 22%, transparent);
    stroke: var(--sse-accent);
    stroke-width: 1;
  }
  .engulfed {
    stroke: var(--sse-accent-ink);
    stroke-width: 2;
  }
  .upheaval {
    fill: var(--sse-accent);
    opacity: 0.12;
  }
  .up-t {
    fill: var(--sse-accent-ink);
  }
  .belt {
    opacity: 0.55;
  }
  .cloud {
    fill: var(--sse-ink-faint);
    opacity: 0.25;
  }
  .cline {
    fill: none;
    stroke: var(--sse-ink-muted);
    stroke-width: var(--sse-line);
    stroke-dasharray: 2 4;
  }
  .cline.water {
    stroke: var(--sse-accent);
    stroke-width: calc(var(--sse-line) * 1.5);
    stroke-dasharray: 5 4;
  }
  .water-t {
    fill: var(--sse-accent-ink);
    text-transform: none;
  }
  .track line {
    stroke: var(--sse-ink);
    stroke-linecap: round;
  }
  .track.lost line {
    stroke: var(--sse-ink-muted);
  }
  .track.selected line {
    stroke: var(--sse-accent);
  }
  .track:hover line {
    stroke: var(--sse-accent-ink);
  }
  .track .join {
    stroke-width: 1;
    stroke-dasharray: 2 2;
  }
  .track .thrown {
    stroke: var(--sse-accent);
    stroke-width: 1.4;
  }
  :global(.arrow) {
    fill: var(--sse-accent);
  }
  .shatter {
    stroke: var(--sse-ink-muted);
    stroke-width: 1.4;
    fill: none;
  }
  .hit:focus {
    outline: none;
  }
  .track:focus-within line {
    stroke: var(--sse-accent-ink);
  }
  .hit {
    fill: none;
    stroke: transparent;
    stroke-width: 12;
    cursor: pointer;
    pointer-events: stroke;
  }
  .name {
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 14px;
    fill: var(--sse-ink);
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 3px;
    pointer-events: none;
  }
  .name.selected {
    font-weight: bold;
    fill: var(--sse-accent-ink);
  }
  .event rect {
    fill: var(--sse-ink-muted);
    stroke: var(--sse-ground);
    stroke-width: 1;
  }
  .event.violent rect {
    fill: var(--sse-accent);
  }
  .ev-t {
    fill: var(--sse-accent-ink);
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 3px;
    pointer-events: none;
  }
  .slice line {
    stroke: var(--sse-accent);
    stroke-width: calc(var(--sse-line) * 1.5);
  }
  .slice-t {
    fill: var(--sse-accent-ink);
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 3px;
  }
  .handle {
    fill: var(--sse-accent);
    cursor: ns-resize;
    transition: transform 120ms ease-out;
    transform-box: fill-box;
    transform-origin: center;
  }
  .handle:hover,
  .handle:focus-visible,
  .dragging .handle {
    transform: scale(1.25);
    outline: none;
  }
  @media (prefers-reduced-motion: reduce) {
    .handle {
      transition: none;
    }
  }
</style>
