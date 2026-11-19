<!--
  The Window (V8). Pick where the ship is (beside a world), what to look at
  and which sensor; the kernel's window_view says what that sensor picks up
  and the camera points at it. Only what the sensor sees is drawn: points
  for what it can't resolve, lit discs (with their phase) for what it can.
  Presentation only.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { SENSORS, YEAR_S, angleWords, camera, discPx, lightWords, magWords, navFromScene, pointPx, project, towards, type Json, type Seen, type SystemScene, type ViewContext, type WindowView } from "../../../ui-contract/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const first = untrack(() => ({ sensor: String(params.sensor ?? "eye"), missed: params.show_missed === true }));
  const YEAR = Number(YEAR_S);

  const fromScene = $derived(navFromScene(scene));
  const bodies = $derived(fromScene.bodies);
  let fromAddr = $state("");
  let target = $state("star");
  let sensor = $state(SENSORS.some((s) => s.id === first.sensor) ? first.sensor : "eye");
  // Where the camera points, nudged from the target.
  let dLon = $state(0);
  let dLat = $state(0);
  let zoom = $state(1);
  let showMissed = $state(first.missed);

  // From the first world in the temperate zone, as Route does.
  $effect(() => {
    const bs = bodies;
    if (!bs.length) return;
    untrack(() => {
      if (bs.some((b) => b.address === fromAddr)) return;
      const zone = scene.subjects.find((s) => s.kind === "zone" && s.band_au);
      const i = zone
        ? bs.findIndex((b) => {
            const a = scene.subjects.find((s) => s.address === b.address)?.orbit?.a_au ?? 0;
            return a >= zone.band_au![0] && a <= zone.band_au![1];
          })
        : -1;
      fromAddr = bs[i >= 0 ? i : 0]!.address;
    });
  });

  let view = $state<WindowView | null>(null);
  let failed = $state<string | null>(null);
  let gen = 0;
  const now_s = $derived.by(() => {
    const c = (ctx.workspace as { campaign_now_yr?: number }).campaign_now_yr;
    return typeof c === "number" && Number.isFinite(c) ? c * YEAR : (fromScene.nav?.t0_s ?? 0);
  });
  $effect(() => {
    const nav = fromScene.nav;
    const from = fromAddr;
    const s = sensor;
    const t = now_s;
    if (!nav || !from) return;
    const g = ++gen;
    ctx.session.windowView({ nav, from, sensor: s, at_s: t }).then((r) => {
      if (g !== gen) return;
      if (!r.ok) return void ((failed = r.message), (view = null));
      failed = null;
      view = r.view;
    });
  });
  function choose(t: string) {
    target = t;
    dLon = 0;
    dLat = 0;
  }

  const aim = $derived(view?.seen.find((s) => s.address === target) ?? view?.seen[0] ?? null);
  const field = $derived(view ? view.sensor.field_deg / zoom : 60);
  const SIZE = 560;
  const cam = $derived(aim ? camera(aim.lon_deg + dLon * field * 0.25, aim.lat_deg + dLat * field * 0.25, field, SIZE) : null);
  const seenNow = $derived(view ? view.seen.filter((s) => s.detected) : []);
  const missedNow = $derived(view ? view.seen.filter((s) => !s.detected) : []);
  const inFrame = $derived(cam ? seenNow.filter((s) => project(cam, s.lon_deg, s.lat_deg)) : []);
  const nameOf = (addr: string) => bodies.find((b) => b.address === addr)?.label ?? addr;

  // Drawing in the theme's tokens: a star chart, the sky in ground colour and the lights in ink.
  let canvas = $state<HTMLCanvasElement | null>(null);
  let tick = $state(0);
  $effect(() => {
    const o = new MutationObserver(() => (tick += 1));
    o.observe(document.documentElement, { attributes: true, attributeFilter: ["style"] });
    return () => o.disconnect();
  });
  function token(el: Element, name: string): string {
    return getComputedStyle(el).getPropertyValue(name).trim() || "currentColor";
  }
  function lit(g: CanvasRenderingContext2D, x: number, y: number, r: number, angle: number, phase_deg: number) {
    const a = (phase_deg * Math.PI) / 180;
    g.save();
    g.translate(x, y);
    g.rotate(angle);
    g.beginPath();
    g.arc(0, 0, r, -Math.PI / 2, Math.PI / 2);
    g.ellipse(0, 0, Math.max(0.01, r * Math.abs(Math.cos(a))), r, 0, Math.PI / 2, -Math.PI / 2, a > Math.PI / 2);
    g.closePath();
    g.fill();
    g.restore();
  }
  $effect(() => {
    void tick;
    const cv = canvas;
    const c = cam;
    const v = view;
    if (!cv || !c || !v) return;
    const g = cv.getContext("2d");
    if (!g) return;
    const ink = token(cv, "--sse-ink");
    const ground = token(cv, "--sse-ground");
    const muted = token(cv, "--sse-ink-muted");
    g.fillStyle = ground;
    g.fillRect(0, 0, SIZE, SIZE);
    // The ecliptic, faint, where it crosses the picture.
    g.strokeStyle = muted;
    g.globalAlpha = 0.5;
    g.setLineDash([4, 6]);
    g.beginPath();
    let on = false;
    for (let l = 0; l <= 360; l += 0.25) {
      const p = project(c, l, 0);
      if (p) {
        if (on) g.lineTo(p.x, p.y);
        else g.moveTo(p.x, p.y);
        on = true;
      } else on = false;
    }
    g.stroke();
    g.setLineDash([]);
    g.globalAlpha = 1;
    const star = v.seen.find((s) => s.kind === "star")!;
    // Labels that would land on one another are left to the table.
    const labels: { x: number; y: number; w: number }[] = [];
    const named: { text: string; x: number; y: number; mag: number }[] = [];
    g.font = `12px ${token(cv, "--sse-font-body")}`;
    // Faintest first, so bright lights sit on top.
    for (const s of [...v.seen].filter((s) => s.detected).sort((a, b) => b.magnitude - a.magnitude)) {
      const p = project(c, s.lon_deg, s.lat_deg, 40);
      if (!p) continue;
      const rd = discPx(c, s.angular_diameter_deg);
      g.fillStyle = ink;
      if (s.resolved && rd >= 1.5) {
        if (s.kind === "star") {
          g.globalAlpha = 0.18;
          g.beginPath();
          g.arc(p.x, p.y, Math.max(rd * 2.2, 16), 0, Math.PI * 2);
          g.fill();
          g.globalAlpha = 1;
          g.beginPath();
          g.arc(p.x, p.y, rd, 0, Math.PI * 2);
          g.fill();
        } else {
          // The night side, faint; the lit side toward the star.
          g.globalAlpha = 0.12;
          g.beginPath();
          g.arc(p.x, p.y, rd, 0, Math.PI * 2);
          g.fill();
          g.globalAlpha = 1;
          lit(g, p.x, p.y, rd, towards(c, p, star.lon_deg, star.lat_deg), s.phase_deg);
        }
      } else {
        const r = s.kind === "star" ? 6 : pointPx(s.magnitude, v.sensor.limit_mag);
        g.beginPath();
        g.arc(p.x, p.y, r, 0, Math.PI * 2);
        g.fill();
        if (s.kind === "star") {
          g.globalAlpha = 0.25;
          g.beginPath();
          g.arc(p.x, p.y, 16, 0, Math.PI * 2);
          g.fill();
          g.globalAlpha = 1;
        }
      }
      named.push({ text: s.kind === "star" ? "the star" : s.name, x: p.x + Math.max(rd, 6) + 4, y: p.y - 4, mag: s.magnitude });
    }
    // Brightest labelled first; one that would land on another is left to the table.
    g.fillStyle = muted;
    for (const n of named.sort((a, b) => a.mag - b.mag)) {
      const w = g.measureText(n.text).width;
      if (labels.some((l) => n.x < l.x + l.w + 4 && l.x < n.x + w + 4 && Math.abs(l.y - n.y) < 14)) continue;
      labels.push({ x: n.x, y: n.y, w });
      g.fillText(n.text, n.x, n.y);
    }
  });

  const status = (s: Seen) => (!s.detected ? "too faint" : s.resolved ? "a disc" : "a point of light");
</script>

<div class="win" data-window={view ? `${view.sensor_id}:${view.beside ?? ""}` : ""}>
  <div class="bar">
    <label>From beside
      <select bind:value={fromAddr} data-from>
        {#each bodies as b (b.address)}<option value={b.address}>{b.label}</option>{/each}
      </select>
    </label>
    <label>Look at
      <select value={target} onchange={(e) => choose(e.currentTarget.value)} data-target>
        <option value="star">the star</option>
        {#each bodies.filter((b) => b.address !== fromAddr) as b (b.address)}<option value={b.address}>{b.label}</option>{/each}
      </select>
    </label>
    <span class="seg" role="group" aria-label="Sensor">
      {#each SENSORS as s (s.id)}<button type="button" class:on={sensor === s.id} aria-pressed={sensor === s.id} data-sensor={s.id} onclick={() => (sensor = s.id)}>{s.label}</button>{/each}
    </span>
  </div>

  <header class="head">
    <h2>The window</h2>
    <span class="stamp">only what the sensor sees</span>
    {#if view && aim}
      <p class="lede" data-lede>
        From beside {nameOf(fromAddr)}, through {view.sensor.name.toLowerCase()}, looking toward {aim.kind === "star" ? "the star" : aim.name}{dLon || dLat ? " (turned a little)" : ""}: {inFrame.length} {inFrame.length === 1 ? "light" : "lights"} in a {angleWords(field)} field. {aim.detected ? `${aim.kind === "star" ? "The star" : aim.name} is ${status(aim)}, ${magWords(aim.magnitude)}.` : `${aim.name} is too faint for this sensor; it is not drawn.`}
      </p>
    {:else if failed}
      <p class="lede" role="alert">{failed}</p>
    {:else if !fromScene.nav}
      <p class="lede">{fromScene.notes[0] ?? "This chart has no star to look from."}</p>
    {:else}
      <p class="lede">Opening the window…</p>
    {/if}
  </header>

  <div class="split">
    <div class="pane">
      <canvas bind:this={canvas} width={SIZE} height={SIZE} data-sky aria-label="The view from the window"></canvas>
      <div class="nudge" role="group" aria-label="Turn and zoom">
        <button type="button" onclick={() => (dLon += 1)} aria-label="Turn left">←</button>
        <button type="button" onclick={() => (dLat += 1)} aria-label="Turn up">↑</button>
        <button type="button" onclick={() => (dLat -= 1)} aria-label="Turn down">↓</button>
        <button type="button" onclick={() => (dLon -= 1)} aria-label="Turn right">→</button>
        <button type="button" onclick={() => (zoom = Math.min(64, zoom * 2))} data-zoom-in aria-label="Zoom in">+</button>
        <button type="button" onclick={() => (zoom = Math.max(1, zoom / 2))} aria-label="Zoom out">−</button>
        <span class="hint">{angleWords(field)} across · ecliptic dashed · north up</span>
      </div>
    </div>
    <aside class="ledger">
      <h3>Seen through {view?.sensor.name.toLowerCase() ?? "the sensor"}</h3>
      {#if view}
        <p class="hint">{view.sensor.why}.</p>
        <table>
          <thead><tr><th>Object</th><th>Looks like</th><th>Size</th><th>Brightness</th><th>Light takes</th></tr></thead>
          <tbody>
            {#each seenNow as s (s.address)}
              <tr data-seen={s.address}>
                <td><button type="button" class="link" onclick={() => choose(s.address)}>{s.kind === "star" ? "the star" : s.name}</button></td>
                <td>{status(s)}{s.kind !== "star" && s.resolved ? `, ${Math.round(s.lit_fraction * 100)}% lit` : ""}</td>
                <td class="v">{angleWords(s.angular_diameter_deg)}</td>
                <td class="v" title={magWords(s.magnitude)}>mag {s.magnitude.toFixed(1)}</td>
                <td class="v">{lightWords(s.light_s)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        {#if missedNow.length}
          <p class="gm">
            <button type="button" class="link" onclick={() => (showMissed = !showMissed)} data-missed-toggle>{showMissed ? "Hide" : "Show"} what it misses ({missedNow.length})</button>
            <span class="hint">GM only: the crew does not know these are there.</span>
          </p>
          {#if showMissed}
            <ul class="missed">{#each missedNow as s (s.address)}<li data-missed={s.address}>{s.name}: mag {s.magnitude.toFixed(1)}, below this sensor's {view.sensor.limit_mag}</li>{/each}</ul>
          {/if}
        {/if}
        <h4>The model's limits</h4>
        <ul>{#each [...view.model, ...fromScene.notes] as m (m)}<li>{m}</li>{/each}</ul>
      {/if}
    </aside>
  </div>
</div>

<style>
  .win {
    color: var(--sse-ink);
    font-size: 13px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 1180px;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: center;
  }
  label {
    display: inline-flex;
    gap: 6px;
    align-items: center;
  }
  select {
    font: inherit;
    min-height: 34px;
    background: var(--sse-ground);
    color: var(--sse-ink);
    border: 1px solid var(--sse-rule);
  }
  .seg {
    display: inline-flex;
    flex-wrap: wrap;
  }
  .seg button,
  .nudge button {
    font: inherit;
    min-height: 34px;
    min-width: 34px;
    padding: 0 12px;
    border: 1px solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    margin-right: -1px;
    cursor: pointer;
  }
  .seg button.on {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 12px;
    align-items: baseline;
  }
  h2 {
    font-family: var(--sse-font-display, serif);
    font-size: 24px;
    font-weight: 600;
    margin: 0;
  }
  h3 {
    font-family: var(--sse-font-display, serif);
    font-size: 18px;
    font-weight: 600;
    margin: 0 0 6px;
  }
  h4 {
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    margin: 12px 0 4px;
    font-weight: 500;
  }
  .lede {
    margin: 0;
    flex-basis: 100%;
    font-size: 14px;
  }
  .stamp {
    border: 1.5px solid var(--sse-accent);
    color: var(--sse-accent-ink);
    font-size: 10px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    padding: 2px 8px;
  }
  .split {
    display: grid;
    grid-template-columns: minmax(0, 560px) minmax(260px, 1fr);
    gap: 20px;
    align-items: start;
  }
  .pane {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  canvas {
    width: 100%;
    height: auto;
    aspect-ratio: 1;
    border: 1px solid var(--sse-ink);
    display: block;
  }
  .nudge {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }
  .ledger {
    border: 1px solid var(--sse-ink);
    padding: 14px 18px;
    background: var(--sse-paper, var(--sse-ground));
    min-width: 0;
    overflow-x: auto;
  }
  table {
    border-collapse: collapse;
    width: 100%;
  }
  th {
    text-align: left;
    font-weight: 500;
    color: var(--sse-ink-muted);
    font-size: 11px;
    border-bottom: 1px solid var(--sse-rule);
    padding: 4px 6px 4px 0;
  }
  td {
    padding: 5px 6px 5px 0;
    border-bottom: 1px dotted var(--sse-rule);
  }
  .v {
    font-family: var(--sse-font-mono, monospace);
    white-space: nowrap;
  }
  .link {
    font: inherit;
    background: none;
    border: none;
    padding: 0;
    color: var(--sse-ink);
    text-decoration: underline;
    cursor: pointer;
  }
  .hint {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  p {
    margin: 4px 0;
  }
  ul {
    margin: 0;
    padding-left: 18px;
  }
  .gm {
    border-left: 3px solid var(--sse-accent);
    padding-left: 8px;
  }
  @media (max-width: 760px) {
    .split {
      grid-template-columns: 1fr;
    }
  }
</style>
