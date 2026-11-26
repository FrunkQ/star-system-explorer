<!--
  Make a system (V13, milestone M2): import a V3 save, or grow a new system
  with the formation pipeline. Both kernel ops are pure, so this is a
  preview: the campaign's universe and state hash do not move. The preview
  chart and card are the ordinary view modules, mounted on a workspace of
  their own in view mode. "Add to the campaign" (part 2) writes the
  previewed save or run into the universe as history with the kernel's
  adopt op; the chart then reads the system back from the universe.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { roundTo, importedSystems, noWorldsWhy, reportSummary, resolveParams, sceneFromFormation, sceneFromImport, worldWords, type FormationRan, type FormationRequest, type SystemScene, type V3Imported, type ViewContext, type ViewHandle, type ViewModule, type ViewSession } from "../../ui-contract/src/index.ts";
  import { systemChart } from "../../views/system-chart/src/index.ts";
  import { bodyCard } from "../../views/body-card/src/index.ts";
  import FormationReplay from "../../views/system-chart/src/FormationReplay.svelte";
  import { createWorkspace } from "./workspace.svelte.ts";
  import StarGallery from "../../views/reference/src/StarGallery.svelte";
  import { STAND_IN, tilesFromScheme, type GalleryStar, type StarTile } from "../../views/reference/src/star-gallery.ts";
  import { classesOnStack, className, sayNumber, type PackDoc } from "../../ui-contract/src/index.ts";

  let { session, context, onadopted, start = "import", view, onview, file = null }: { file?: { text: string; name: string } | null; view?: import("../../ui-contract/src/index.ts").ViewChoice; onview?: (c: import("../../ui-contract/src/index.ts").ViewChoice) => void; session: ViewSession; context: Omit<ViewContext, "workspace" | "params" | "session">; onadopted?: (systems: { system: string; title: string }[], note: string, state_hash: string) => void; start?: "import" | "generate" } = $props();

  let tab = $state<"import" | "generate">(untrack(() => start));
  let busy = $state(false);
  let problem = $state<string | null>(null);

  // ── Import ──
  let imported = $state<V3Imported | null>(null);
  let fileName = $state("");
  let pick = $state("");
  async function read(text: string, name: string) {
    busy = true;
    problem = null;
    const r = await session.importV3(text);
    busy = false;
    if (!r.ok) return void ((problem = r.code === "import_failed" ? `That file is not a V3 save the importer can read (${r.message}).` : r.message), (imported = null));
    imported = r.result;
    importText = text;
    fileName = name;
    pick = importedSystems(r.result)[0]?.address ?? "";
  }
  // A V3 save handed over by the one Open door is read straight away.
  $effect(() => {
    const f = file;
    if (f) untrack(() => void read(f.text, f.name));
  });
  async function onFile(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (!f) return;
    if (/\.zip$/i.test(f.name)) return void (problem = "Unpack the .sse.zip first and choose the starmap.json or system.json inside it.");
    await read(await f.text(), f.name);
  }
  async function sample() {
    const sampleUrl = String("samples/synthetic-sol.json");
    const t = await (await fetch(new URL(sampleUrl, document.baseURI))).text();
    await read(t, "synthetic-sol.json (sample)");
  }
  const systems = $derived(imported ? importedSystems(imported) : []);
  const summary = $derived(imported ? reportSummary(imported.report) : null);
  const reportLink = $derived(imported ? URL.createObjectURL(new Blob([JSON.stringify(imported.report, null, 1)], { type: "application/json" })) : null);
  $effect(() => {
    const l = reportLink;
    return () => l && URL.revokeObjectURL(l);
  });

  // ── Generate ──
  let seed = $state("first light");
  let pinMass = $state(true);
  let mass = $state(1);
  let block = $state<"modern" | "classic" | "classic-corrected">("modern");
  let grown = $state<FormationRan | null>(null);
  let grownSeed = $state("");
  // The formation replay (for "how it formed"): opened over the whole page, from the run already grown.
  let replayOpen = $state(false);
  let replayDlg = $state<HTMLDialogElement | undefined>();
  $effect(() => {
    if (!replayDlg) return;
    if (replayOpen && !replayDlg.open) (replayDlg.showModal(), replayDlg.querySelector<HTMLElement>(".replay")?.focus());
    else if (!replayOpen && replayDlg.open) replayDlg.close();
  });
  let grownReq = $state<FormationRequest | null>(null);

  // ── The star, picked from the star gallery (SC1, Alex 26 Nov: "That gallery could be the picker") ──
  // Its mass, age and metals then hold in the formation run ("work out what happened from its birth to now"), or it
  // stands alone in a new system for the game master to fill by hand.
  let picked = $state<GalleryStar | null>(null);
  let galleryOpen = $state(false);
  let galleryDlg = $state<HTMLDialogElement | undefined>();
  $effect(() => {
    if (!galleryDlg) return;
    if (galleryOpen && !galleryDlg.open) galleryDlg.showModal();
    else if (!galleryOpen && galleryDlg.open) galleryDlg.close();
  });
  // The star classes from the base atlas's star scheme on the stack, else the shipped atlas, else the stand-in list.
  let starTiles = $state.raw<readonly StarTile[]>(STAND_IN);
  $effect(() => {
    if (!galleryOpen) return;
    untrack(() => void starClassTiles().then((t) => (starTiles = t)));
  });
  async function starClassTiles(): Promise<readonly StarTile[]> {
    const info = session.stack();
    const on = classesOnStack([...info.base, ...(info.campaign ? [info.campaign] : [])]).schemes.find((s) => s.id === "star");
    if (on) return tilesFromScheme(on, (c) => className(c));
    // The shipped atlas as the shell serves it (scripts/build-shell.sh), as the Classes page offers it.
    const atlasUrl = String("packs/base-atlas.json");
    const doc = await fetch(new URL(atlasUrl, document.baseURI)).then((r) => (r.ok ? (r.json() as Promise<PackDoc>) : null)).catch(() => null);
    const shipped = doc ? classesOnStack([doc]).schemes.find((s) => s.id === "star") : null;
    return shipped ? tilesFromScheme(shipped, (c) => className(c)) : STAND_IN;
  }
  function pickStar(g: GalleryStar) {
    picked = g;
    pinMass = true;
    mass = g.ask.mass_msun;
    galleryOpen = false;
    grown = null;
  }
  const pickedCode = $derived((picked?.desc.now as { class?: { code?: string } } | undefined)?.class?.code ?? null);
  /** The picked star's pins: its mass, alone, at its age and metals. */
  const pickedPins = $derived(picked ? { star_pins: { primary_mass: picked.ask.mass_msun, count: 1 }, inherit_pins: { age_gyr: picked.ask.age_yr / 1e9, feh: picked.ask.feh ?? picked.desc.feh } } : {});

  // "Fill it in myself": a system holding only the picked star, added to the campaign at once, for the game master to
  // add worlds to by hand (Add here). Written through the V3 importer's door, the one way an authored system enters
  // the chronicle today: a single star node with its mass, radius, temperature and the system's age.
  async function addStarAlone() {
    if (!picked) return;
    const n = picked.desc.now;
    const name = seed.trim() || picked.tile.name;
    const id = slug(name);
    const v3 = { id: `sys-${id}`, name, seed: id, epochT0: 0, age_Gyr: picked.ask.age_yr / 1e9, nodes: [{ id: "star", name: `${name} A`, parentId: null, kind: "body", roleHint: "star", massKg: n.mass_kg, radiusKm: n.radius_m / 1000, temperatureK: n.teff_k, ...(pickedCode ? { classes: [`star/${pickedCode.replace(/\s+/g, "")}`] } : {}) }] };
    busy = true;
    problem = null;
    const r = await session.adopt({ import_v3: JSON.stringify(v3) });
    busy = false;
    if (!r.ok) return void (problem = r.code === "already_adopted" ? "A system of this name is already in the campaign: change the seed to name it differently." : r.message);
    const list = r.result.systems.map((x) => ({ system: x, title: name }));
    const note = `Added ${name} to the campaign: its star alone, ready for you to add worlds by hand.`;
    adoptedNote = note;
    onadopted?.(list, note, r.result.state_hash);
  }
  let importText = "";
  async function grow() {
    busy = true;
    problem = null;
    const req: FormationRequest = { seed, block, system: `${universeRoot}/sys:@${slug(seed)}`, ...(picked ? pickedPins : pinMass ? { star_pins: { primary_mass: Number(mass) } } : {}) };
    const r = await session.runFormation(req);
    busy = false;
    if (!r.ok) return void ((problem = r.message), (grown = null));
    grown = r.result;
    grownReq = req;
    grownSeed = seed;
  }

  // ── Adopt (part 2) ──
  // A grown system takes an address under the campaign's universe named for its seed, so the same seed lands in the same place.
  const universeRoot = $derived(session.universe.root);
  const slug = (t: string) => t.trim().toLowerCase().replace(/[^a-z0-9._:-]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 40) || "seed";
  let adoptedNote = $state<string | null>(null);
  async function adopt() {
    busy = true;
    problem = null;
    const r = tab === "import" ? await session.adopt({ import_v3: importText }) : grownReq ? await session.adopt({ formation: grownReq }) : null;
    busy = false;
    if (!r) return;
    if (!r.ok) return void (problem = r.code === "already_adopted" ? "This system is already in the campaign." : r.message);
    // The import was previewed under its own universe id; adopted, the same systems sit under the campaign's root.
    const tail = (a: string) => a.split("/").slice(1).join("/");
    const names = tab === "import" ? new Map(systems.map((x) => [tail(x.address), x.name])) : new Map([[tail(r.result.systems[0] ?? ""), `Seed "${grownSeed}"`]]);
    const list = r.result.systems.map((x) => ({ system: x, title: names.get(tail(x)) ?? x.split("/").pop()! }));
    const note = `Added ${list.map((x) => x.title).join(", ")} to the campaign: ${r.result.facts} facts in ${r.result.events.length} ${r.result.events.length === 1 ? "event" : "events"} on the chronicle.`;
    adoptedNote = note;
    onadopted?.(list, note, r.result.state_hash);
  }

  // ── Preview ──
  const preview = $derived.by((): SystemScene | null => {
    if (tab === "import") return imported && pick ? sceneFromImport(imported, pick) : null;
    return grown ? sceneFromFormation(grown, grownSeed) : null;
  });
  // V3 belts read as shells (the Oort cloud): said in the import report, because V3 itself drew them as rings.
  const shells = $derived(tab === "import" && preview ? preview.subjects.filter((x) => x.shape === "sphere").map((x) => x.label) : []);
  const ws = createWorkspace(0);
  ws.mode = "view";
  const handles: ViewHandle[] = [];
  $effect(() => {
    const p = preview;
    if (!p) return;
    untrack(() => {
      ws.now_yr = p.now_yr;
      ws.selection = null;
    });
    for (const h of handles) h.update(p);
  });
  function mount(node: HTMLElement, module: ViewModule) {
    const h = module.mount(node, { ...context, session, workspace: ws, params: resolveParams(module.params, {}).params }, preview!);
    handles.push(h);
    return { destroy: () => (handles.splice(handles.indexOf(h), 1), h.dispose()) };
  }
  const fmt = (v: number) => (Math.abs(v) >= 1e5 || (Math.abs(v) < 1e-3 && v !== 0) ? v.toExponential(4) : String(roundTo(v, 6)));
  const pct = (v: number) => `${roundTo(v * 100, 2)} %`;
  const counts = (o: Record<string, number>) => Object.entries(o).map(([k, n]) => (n > 1 ? `${k} (${n})` : k));
</script>

<section class="make" aria-label="Make a system" data-busy={busy} data-preview={preview?.system ?? ""}>
  <h2>Make a system</h2>
  <div class="tabs" role="tablist" aria-label="How">
    <button type="button" role="tab" aria-selected={tab === "import"} onclick={() => (tab = "import")}>Import a V3 save</button>
    <button type="button" role="tab" aria-selected={tab === "generate"} onclick={() => (tab = "generate")}>Grow a new system</button>
  </div>
  <p class="note">A preview: nothing enters the campaign or changes its history until you add it.</p>

  {#if tab === "import"}
    <div class="row">
      <label class="file">V3 save (starmap.json or system.json)<input type="file" accept=".json,application/json,.zip" onchange={onFile} /></label>
      <button type="button" class="linkbtn" onclick={sample}>Try a sample save</button>
    </div>
    {#if imported && summary}
      <div class="report" aria-label="Import report" data-outside={summary.outside.length} data-within={summary.within}>
        <p><b>{fileName}</b>: {summary.line}. Read as a V3 {imported.report.source_kind}{imported.report.v3_app_version ? ` from V3 ${imported.report.v3_app_version}` : ""} by importer {imported.report.importer_version}.</p>
        <p>
          {summary.within} recomputed {summary.within === 1 ? "value agrees" : "values agree"} with V3 within tolerance{#if summary.outside.length}; <b>{summary.outside.length} {summary.outside.length === 1 ? "differs" : "differ"}</b>, listed below{/if}.
          {#if imported.report.honesty_marks.length}{imported.report.honesty_marks.length} {imported.report.honesty_marks.length === 1 ? "body carries" : "bodies carry"} an honesty mark (a value set against the physics in V3).{/if}
        </p>
        {#if shells.length}
          <p data-shells={shells.length}>{shells.join(", ")} {shells.length === 1 ? "is" : "are"} drawn as a sphere round the system, not a ring: V3 gives a belt no shape, and one named for the Oort cloud or starting beyond 1,000 AU is a shell (night-sky design, section 10).</p>
        {/if}
        {#if systems.length > 1}
          <label>System to preview <select bind:value={pick} aria-label="System to preview">{#each systems as s (s.address)}<option value={s.address}>{s.name} ({s.bodies})</option>{/each}</select></label>
        {/if}
        {#if imported.report.differences.length}
          <details open={summary.outside.length > 0}>
            <summary>Recomputed values against V3 ({imported.report.differences.length})</summary>
            <div class="scroll">
              <table>
                <thead><tr><th>Body</th><th>Field</th><th>V3</th><th>V4</th><th>Off by</th><th>Allowed</th><th>From</th></tr></thead>
                <tbody>
                  {#each imported.report.differences as d (d.address + d.field)}
                    <tr data-within={d.within} class:out={!d.within}><td>{d.name}</td><td>{d.field}</td><td>{fmt(d.v3)}</td><td>{fmt(d.v4)}</td><td>{pct(d.relative)}</td><td>{pct(d.tolerance)}</td><td>{d.via}</td></tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </details>
        {/if}
        <details>
          <summary>What was kept, set aside and dropped</summary>
          <dl>
            <dt>Recomputed by V4, not compared</dt><dd>{counts(imported.report.derived_not_compared).join(", ") || "none"}</dd>
            <dt>Kept as unclassified (V4 has no field for it yet)</dt><dd>{counts(imported.report.unclassified).join(", ") || "none"}</dd>
            <dt>Presentation only</dt><dd>{counts(imported.report.presentation).join(", ") || "none"}</dd>
            <dt>Dropped (V3 bookkeeping)</dt><dd>{counts(imported.report.dropped).join(", ") || "none"}</dd>
            {#if imported.report.warnings.length}<dt>Warnings</dt><dd>{imported.report.warnings.join("; ")}</dd>{/if}
          </dl>
        </details>
        {#if reportLink}<a href={reportLink} download="v3-import-report.json" data-report-link>Download the full report</a>{/if}
      </div>
    {/if}
  {:else}
    <div class="row pickrow">
      <button type="button" class="linkbtn" data-star-gallery-open onclick={() => (galleryOpen = true)}>{picked ? "Choose another star…" : "Choose the star from the gallery…"}</button>
      {#if picked}
        <span data-picked={picked.tile.id}><b>{picked.tile.name}</b>{pickedCode ? ` (${pickedCode})` : ""}: born with {sayNumber(picked.ask.mass_msun, 3)} M☉, {sayNumber(picked.ask.age_yr / 1e9, 3)} billion years ago, now {picked.desc.now.label}.</span>
        <button type="button" class="linkbtn" onclick={() => ((picked = null), (grown = null))}>Let the seed choose instead</button>
      {/if}
    </div>
    {#if picked}
      <div class="row">
        <button type="button" class="go" data-grow-picked onclick={grow} disabled={busy || !seed.trim()}>{busy ? "Working it out…" : "Work out what happened since its birth"}</button>
        <button type="button" class="go plain" data-alone-picked onclick={addStarAlone} disabled={busy}>Fill it in myself</button>
      </div>
      <p class="note">Working it out runs the formation pipeline with this star held at its mass, age and metals: the disc, the worlds and what became of them by now. Filling it in yourself adds a system with only this star; add its worlds on the chart.</p>
    {/if}
    <div class="row form">
      <label>{picked ? "Name (and seed)" : "Seed"}<input type="text" bind:value={seed} aria-label="Seed" /></label>
      {#if !picked}
        <label class="check"><input type="checkbox" bind:checked={pinMass} /> Pin the star's mass</label>
        {#if pinMass}<label>Star mass (M☉)<input type="number" min="0.1" max="8" step="0.05" bind:value={mass} aria-label="Star mass" /></label>{/if}
      {/if}
      <label>Formation rule
        <select bind:value={block} aria-label="Formation rule">
          <option value="modern">Modern (pebbles and planetesimals)</option>
          <option value="classic">Classic Accrete</option>
          <option value="classic-corrected">Classic Accrete, corrected</option>
        </select>
      </label>
      {#if !picked}<button type="button" class="go" onclick={grow} disabled={busy || !seed.trim()}>{busy ? "Growing…" : "Grow it"}</button>{/if}
    </div>
    <p class="note">One run of the nine-stage formation pipeline: medium cost, a second or two on most devices. The same seed and settings always grow the same system.</p>
    {#if grown}
      <div class="report" aria-label="Formation result" data-worlds={grown.run.worlds.length}>
        {#if noWorldsWhy(grown)}<p role="note">{noWorldsWhy(grown)}</p>{:else}
          <p><b>{grown.run.worlds.length} {grown.run.worlds.length === 1 ? "world" : "worlds"}</b> round a {preview?.subjects[0]?.subtitle ?? "star"}, {Number(grown.run.history.inheritance.age_gyr.toFixed(2))} Gyr old, [Fe/H] {grown.run.history.inheritance.feh.toFixed(2)}.</p>
          <ol class="worlds">{#each grown.run.worlds as w, i (w.address)}<li>World {i + 1}: {worldWords(w)}</li>{/each}</ol>
        {/if}
        {#each grown.run.notes as n (n)}<p class="note">{n}</p>{/each}
        {#if grown.run.worlds.length}<button type="button" class="go watch" data-watch-form onclick={() => (replayOpen = true)}>Watch it form ›</button>{/if}
        <details>
          <summary>How it formed ({grown.run.history.events.length + grown.run.events.length} events)</summary>
          <ol class="events">
            {#each [...grown.run.history.events, ...grown.run.events].filter((e) => e.text) as e, i (i)}<li><span class="t">{e.t_myr < 1 ? `${Math.round(e.t_myr * 1000)} kyr` : e.t_myr < 1000 ? `${Number(e.t_myr.toPrecision(3))} Myr` : `${Number((e.t_myr / 1000).toPrecision(3))} Gyr`}</span> {e.text}</li>{/each}
          </ol>
        </details>
      </div>
    {/if}
  {/if}

  {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
  {#if preview && (tab === "import" ? imported : grown && grown.run.worlds.length)}
    <div class="row adopt">
      <button type="button" class="go" onclick={adopt} disabled={busy}>{busy ? "Adding…" : tab === "import" ? `Add ${systems.length === 1 ? "this system" : `these ${systems.length} systems`} to the campaign` : "Add this system to the campaign"}</button>
      <span class="note">Written to the chronicle as history, with each value's evidence kept{tab === "import" ? " (authored, catalogued or set against the physics)" : ""}.</span>
    </div>
  {/if}
  {#if adoptedNote}<p role="status" data-adopted>{adoptedNote}</p>{/if}

  <dialog class="gallery-dlg" bind:this={galleryDlg} aria-label="Choose the star" onclose={() => (galleryOpen = false)}>
    {#if galleryOpen}
      <div class="dlg-head"><h2>Choose the star</h2><button type="button" class="linkbtn" onclick={() => (galleryOpen = false)}>Close</button></div>
      <StarGallery {session} tiles={starTiles} onpick={pickStar} pickWords="Make a system round this star" />
    {/if}
  </dialog>

  {#if grown}
    <dialog class="replay-dlg" bind:this={replayDlg} aria-label="How {grownSeed} formed" onclose={() => (replayOpen = false)}>
      {#if replayOpen}<FormationReplay run={grown} title={`Seed "${grownSeed}"`} onclose={() => (replayOpen = false)} {view} {onview} />{/if}
    </dialog>
  {/if}

  {#if preview}
    {#key preview.system + tab}
      <div class="preview" aria-label="Preview">
        <div class="chart" use:mount={systemChart as unknown as ViewModule}></div>
        <div class="card" use:mount={bodyCard as unknown as ViewModule}></div>
      </div>
    {/key}
  {/if}
</section>

<style>
  .gallery-dlg {
    width: min(1180px, calc(100vw - 32px));
    max-height: calc(100vh - 32px);
    padding: 16px;
    border: var(--sse-line, 1px) solid var(--sse-ink);
    background: var(--sse-ground, Canvas);
    color: var(--sse-ink);
  }
  .dlg-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }
  .go.plain {
    background: var(--sse-ground, Canvas);
    color: var(--sse-ink);
  }
  .make {
    display: grid;
    gap: 10px;
    grid-template-columns: minmax(0, 1fr);
    font-family: var(--sse-font-body);
  }
  .make > :global(*),
  .report > :global(*) {
    min-width: 0;
  }
  h2 {
    margin: 0;
    font-family: var(--sse-font-display, inherit);
  }
  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .tabs button {
    font: inherit;
    padding: 6px 12px;
    border: var(--sse-line, 1px) solid var(--sse-ink);
    background: var(--sse-ground, Canvas);
    color: var(--sse-ink);
    cursor: pointer;
  }
  .tabs button[aria-selected="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 12px 18px;
    align-items: end;
  }
  label {
    display: grid;
    gap: 2px;
    font-size: 13px;
  }
  .check {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  input[type="text"],
  input[type="number"],
  select {
    font: inherit;
    padding: 4px 6px;
    background: transparent;
    color: inherit;
    border: 1px solid var(--sse-rule, currentColor);
  }
  .go {
    font: inherit;
    padding: 6px 14px;
    background: var(--sse-ink);
    color: var(--sse-on-ink);
    border: 0;
    cursor: pointer;
  }
  .watch {
    justify-self: start;
  }
  .linkbtn {
    font: inherit;
    background: none;
    border: 0;
    padding: 0;
    color: var(--sse-accent);
    text-decoration: underline;
    cursor: pointer;
  }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .problem {
    margin: 0;
    color: var(--sse-accent);
  }
  .report {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 6px;
    font-size: 13px;
    border-left: 3px solid var(--sse-rule, currentColor);
    padding-left: 10px;
  }
  .report p {
    margin: 0;
    line-height: 1.45;
  }
  .scroll {
    overflow-x: auto;
  }
  table {
    border-collapse: collapse;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }
  th,
  td {
    text-align: left;
    padding: 2px 8px;
    border-bottom: 1px solid var(--sse-rule, currentColor);
    white-space: nowrap;
  }
  tr.out td {
    color: var(--sse-accent);
    font-weight: 600;
  }
  dl {
    margin: 4px 0;
    display: grid;
    gap: 2px;
  }
  dt {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--sse-ink-muted);
  }
  dd {
    margin: 0 0 4px;
  }
  .replay-dlg {
    width: min(1400px, 100vw);
    height: min(920px, 100dvh);
    max-width: 100vw;
    max-height: 100dvh;
    padding: 0;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
  }
  .replay-dlg::backdrop {
    background: color-mix(in srgb, var(--sse-ink) 40%, transparent);
  }
  .worlds,
  .events {
    margin: 0;
    padding-left: 20px;
    font-size: 12px;
    max-height: 180px;
    overflow: auto;
  }
  .t {
    font-variant-numeric: tabular-nums;
    color: var(--sse-ink-muted);
  }
  .preview {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(240px, 1fr);
    gap: 10px;
    min-height: 460px;
  }
  .chart {
    position: relative;
    overflow: hidden;
    min-width: 0;
    min-height: 460px;
    border: var(--sse-line, 1px) solid var(--sse-ink);
  }
  .card {
    min-height: 200px;
    overflow: auto;
  }
  @media (max-width: 720px) {
    .preview {
      grid-template-columns: minmax(0, 1fr);
    }
    .chart {
      min-height: 340px;
    }
  }
</style>
