<!--
  The galaxy chart (U6). One galaxy from the kernel's galaxy_chart: face-on,
  rings shaded by a layer, arm crests and the green band at the chosen time,
  history curves on cosmic time with the epochs and active-nucleus episodes,
  and the ledger of a picked ring (what died there, conditions now, whether
  it is green and why). Presentation only.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { announcePlace, cellHalo, GROUP_WORDS, LAYER_WORDS, MORPHOLOGY_WORDS, NAMED_HALOES, armPath, cosmicAxis, dex, editWords, toggleRemoval, discFrame, epochTicks, greenAt, logCurve, ringAt, ringShades, sci, timesSun, type GalaxyChart, type GalaxyEdit, type Group, type Json, type RingLayer, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";

  let { ctx, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const first = untrack(() => ({ halo: String(params.halo ?? "milky-way-like"), layer: String(params.layer ?? "metals") }));
  // A named halo, or a catalogued halo's address picked on the cluster map (V9).
  type HaloKey = string;
  // A link's place (U24) opens first, then a galaxy picked on the cluster map, then the screen's own choice.
  const named = (k: unknown): k is string => typeof k === "string" && NAMED_HALOES.some((h) => h.key === k);
  const startHalo = untrack(() => {
    const p = params.place;
    if (typeof p === "string" && (cellHalo(p) || named(p))) return p;
    const sel = ctx.workspace.selection;
    return cellHalo(sel) ? sel! : named(first.halo) ? first.halo : "milky-way-like";
  });
  let halo = $state<HaloKey>(startHalo);
  const catalogued = $derived(cellHalo(halo));
  // Picking a galaxy on the cluster map charts it here.
  $effect(() => {
    const sel = ctx.workspace.selection;
    if (cellHalo(sel)) untrack(() => chooseHalo(sel!));
  });
  let layer = $state<RingLayer>((["metals", "heavy", "stars", "supernovae"].includes(first.layer) ? first.layer : "metals") as RingLayer);
  let at = $state<number | null>(null);
  let picked = $state<number | null>(null);
  let chart = $state<GalaxyChart | null>(null);
  let failed = $state<string | null>(null);
  let busy = $state(false);
  let gen = 0;
  // Inked history (G6): previewed by the kernel, kept in this browser per galaxy.
  const EDIT_KEY = (h: string) => `sse.galaxy.edits.${h}`;
  function loadEdits(h: string): GalaxyEdit[] {
    try {
      const v = JSON.parse(localStorage.getItem(EDIT_KEY(h)) ?? "[]");
      return Array.isArray(v) ? (v as GalaxyEdit[]) : [];
    } catch {
      return [];
    }
  }
  function keepEdits(h: string, e: GalaxyEdit[]) {
    try {
      localStorage.setItem(EDIT_KEY(h), JSON.stringify(e));
    } catch {
      /* private window: the edits last for this visit */
    }
  }
  let edits = $state<GalaxyEdit[]>(untrack(() => loadEdits(startHalo)));
  function setEdits(e: GalaxyEdit[]) {
    edits = e;
    keepEdits(halo, e);
  }
  // The ink form.
  let inkKind = $state<"merger" | "starburst" | "quiet_nucleus">("merger");
  let inkT = $state(8);
  let inkTo = $state(10);
  let inkRatio = $state(0.33);
  let inkGas = $state(0.3);
  function ink() {
    const t = Math.round(inkT * 10) / 10;
    const e: GalaxyEdit = inkKind === "merger" ? { kind: "merger", t_gyr: t, ratio: inkRatio } : inkKind === "starburst" ? { kind: "starburst", t_gyr: t, gas_fraction: inkGas } : { kind: "quiet_nucleus", from_gyr: t, to_gyr: Math.max(t + 0.1, Math.round(inkTo * 10) / 10) };
    setEdits([...edits, e]);
  }

  // One call returns the galaxy, its rings and the conditions at the picked ring.
  $effect(() => {
    const h = halo;
    const t = at;
    const p = picked;
    const ed = edits;
    const r = untrack(() => (p !== null && chart && (chart.key === h || chart.halo === h) ? chart.rings[p]?.radius_kpc : undefined));
    const g = ++gen;
    busy = true;
    const ch = cellHalo(h);
    ctx.session.galaxyChart({ ...(ch ? { cell: ch.cell, halo: ch.halo } : { halo: h }), ...(t !== null ? { at_gyr: t } : {}), ...(r !== undefined ? { place_kpc: r } : {}), ...(ed.length ? { edits: ed } : {}) }).then((res) => {
      if (g !== gen) return;
      busy = false;
      if (!res.ok) return void ((failed = res.message), (chart = null));
      failed = null;
      chart = res.chart;
      if (picked === null) picked = ringAt(res.chart.rings, res.chart.sun_radius_kpc);
    });
  });
  function chooseHalo(h: HaloKey) {
    if (h === halo) return;
    picked = null;
    at = null;
    edits = loadEdits(h);
    halo = h;
  }

  // The place line: this galaxy is the place in view. Nothing is placed in it below yet (systems join with GH).
  let root = $state<HTMLDivElement | null>(null);
  const galaxyName = (h: string) => NAMED_HALOES.find((n) => n.key === h)?.label ?? (cellHalo(h) ? `Halo ${cellHalo(h)!.key}, cell ${cellHalo(h)!.cell.join(".")}` : h);
  $effect(() => {
    const h = halo;
    if (root) announcePlace(root, { scale: "galaxy", id: h, name: galaxyName(h) });
  });

  const W = 520;
  const f = $derived(chart ? discFrame(W, W, chart) : null);
  const shades = $derived(chart ? ringShades(chart.rings, layer) : []);
  const tNow = $derived(chart ? chart.at_gyr : 13.8);
  const green = $derived(chart ? greenAt(chart, tNow) : null);
  const ring = $derived(chart && picked !== null ? chart.rings[picked] : null);
  // The galaxy as it was at the chosen time (the history point at or before it).
  const then = $derived.by(() => {
    if (!chart) return null;
    let s = chart.history[0] ?? chart.now;
    for (const h of chart.history) if (h.t_gyr <= chart.at_gyr + 1e-9) s = h;
    return chart.at_gyr >= chart.now_gyr - 1e-6 ? chart.now : s;
  });
  const sunRing = $derived(chart ? ringAt(chart.rings, chart.sun_radius_kpc) : -1);

  // History curves.
  const HW = 880;
  const HH = 150;
  const ax = $derived(chart ? cosmicAxis(0, chart.now_gyr, HW, 10) : null);
  const curves = $derived.by(() => {
    if (!chart || !ax) return [];
    const series: { id: string; label: string; v: (s: GalaxyChart["now"]) => number }[] = [
      { id: "stars", label: "stars", v: (s) => s.stars },
      { id: "gas", label: "gas", v: (s) => s.gas },
      { id: "sfr", label: "star formation", v: (s) => s.sfr },
    ];
    const c = chart;
    return series.map((s) => {
      const cur = logCurve(c.history.map((h) => ({ t: h.t_gyr, v: s.v(h) })), ax, 22, HH - 44);
      // The galaxy as grown, before the inked edits: a ghost on the same scale.
      const was = c.compare?.curves.find((k) => k.name === s.id);
      const ghost = was ? logCurve(was.t_gyr.map((t, i) => ({ t, v: was.before[i]! })), ax, 22, HH - 44, cur).d : "";
      return { ...s, ...cur, ghost };
    });
  });
  const ticks = $derived(chart && ax ? epochTicks(chart.epochs, ax).filter((t, i, a) => i === 0 || t.x - a[i - 1]!.x > 40) : []);
  const reached = $derived.by(() => {
    if (!chart || !ring) return null;
    const b = chart.green_bands.find((g) => g.habitable && ring.radius_kpc >= g.habitable[0] && ring.radius_kpc <= g.habitable[1]);
    return b ? b.t_gyr : null;
  });
  const gyr = (t: number) => `${t.toFixed(t < 1 ? 2 : 1)} Gyr`;
  const CRITERIA: { key: keyof NonNullable<GalaxyChart["place"]>["green"]; words: string }[] = [
    { key: "metals_enough", words: "enough metals for rocky worlds" },
    { key: "metals_not_too_many", words: "not so metal-rich that migrating giants dominate" },
    { key: "few_supernovae", words: "few sterilising supernovae nearby" },
    { key: "few_passages", words: "few close stellar passes" },
    { key: "few_bursts", words: "few gamma-ray bursts" },
    { key: "nucleus_quiet", words: "the central black hole quiet enough" },
    { key: "quiet_long_enough", words: "quiet long enough for complex life" },
  ];
</script>

<div class="gal" data-galaxy={chart?.halo ?? ""} bind:this={root}>
  <div class="bar">
    <label>Galaxy
      <select value={halo} onchange={(e) => chooseHalo(e.currentTarget.value as HaloKey)} data-halo>
        {#each NAMED_HALOES as h (h.key)}<option value={h.key}>{h.label}</option>{/each}
        {#if catalogued}<option value={halo}>Halo {catalogued.key} of cell {catalogued.cell.join(".")} (cluster map)</option>{/if}
      </select>
    </label>
    <div class="seg" role="group" aria-label="Shade the rings by">
      {#each Object.entries(LAYER_WORDS) as [id, w] (id)}<button type="button" class:on={layer === id} aria-pressed={layer === id} onclick={() => (layer = id as RingLayer)} data-layer={id}>{w.label}</button>{/each}
    </div>
    {#if busy}<span class="hint">growing…</span>{/if}
  </div>

  {#if failed}
    <p class="note" role="alert">Could not chart the galaxy: {failed}</p>
  {:else if chart && f}
    <header class="head">
      <h2>{galaxyName(halo)}</h2>
      <span class="stamp">grown, not prebaked</span>
      {#if chart.edits.length}<span class="stamp authored" data-inked>history inked by the GM</span>{/if}
      <p class="lede">
        {#if then && chart.at_gyr < chart.now_gyr - 1e-6}
          At {gyr(chart.at_gyr)} after the Big Bang: {sci(then.stars)} solar masses of stars, forming {sci(then.sfr)} a year, with {sci(then.gas)} of gas. Today it is a {MORPHOLOGY_WORDS[chart.morphology] ?? chart.morphology}.
        {:else}
          A {MORPHOLOGY_WORDS[chart.morphology] ?? chart.morphology} of {sci(chart.now.stars)} solar masses of stars, forming {sci(chart.now.sfr)} a year;
          {sci(chart.core_collapse_per_century)} core-collapse and {sci(chart.ia_per_century)} white-dwarf supernovae a century. Shown today, {gyr(chart.at_gyr)} after the Big Bang.
        {/if}
      </p>
      {#if chart.compare}
        <p class="lede" data-compare>
          {#if chart.compare.first_change_gyr !== null}
            The inked history changes this galaxy from {gyr(chart.compare.first_change_gyr)} on; everything before is as grown.
            {chart.compare.morphology[0] !== chart.compare.morphology[1] ? `It was a ${MORPHOLOGY_WORDS[chart.compare.morphology[0]] ?? chart.compare.morphology[0]}; now it is a ${MORPHOLOGY_WORDS[chart.compare.morphology[1]] ?? chart.compare.morphology[1]}.` : `It is still a ${MORPHOLOGY_WORDS[chart.morphology] ?? chart.morphology}.`}
            Stars today {sci(chart.compare.curves.find((c) => c.name === "stars")?.before.at(-1) ?? 0)} → {sci(chart.now.stars)} M☉.
          {:else}
            The inked edits change nothing in this galaxy's history.
          {/if}
        </p>
      {/if}
    </header>

    <div class="cols">
      <div class="disc">
        <svg viewBox="0 0 {W} {W}" role="img" aria-label="The galaxy face-on at {gyr(chart.at_gyr)}: {chart.rings.length} rings shaded by {LAYER_WORDS[layer].label.toLowerCase()}, {chart.arms.length} arms, the green band{green?.habitable ? ` from ${green.habitable[0].toFixed(1)} to ${green.habitable[1].toFixed(1)} kpc` : ' absent'}">
          {#each chart.rings as g, i (i)}
            <circle class="ring" cx={f.cx} cy={f.cy} r={((g.inner_kpc + g.outer_kpc) / 2) * f.k} stroke-width={Math.max(1, (g.outer_kpc - g.inner_kpc) * f.k)} style="stroke-opacity: {0.06 + shades[i]! * 0.6}" onclick={() => (picked = i)} role="presentation"><title>Ring {i + 1}: {g.radius_kpc.toFixed(1)} kpc</title></circle>
          {/each}
          {#if green?.habitable}
            <circle class="green" cx={f.cx} cy={f.cy} r={((green.habitable[0] + green.habitable[1]) / 2) * f.k} stroke-width={(green.habitable[1] - green.habitable[0]) * f.k} />
            {#each green.habitable as edge, i (i)}<circle class="greenedge" cx={f.cx} cy={f.cy} r={edge * f.k} />{/each}
          {/if}
          {#each chart.arms as a, i (i)}<path class="arm" d={armPath(a, f)} style="stroke-width: {1 + chart.arm_contrast * 4}" />{/each}
          {#if chart.barred}<line class="barline" x1={f.cx - chart.disc_kpc * f.k} y1={f.cy} x2={f.cx + chart.disc_kpc * f.k} y2={f.cy} />{/if}
          <circle class="sun" cx={f.cx + chart.sun_radius_kpc * f.k} cy={f.cy} r="4"><title>At the Sun's distance from the centre, scaled to this disc</title></circle>
          {#if ring}<circle class="pick" cx={f.cx} cy={f.cy} r={ring.radius_kpc * f.k} />{/if}
          <circle class="core" cx={f.cx} cy={f.cy} r="3" />
        </svg>
        <p class="hint">
          {chart.rings.length} rings out to {chart.rings.at(-1)!.outer_kpc.toFixed(0)} kpc, narrow near the centre and wider outside. {LAYER_WORDS[layer].legend}. Arms in ink; the green band{green?.habitable ? "" : " (none at this time)"} where simple life could start; the dot at the Sun's distance scaled to this disc. Pick a ring for its ledger.
        </p>
        <label class="time">Time
          <input type="range" min={Math.ceil(chart.t_start_gyr * 10) / 10} max={chart.now_gyr} step="0.1" value={chart.at_gyr} onchange={(e) => (at = Number(e.currentTarget.value))} aria-valuetext={gyr(chart.at_gyr)} data-time />
          <span class="v">{gyr(chart.at_gyr)}</span>
        </label>
      </div>

      {#if ring}
        <section class="ledger" aria-label="Ring {picked! + 1}" data-ring={picked}>
          <h3>Ring {picked! + 1}{picked === sunRing ? " · at the Sun's distance" : ""}</h3>
          <label class="ringpick">Ring
            <select value={picked} onchange={(e) => (picked = Number(e.currentTarget.value))} data-ring-pick>
              {#each chart.rings as g, i (i)}<option value={i}>{i + 1}: {g.radius_kpc.toFixed(1)} kpc{i === sunRing ? " (Sun's distance)" : ""}</option>{/each}
            </select>
          </label>
          <div class="row"><span>Distance from centre</span><span class="v">{ring.radius_kpc.toFixed(2)} kpc</span></div>
          <div class="row"><span>Iron, relative to the Sun</span><span class="v">{dex(ring.abundances.fe)}</span></div>
          {#each ["alpha", "cn", "r", "k40"] as const as gk (gk)}
            <div class="row"><span>{GROUP_WORDS[gk as Group]}</span><span class="v">{timesSun(ring.abundances[gk as Group])}</span></div>
          {/each}
          <h4>What died here</h4>
          <div class="row"><span>Massive-star deaths (core collapse)</span><span class="v">{sci(ring.deaths.core_collapse)}</span></div>
          <div class="row"><span>White-dwarf blasts (type Ia)</span><span class="v">{sci(ring.deaths.type_ia)}</span></div>
          <div class="row"><span>Neutron-star mergers</span><span class="v">{ring.deaths.neutron_star_mergers.toLocaleString("en-GB")}</span></div>
          <div class="row"><span>Gas returned by dying giants</span><span class="v">{sci(ring.deaths.agb_returned)} M☉</span></div>
          {#if chart.place}
            {@const l = chart.place.local}
            <h4>Conditions at {gyr(l.place.t_gyr)}</h4>
            <div class="row"><span>Star systems per cubic parsec</span><span class="v">{sci(l.systems_pc3)}</span></div>
            <div class="row"><span>Sterilising supernovae nearby, per Gyr</span><span class="v">{sci(l.sterilising_per_gyr)}</span></div>
            <div class="row"><span>Close stellar passes, per Myr</span><span class="v">{sci(l.encounters_per_myr)}</span></div>
            <div class="row"><span>Lethal gamma-ray bursts, per Gyr</span><span class="v">{sci(l.grb_per_gyr)}</span></div>
            <div class="row"><span>Carbon to oxygen</span><span class="v">{l.c_o.toFixed(2)}</span></div>
            <p class="verdict" data-green={chart.place.green.complex ? "complex" : chart.place.green.habitable ? "habitable" : "no"}>
              {chart.place.green.complex ? "Green: complex life has had time here." : chart.place.green.habitable ? "Green for simple life; not yet quiet long enough for complex life." : "Not green here at this time."}
            </p>
            <ul class="crit">{#each CRITERIA as c (c.key)}<li class:fail={!chart.place.green[c.key]}>{chart.place.green[c.key] ? "✓" : "✗"} {c.words}</li>{/each}</ul>
            {#if reached !== null}<p class="hint">The green band first reached this ring at {gyr(reached)}.</p>{/if}
          {/if}
        </section>
      {/if}
    </div>

    <section aria-label="History" class="hist">
      <h4>History, on cosmic time</h4>
      {#if ax}
        <svg viewBox="0 0 {HW} {HH}" class="curves" role="img" aria-label="Stars, gas and star formation over {gyr(chart.now_gyr)}">
          {#each ticks as t (t.e.id)}<line class="tick" class:spec={t.e.speculative} x1={t.x} x2={t.x} y1="14" y2={HH - 18}><title>{t.e.name}: {t.e.why}</title></line><text class="tlabel" x={t.x + 2} y="11">{t.e.name}</text>{/each}
          {#each chart.active_nuclei as [t, m], i (i)}<circle class="agn" cx={ax.x(t)} cy={HH - 22} r="3"><title>Active nucleus at {gyr(t)}: the black hole gained {sci(m)} M☉</title></circle>{/each}
          <line class="nowline" x1={ax.x(chart.at_gyr)} x2={ax.x(chart.at_gyr)} y1="14" y2={HH - 18} />
          {#each curves as c (c.id)}{#if c.ghost}<path class="ghost {c.id}" d={c.ghost}><title>{c.label} as grown, before the inked history</title></path>{/if}{/each}
          {#each chart.events.filter((e) => e.kind === "merger" || e.kind === "starburst") as e (e.id)}
            {@const x = ax.x(e.t_myr / 1000)}
            <path class="mark {e.kind}" class:inked={e.id.startsWith("edit/")} d="M{x} {HH - 30} l5 6 l-5 6 l-5 -6 z" data-event={e.id}><title>{e.kind === "merger" ? "Merger" : "Starburst"} at {gyr(e.t_myr / 1000)}{e.id.startsWith("edit/") ? " (inked by the GM)" : ""}</title></path>
          {/each}
          {#each curves as c (c.id)}{#if c.d}<path class="curve {c.id}" d={c.d}><title>{c.label}, log scale 10^{c.lo} to 10^{c.hi}</title></path>{/if}{/each}
          <text class="axis" x="10" y={HH - 4}>Big Bang</text>
          <text class="axis end" x={HW - 10} y={HH - 4}>{gyr(chart.now_gyr)}</text>
        </svg>
        <p class="hint key"><span class="sw stars"></span> stars <span class="sw gas"></span> gas <span class="sw sfr"></span> star formation (each on its own log scale, three decades tall) · dots: active-nucleus episodes · diamonds: mergers (filled when inked) · ticks: cosmic epochs{chart.compare ? " · faint: as grown, before the inked history" : ""}</p>
      {/if}
      <p class="hint">Model: {chart.model.join("; ")}.</p>
    </section>

    <section aria-label="Ink history" class="ink" data-ink>
      <h4>Ink history</h4>
      <p class="hint">Rewrite this galaxy's past: each edit acts from its own time on, so what came before is untouched. A preview kept in this browser; nothing is sent to players.</p>
      <div class="inkform">
        <label>Event
          <select bind:value={inkKind} data-ink-kind>
            <option value="merger">A merger</option>
            <option value="starburst">A starburst</option>
            <option value="quiet_nucleus">A quiet nucleus</option>
          </select>
        </label>
        <label>{inkKind === "quiet_nucleus" ? "From" : "At"} <input type="number" min={Math.ceil(chart.t_start_gyr * 10) / 10} max={chart.now_gyr} step="0.1" bind:value={inkT} data-ink-t /> Gyr</label>
        {#if inkKind === "merger"}
          <label>Size
            <select bind:value={inkRatio} data-ink-ratio>
              <option value={0.1}>minor (1 : 10)</option>
              <option value={0.33}>major (1 : 3)</option>
              <option value={1}>equal (1 : 1)</option>
            </select>
          </label>
        {:else if inkKind === "starburst"}
          <label>Gas into stars
            <select bind:value={inkGas}>
              <option value={0.1}>10%</option>
              <option value={0.3}>30%</option>
              <option value={0.6}>60%</option>
            </select>
          </label>
        {:else}
          <label>To <input type="number" min={inkT} max={chart.now_gyr} step="0.1" bind:value={inkTo} /> Gyr</label>
        {/if}
        <button type="button" class="act" onclick={ink} data-ink-add>Ink it</button>
      </div>
      {#if edits.length}
        <ul class="edits">
          {#each edits as e, i (i)}<li data-edit={i}>{editWords(e, chart.mergers)} <button type="button" class="x" aria-label="Remove this edit" onclick={() => setEdits(edits.filter((_, j) => j !== i))}>✕</button></li>{/each}
        </ul>
        <button type="button" class="act" onclick={() => setEdits([])} data-ink-clear>Back to the galaxy as grown</button>
      {/if}
      {#if chart.mergers.length}
        <h4>The halo's own mergers</h4>
        <ul class="edits">
          {#each chart.mergers as mg (mg.index)}
            <li class:out={mg.removed} data-merger={mg.index}>{gyr(mg.t_gyr)}, {mg.ratio >= 0.25 ? "major" : "minor"} (1 : {Math.round(1 / mg.ratio)}) <button type="button" class="act small" onclick={() => setEdits(toggleRemoval(edits, mg.index))}>{mg.removed ? "Put back" : "Take out"}</button></li>
          {/each}
        </ul>
      {/if}
    </section>
  {:else}
    <p class="note">Growing the galaxy from the universe's seed…</p>
  {/if}
</div>

<style>
  .stamp.authored {
    border-style: dashed;
  }
  .ghost {
    fill: none;
    stroke: var(--sse-ink-muted);
    stroke-width: 1;
    stroke-dasharray: 3 3;
    opacity: 0.7;
  }
  .mark {
    fill: var(--sse-ground);
    stroke: var(--sse-ink);
    stroke-width: 1.2;
  }
  .mark.inked {
    fill: var(--sse-accent);
    stroke: var(--sse-accent);
  }
  .ink {
    border-top: 1px solid var(--sse-rule);
    padding-top: 8px;
  }
  .inkform {
    display: flex;
    flex-wrap: wrap;
    gap: 10px 16px;
    align-items: center;
  }
  .inkform input {
    font: inherit;
    width: 5em;
    min-height: 30px;
    background: var(--sse-ground);
    color: var(--sse-ink);
    border: 1px solid var(--sse-rule);
  }
  .act {
    font: inherit;
    min-height: 34px;
    padding: 0 12px;
    border: 1px solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .act.small {
    min-height: 26px;
    padding: 0 8px;
    margin-left: 8px;
  }
  .edits {
    list-style: none;
    padding: 0;
    margin: 6px 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .edits li.out {
    text-decoration: line-through;
    color: var(--sse-ink-muted);
  }
  .x {
    font: inherit;
    border: none;
    background: transparent;
    color: var(--sse-ink-muted);
    cursor: pointer;
    min-width: 28px;
    min-height: 28px;
  }
  .gal {
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
  .seg button {
    font: inherit;
    min-height: 34px;
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
    letter-spacing: 0.12em;
    text-transform: uppercase;
    padding: 1px 6px;
  }
  .hint,
  .note {
    color: var(--sse-ink-muted);
    margin: 0;
  }
  .hint {
    font-size: 12px;
  }
  .cols {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(260px, 1fr);
    gap: 18px;
    align-items: start;
  }
  @media (max-width: 900px) {
    .cols {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .disc {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .disc svg {
    width: 100%;
    max-width: 560px;
    height: auto;
    align-self: center;
  }
  .ring {
    fill: none;
    stroke: var(--sse-ink);
    cursor: pointer;
  }
  .green {
    fill: none;
    stroke: var(--sse-accent);
    stroke-opacity: 0.1;
    pointer-events: none;
  }
  .greenedge {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 1.5;
    stroke-dasharray: 2 4;
    pointer-events: none;
  }
  .arm {
    fill: none;
    stroke: var(--sse-ink);
    stroke-opacity: 0.75;
    stroke-linecap: round;
    pointer-events: none;
  }
  .barline {
    stroke: var(--sse-ink);
    stroke-width: 6;
    stroke-linecap: round;
    stroke-opacity: 0.6;
  }
  .sun {
    fill: var(--sse-ground);
    stroke: var(--sse-ink);
    stroke-width: 1.5;
  }
  .pick {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 2;
    stroke-dasharray: 5 3;
    pointer-events: none;
  }
  .core {
    fill: var(--sse-ink);
  }
  .time input {
    flex: 1;
    min-width: 160px;
    accent-color: var(--sse-accent);
  }
  .time {
    display: flex;
  }
  .v {
    font-family: var(--sse-font-mono, monospace);
  }
  .ledger {
    background: var(--sse-panel-ground, var(--sse-ground));
    border: var(--sse-frame-width, 1px) solid var(--sse-frame-line, var(--sse-ink));
    box-shadow: var(--sse-frame-shadow, none);
    border-radius: var(--sse-frame-radius, 0);
    padding: 12px 16px;
  }
  .ringpick {
    margin-bottom: 6px;
  }
  .row {
    display: flex;
    justify-content: space-between;
    gap: 10px;
    padding: 4px 0;
    border-bottom: 1px dotted var(--sse-rule);
  }
  .verdict {
    margin: 10px 0 4px;
    font-weight: 600;
  }
  .crit {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .crit .fail {
    color: var(--sse-accent-ink);
  }
  .curves {
    width: 100%;
    height: auto;
    border: 1px solid var(--sse-rule);
    background: var(--sse-chart-ground, transparent);
  }
  .curve {
    fill: none;
    stroke-width: 2;
  }
  .curve.stars {
    stroke: var(--sse-ink);
  }
  .curve.gas {
    stroke: var(--sse-ink-muted);
    stroke-dasharray: 5 3;
  }
  .curve.sfr {
    stroke: var(--sse-accent);
  }
  .tick {
    stroke: var(--sse-rule);
  }
  .tick.spec {
    stroke-dasharray: 2 3;
  }
  .nowline {
    stroke: var(--sse-accent);
    stroke-width: 1.5;
  }
  .agn {
    fill: var(--sse-accent);
  }
  svg text {
    fill: var(--sse-ink-muted);
    font-size: 10px;
  }
  .axis.end {
    text-anchor: end;
  }
  .key {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }
  .sw {
    display: inline-block;
    width: 18px;
    height: 0;
    border-top: 2px solid var(--sse-ink);
  }
  .sw.gas {
    border-top: 2px dashed var(--sse-ink-muted);
  }
  .sw.sfr {
    border-top-color: var(--sse-accent);
  }
</style>
