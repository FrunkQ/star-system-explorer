<!--
  The star's end, close up (formation design section 2, stage 9's close; F10c). On the main chart the whole giant
  phase is a sliver at the foot, so here it is opened out: distance across on the main chart's scale, and down the
  page the time left before the star sheds its outer layers, on a log scale, so the last few million years, when it
  swells fastest and swallows its worlds, get room. The star is the shaded swell from the left; each world's track
  widens as the star loses mass, or stops where the surface reaches it. A strip at the foot shows what is left.
  From ui-contract formation-fate.ts.
-->
<script lang="ts">
  import { chartAu, chartTime, massWidth, type ChartFate, type FormationChart } from "../../../ui-contract/src/index.ts";

  let { chart, fate, selected = null, onpick }: { chart: FormationChart; fate: ChartFate; selected?: string | null; onpick: (id: string) => void } = $props();

  let w = $state(760);
  const L = 58;
  const R = 28;
  const T = 18;
  const H1 = 190;
  const AFTER = 44;
  const h = T + H1 + AFTER + 20;
  const pw = $derived(Math.max(120, w - L - R));
  const lg = Math.log10;
  const X = (a: number) => L + ((lg(Math.max(a, chart.a_in)) - lg(chart.a_in)) / (lg(chart.a_out) - lg(chart.a_in))) * pw;
  // Time left before the end, on a log scale from where hydrogen runs out down to a tenth of a million years.
  const EPS = 0.1;
  const span = $derived(Math.max(fate.end - fate.ms_end, EPS * 10));
  const Y = (t: number) => {
    if (t >= fate.end) return T + H1;
    const left = Math.max(fate.end - t, EPS);
    return T + (1 - lg(left / EPS) / lg(span / EPS)) * H1;
  };
  const ticks = $derived.by(() => {
    const out: { y: number; label: string }[] = [];
    for (let e = Math.floor(lg(span)); e >= 0; e--) {
      const left = Math.pow(10, e);
      if (left > span) continue;
      out.push({ y: Y(fate.end - left), label: `${chartTime(left)} before` });
    }
    return out;
  });
  const swell = $derived.by(() => {
    const pts = fate.star.filter((s) => s.t >= fate.ms_end && s.t < fate.end);
    if (!pts.length) return "";
    const edge = pts.map((s) => `${X(s.r_au)},${Y(s.t)}`);
    return [`${L},${Y(pts[0]!.t)}`, ...edge, `${L},${Y(pts[pts.length - 1]!.t)}`].join(" ");
  });
  const peak = $derived(fate.star.filter((s) => s.t < fate.end).reduce((a, s) => (s.r_au > a.r_au ? s : a), { r_au: 0, t: fate.ms_end }));
  const marks = $derived.by(() => {
    const used: number[] = [];
    return fate.marks
      .filter((m) => m.kind !== "engulfed" || m.ids.length)
      .map((m) => {
        const y = Y(m.t);
        const show = !used.some((u) => Math.abs(u - y) < 12);
        if (show) used.push(y);
        return { m, y, show };
      });
  });
  const kept = $derived(fate.worlds.filter((x) => x.engulfed === undefined));
</script>

<section class="fate" aria-label="The star's end" data-fate>
  <h3><span class="k">The star's end</span> <span class="future">one plausible future · nothing on the campaign changes</span></h3>
  <div class="box" bind:clientWidth={w}>
    <svg width={w} height={h} viewBox="0 0 {w} {h}" role="img" aria-label={fate.text}>
      <rect class="frame" x={L} y={T} width={pw} height={H1} />
      {#each chart.ticks_a as k (k.a)}
        <line class="grid" x1={X(k.a)} x2={X(k.a)} y1={T} y2={T + H1 + AFTER} />
        <text class="axis" x={X(k.a)} y={h - 4} text-anchor="middle">{k.label}</text>
      {/each}
      {#each ticks as k (k.label)}
        <line class="grid" x1={L} x2={L + pw} y1={k.y} y2={k.y} />
        <text class="axis" x={L - 6} y={k.y + 3} text-anchor="end">{k.label.replace(" before", "")}</text>
      {/each}
      <text class="axis" x={L - 6} y={T + H1 + 3} text-anchor="end">end</text>
      <text class="kicker" x={L} y={T - 6}>Time left before it sheds its shell ↓ (log)</text>

      <!-- The star swelling from the left. -->
      {#if swell}<polygon class="star" points={swell} data-fate-star><title>The star's surface: {chartAu(peak.r_au)} at its largest, at {chartTime(peak.t)}</title></polygon>{/if}

      <!-- The worlds through it. -->
      {#each fate.worlds as v (v.id)}
        {@const pts = v.pts.filter((p) => p.t >= fate.ms_end && p.t <= fate.end)}
        {#if pts.length}
          <g class="world" class:selected={v.id === selected} class:lost={v.engulfed !== undefined} data-fate-world={v.id} data-fate-end={v.engulfed !== undefined ? "engulfed" : "lasts"}>
            <polyline points={pts.map((p) => `${X(p.a)},${Y(p.t)}`).join(" ")} stroke-width={Math.max(1, massWidth(v.m) * 0.6)} />
            {#if v.engulfed !== undefined}
              {@const p = pts[pts.length - 1]!}
              <path class="gone" d="M{X(p.a) - 4} {Y(p.t) - 4}l8 8m0 -8l-8 8" />
            {/if}
            <polyline class="hit" points={pts.map((p) => `${X(p.a)},${Y(p.t)}`).join(" ")} role="button" tabindex="-1" aria-label={v.label} onclick={() => onpick(v.id)} onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), onpick(v.id))}>
              <title>{v.label}{v.engulfed !== undefined ? `: swallowed at ${chartTime(v.engulfed)}` : ""}</title>
            </polyline>
          </g>
        {/if}
      {/each}
      {#each marks as v, i (i)}
        <g class="mark" class:violent={v.m.kind === "engulfed"} data-fate-mark={v.m.kind}>
          <line x1={L} x2={L + 6} y1={v.y} y2={v.y} />
          {#if v.show}<text x={L + pw - 4} y={v.y < T + 14 ? v.y + 12 : v.y - 3} text-anchor="end">{v.m.words}<title>{chartTime(v.m.t)}: {v.m.why}</title></text>{/if}
        </g>
      {/each}

      <!-- After: the remnant and what is left round it. -->
      <rect class="after" x={L} y={T + H1} width={pw} height={AFTER} />
      <text class="note" x={L + 6} y={T + H1 + 14}>After: a {fate.remnant}, cooling · {kept.length ? `${kept.length} ${kept.length === 1 ? "world" : "worlds"} left` : "no worlds left"}</text>
      {#each kept as v (v.id)}
        {@const a = v.pts[v.pts.length - 1]!.a}
        <circle class="left" cx={X(a)} cy={T + H1 + AFTER * 0.65} r={Math.max(2.5, massWidth(v.m) * 0.7)} data-fate-left={v.id}><title>{v.label}: {chartAu(v.pts[0]!.a)} to {chartAu(a)}</title></circle>
      {/each}
    </svg>
  </div>
  <p class="words" data-fate-words>{fate.text}</p>
  <p class="foot">Aged by the time model (stellar tracks of Hurley, Pols and Tout 2000; orbits widening as the star loses mass, tides drawing the near ones in), assuming {fate.assumed.join("; ")}. Beyond {chartTime(fate.plausible_after)} this is one plausible future, not a forecast.</p>
</section>

<style>
  .fate {
    margin-top: 6px;
  }
  h3 {
    margin: 0 0 4px;
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
    flex-wrap: wrap;
  }
  .k {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .future {
    font-size: 12px;
    color: var(--sse-ink-muted);
    font-weight: normal;
  }
  .box {
    width: 100%;
    min-width: 0;
    overflow: hidden;
  }
  svg {
    display: block;
    font-family: var(--sse-font-mono);
  }
  .frame {
    fill: none;
    stroke: var(--sse-ink);
    stroke-width: var(--sse-line);
  }
  .grid {
    stroke: var(--sse-ink-faint);
    stroke-width: 0.5;
  }
  .axis,
  .kicker,
  .note {
    font-size: 10.5px;
    fill: var(--sse-ink-muted);
  }
  .kicker {
    letter-spacing: 0.08em;
  }
  .star {
    fill: color-mix(in srgb, var(--sse-accent) 30%, var(--sse-ground));
    stroke: var(--sse-accent);
    stroke-width: 1;
  }
  .world polyline {
    fill: none;
    stroke: var(--sse-ink);
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .world.lost polyline {
    stroke: var(--sse-ink-muted);
  }
  .world.selected polyline:not(.hit) {
    stroke: var(--sse-accent-ink);
  }
  .world .hit {
    stroke: transparent;
    stroke-width: 12;
    cursor: pointer;
    pointer-events: stroke;
  }
  .gone {
    stroke: var(--sse-accent-ink);
    stroke-width: 2;
  }
  .mark line {
    stroke: var(--sse-ink);
  }
  .mark text {
    font-size: 11px;
    fill: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  .mark.violent text {
    fill: var(--sse-accent-ink);
    font-weight: 600;
  }
  .after {
    fill: color-mix(in srgb, var(--sse-ink) 5%, var(--sse-ground));
    stroke: var(--sse-ink);
    stroke-width: var(--sse-line);
  }
  .left {
    fill: var(--sse-ink);
  }
  .words {
    margin: 6px 0 0;
    font-size: 13.5px;
    line-height: 1.45;
    max-width: 60em;
  }
  .foot {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
    max-width: 60em;
  }
</style>
