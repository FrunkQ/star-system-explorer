<!--
  Formation replay: a grown system's nine stages played over time, read from
  the formation run's own record (ui-contract replay.ts). Top down on a log
  scale like the system chart: the disc and its material bands with the
  condensation lines sweeping inward, embryos gathering and growing from
  their ledgers, the violent phase (chains breaking, worlds flung, giant
  impacts, rogues thrown out), then the leftovers. Play steps through the
  run's beats (every event lingers), the scrubber moves in log time, and the
  stage chips jump. Nothing here simulates.
-->
<script lang="ts">
  import { bandWords, isShell, replayAt, replayClock, replayModel, flatChart, DEFAULT_VIEW, type FormationRan, type ReplayFrame, type ViewChoice } from "../../../ui-contract/src/index.ts";
  import { radius, scaleRings, reducedMotion, ViewControls } from "../../../chart-grammar/src/index.ts";
  import { onDestroy } from "svelte";

  let { run, title, onclose, view = DEFAULT_VIEW, onview }: { run: FormationRan; title: string; onclose?: () => void; view?: ViewChoice; onview?: (c: ViewChoice) => void } = $props();
  // The standard view controls (rule 12): the replay is drawn from above for now; the rest is listed and says why.
  const CONTROLS = flatChart("The replay is drawn from above for now");

  const m = $derived(replayModel(run));
  let w = $state(900);
  let h = $state(620);
  const cx = $derived(w / 2);
  const cy = $derived(h / 2);
  const scale = $derived({ inner_au: m.inner_au, outer_au: m.outer_au, r_min: 26, r_max: Math.max(80, Math.min(w, h) * 0.47) });
  const r = (a: number) => radius(a, scale);
  const rings = $derived(scaleRings(scale, Math.hypot(cx, cy)).filter((g) => g.major));

  // Playback position: a fractional beat index; the time is log-interpolated between beats.
  let pos = $state(0);
  let playing = $state(false);
  const lg = Math.log10;
  const timeAt = (p: number) => {
    const b = m.beats;
    const i = Math.max(0, Math.min(b.length - 1, Math.floor(p)));
    const j = Math.min(b.length - 1, i + 1);
    const f = p - i;
    return Math.pow(10, lg(b[i]!) + (lg(b[j]!) - lg(b[i]!)) * f);
  };
  const t = $derived(timeAt(pos));
  const frame: ReplayFrame = $derived(replayAt(m, t));
  const hasEventAt = $derived(new Set(m.events.map((e) => Number(e.t_myr.toPrecision(9)))));

  let raf = 0;
  let last = 0;
  function tick(now: number) {
    if (!playing) return;
    const dt = last ? now - last : 0;
    last = now;
    const i = Math.floor(pos);
    // Each beat takes a moment; a beat with an event lingers so it can be read.
    const linger = hasEventAt.has(Number((m.beats[i] ?? 0).toPrecision(9))) ? 1400 : 260;
    pos = Math.min(m.beats.length - 1, pos + dt / linger);
    if (pos >= m.beats.length - 1) playing = false;
    else raf = requestAnimationFrame(tick);
  }
  function play() {
    if (playing) return void (playing = false);
    if (pos >= m.beats.length - 1) pos = 0;
    playing = true;
    last = 0;
    raf = requestAnimationFrame(tick);
  }
  onDestroy(() => cancelAnimationFrame(raf));
  function jumpTo(myr: number) {
    playing = false;
    const i = m.beats.findIndex((b) => b >= myr);
    pos = i < 0 ? m.beats.length - 1 : i;
  }
  // The scrubber runs in log time, 0 to 1000.
  const slider = $derived(Math.round(((lg(t) - lg(m.beats[0]!)) / (lg(m.t1) - lg(m.beats[0]!))) * 1000));
  function scrub(v: number) {
    playing = false;
    const target = Math.pow(10, lg(m.beats[0]!) + (v / 1000) * (lg(m.t1) - lg(m.beats[0]!)));
    let i = m.beats.findIndex((b) => b >= target);
    if (i <= 0) i = Math.max(0, i);
    const b0 = m.beats[Math.max(0, i - 1)]!;
    const b1 = m.beats[i]!;
    pos = i <= 0 ? 0 : i - 1 + (b1 > b0 ? (lg(target) - lg(b0)) / (lg(b1) - lg(b0)) : 0);
  }
  const markX = (myr: number) => ((lg(myr) - lg(m.beats[0]!)) / (lg(m.t1) - lg(m.beats[0]!))) * 100;
  function onkey(e: KeyboardEvent) {
    if (e.key === " " || e.key === "k") (e.preventDefault(), play());
    else if (e.key === "ArrowRight") (e.preventDefault(), (playing = false), (pos = Math.min(m.beats.length - 1, Math.floor(pos) + 1)));
    else if (e.key === "ArrowLeft") (e.preventDefault(), (playing = false), (pos = Math.max(0, Math.ceil(pos) - 1)));
    else if (e.key === "Escape" && onclose) onclose();
  }

  // Bands: hot inner gas through rock to the ices.
  const BAND_INK: Record<string, string> = {
    vapour: "color-mix(in srgb, var(--sse-accent) 26%, var(--sse-ground))",
    rock: "color-mix(in srgb, var(--sse-accent) 14%, var(--sse-ink-faint))",
    carbon: "color-mix(in srgb, var(--sse-ink) 22%, var(--sse-ground))",
    "water-ice": "color-mix(in srgb, var(--sse-zone-ink) 70%, var(--sse-ground))",
    "co2-ice": "color-mix(in srgb, var(--sse-zone-ink) 45%, var(--sse-ground))",
    "co-ice": "color-mix(in srgb, var(--sse-zone-ink) 25%, var(--sse-ground))",
  };
  const LINE_WORDS = ["rock line", "carbon line", "water ice line", "CO₂ ice line", "CO ice line"];
  const ring = (a0: number, a1: number) => {
    const r0 = Math.max(0, r(a0));
    const r1 = r(a1);
    return `M${cx - r1} ${cy}a${r1} ${r1} 0 1 0 ${r1 * 2} 0a${r1} ${r1} 0 1 0 ${-r1 * 2} 0Z M${cx - r0} ${cy}a${r0} ${r0} 0 1 0 ${r0 * 2} 0a${r0} ${r0} 0 1 0 ${-r0 * 2} 0Z`;
  };
  const discBands = $derived.by(() => {
    const d = frame.disc;
    if (!d) return [];
    const edges = [d.inner_au, ...d.lines, d.outer_au];
    return m.disc.bands.map((key, i) => ({ key, a0: edges[i]!, a1: Math.max(edges[i]!, edges[i + 1] ?? d.outer_au) })).filter((b) => b.a1 > b.a0);
  });
  // An orbit with eccentricity e on a log scale: periapsis and apoapsis placed honestly, an ellipse through them.
  const orbitPath = (a: number, e: number, ang: number) => {
    const q = r(Math.max(a * (1 - e), m.inner_au));
    const Q = r(a * (1 + e));
    const rx = (q + Q) / 2;
    const off = (Q - q) / 2;
    const rad = ((ang + 180) * Math.PI) / 180;
    const ry = e < 0.01 ? rx : Math.sqrt(q * Q);
    return { cx: cx + off * Math.cos(rad), cy: cy - off * Math.sin(rad), rx, ry, rot: -(ang + 180) };
  };
  const pt = (a: number, ang: number): [number, number] => [cx + r(a) * Math.cos((ang * Math.PI) / 180), cy - r(a) * Math.sin((ang * Math.PI) / 180)];
  // Marks grow with the cube root of mass: an embryo of a thousandth of an Earth is a speck, a giant a disc.
  const size = (mEarth: number) => Math.max(2.4, Math.min(16, 5.5 * Math.cbrt(Math.max(mEarth, 1e-6))));
  const freshIds = $derived(new Set(frame.fresh.flatMap((e) => e.ids)));
  const freshImpact = $derived(new Set(frame.fresh.filter((e) => e.kind.startsWith("impact")).flatMap((e) => e.ids)));
  const starR = $derived(Math.max(5, Math.min(16, 7 * Math.pow(frame.luminosity, 0.25))));
  const motionOk = !reducedMotion();
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions (keys: space plays, arrows step, Escape closes) -->
<div class="replay" role="region" aria-label="How {title} formed" data-replay-stage={frame.stage.n} data-replay-t={t.toPrecision(4)} tabindex="-1" onkeydown={onkey}>
  <header>
    <div class="clock"><b data-replay-clock>{replayClock(t)}</b> <span>after the disc formed</span></div>
    <div class="stage"><span class="k">Stage {frame.stage.n} of 9</span> <b>{frame.stage.name}</b></div>
    <ViewControls spec={CONTROLS} choice={view} onchange={(c) => onview?.(c)} />
    {#if onclose}<button type="button" class="close" onclick={onclose} aria-label="Close the replay">×</button>{/if}
  </header>

  <div class="chart" bind:clientWidth={w} bind:clientHeight={h}>
    <svg width={w} height={h} viewBox="0 0 {w} {h}" role="img" aria-label="{title} at {replayClock(t)}: {frame.bodies.length} {frame.bodies.length === 1 ? 'body' : 'bodies'}{frame.disc ? ', the gas disc' : ''}">
      {#if frame.disc}
        <g class="disc" style:opacity={Math.min(0.6, 0.12 + 0.5 * frame.disc.opacity)} data-disc>
          {#each discBands as b (b.key)}<path d={ring(b.a0, b.a1)} fill={BAND_INK[b.key] ?? "var(--sse-ink-faint)"} fill-rule="evenodd"><title>{bandWords(b.key)}</title></path>{/each}
          {#each frame.disc.lines as a, i (i)}
            <circle class="line" class:water={i === 2} cx={cx} cy={cy} r={r(a)} data-line={LINE_WORDS[i]} />
            {#if i === 2}<text class="line-t" x={cx} y={cy + r(a) + 13} text-anchor="middle">{LINE_WORDS[i]} · {Number(a.toPrecision(2))} AU</text>{/if}
          {/each}
        </g>
      {/if}
      {#each frame.clouds as c, i (i)}
        {#if isShell(c)}
          <circle class="shell" cx={cx} cy={cy} r={Math.min(r(c.a_max_au), Math.hypot(w, h))} />
          <text class="cloud-t" x={cx} y={Math.min(h - 8, cy + r(c.a_min_au) + 12)} text-anchor="middle">{c.kind.replace(/-/g, " ")} · sphere</text>
        {:else}
          <path class="belt" d={ring(c.a_min_au, c.a_max_au)} fill-rule="evenodd" data-belt={c.kind} />
        {/if}
      {/each}
      {#each rings as g (g.au)}
        <circle class="scale" cx={cx} cy={cy} r={r(g.au)} />
        {#if cy - r(g.au) > 14}<text class="scale-t" x={cx + 4} y={cy - r(g.au) - 4}>{g.units.toLocaleString("en-GB")} AU</text>{/if}
      {/each}
      <circle class="star" cx={cx} cy={cy} r={starR} />
      {#each frame.bodies as b (b.id)}
        {@const o = orbitPath(b.a_au, b.e, b.angle_deg)}
        {@const [x, y] = pt(b.a_au, b.angle_deg)}
        <ellipse class="orbit" class:stretched={b.e > 0.1} cx={o.cx} cy={o.cy} rx={o.rx} ry={o.ry} transform="rotate({o.rot} {o.cx} {o.cy})" />
        {#if freshImpact.has(b.id) && motionOk}<circle class="burst" cx={x} cy={y} r={size(b.m) + 4} />{/if}
        <circle class="body" class:giant={b.giant} class:fresh={freshIds.has(b.id)} cx={x} cy={y} r={size(b.m)} data-replay-body={b.id}><title>{b.label}: {Number(b.m.toPrecision(2))} Earth masses at {Number(b.a_au.toPrecision(2))} AU</title></circle>
        {#if b.m > 0.05 || freshIds.has(b.id)}<text class="name" class:fresh={freshIds.has(b.id)} x={x + size(b.m) + 4} y={y - size(b.m) - 2}>{b.label}</text>{/if}
        {#each frame.moons.filter((x) => x.parent === b.id) as mo, k (k)}
          <circle class="moon" cx={x + (size(b.m) + 5) * Math.cos(k * 2.4)} cy={y - (size(b.m) + 5) * Math.sin(k * 2.4)} r="1.8" />
        {/each}
      {/each}
    </svg>
    <div class="captions">
      <p class="stage-text" data-stage-text>{frame.stage.text}</p>
      {#if frame.last}<p class="event" class:fresh={frame.fresh.includes(frame.last)} data-replay-event={frame.last.kind}>{frame.last.text}</p>{/if}
    </div>
  </div>

  <div class="controls">
    <button type="button" class="play" onclick={play} aria-pressed={playing} data-replay-play>{playing ? "Pause" : pos >= m.beats.length - 1 ? "Replay" : "Play"}</button>
    <div class="scrub">
      <input type="range" min="0" max="1000" value={slider} oninput={(e) => scrub(Number((e.target as HTMLInputElement).value))} aria-label="Time since the disc formed" aria-valuetext={replayClock(t)} />
      <div class="marks" aria-hidden="true">
        {#each m.events.filter((x) => x.t_myr <= m.t1) as e, i (i)}<span class="mark" class:violent={e.kind.startsWith("impact") || e.kind === "scatter" || e.kind === "ejected"} style:left="{markX(e.t_myr)}%"></span>{/each}
      </div>
    </div>
    <ol class="stages" aria-label="Stages">
      {#each [...m.stages].sort((a, b) => a.n - b.n) as s (s.n)}
        <li><button type="button" aria-current={frame.stage.n === s.n ? "step" : undefined} onclick={() => jumpTo(s.from_myr)} title={s.name}>{s.n}</button></li>
      {/each}
    </ol>
  </div>
  <p class="honest">From this run's own record: the disc's lines and mass, each embryo's ledger, the upheaval's events. Orbits between recorded moments are interpolated; the turning round the star is for display. Distances on a log scale.</p>
</div>

<style>
  .replay {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) auto auto;
    height: 100%;
    min-height: 420px;
    gap: 8px;
    color: var(--sse-ink);
    background: var(--sse-ground);
    font-family: var(--sse-font-body);
    outline: none;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 18px;
    padding: 10px 16px 0;
  }
  .clock b {
    font-family: var(--sse-font-display);
    font-size: 22px;
    font-weight: normal;
  }
  .clock span,
  .k {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .k {
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-accent-ink);
  }
  .stage {
    flex: 1;
  }
  .close {
    border: 0;
    background: transparent;
    font-size: 22px;
    color: var(--sse-ink);
    cursor: pointer;
    min-width: 36px;
    min-height: 36px;
  }
  .chart {
    position: relative;
    min-height: 0;
    overflow: hidden;
  }
  svg {
    display: block;
  }
  .scale {
    fill: none;
    stroke: var(--sse-ink-faint);
    stroke-dasharray: 1 3;
    stroke-width: var(--sse-line);
  }
  .scale-t,
  .cloud-t {
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 3px;
  }
  .scale-t,
  .line-t,
  .cloud-t {
    fill: var(--sse-ink-muted);
    font-family: var(--sse-font-mono);
    font-size: 10px;
  }
  .disc {
    transition: opacity 0.3s;
  }
  .line {
    fill: none;
    stroke: var(--sse-ink-muted);
    stroke-width: var(--sse-line);
    stroke-dasharray: 3 3;
  }
  .line.water {
    stroke: var(--sse-zone-ink);
    stroke-width: calc(var(--sse-line) * 2);
  }
  .belt {
    fill: var(--sse-ink-faint);
    opacity: 0.5;
  }
  .shell {
    fill: var(--sse-ink);
    fill-opacity: 0.04;
    stroke: var(--sse-ink-faint);
    stroke-dasharray: 1 3;
  }
  .star {
    fill: var(--sse-accent);
  }
  .orbit {
    fill: none;
    stroke: var(--sse-ink-faint);
    stroke-width: var(--sse-line);
  }
  .orbit.stretched {
    stroke: var(--sse-accent);
    stroke-dasharray: 4 3;
  }
  .body {
    fill: var(--sse-ink);
  }
  .body.giant {
    fill: var(--sse-ground);
    stroke: var(--sse-ink);
    stroke-width: 2;
  }
  .body.fresh {
    fill: var(--sse-accent);
  }
  .burst {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 2;
    animation: burst 0.9s ease-out infinite;
    transform-box: fill-box;
    transform-origin: center;
  }
  @keyframes burst {
    from {
      opacity: 1;
      transform: scale(0.6);
    }
    to {
      opacity: 0;
      transform: scale(2.4);
    }
  }
  .moon {
    fill: var(--sse-ink-muted);
  }
  .name {
    fill: var(--sse-ink-muted);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 12px;
  }
  .name.fresh {
    fill: var(--sse-accent-ink);
  }
  .captions {
    position: absolute;
    left: 16px;
    bottom: 12px;
    max-width: min(30em, 46%);
    display: grid;
    gap: 6px;
    pointer-events: none;
  }
  .captions p {
    margin: 0;
    padding: 8px 12px;
    background: color-mix(in srgb, var(--sse-ground) 88%, transparent);
    border-left: 3px solid var(--sse-ink-faint);
    font-size: 14px;
    line-height: 1.45;
  }
  .captions .event {
    border-left-color: var(--sse-accent);
  }
  .captions .event.fresh {
    color: var(--sse-accent-ink);
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 16px;
    flex-wrap: wrap;
  }
  .play {
    font: inherit;
    min-width: 84px;
    min-height: 36px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ink);
    color: var(--sse-ground);
    cursor: pointer;
  }
  .scrub {
    position: relative;
    flex: 1;
    min-width: 200px;
  }
  .scrub input {
    width: 100%;
  }
  .marks {
    position: relative;
    height: 8px;
    margin: 0 8px;
  }
  .mark {
    position: absolute;
    top: 0;
    width: 1px;
    height: 6px;
    background: var(--sse-ink-faint);
  }
  .mark.violent {
    background: var(--sse-accent);
    height: 8px;
  }
  .stages {
    display: flex;
    gap: 2px;
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .stages button {
    font: inherit;
    font-size: 12px;
    min-width: 28px;
    min-height: 28px;
    border: var(--sse-line) solid var(--sse-ink-faint);
    background: transparent;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .stages button[aria-current] {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .honest {
    margin: 0;
    padding: 0 16px 10px;
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  @media (max-width: 700px) {
    .captions {
      max-width: calc(100% - 32px);
    }
    .captions .stage-text {
      display: none;
    }
  }
</style>
