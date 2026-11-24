<!--
  Hazard map (exotic objects design 3.8, XO18; Alex 24 Nov 2026: "a hazard map that can be brought up", for any
  hazardous body, "Jupiter radiation belts and such need to be treated carefully"). The Profile: distance from the
  body on a stretched scale, one thin row per hazard in three tones (clear, harsh, deadly), along a stated line (in
  the equator's plane, or over the poles), with the body's fixed marks under it and the ship's two marks over it:
  Park, the closest orbit it can settle into and still climb out of, and Dive, the closest a fall-through can pass
  and still come back. Inside Dive is "too late". Every number is the kernel's (`hazard_map`); danger is drama,
  never forbidden, so nothing here stops a ship. The cut through the body (the Map) comes with the Mag layer's tracer.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { DEFAULT_VIEW, viewOptions, type Json, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { Card } from "../../../chart-grammar/src/index.ts";
  import { HAZARD_CONTROLS } from "./controls.ts";
  import { distanceWords, labelled, radiusTicks, xOf, type HazardMapRead, type Tone } from "./profile.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  const ws = $derived(ctx.workspace);
  const hud = $derived(viewOptions(HAZARD_CONTROLS, ws.view ?? DEFAULT_VIEW).shown.hud);
  const isSol = $derived(/\/sys:sol$/.test(scene.system));
  // The body in view: the selection, else the system's star.
  const subject = $derived(scene.subjects.find((s) => s.address === ws.selection) ?? scene.subjects.find((s) => s.kind === "star"));
  // The line starts from the setting; pressing the switch changes it for this visit.
  let line = $state<"equator" | "poles">(untrack(() => params.line) === "poles" ? "poles" : "equator");
  let map = $state.raw<HazardMapRead | null>(null);
  let problem = $state("");
  let loading = $state(false);

  $effect(() => {
    const s = subject;
    const l = line;
    map = null;
    problem = "";
    if (!s || !isSol) return;
    let live = true;
    loading = true;
    void ctx.session.hazardMap({ fixture: "solar-system", body: s.label.toLowerCase(), line: l }).then((r) => {
      if (!live) return;
      loading = false;
      if (r.ok) map = (r.maps[0] as unknown as HazardMapRead) ?? null;
      else problem = r.message;
    });
    return () => {
      live = false;
    };
  });

  const W = 760;
  const X0 = 150;
  const X1 = W - 24;
  const ROW = 18;
  const TOP = 40;
  const rows = $derived(map?.rows ?? []);
  const base = $derived(TOP + rows.length * (ROW + 6) + 6);
  const H = $derived(TOP + rows.length * (ROW + 6) + 70);
  const x = (d: number) => (map ? xOf(map, d, X0, X1) : X0);
  const radius = $derived(map?.marks.find((m) => m.what === "surface")?.distance_m ?? map?.from_m ?? 1);
  const TONE: Record<Tone, string> = {
    clear: "transparent",
    harsh: "color-mix(in srgb, var(--sse-alert, #d98a00) 45%, transparent)",
    deadly: "var(--sse-danger, #c0392b)",
  };
  const ticks = $derived(map ? radiusTicks(map) : []);
  const fixed = $derived(map ? labelled(map.marks.map((m) => ({ ...m, x: x(m.distance_m) }))) : []);
  const shipMarks = $derived(
    map
      ? [
          ...(map.park ? [{ key: "park", name: "Park", ...map.park }] : []),
          ...(map.dive ? [{ key: "dive", name: "Dive", ...map.dive }] : []),
        ].map((m) => ({ ...m, x: x(m.distance_m) }))
      : [],
  );
  const word = (h: string) => ({ heat: "heat", radiation_belts: "radiation from the belts", air: "air", climbing_out: "fuel to climb out", surface: "surface" })[h] ?? h;
  const moonName = (a: string) => {
    const n = a.split("/").pop() ?? a;
    return n.charAt(0).toUpperCase() + n.slice(1);
  };
</script>

<div class="wrap" data-hazard-map={map ? map.address : loading ? "loading" : "none"} data-line={line}>
  {#if !subject}
    <p class="note">Pick a body to read the dangers round it.</p>
  {:else if !isSol}
    <p class="note">The Hazard map reads the Solar System's measured bodies for now. Other systems' worlds join when their fields are published to the map.</p>
  {:else if problem}
    <p class="note" data-hazard-problem>{problem}</p>
  {:else if map}
    <Card title="{subject.label}: hazard map" subtitle="How close can a ship go, and which hazard stops it">
      <div class="lines" role="group" aria-label="Read along">
        <button class:on={line === "equator"} aria-pressed={line === "equator"} data-line-pick="equator" onclick={() => (line = "equator")}>In the equator's plane</button>
        <button class:on={line === "poles"} aria-pressed={line === "poles"} data-line-pick="poles" onclick={() => (line = "poles")}>Over the poles</button>
      </div>
      <figure>
        <svg viewBox="0 0 {W} {H}" role="img" aria-label="Hazards round {subject.label} by distance, {line === 'poles' ? 'over the poles' : 'in the equator\'s plane'}. The lines below say the same in words.">
          {#each rows as r, i (r.hazard)}
            {@const y = TOP + i * (ROW + 6)}
            <g data-row={r.hazard}>
              <text x={X0 - 8} y={y + ROW / 2 + 4} class="row" text-anchor="end">{r.name}</text>
              <rect x={X0} y={y} width={X1 - X0} height={ROW} fill="none" stroke="var(--sse-rule, var(--sse-ink-muted))" stroke-width="0.6" />
              {#each r.bands as b, j (j)}
                {#if b.tone !== "clear"}
                  <rect x={x(b.from_m)} y={y} width={Math.max(0.8, x(b.to_m) - x(b.from_m))} height={ROW} fill={TONE[b.tone]} data-tone={b.tone}><title>{b.tone}: {distanceWords(b.from_m, radius)} to {distanceWords(b.to_m, radius)}</title></rect>
                {/if}
              {/each}
            </g>
          {/each}
          {#each map.safe_bands as sb, i (i)}
            <line x1={x(sb[0])} x2={x(sb[1])} y1={TOP - 6} y2={TOP - 6} stroke="var(--sse-accent)" stroke-width="2" opacity="0.5" data-safe-band={i} />
          {/each}
          {#each shipMarks as m (m.key)}
            <g data-ship-mark={m.key}>
              <line x1={m.x} x2={m.x} y1={TOP - 14} y2={TOP + rows.length * (ROW + 6)} stroke="var(--sse-accent)" stroke-width="2" stroke-dasharray={m.key === "dive" ? "4 3" : undefined} />
              {#if hud}<text x={m.x + (m.key === "dive" ? -4 : 4)} y={TOP - 18} class="ship" text-anchor={m.key === "dive" ? "end" : "start"}>{m.name}: {distanceWords(m.distance_m, radius)}</text>{/if}
            </g>
          {/each}
          <line x1={X0} x2={X1} y1={base} y2={base} stroke="var(--sse-ink)" stroke-width="0.8" />
          {#each ticks as t (t)}
            <line x1={x(t)} x2={x(t)} y1={base} y2={base + 4} stroke="var(--sse-ink)" />
            {#if hud}<text x={x(t)} y={base + 14} class="tick" text-anchor="middle">{distanceWords(t, radius)}</text>{/if}
          {/each}
          {#each fixed as f (f.what)}
            <g data-mark={f.what}>
              <circle cx={f.x} cy={base + 26} r="2.6" fill="var(--sse-ink)" />
              {#if hud && f.show}<text x={f.x} y={base + 40} class="mark" text-anchor="middle">{f.what}</text>{/if}
              <title>{f.what}: {distanceWords(f.distance_m, radius)}</title>
            </g>
          {/each}
        </svg>
      </figure>
      <ul class="legend" aria-label="Tones">
        <li><span class="sw clear"></span>clear</li>
        <li><span class="sw harsh"></span>harsh</li>
        <li><span class="sw deadly"></span>deadly</li>
        <li><span class="sw park"></span>Park and Dive for {map.stock_ship ? "the stock ship" : map.ship.name} ({(map.ship.delta_v_m_s / 1e3).toFixed(1)} km/s, {(map.ship.shield_kg_m2 / 10).toFixed(0)} g/cm² shield)</li>
      </ul>
      <div class="words">
        {#if map.park}<p data-says="park"><b>Park</b> {distanceWords(map.park.distance_m, radius)}, set by the {word(map.park.set_by)}. {map.park.says}</p>{/if}
        {#if map.dive}<p data-says="dive"><b>Dive</b> {distanceWords(map.dive.distance_m, radius)}, set by the {word(map.dive.set_by)}. {map.dive.says} Inside it is too late.</p>{/if}
        {#if map.pass}<p data-says="pass">That pass: {map.pass.dose_sv.toFixed(2)} Sv soaked up{map.pass.peak_hull_k > 0 ? `, hull peaks at ${map.pass.peak_hull_k.toFixed(0)} K` : ""}{map.pass.hours_in_belts > 0 ? `, ${map.pass.hours_in_belts.toFixed(1)} hours inside the belts` : ""}.</p>{/if}
        {#each map.rows as r (r.hazard)}<p class="say" data-says={r.hazard}>{r.name}: {r.says}.</p>{/each}
        {#if map.quiet.length}<p class="say" data-says="quiet">{map.quiet.join(", ")}: nothing to worry about here.</p>{/if}
        {#if map.moons.length}
          <table class="moons" data-moons>
            <thead><tr><th>Beside</th><th>Dose a day</th><th></th></tr></thead>
            <tbody>
              {#each map.moons as mo (mo.moon)}
                <tr data-moon={mo.moon} data-tone={mo.tone}><td>{moonName(mo.moon)}</td><td>{mo.sv_per_day < 0.01 ? mo.sv_per_day.toExponential(1) : mo.sv_per_day.toFixed(2)} Sv</td><td class="t-{mo.tone}">{mo.tone}</td></tr>
              {/each}
            </tbody>
          </table>
        {/if}
        {#if map.stand_ins.length}
          <details class="limits">
            <summary>What this leaves out</summary>
            <ul>{#each map.stand_ins as s (s)}<li>{s}</li>{/each}</ul>
          </details>
        {/if}
      </div>
    </Card>
  {:else if loading}
    <p class="note">Reading the dangers round {subject.label}…</p>
  {/if}
</div>

<style>
  .wrap {
    padding: 12px;
    color: var(--sse-ink);
    font-family: var(--sse-font-body, inherit);
    max-width: 820px;
  }
  .note {
    color: var(--sse-ink-muted);
  }
  .lines {
    display: flex;
    gap: 6px;
    margin: 4px 0 8px;
    flex-wrap: wrap;
  }
  .lines button {
    border: 1px solid var(--sse-ink-muted);
    background: transparent;
    color: var(--sse-ink);
    border-radius: 999px;
    padding: 3px 10px;
    cursor: pointer;
    font: inherit;
  }
  .lines button.on {
    background: var(--sse-accent);
    color: var(--sse-accent-ink, var(--sse-ground));
    border-color: var(--sse-accent);
  }
  figure {
    margin: 0;
  }
  svg {
    width: 100%;
    height: auto;
    overflow: visible;
  }
  .row,
  .tick,
  .mark,
  .ship {
    font-size: 11px;
    fill: var(--sse-ink);
  }
  .tick,
  .mark {
    fill: var(--sse-ink-muted);
    font-size: 10px;
  }
  .ship {
    fill: var(--sse-accent);
    font-weight: 600;
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    list-style: none;
    padding: 0;
    margin: 6px 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .sw {
    display: inline-block;
    width: 14px;
    height: 10px;
    margin-right: 4px;
    vertical-align: middle;
    border: 1px solid var(--sse-ink-muted);
  }
  .sw.harsh {
    background: color-mix(in srgb, var(--sse-alert, #d98a00) 45%, transparent);
  }
  .sw.deadly {
    background: var(--sse-danger, #c0392b);
  }
  .sw.park {
    background: var(--sse-accent);
    border-color: var(--sse-accent);
  }
  .words p {
    margin: 4px 0;
  }
  .say {
    color: var(--sse-ink-muted);
    font-size: 13px;
  }
  .moons {
    border-collapse: collapse;
    margin: 6px 0;
    font-size: 13px;
  }
  .moons th,
  .moons td {
    padding: 2px 10px 2px 0;
    text-align: left;
  }
  .t-deadly {
    color: var(--sse-danger, #c0392b);
    font-weight: 600;
  }
  .t-harsh {
    color: var(--sse-alert, #d98a00);
  }
  .limits {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
</style>
