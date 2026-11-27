<!--
  Size comparison (V14). Bodies stand side by side on one baseline at one
  true scale, so a moon, a world and a star can be shown off against each
  other and against things people know. The lineup starts as this system's
  own worlds; "Add" brings in any body from any system in the campaign, or a
  stock reference object (one of each standard planet and star type, with
  measured radii and references). A body that dwarfs the rest is drawn as a
  limb from the left edge, as on a poster; anything under a pixel is marked,
  never dropped. Radii come from the bodies' own facts: a body without one is
  not offered, and the panel says how many. In 3D, True and Wireframe the same
  lineup stands up as globes, each turning on its own axis (Lineup3D).
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { sayNumber, checkReferences, formatKmLength, itemFromReference, itemFromSubject, layoutLineup, listedTitle, sceneFromResolved, scaleBar, sizeWords, starColour, subjectOf, viewOptions, pictureSpan, rateWords, tooFast, DEFAULT_VIEW, SEEN_ELEVATION, type CompareItem, type Json, type ReferenceDoc, type SceneSubject, type SolDoc, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { HatchDefs, markPattern } from "../../../chart-grammar/src/index.ts";
  import { lineups } from "./lineup.svelte.ts";
  import { COMPARE_CONTROLS } from "./controls.ts";
  import Lineup3D from "./Lineup3D.svelte";
  import { SECONDS_PER_TURN } from "../../../renderer/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  const ws = $derived(ctx.workspace);
  const shown = (s: SceneSubject) => s.kind === "star" || s.kind === "planet" || s.kind === "moon";
  const own = $derived(scene.subjects.filter(shown).map((s) => itemFromSubject(s, scene.title, scene.subjects)).filter((x): x is CompareItem => x !== null));
  const missing = $derived(scene.subjects.filter(shown).length - own.length);
  // This system's worlds and star, in orbit order: where the lineup starts.
  const starting = $derived(own.filter((i) => i.kind !== "moon"));
  const key = $derived(scene.system);
  $effect(() => {
    if (!lineups[key]) lineups[key] = starting.map((i) => ({ ...i }));
  });
  const items = $derived(lineups[key] ?? []);
  const inLineup = (k: string) => items.some((i) => i.key === k);
  function add(i: CompareItem) {
    if (!inLineup(i.key)) lineups[key] = [...items, { ...i }];
    note = `Added ${i.label}.`;
  }
  function remove(i: CompareItem) {
    lineups[key] = items.filter((x) => x.key !== i.key);
    note = `Removed ${i.label}.`;
  }
  let note = $state("");
  let sorted = $state(false);
  const order = $derived(sorted ? [...items].sort((a, b) => a.radius_km - b.radius_km) : items);
  // Scale is the view controls' (one set of controls, Alex 23 Nov): Real is one true scale, Readable a log scale.
  const shownView = $derived(viewOptions(COMPARE_CONTROLS, ws.view ?? DEFAULT_VIEW).shown);
  const mode = $derived(shownView.scale === "real" ? "true" : "log");
  // 3D, True and Wireframe: the lineup as turning globes, unless this device cannot draw them.
  let noGpu = $state(false);
  const globes = $derived(shownView.style === "3d" || shownView.style === "true" || shownView.style === "wireframe");
  const look = $derived(globes && !noGpu ? (shownView.style as "3d" | "true" | "wireframe") : null);
  let pace = $state<"same" | "clock">("same");
  // Seen from tips every globe together over their orbits' plane, the row staying put; Free is turned by hand.
  let freeElev = $state(SEEN_ELEVATION.tilted);
  let yaw = $state(0);
  const globeView = $derived({ elevation: shownView.seen === "free" ? freeElev : SEEN_ELEVATION[shownView.seen], yaw });
  $effect(() => {
    void shownView.framed;
    untrack(() => {
      if (shownView.seen !== "free") ((yaw = 0), (freeElev = SEEN_ELEVATION[shownView.seen]));
    });
  });
  let drag: { x: number; y: number; moved: number } | null = null;
  let dragged = false;
  function down(e: PointerEvent) {
    if (!look || shownView.seen !== "free") return;
    drag = { x: e.clientX, y: e.clientY, moved: 0 };
    dragged = false;
  }
  function move(e: PointerEvent) {
    if (!drag) return;
    const dx = e.clientX - drag.x;
    const dy = e.clientY - drag.y;
    drag = { x: e.clientX, y: e.clientY, moved: drag.moved + Math.abs(dx) + Math.abs(dy) };
    if (drag.moved > 4) {
      dragged = true;
      yaw += dx * 0.01;
      freeElev = Math.max(-Math.PI / 2, Math.min(Math.PI / 2, freeElev + dy * 0.01));
    }
  }
  const up = () => (drag = null);

  // Reference objects: a document the shell serves (a pack may ship more).
  let refs = $state<ReferenceDoc | null>(null);
  let refProblem = $state<string | null>(null);
  $effect(() => {
    if (refs) return;
    fetch(new URL(String(params.references ?? "documents/reference-objects.json"), document.baseURI))
      .then((r) => r.json())
      .then((d) => {
        const p = checkReferences(d);
        if (p.length) refProblem = p.join("; ");
        else refs = d as ReferenceDoc;
      })
      .catch((e) => (refProblem = String(e)));
  });
  // The Solar System's measured days, tilts and rings (base-sol pack) for the reference bodies that name a row in it.
  const SOL = "packs/base-sol.json";
  let solDoc = $state<SolDoc | null>(null);
  $effect(() => {
    if (solDoc) return;
    fetch(new URL(SOL, document.baseURI))
      .then((r) => r.json())
      .then((d) => (solDoc = (d?.provides?.other?.sol as SolDoc | undefined) ?? null))
      .catch(() => {});
  });

  // Everything else in the campaign: each system the kernel lists, read back with its facts.
  type Pool = { title: string; system: string; items: CompareItem[]; without: number }[];
  let pool = $state<Pool | null>(null);
  let poolLoading = $state(false);
  let poolProblem = $state<string | null>(null);
  async function loadPool(): Promise<Pool> {
    if (pool) return pool;
    poolLoading = true;
    const out: Pool = [];
    const listed = await ctx.session.listSystems();
    if (!listed.ok) poolProblem = listed.message;
    else
      for (const x of listed.systems) {
        if (x.system === scene.system) continue;
        const r = await ctx.session.resolvedSystem(x.system);
        if (!r.ok) continue;
        const sc = sceneFromResolved(r.result, listedTitle(x));
        const its = sc.subjects.filter(shown).map((s) => itemFromSubject(s, sc.title, sc.subjects)).filter((v): v is CompareItem => v !== null);
        out.push({ title: sc.title, system: x.system, items: its, without: sc.subjects.filter(shown).length - its.length });
      }
    poolLoading = false;
    pool = out;
    return out;
  }

  let adding = $state<"system" | "campaign" | "reference" | null>(null);
  function openAdd(t: "system" | "campaign" | "reference") {
    adding = adding === t ? null : t;
    if (t === "campaign") void loadPool();
  }
  async function surprise() {
    const p = await loadPool();
    const choices = [...own, ...p.flatMap((x) => x.items), ...(refs?.objects.map((o) => itemFromReference(o, solDoc)) ?? [])].filter((i) => !inLineup(i.key));
    if (!choices.length) {
      note = "Everything there is to compare is already in the lineup.";
      return;
    }
    add(choices[Math.floor(Math.random() * choices.length)]!);
  }

  // Layout.
  let w = $state(700);
  const narrow = $derived(w < 560);
  const H = $derived(narrow ? 220 : 300);
  const labelH = 78;
  const base = $derived(H - labelH);
  const lineup = $derived(layoutLineup(order, base - 16, Math.max(280, w - 8), mode, Number(params.limb_ratio ?? 5), narrow ? 64 : 76, globes));
  // Each label (its name and size line) takes the first of three rows where it clears the label before it; a crowded
  // row of small bodies staggers instead of overprinting.
  const labelRows = $derived.by(() => {
    const ends = [-Infinity, -Infinity, -Infinity];
    return lineup.placed.map((p) => {
      const half = Math.max(p.item.label.length * 7, (p.tiny ? 13 : sizeWords(p.item.radius_km).length) * 6) / 2 + 4;
      // Kept inside the view at either end.
      const x = Math.min(Math.max(lineup.width, w - 8) - half, Math.max(half, p.x));
      const free = ends.findIndex((e) => e <= x - half);
      const row = free >= 0 ? free : ends.indexOf(Math.min(...ends));
      ends[row] = x + half;
      return { row, x };
    });
  });
  const svgW = $derived(Math.max(w - 8, lineup.width));
  const vis = $derived(Math.min(svgW, Math.max(280, w - 8)));
  const bar = $derived(mode === "true" && Number.isFinite(lineup.km_per_px) ? scaleBar(lineup.km_per_px, narrow ? 80 : 120) : null);

  // Colour from the symbology (colour-coded skins colour by class); a pattern where the symbology draws one.
  const markOf = (i: CompareItem) => (i.kind === "star" ? null : (ctx.symbology?.marker().mark(subjectOf(i)) ?? null));
  function fill(i: CompareItem): string {
    if (i.kind === "star") return i.colour ?? starColour(i.teff_k);
    return markOf(i)?.colour ?? i.colour ?? "var(--sse-chart-ground, var(--sse-ground))";
  }
  function pattern(i: CompareItem): string | null {
    const m = markOf(i);
    const p = m ? markPattern(m) : null;
    return p ? `url(#cmp-${p})` : null;
  }
  // A star's figures as the scene holds them (the same three the system's True sends with a fixture's star).
  const starGiven = (i: CompareItem) => {
    const f = scene.subjects.find((s) => s.address === i.key)?.facts ?? [];
    const num = (k: string) => Number(f.find((x) => x.field === k)?.value);
    return { teff_k: num("teff_k"), mass_sun: num("mass_sun"), radius_sun: num("radius_sun") };
  };
  const selectable = (i: CompareItem) => scene.subjects.some((s) => s.address === i.key);
  function pick(i: CompareItem) {
    if (dragged) return void (dragged = false);
    if (selectable(i)) ws.select(i.key);
    else note = `${i.label}: ${i.type ? `${i.type}, ` : ""}${tidy(i.radius_km)}, from ${i.where}.${i.ref ? ` ${i.ref.cite}.` : ""}`;
  }
  const ratio = (i: CompareItem) => {
    const sel = items.find((x) => x.key === ws.selection);
    if (!sel || sel.key === i.key) return "";
    const k = i.radius_km / sel.radius_km;
    return `${sayNumber(k, 0, { decimals: k >= 10 ? 0 : k >= 1 ? 1 : 2 })} × ${sel.label}`;
  };
  const tidy = (km: number) => `${sayNumber(km, 0, { upTo: km < 100 ? 1 : 0 })} km`;
  // On the clock (CLK1): the globes that turn faster than the screen's pictures can follow at the clock's rate.
  const fastSpins = $derived.by(() => {
    const c = ws.clock;
    if (!c || !(c.rate > 0)) return "";
    const names = order.filter((i) => i.day_h && tooFast(Math.abs(i.day_h) * 3600, pictureSpan(c, 60))).map((i) => i.label);
    return names.length > 1 ? `${names.slice(0, -1).join(", ")} and ${names[names.length - 1]} turn` : names.length ? `${names[0]} turns` : "";
  });
  const stillOnes = $derived(order.filter((i) => !i.day_h).length);
  const summary = $derived(order.map((i) => `${i.label} ${tidy(i.radius_km)}`).join(", "));
</script>

<section class="compare" aria-label="Size comparison" data-compare={items.length} data-mode={mode}>
  <header>
    <h2>Size comparison</h2>
    <div class="actions">
      <button type="button" aria-expanded={adding === "system"} onclick={() => openAdd("system")}>From {scene.title}</button>
      <button type="button" aria-expanded={adding === "campaign"} onclick={() => openAdd("campaign")}>From the campaign</button>
      <button type="button" aria-expanded={adding === "reference"} onclick={() => openAdd("reference")}>Reference objects</button>
      <button type="button" onclick={surprise}>Surprise me</button>
      <span class="sep"></span>
      <button type="button" aria-pressed={sorted} onclick={() => (sorted = !sorted)}>By size</button>
      <button type="button" onclick={() => ((lineups[key] = starting.map((i) => ({ ...i }))), (note = `Back to ${scene.title}'s own worlds.`))}>Start again</button>
      <button type="button" onclick={() => ((lineups[key] = []), (note = "Cleared."))} disabled={!items.length}>Clear</button>
    </div>
  </header>

  {#if adding}
    <div class="adder" role="group" aria-label={adding === "system" ? `Add from ${scene.title}` : adding === "campaign" ? "Add from the campaign" : "Add a reference object"}>
      {#if adding === "system"}
        <ul class="picks">
          {#each own as i (i.key)}<li><button type="button" disabled={inLineup(i.key)} onclick={() => add(i)}>{i.label}</button></li>{/each}
        </ul>
        {#if missing > 0}<p class="note">{missing} {missing === 1 ? "body has" : "bodies have"} no radius yet, so {missing === 1 ? "it is" : "they are"} not offered.</p>{/if}
      {:else if adding === "campaign"}
        {#if poolLoading}
          <p class="note">Reading the campaign's systems…</p>
        {:else if poolProblem}
          <p class="note">{poolProblem}</p>
        {:else if pool && !pool.length}
          <p class="note">No other systems in the campaign yet. Grow or import one with Make a system.</p>
        {:else if pool}
          {#each pool as p (p.system)}
            <h3>{p.title}</h3>
            <ul class="picks">
              {#each p.items as i (i.key)}<li><button type="button" disabled={inLineup(i.key)} onclick={() => add(i)}>{i.label}</button></li>{/each}
            </ul>
            {#if p.without}<p class="note">{p.without} without a radius, not offered.</p>{/if}
          {/each}
        {/if}
      {:else if refs}
        {#each refs.groups as g (g.id)}
          <h3>{g.label}</h3>
          <ul class="picks">
            {#each refs.objects.filter((o) => o.group === g.id) as o (o.id)}
              {@const i = itemFromReference(o, solDoc)}
              <li><button type="button" disabled={inLineup(i.key)} onclick={() => add(i)} title={o.type}>{o.label}</button></li>
            {/each}
          </ul>
        {/each}
      {:else}
        <p class="note">{refProblem ?? "Loading reference objects…"}</p>
      {/if}
    </div>
  {/if}

  <div class="stage" bind:clientWidth={w}>
    {#if !items.length}
      <p class="note empty">Nothing to compare. Add bodies from this system, the campaign or the reference objects.</p>
    {:else}
      <div class="frame" class:globes={look} class:space={look === "true"} class:free={look && shownView.seen === "free"} onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up} role="presentation">
      {#if look}<Lineup3D {ctx} system={scene.system} placed={lineup.placed} width={svgW} height={H} {base} {look} view={globeView} {pace} colour={fill} own={selectable} given={starGiven} onfail={() => (noGpu = true)} />{/if}
      <svg width={svgW} height={H} role="img" aria-label={`Size comparison${mode === "log" ? ", not to scale (log)" : ""}: ${summary}`}>
        <HatchDefs prefix="cmp" />
        {#if lineup.limb}
          {@const l = lineup.limb}
          <g data-item={l.item.key} data-limb="true">
            <circle cx={l.x} cy={base / 2} r={l.r} fill={fill(l.item)} stroke="var(--sse-ink)" stroke-width="1" />
            <text class="limb-label" x={8} y={base / 2 - 6}>{l.item.label}</text>
            <text class="limb-label sub" x={8} y={base / 2 + 10}>edge only · {sizeWords(l.item.radius_km)}</text>
          </g>
        {/if}
        <line class="baseline" x1="0" x2={svgW} y1={base} y2={base} />
        {#each lineup.placed as p, n (p.item.key)}
          {@const row = labelRows[n]?.row ?? 0}
          {@const lx = labelRows[n]?.x ?? p.x}
          <g
            class="disc"
            data-item={p.item.key}
            data-tiny={p.tiny || undefined}
            role="button"
            tabindex="0"
            aria-label={`${p.item.label}, ${tidy(p.item.radius_km)}`}
            aria-pressed={ws.selection === p.item.key}
            onclick={() => pick(p.item)}
            onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), pick(p.item))}
          >
            <circle class="face" cx={p.x} cy={base - p.r} r={p.r} fill={fill(p.item)} stroke="var(--sse-ink)" stroke-width={p.r < 3 ? 0.6 : 1} />
            {#if pattern(p.item) && p.r > 4 && !look}<circle cx={p.x} cy={base - p.r} r={p.r} fill={pattern(p.item)} />{/if}
            {#if p.tiny}<circle class="tinyring" cx={p.x} cy={base - p.r} r={p.r + 5} />{/if}
            {#if ws.selection === p.item.key}<circle class="sel" cx={p.x} cy={base - p.r} r={p.r + (p.tiny ? 9 : 4)} />{/if}
            <text class="lab" x={lx} y={base + 16 + row * 22} text-anchor="middle">{p.item.label}</text>
            <text class="lab sub" x={lx} y={base + 28 + row * 22} text-anchor="middle">{p.tiny ? "under a pixel" : sizeWords(p.item.radius_km)}</text>
          </g>
        {/each}
        {#if bar}
          <g class="bar" aria-hidden="true">
            <line x1={vis - bar.px - 16} x2={vis - 16} y1={14} y2={14} />
            <line x1={vis - bar.px - 16} x2={vis - bar.px - 16} y1={9} y2={19} />
            <line x1={vis - 16} x2={vis - 16} y1={9} y2={19} />
            <text x={vis - 16 - bar.px / 2} y={32} text-anchor="middle">{formatKmLength(bar.km)}</text>
          </g>
        {:else if mode === "log"}
          <text class="stamp" x={vis - 16} y={20} text-anchor="end">Not to scale: radius on a log scale</text>
        {/if}
      </svg>
      </div>
    {/if}
  </div>
  {#if look}
    <p class="note turning" data-pace={pace}>
      <button type="button" aria-pressed={pace === "clock"} onclick={() => (pace = pace === "same" ? "clock" : "same")}>On the clock</button>
      {#if pace === "same"}Each body turns once every {SECONDS_PER_TURN} seconds, its own way round and leaning by its own tilt: an aid for comparing, not their real speeds.{:else}On the campaign's clock, {rateWords(ws.clock?.rate ?? 0)}: each body turns at its own real speed{(ws.clock?.rate ?? 0) > 0 ? "" : ", so with the clock stopped they stand still (press play on the ruler to set them turning)"}{fastSpins ? `; ${fastSpins} faster than the screen can show, drawn where ${fastSpins.includes(" and ") ? "they are" : "it is"} at each picture` : ""}.{/if}
      {#if stillOnes}{stillOnes === 1 ? "One body's day is" : `${stillOnes} bodies' days are`} not known, so {stillOnes === 1 ? "it stands" : "they stand"} still.{/if}
    </p>
  {/if}
  {#if noGpu && globes}<p class="note" data-no-globes>This device cannot draw the globes, so the lineup is drawn flat.</p>{/if}
  {#if note}<p class="note" role="status">{note}</p>{/if}
  {#if mode === "true" && lineup.placed.some((p) => p.tiny)}<p class="note" data-hint="tiny">Some bodies are under a pixel beside the largest: try Scale: Readable (a log scale), or remove the largest.</p>{/if}

  {#if items.length}
    <table aria-label="Sizes in the lineup">
      <thead><tr><th scope="col">Body</th><th scope="col">From</th><th scope="col">Radius</th><th scope="col">Size</th><th scope="col">Rests on</th><th scope="col"><span class="sr">Remove</span></th></tr></thead>
      <tbody>
        {#each order as i (i.key)}
          <tr data-row={i.key}>
            <th scope="row">{#if selectable(i)}<button type="button" class="linkbtn" onclick={() => pick(i)}>{i.label}</button>{:else}{i.label}{/if}{#if i.type}<span class="type">{i.type}</span>{/if}</th>
            <td>{i.where}</td>
            <td class="num">{tidy(i.radius_km)}</td>
            <td>{sizeWords(i.radius_km)}{#if ratio(i)}<span class="type">{ratio(i)}</span>{/if}</td>
            <td>{#if i.ref}<a href={i.ref.url} target="_blank" rel="noopener" title={i.ref.cite}>{i.basis === "typical" ? "typical value" : "measured"}</a>{:else}{i.basis}{/if}</td>
            <td><button type="button" class="x" aria-label={`Remove ${i.label}`} onclick={() => remove(i)}>×</button></td>
          </tr>
        {/each}
      </tbody>
    </table>
    <p class="note">Radii are mean radii. Click a body from {scene.title} to see its card; with one selected, the table gives every size as a multiple of it.</p>
  {/if}
</section>

<style>
  .compare {
    display: flex;
    flex-direction: column;
    gap: 10px;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    padding: 4px 4px 16px;
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 8px 16px;
  }
  h2 {
    margin: 0;
    font-family: var(--sse-font-display, var(--sse-font-body));
    font-size: 22px;
  }
  h3 {
    margin: 8px 0 4px;
    font-size: 13px;
    font-weight: 600;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }
  .sep {
    width: 8px;
  }
  button {
    min-height: 32px;
    padding: 0 10px;
    border: var(--sse-line, 1px) solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  button[aria-pressed="true"],
  button[aria-expanded="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink, var(--sse-ground));
  }
  button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .adder {
    border: var(--sse-line, 1px) solid var(--sse-rule, var(--sse-ink));
    padding: 8px 12px;
    max-height: 240px;
    overflow: auto;
  }
  .picks {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .stage {
    overflow-x: auto;
    max-width: 100%;
  }
  .frame {
    position: relative;
    width: max-content;
  }
  .frame svg {
    position: relative;
  }
  /* Globes: the lineup's discs become the picking spots over the GPU's globes; labels keep a halo of the ground. */
  .globes .face {
    fill: transparent;
    stroke: none;
  }
  .globes text {
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 3px;
    stroke-linejoin: round;
  }
  /* True: the labels sit over black space, so they are light on a dark halo (the chart's ground halo read as hatching). */
  .space text {
    fill: white;
    stroke: black;
  }
  .space text.sub {
    fill: silver;
  }
  /* The limb is a star's bright edge: its words go dark on it. */
  .space text.limb-label {
    fill: black;
    stroke: none;
  }
  .space .baseline {
    stroke: var(--sse-ink-muted);
  }
  .free {
    cursor: grab;
    touch-action: none;
  }
  .turning {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 10px;
    align-items: center;
  }
  svg {
    display: block;
  }
  .baseline {
    stroke: var(--sse-ink);
    stroke-width: 1;
  }
  .disc {
    cursor: pointer;
  }
  .disc:focus-visible {
    outline: 2px solid var(--sse-accent);
  }
  .tinyring {
    fill: none;
    stroke: var(--sse-ink-muted);
    stroke-dasharray: 2 2;
  }
  .sel {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 1.5;
  }
  .lab {
    font-size: 12px;
    fill: var(--sse-ink);
  }
  .sub,
  .limb-label.sub {
    font-size: 10px;
    fill: var(--sse-ink-muted);
  }
  .limb-label {
    font-size: 12px;
    fill: var(--sse-ink);
  }
  .bar line {
    stroke: var(--sse-ink);
    stroke-width: 1.2;
  }
  .bar text,
  .stamp {
    font-size: 11px;
    fill: var(--sse-ink);
    font-family: var(--sse-font-mono, monospace);
  }
  .note {
    font-size: 13px;
    color: var(--sse-ink-muted);
    margin: 0;
  }
  .empty {
    padding: 24px 0;
  }
  table {
    border-collapse: collapse;
    font-size: 13px;
    width: 100%;
    max-width: 860px;
  }
  th,
  td {
    text-align: left;
    padding: 4px 8px 4px 0;
    border-bottom: 1px solid var(--sse-rule, var(--sse-ink-muted));
    vertical-align: top;
  }
  thead th {
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--sse-ink-muted);
  }
  .num {
    font-family: var(--sse-font-mono, monospace);
    white-space: nowrap;
  }
  .type {
    display: block;
    font-size: 11px;
    font-weight: normal;
    color: var(--sse-ink-muted);
  }
  .linkbtn {
    border: 0;
    padding: 0;
    min-height: 0;
    text-decoration: underline;
    font-weight: 600;
  }
  .x {
    min-height: 28px;
    width: 28px;
    padding: 0;
  }
  a {
    color: var(--sse-accent);
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
  @media (max-width: 560px) {
    table {
      font-size: 12px;
    }
    th:nth-child(2),
    td:nth-child(2),
    th:nth-child(5),
    td:nth-child(5) {
      display: none;
    }
  }
</style>
