<!--
  Biosphere view and life timeline (U7). One world's life, from the kernel's
  life_view: the timeline (named epochs as bands, production on a log axis,
  catastrophes and origins as ticks), the food web by feeding level, the
  guilds in words, traits, what a scan would show at each tier, the dials the
  story leans on, and story hooks. Every reason is told in three lenses.
  The kernel builds a biosphere for its example worlds today; for a body on
  the chart it says when it has none yet, and the view shows an example,
  labelled as one.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import LifeFlow from "./LifeFlow.svelte";
  import { LIFE_EXAMPLES, MARKED_KINDS, carbonWords, epochBands, foodWeb, lifeWords, productionPath, scanTiers, timeAxis, type Json, type LifeEpoch, type LifeView, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  type Lens = "physics" | "ecology" | "history";
  const LENSES: { id: Lens; label: string }[] = [
    { id: "ecology", label: "Ecology" },
    { id: "physics", label: "Physics" },
    { id: "history", label: "History" },
  ];
  const first = untrack(() => ({ lens: params.lens, example: params.example }));
  let lens = $state<Lens>(first.lens === "physics" || first.lens === "history" ? first.lens : "ecology");
  // Story (timeline, food web, scan) or Board (the LifeFlow: the pipeline that worked it out).
  let shape = $state<"story" | "board">(untrack(() => (params.show === "board" ? "board" : "story")));

  const ws = $derived(ctx.workspace);
  const subject = $derived(scene.subjects.find((s) => s.address === ws.selection && s.kind !== "star"));
  let example = $state(LIFE_EXAMPLES.some((e) => e.fixture === first.example) ? String(first.example) : "earth");
  // "body": the selected world, when the kernel has its biosphere; otherwise an example.
  let source = $state<{ kind: "body"; address: string; label: string } | { kind: "example"; fixture: string } | null>(null);
  let view = $state<LifeView | null>(null);
  let bodyNote = $state<string | null>(null);
  let failed = $state<string | null>(null);
  let gen = 0;
  $effect(() => {
    const sub = subject;
    const ex = example;
    const g = ++gen;
    failed = null;
    (async () => {
      if (sub) {
        const r = await ctx.session.lifeView({ body: sub.address });
        if (g !== gen) return;
        if (r.ok) {
          ((source = { kind: "body", address: sub.address, label: sub.label }), (view = r.view), (bodyNote = null));
          return;
        }
        bodyNote = `The kernel has no biosphere for ${sub.label} yet (worlds on the chart get one when systems carry habitat graphs). Showing an example world.`;
      } else bodyNote = null;
      const r = await ctx.session.lifeView({ fixture: ex });
      if (g !== gen) return;
      if (!r.ok) return void ((failed = r.message), (view = null));
      source = { kind: "example", fixture: ex };
      view = r.view;
    })();
  });

  const name = $derived(source?.kind === "body" ? source.label : (LIFE_EXAMPLES.find((e) => source?.kind === "example" && e.fixture === source.fixture)?.label ?? ""));
  const shows = $derived.by(() => {
    const src = source;
    return src?.kind === "example" ? (LIFE_EXAMPLES.find((e) => e.fixture === src.fixture)?.shows ?? null) : null;
  });

  // Timeline geometry.
  const TW = 880;
  const TH = 150;
  const bandTop = 26;
  const bandH = 30;
  const curveTop = bandTop + bandH + 10;
  const curveH = TH - curveTop - 18;
  const axis = $derived(view ? timeAxis(view, TW, 8) : null);
  const bands = $derived(view && axis ? epochBands(view, axis) : []);
  const prod = $derived(view && axis ? productionPath(view, axis, curveTop, curveH) : { d: "", lo: 0, hi: 0 });
  const marked = $derived(view ? view.events.filter((e) => (MARKED_KINDS as readonly string[]).includes(e.kind)) : []);
  const gains = $derived(view ? view.events.filter((e) => e.kind === "capability") : []);
  const ticks = $derived.by(() => {
    if (!axis) return [];
    const step = axis.to > 3000 ? 1000 : axis.to > 600 ? 200 : 50;
    const out: number[] = [];
    for (let t = 0; t <= axis.to + 1e-9; t += step) out.push(t);
    return out;
  });
  let picked = $state<number | null>(null);
  $effect(() => {
    void view;
    picked = null;
  });
  const epoch = $derived<LifeEpoch | null>(view && picked !== null ? (view.epochs[picked] ?? null) : null);
  const nowEpoch = $derived(view?.epochs.at(-1) ?? null);

  // Food web.
  const WW = 520;
  const WH = 260;
  const web = $derived(view ? foodWeb(view.guilds, WW, WH) : { nodes: [], links: [], levels: 0 });
  const at = $derived(new Map(web.nodes.map((n) => [n.guild.id, n])));
  let guildPick = $state<string | null>(null);
  const guild = $derived(view?.guilds.find((g) => g.id === guildPick) ?? null);
  const guildsBySize = $derived(view ? [...view.guilds].sort((a, b) => Math.max(b.production_gc_yr, b.intake_gc_yr) - Math.max(a.production_gc_yr, a.intake_gc_yr)) : []);

  const tiers = $derived(view ? scanTiers(view.signatures) : []);
  const myr = (m: number) => (m >= 1000 ? `${(m / 1000).toLocaleString("en-GB", { maximumFractionDigits: 2 })} Gyr` : `${Math.round(m).toLocaleString("en-GB")} Myr`);
  const traitRows = $derived.by(() => {
    const t = view?.traits;
    if (!t) return [];
    const show: [string, string, (v: Json) => string][] = [
      ["producer_colour", "Producer colour", (v) => String(v)],
      ["photon_peak_nm", "Starlight peaks at", (v) => `${Math.round(Number(v))} nm`],
      ["solvent", "Solvent", (v) => String(v)],
      ["handedness", "Chemistry", (v) => String(v)],
      ["tallest_plant_m", "Tallest producers", (v) => `about ${Math.round(Number(v))} m`],
      ["max_land_animal_kg", "Largest land animal", (v) => `about ${Number(v).toLocaleString("en-GB")} kg`],
      ["flight", "Flight", (v) => String(v)],
      ["max_flyer_kg", "Largest flyer", (v) => `about ${Number(v).toLocaleString("en-GB")} kg`],
      ["senses", "Senses", (v) => (Array.isArray(v) ? v.join(", ") : String(v))],
      ["day_strategies", "Days and nights", (v) => (Array.isArray(v) ? v.join(", ") : String(v))],
      ["to_visitors", "To visitors", (v) => String(v)],
      ["origin", "Origin", (v) => String(v)],
    ];
    return show.filter(([k]) => t[k] !== undefined && t[k] !== null).map(([k, label, f]) => ({ label, text: f(t[k]!) }));
  });
  const dialWords: Record<string, string> = { origin: "How easily life starts", oxygenic: "Oxygen-making light harvest", complex_cells: "Complex cells", minds: "Minds" };
</script>

<div class="bio" data-biosphere={source ? (source.kind === "body" ? source.address : `fixture:${source.fixture}`) : ""}>
  <div class="pick">
    <label>
      Example world
      <select aria-label="Example world" bind:value={example}>
        {#each LIFE_EXAMPLES as e (e.fixture)}<option value={e.fixture}>{e.label}</option>{/each}
      </select>
    </label>
    {#if bodyNote}<p class="note" data-body-note>{bodyNote}</p>{:else if !subject}<p class="note">Pick a world on the chart to ask for its biosphere; until then an example world is shown.</p>{/if}
  </div>

  {#if failed}
    <p class="note" role="alert">Could not read the biosphere: {failed}</p>
  {:else if view}
    <header class="head">
      <h2>{view.signatures.alive ? "The living world of" : view.signatures.ever_alive ? "Life that was, on" : "Life on"} {name}</h2>
      <span class="stamp" title="How far this rests on physics, models or dials">{view.evidence}</span>
      {#if source?.kind === "example"}<span class="stamp quiet" data-example>example world</span>{/if}
    </header>
    <p class="lede">
      {view.signatures.summary ? `${view.signatures.summary[0]!.toUpperCase()}${view.signatures.summary.slice(1)}.` : view.signatures.alive ? "Alive." : view.signatures.ever_alive ? "Life began here and ended." : "Life never began here."}
      {#if shows}It shows {shows}.{/if}
    </p>

    <div class="lenses" role="group" aria-label="Tell reasons as">
      {#each LENSES as l (l.id)}<button type="button" class:on={lens === l.id} aria-pressed={lens === l.id} onclick={() => (lens = l.id)}>{l.label}</button>{/each}
    </div>

    <div class="lenses" role="group" aria-label="Show">
      <button type="button" class:on={shape === "story"} aria-pressed={shape === "story"} data-show="story" onclick={() => (shape = "story")}>Story</button>
      <button type="button" class:on={shape === "board"} aria-pressed={shape === "board"} data-show="board" onclick={() => (shape = "board")}>Board</button>
    </div>

    {#if shape === "board"}
      <LifeFlow {view} {lens} {dialWords} />
    {:else}
    <section aria-labelledby="tl-h">
      <h3 id="tl-h">Life on {name}, {axis ? myr(axis.to) : ""}: epochs named after the fact</h3>
      {#if view.epochs.length && axis}
        <div class="tlbox">
          <svg viewBox="0 0 {TW} {TH}" width="100%" role="img" aria-label="Epochs, events and production over {myr(axis.to)}">
            {#each bands as b (b.index)}
              <rect class="band" class:major={b.epoch.major} class:picked={picked === b.index} x={b.x0} y={bandTop} width={b.x1 - b.x0} height={bandH} onclick={() => (picked = picked === b.index ? null : b.index)} role="presentation"><title>{b.epoch.name}, {myr(b.epoch.from_myr)} to {myr(b.epoch.to_myr)}</title></rect>
              {#if b.x1 - b.x0 > 70}<text class="bandname" x={(b.x0 + b.x1) / 2} y={bandTop + bandH / 2 + 4}>{b.epoch.name.replace(/^The /, "")}</text>{/if}
            {/each}
            {#if prod.d}<path class="prod" d={prod.d} />{/if}
            {#each gains as e (e.id)}<circle class="gain" cx={axis.x(e.at_myr)} cy={curveTop + curveH + 6} r="2"><title>{myr(e.at_myr)}: {lifeWords(e.subject)}</title></circle>{/each}
            {#each marked as e (e.id)}
              <line class="tick" class:loss={e.kind !== "origin"} x1={axis.x(e.at_myr)} x2={axis.x(e.at_myr)} y1={bandTop - 6} y2={TH - 12}><title>{myr(e.at_myr)}: {e.why}</title></line>
            {/each}
            {#each ticks as t (t)}<text class="axis" class:left={t === 0} x={axis.x(t)} y={TH - 1}>{t === 0 ? "formed" : myr(t)}</text>{/each}
            <text class="axis left" x="8" y="12">{prod.d ? `production, log scale (10^${prod.lo} to 10^${prod.hi} g C/yr)` : "no production"}</text>
          </svg>
        </div>
        <label class="epochpick">
          Epoch
          <select aria-label="Epoch" value={picked === null ? "" : String(picked)} onchange={(e) => (picked = e.currentTarget.value === "" ? null : Number(e.currentTarget.value))}>
            <option value="">Now</option>
            {#each view.epochs as ep, i (i)}<option value={String(i)}>{ep.name} ({myr(ep.from_myr)} to {myr(ep.to_myr)})</option>{/each}
          </select>
        </label>
        <div class="why" aria-live="polite" data-epoch={epoch?.name ?? ""}>
          {#if epoch}
            <b>{epoch.name}</b>, {myr(epoch.from_myr)} to {myr(epoch.to_myr)}{epoch.major ? " · a major epoch" : ""}.
            {epoch.why[lens]}
          {:else if nowEpoch}
            Now: <b>{nowEpoch.name}</b>. {nowEpoch.why[lens]} <span class="hint">Pick an epoch on the timeline or in the list for its story.</span>
          {/if}
        </div>
      {:else}
        <p class="note">No epochs: {view.signatures.ever_alive ? "the kernel has no history for this world's life yet." : "life never began, so there is no history to tell."}</p>
      {/if}
      {#if marked.length}
        <ol class="events">
          {#each marked as e (e.id)}<li data-event={e.kind}><span class="when">{myr(e.at_myr)}</span> {e.why}{e.dials.length ? ` (leans on: ${e.dials.map((d) => dialWords[d] ?? lifeWords(d)).join(", ")})` : ""}</li>{/each}
        </ol>
      {/if}
    </section>

    <div class="cols">
      <section aria-labelledby="web-h" class="webcol">
        <h3 id="web-h">Who feeds whom</h3>
        {#if web.nodes.length}
          <svg viewBox="0 0 {WW} {WH}" width="100%" role="img" aria-label="Food web: {view.guilds.length} guilds over {web.levels} feeding levels">
            {#each web.links as l, i (i)}
              {@const a = at.get(l.from)}
              {@const b = at.get(l.to)}
              {#if a && b}<line class="link" class:recycle={l.recycle} x1={a.x} y1={a.y} x2={b.x} y2={b.y} />{/if}
            {/each}
            {#each web.nodes as n (n.guild.id)}
              <circle class="node {n.part}" class:picked={guildPick === n.guild.id} cx={n.x} cy={n.y} r={n.r} onclick={() => (guildPick = guildPick === n.guild.id ? null : n.guild.id)} role="presentation"><title>{n.guild.plain_name}</title></circle>
            {/each}
          </svg>
          <p class="hint">Filled: makes food. Open: eats. Dashed: recycles the dead. Ringed: lives with others. Bigger discs move more carbon (by order of magnitude); the side column holds recyclers and partners.</p>
          <table class="guilds">
            <caption class="sr">Guilds</caption>
            <thead><tr><th scope="col">Guild</th><th scope="col">Role</th><th scope="col">Lives in</th><th scope="col">Carbon</th></tr></thead>
            <tbody>
              {#each guildsBySize as g (g.id)}
                <tr class:picked={guildPick === g.id} data-guild={g.id}>
                  <th scope="row"><button type="button" class="linkbtn" onclick={() => (guildPick = guildPick === g.id ? null : g.id)}>{g.plain_name}</button></th>
                  <td>{g.role}, level {g.level}</td>
                  <td>{lifeWords(g.habitat_kind).replace(/-/g, " ")}</td>
                  <td>{carbonWords(Math.max(g.production_gc_yr, g.intake_gc_yr))}{g.share > 0 ? ` · ${Math.round(g.share * 100)}% of food` : ""}</td>
                </tr>
              {/each}
            </tbody>
          </table>
          {#if guild}
            <p class="why" data-guild-why>{guild.plain_name}: {guild.physics_name}; {guild.grade}{guild.max_mass_kg ? `, up to about ${Math.round(guild.max_mass_kg).toLocaleString("en-GB")} kg` : ""}{guild.can_fly ? ", flies" : ""}.</p>
          {/if}
        {:else}
          <p class="note">No living guilds.</p>
        {/if}
      </section>

      <section class="side">
        <section aria-labelledby="scan-h">
          <h3 id="scan-h">What a scan would show</h3>
          <dl class="tiers">
            {#each tiers as t (t.tier)}
              <dt data-tier={t.tier}>{t.label}</dt>
              <dd>{#each t.lines as line, i (i)}<span>{line}</span>{/each}</dd>
            {/each}
          </dl>
          <p class="hint">Fed to the players' data views by tier; the GM sees all of it.</p>
        </section>
        {#if view.hooks.length}
          <section aria-labelledby="hooks-h">
            <h3 id="hooks-h">Story hooks</h3>
            {#each view.hooks as h (h.id)}<p class="hook" data-hook={h.id}><b>{h.text}.</b> {h.why[lens]}</p>{/each}
          </section>
        {/if}
        {#if traitRows.length}
          <section aria-labelledby="traits-h">
            <h3 id="traits-h">Traits</h3>
            <dl class="facts">{#each traitRows as r (r.label)}<dt>{r.label}</dt><dd>{r.text}</dd>{/each}</dl>
          </section>
        {/if}
        {#if Object.keys(view.dials).length}
          <section aria-labelledby="dials-h">
            <h3 id="dials-h">The unknowns: dials</h3>
            <dl class="facts">{#each Object.entries(view.dials) as [k, v] (k)}<dt>{dialWords[k] ?? lifeWords(k)}</dt><dd>{v === 1 ? "Earth-like pace" : `${v} × Earth's pace`}</dd>{/each}</dl>
            <p class="hint">Nobody knows these. Results that lean on a dial say so; the universe pack sets them.</p>
          </section>
        {/if}
      </section>
    </div>
    {/if}
  {:else}
    <p class="note">Reading the biosphere…</p>
  {/if}
</div>

<style>
  .bio {
    color: var(--sse-ink);
    font-size: 13px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 1100px;
  }
  .pick {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: center;
  }
  .pick label {
    display: inline-flex;
    gap: 6px;
    align-items: center;
  }
  select {
    font: inherit;
    background: var(--sse-ground);
    color: var(--sse-ink);
    border: 1px solid var(--sse-rule);
  }
  .note,
  .hint {
    color: var(--sse-ink-muted);
    margin: 0;
  }
  .hint {
    font-size: 12px;
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: baseline;
  }
  h2 {
    font-family: var(--sse-font-display, serif);
    font-size: 24px;
    margin: 0;
    font-weight: 600;
  }
  h3 {
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    margin: 12px 0 6px;
    font-weight: 500;
  }
  .stamp {
    border: 1.5px solid var(--sse-accent);
    color: var(--sse-accent-ink);
    font-size: 10px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    padding: 1px 6px;
  }
  .stamp.quiet {
    border-color: var(--sse-rule);
    color: var(--sse-ink-muted);
  }
  .lede {
    margin: 0;
    font-size: 14px;
  }
  .lenses {
    display: inline-flex;
  }
  .lenses button {
    font: inherit;
    padding: 4px 12px;
    border: 1px solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    margin-right: -1px;
  }
  .lenses button.on {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .epochpick {
    display: inline-flex;
    gap: 6px;
    align-items: center;
    margin-top: 4px;
  }
  .band {
    fill: var(--sse-ground-raised, var(--sse-ground));
    stroke: var(--sse-ink);
    stroke-width: 0.6;
    cursor: pointer;
  }
  .band.major {
    fill: color-mix(in srgb, var(--sse-accent) 14%, var(--sse-ground));
  }
  .band.picked {
    fill: color-mix(in srgb, var(--sse-accent) 36%, var(--sse-ground));
  }
  .bandname {
    font-size: 10px;
    text-anchor: middle;
    fill: var(--sse-ink);
    pointer-events: none;
  }
  .prod {
    fill: none;
    stroke: var(--sse-ink);
    stroke-width: 1.4;
  }
  .gain {
    fill: var(--sse-ink-muted);
  }
  .tick {
    stroke: var(--sse-ink);
    stroke-width: 1;
  }
  .tick.loss {
    stroke: var(--sse-accent);
    stroke-dasharray: 3 2;
  }
  .axis {
    font-size: 9px;
    fill: var(--sse-ink-muted);
    text-anchor: middle;
    font-family: var(--sse-font-mono, monospace);
  }
  .axis.left {
    text-anchor: start;
  }
  .why {
    margin: 4px 0 0;
    min-height: 2.6em;
    line-height: 1.5;
  }
  .events {
    margin: 8px 0 0;
    padding-left: 18px;
    line-height: 1.5;
  }
  .when {
    font-family: var(--sse-font-mono, monospace);
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .cols {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr);
    gap: 24px;
  }
  @media (max-width: 900px) {
    .cols {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .link {
    stroke: var(--sse-ink-muted);
    stroke-width: 0.8;
  }
  .link.recycle {
    stroke-dasharray: 2 3;
  }
  .node {
    stroke: var(--sse-ink);
    stroke-width: 1.2;
    cursor: pointer;
  }
  .node.makes {
    fill: var(--sse-ink);
  }
  .node.eats {
    fill: var(--sse-ground);
  }
  .node.recycles {
    fill: var(--sse-ground);
    stroke-dasharray: 3 2;
  }
  .node.with {
    fill: var(--sse-ground);
    stroke-width: 2.4;
  }
  .node.picked {
    stroke: var(--sse-accent);
    stroke-width: 3;
  }
  table {
    border-collapse: collapse;
    width: 100%;
    margin-top: 8px;
  }
  th,
  td {
    text-align: left;
    padding: 3px 6px 3px 0;
    border-bottom: 1px dotted var(--sse-rule);
    font-weight: normal;
    vertical-align: top;
  }
  thead th {
    color: var(--sse-ink-muted);
    font-size: 11px;
  }
  tr.picked th,
  tr.picked td {
    background: color-mix(in srgb, var(--sse-accent) 12%, transparent);
  }
  .linkbtn {
    font: inherit;
    background: none;
    border: 0;
    padding: 0;
    color: var(--sse-ink);
    text-decoration: underline dotted;
    cursor: pointer;
    text-align: left;
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
  .tiers,
  .facts {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 4px 12px;
    margin: 0;
  }
  .tiers dd,
  .facts dd {
    margin: 0;
  }
  .tiers dd span {
    display: block;
  }
  dt {
    color: var(--sse-ink-muted);
  }
  .hook {
    margin: 0 0 6px;
    line-height: 1.5;
  }
</style>
