<!--
  Sector desk (X3c on the chart room): the campaign's mapped systems on the
  stacked pack's hex map, the picked world's code sheet beside it with a way
  into its system, and the sector file in and out (import with a preview first,
  export as a download). Every word, grid and glyph comes from the pack.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { SECTOR_FORMATS, fetchMetadata, fetchSector, travelPens, placeHash, viewOptions, DEFAULT_VIEW, type Json, type SectorFormat, type SectorImported, type SectorMap, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { opensFrom, penOf } from "../../../chart-grammar/src/index.ts";
  import { blockHexes, label as hexLabel } from "../../hex-map/src/grid.ts";
  import HexMap from "../../hex-map/src/HexMap.svelte";
  import WorldCodes from "../../world-codes/src/WorldCodes.svelte";
  import { sift } from "../../hex-map/src/sift.ts";
  import { inputFromPacks, sheetOf, type CodesResponse, type WorldCodesInput } from "../../world-codes/src/sheet.ts";
  import { SECTOR_CONTROLS } from "./controls.ts";
  import { blocksInUse, exportName, hexConfigOf, mapWorlds, openingBlock, rollEveryWorld, rollTallyWords, sceneOf } from "./desk.ts";

  let { ctx, params }: { ctx: ViewContext; params: Record<string, Json> } = $props();
  const session = untrack(() => ctx.session);
  const ws = $derived(ctx.workspace);
  // The shared view controls (V10a): on the Sector map the grid is the map, and HUD (on by default) carries its labels and overlays.
  const hud = $derived(viewOptions(SECTOR_CONTROLS, ws.view ?? DEFAULT_VIEW).shown.hud);
  const gm = session.seat.authority === "gm";

  const packs = $derived.by(() => {
    void ws.revision;
    const s = session.stack();
    return [...s.base, ...(s.campaign ? [s.campaign] : [])] as unknown[];
  });
  const sheet = $derived(sheetOf(packs));
  const config = $derived(hexConfigOf(packs));

  let map = $state<SectorMap | null>(null);
  let failed = $state<string | null>(null);
  let atS = $state<bigint>(0n);
  let block = $state<number | undefined>(undefined);
  let blockChosen = false;
  let selected = $state<string | undefined>(undefined);
  let reach = $state(untrack(() => Number(params.reach ?? 2)));
  let sifter = $state<string>("");

  async function load() {
    if (!sheet || !config) return;
    const now = await session.campaignNow();
    atS = now.ok && now.at_s !== undefined ? now.at_s : 0n;
    const r = await session.sectorMap?.({ scope: session.universe.root, at_s: atS });
    if (!r) return void (failed = "This seat cannot read the sector.");
    if (!r.ok) return void (failed = r.message);
    failed = null;
    map = r.result;
    if (!blockChosen) block = openingBlock(config, mapWorlds(r.result));
  }
  $effect(() => {
    void ws.revision;
    void sheet;
    untrack(() => void load());
  });

  const worlds = $derived(map ? mapWorlds(map) : []);
  const blocks = $derived(config ? blocksInUse(config, worlds) : []);
  const unit = $derived(config?.grid.hex_distance.unit ?? "hex");
  const sifted = $derived.by(() => {
    const s = config?.sifters?.find((x) => x.id === sifter);
    return s ? new Set(sift(s, worlds)) : null;
  });
  // Pencil and ink (SV3): on the GM's Build and Run desks a world the table does not know is drawn in pencil. Only the
  // shown block's worlds are asked about.
  let pens = $state.raw<Record<string, "ink" | "heard" | "pencil">>({});
  let pensGen = 0;
  $effect(() => {
    void ws.revision;
    const gmDesk = gm && (ws.desk === "build" || ws.desk === "run");
    const shown = config && block !== undefined ? new Set(blockHexes(config.grid, block).map((h) => hexLabel(config.grid, h))) : null;
    const asked = worlds.filter((w) => w.address && (!shown || shown.has(w.hex)));
    const g = ++pensGen;
    if (!gmDesk || !asked.length) return void (pens = {});
    untrack(() =>
      void travelPens(session, asked.map((w) => w.address!)).then((r) => {
        if (g !== pensGen) return;
        const out: Record<string, "ink" | "heard" | "pencil"> = {};
        for (const w of asked) {
          const l = r.places.get(w.address!);
          if (l && l !== "known") out[w.hex] = penOf(l);
        }
        pens = out;
      }),
    );
  });
  const scene = $derived(config ? { ...sceneOf(config, sifted ? worlds.filter((w) => sifted.has(w.hex)) : worlds, { ...(block !== undefined ? { block } : {}), ...(selected ? { selected, range: reach } : {}), title: map?.worlds[0]?.sector || "Sector" }), pens } : null);
  const row = $derived(map?.worlds.find((w) => w.hex === selected) ?? null);

  // The picked world's sheet, read for its own body with the system as context.
  let sheetInput = $state<WorldCodesInput | null>(null);
  $effect(() => {
    const r = row;
    sheetInput = null;
    if (!r) return;
    untrack(() => {
      session.codes?.({ scope: r.body, at_s: atS }).then((c) => {
        if (!c.ok || row?.body !== r.body) return;
        sheetInput = inputFromPacks(packs, c.result as unknown as CodesResponse, { selected: r.body, names: { [r.body]: r.name }, title: r.name });
      });
    });
  });

  // ---- Import: text in, a preview, then written as history ----
  let importOpen = $state(false);
  let importBtn = $state<HTMLElement | null>(null);
  let text = $state("");
  let fetchName = $state("");
  let fetchSub = $state("");
  let attribution = $state<string | null>(null);
  let preview = $state<SectorImported | null>(null);
  let grow = $state(false);
  /** The sector's metadata XML (fetched with the sector, or chosen): its routes become links. */
  let metadata = $state<string | null>(null);
  let routesKnown = $state(false);
  let busy = $state<string | null>(null);
  let importNote = $state<string | null>(null);

  async function readFile(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (f) ((text = await f.text()), (preview = null), (attribution = null), (metadata = null));
  }
  async function readMetadata(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (f) ((metadata = await f.text()), (preview = null));
  }
  async function fetchIt() {
    if (!fetchName.trim()) return;
    busy = "Fetching…";
    try {
      const got = await fetchSector((u) => fetch(new URL(u, document.baseURI)), fetchName.trim(), fetchSub.trim() || undefined);
      text = got.text;
      attribution = got.attribution;
      metadata = await fetchMetadata((u) => fetch(new URL(u, document.baseURI)), fetchName.trim());
      preview = null;
      importNote = null;
    } catch (err) {
      importNote = (err as Error).message;
    }
    busy = null;
  }
  async function previewIt() {
    if (!text.trim()) return;
    busy = "Reading…";
    const r = await session.importSector?.({ text, ...(metadata ? { metadata } : {}), at_s: atS });
    busy = null;
    if (!r) return;
    if (!r.ok) return void ((importNote = r.message), (preview = null));
    importNote = null;
    preview = r.result;
  }
  async function applyIt() {
    if (!preview) return;
    busy = grow ? "Growing each system and writing it as history… (this can take a while)" : "Writing the worlds as history…";
    const r = await session.importSector?.({ text, ...(metadata ? { metadata, route_audience: routesKnown ? "everyone" : "gm" } : {}), at_s: atS, apply: true, grow });
    busy = null;
    if (!r) return;
    if (!r.ok) return void (importNote = r.message);
    const linked = (r.result.routes ?? []).filter((x) => x.link).length;
    importNote = `${r.result.summary}. Written as ${r.result.events.length} events in the history${linked ? `, and ${linked} route${linked === 1 ? "" : "s"} as links` : ""}.`;
    preview = null;
    blockChosen = false;
    ws.revision++;
    await load();
  }

  // ---- Export: a download ----
  let exportOpen = $state(false);
  let exportBtn = $state<HTMLElement | null>(null);
  let format = $state<SectorFormat>("t5_tab");
  let exportNote = $state<string | null>(null);
  async function exportIt() {
    const r = await session.exportSector?.({ scope: session.universe.root, format, at_s: atS });
    if (!r) return;
    if (!r.ok) return void (exportNote = r.message);
    const ext = SECTOR_FORMATS.find((f) => f.id === format)?.ext ?? "txt";
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([r.result.text], { type: "text/plain" }));
    a.download = exportName(map, ext);
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
    exportNote = `${r.result.written} world${r.result.written === 1 ? "" : "s"} written${r.result.skipped.length ? `; ${r.result.skipped.length} mapped system${r.result.skipped.length === 1 ? "" : "s"} skipped (no main world)` : ""}.`;
  }

  const counts = (p: SectorImported) => p.import.summary;

  // WBH next: roll a pack's program for every mapped world in one ask. Codes already set stay; only the unset roll.
  type RollDef = { id: string; label?: string; about?: string; sheet: string };
  const programs = $derived(packs.flatMap((p) => ((p as { provides?: { other?: { rolls?: RollDef[] } } })?.provides?.other?.rolls ?? [])));
  let rollOpen = $state(false);
  let rollBtn = $state<HTMLElement | null>(null);
  let rollProgram = $state<string | undefined>(undefined);
  let rollNote = $state<string | null>(null);
  async function rollAll() {
    const program = rollProgram ?? programs[0]?.id;
    if (!program || !map || !session.rollProfile) return;
    const bodies = [...new Set(map.worlds.map((w) => w.body))];
    busy = `Rolling ${bodies.length} world${bodies.length === 1 ? "" : "s"}…`;
    rollNote = null;
    const t = await rollEveryWorld((a) => session.rollProfile!(a), program, bodies, atS);
    busy = null;
    rollNote = rollTallyWords(t);
    if (t.written) ws.revision++;
  }
</script>

<div class="desk" role="region" aria-label="Sector">
  {#if !sheet || !config}
    <p class="lede">No pack in the stack gives a code sheet and a hex map. Add a game pack under Workshop, Packs, and its sector shows here.</p>
  {:else}
    <div class="bar">
      <label class="lbl">Show
        <select bind:value={block} onchange={() => (blockChosen = true)} data-block>
          <option value={undefined}>The whole sheet</option>
          {#each blocks as b}<option value={b.index}>{b.name} ({b.worlds} world{b.worlds === 1 ? "" : "s"})</option>{/each}
        </select>
      </label>
      {#if config.sifters?.length}
        <label class="lbl">Find
          <select bind:value={sifter} data-sifter>
            <option value="">Every world</option>
            {#each config.sifters as s}<option value={s.id} title={s.explain}>{s.label}</option>{/each}
          </select>
        </label>
      {/if}
      {#if selected}
        <label class="lbl">Reach <input type="number" min="1" max="9" bind:value={reach} data-reach /> {unit}</label>
      {/if}
      <span class="grow"></span>
      {#if gm}
        <button type="button" class="quiet-btn" bind:this={importBtn} aria-expanded={importOpen} onclick={() => ((importOpen = !importOpen), (exportOpen = false), (rollOpen = false))} data-import>Import a sector…</button>
        <button type="button" class="quiet-btn" bind:this={exportBtn} aria-expanded={exportOpen} disabled={!worlds.length} onclick={() => ((exportOpen = !exportOpen), (importOpen = false), (rollOpen = false))} data-export>Export…</button>
        {#if programs.length && session.rollProfile}
          <button type="button" class="quiet-btn" bind:this={rollBtn} aria-expanded={rollOpen} disabled={!worlds.length} onclick={() => ((rollOpen = !rollOpen), (importOpen = false), (exportOpen = false))} data-roll-all>Roll every world…</button>
        {/if}
      {/if}
    </div>

    {#if importOpen}
      <section class="pop" aria-label="Import a sector" use:opensFrom={() => importBtn}>
        <p class="small">Paste a sector file (tab-delimited, SEC columns or one line per world), choose one, or fetch one by name. You see what it gives before anything is written.</p>
        <textarea rows="6" bind:value={text} placeholder="Hex  Name  UWP …" spellcheck="false" data-import-text oninput={() => (preview = null)}></textarea>
        <div class="row">
          <label class="quiet-btn file">Choose a file <input type="file" accept=".tab,.sec,.txt,text/plain" onchange={readFile} /></label>
          <label class="quiet-btn file" title="The sector's metadata file (XML): its routes become links">{metadata ? "Routes file chosen" : "Routes (XML)…"} <input type="file" accept=".xml,application/xml,text/xml" onchange={readMetadata} data-import-metadata /></label>
          <input type="text" bind:value={fetchName} placeholder="Sector name" aria-label="Sector name to fetch" />
          <input type="text" bind:value={fetchSub} placeholder="Subsector (A to P)" aria-label="Subsector" size="10" />
          <button type="button" class="quiet-btn" onclick={fetchIt} disabled={!fetchName.trim() || !!busy}>Fetch</button>
          <button type="button" class="go" onclick={previewIt} disabled={!text.trim() || !!busy} data-preview>Preview</button>
        </div>
        {#if attribution}<p class="small muted">{attribution}</p>{/if}
        {#if busy}<p class="small" aria-live="polite">{busy}</p>{/if}
        {#if importNote}<p class="small" aria-live="polite" data-import-note>{importNote}</p>{/if}
        {#if preview}
          {@const c = counts(preview)}
          <div class="preview" data-preview-result>
            <p><b>{c.imported}</b> of {c.lines} lines read{c.refused ? `, ${c.refused} refused` : ""}. {c.impossible ? `${c.impossible} with values physics does not allow (kept and marked, never altered). ` : ""}{c.guessed ? `${c.guessed} needed a guess. ` : ""}{c.differ ? `${c.differ} list codes the physics reads differently.` : ""}</p>
            <ul class="worlds">
              {#each preview.import.worlds as w}
                <li class:refused={!!w.refused}>
                  <span class="mono">{w.hex}</span> {w.name}
                  {#if w.refused}<span class="muted"> · not imported: {w.refused}</span>{/if}
                  {#if w.world_edits.some((e) => e.impossible)}<span class="mark"> · impossible values kept</span>{/if}
                  {#if w.guessed.length}<span class="muted" title={w.guessed.join("; ")}> · {w.guessed.length} guess{w.guessed.length === 1 ? "" : "es"}</span>{/if}
                  {#if w.differs.length}<span class="muted" title={w.differs.map((d) => `${d.group}: listed ${d.listed_only.join(" ") || "nothing more"}, physics reads ${d.read_only.join(" ") || "nothing more"}`).join("; ")}> · differs in {w.differs.map((d) => d.group).join(", ")}</span>{/if}
                </li>
              {/each}
            </ul>
            {#if preview.routes?.length}
              {@const kept = preview.routes.filter((x) => !x.left_out)}
              <p class="small" data-preview-routes><b>{kept.length}</b> route{kept.length === 1 ? "" : "s"} become{kept.length === 1 ? "s" : ""} links between worlds{preview.routes.length > kept.length ? `; ${preview.routes.length - kept.length} left out` : ""}.</p>
              {#if preview.routes.length > kept.length}
                <ul class="worlds">
                  {#each preview.routes.filter((x) => x.left_out) as x}<li class="refused"><span class="mono">{x.start}–{x.end}</span><span class="muted"> · {x.left_out}</span></li>{/each}
                </ul>
              {/if}
              <label class="tick"><input type="checkbox" bind:checked={routesKnown} data-routes-known /> The players know these routes (otherwise only you, until you tell them)</label>
            {/if}
            <label class="tick"><input type="checkbox" bind:checked={grow} /> Grow each world's whole system first (its star, giants and belts to match the codes; slow for a whole sector)</label>
            <button type="button" class="go" onclick={applyIt} disabled={!!busy || !c.imported} data-apply>Write {c.imported} world{c.imported === 1 ? "" : "s"} as history</button>
          </div>
        {/if}
      </section>
    {/if}

    {#if exportOpen}
      <section class="pop" aria-label="Export the sector" use:opensFrom={() => exportBtn}>
        <p class="small">Every mapped system's main world, read from its physics now, as a sector file.</p>
        <div class="row">
          <select bind:value={format} aria-label="File format">
            {#each SECTOR_FORMATS as f}<option value={f.id}>{f.label}</option>{/each}
          </select>
          <button type="button" class="go" onclick={exportIt} data-download>Download</button>
        </div>
        {#if exportNote}<p class="small" aria-live="polite" data-export-note>{exportNote}</p>{/if}
      </section>
    {/if}

    {#if rollOpen}
      <section class="pop" aria-label="Roll every world" use:opensFrom={() => rollBtn}>
        <p class="small">Rolls the codes the physics cannot decide for every mapped world, from the universe's seed (the same dice on every machine). Codes already set, by hand or by an imported profile (bases included), stay as they are.</p>
        <div class="row">
          {#if programs.length > 1}
            <select bind:value={rollProgram} aria-label="What to roll">
              {#each programs as p, i}<option value={i === 0 ? undefined : p.id}>{p.label ?? p.id}</option>{/each}
            </select>
          {/if}
          <button type="button" class="go" onclick={rollAll} disabled={!!busy} title={(programs.find((p) => p.id === (rollProgram ?? programs[0]?.id)) ?? programs[0])?.about} data-roll-all-go>Roll and write for {worlds.length} world{worlds.length === 1 ? "" : "s"}</button>
        </div>
        {#if busy}<p class="small" aria-live="polite">{busy}</p>{/if}
        {#if rollNote}<p class="small" aria-live="polite" data-roll-all-note>{rollNote}</p>{/if}
      </section>
    {/if}

    {#if failed}<p class="warn">{failed}</p>{/if}

    <div class="body">
      <div class="map">
        {#if scene}
          {#if !worlds.length}
            <p class="lede">No mapped systems yet. Import a sector, and each world lands on its hex with its physics behind it.</p>
          {:else}
            <HexMap {scene} {hud} params={params as Record<string, unknown>} onpick={(h) => (selected = selected === h ? undefined : h)} />
          {/if}
        {/if}
      </div>
      {#if row}
        <aside class="side" aria-label={row.name} use:opensFrom={() => document.querySelector(`[aria-label="hex ${row.hex}"]`)}>
          <header>
            <h3>{row.name} <span class="mono muted">{row.hex}</span></h3>
            <nav class="links">
              <a href={placeHash({ scale: "system", system: row.system })} data-open-system>Open the system</a>
              <a href={placeHash({ scale: "world", system: row.system, world: row.body })} data-open-world>Open the world</a>
            </nav>
          </header>
          {#if sheetInput}
            <WorldCodes scene={sheetInput} params={{ mode: "sheet" }} />
          {:else}
            <p class="small muted">Reading its codes…</p>
          {/if}
        </aside>
      {/if}
    </div>
  {/if}
</div>

<style>
  .desk { color: var(--sse-ink); font-size: 13px; display: flex; flex-direction: column; gap: 12px; }
  .bar, .row { display: flex; flex-wrap: wrap; gap: 8px 12px; align-items: center; }
  .grow { flex: 1; }
  .lbl { font-family: var(--sse-font-mono, monospace); display: inline-flex; gap: 6px; align-items: center; }
  select, input[type="text"], input[type="number"], textarea { font: inherit; min-height: 34px; color: var(--sse-ink); background: var(--sse-ground); border: 1px solid var(--sse-rule); }
  input[type="number"] { width: 4em; }
  textarea { width: 100%; box-sizing: border-box; font-family: var(--sse-font-mono, monospace); font-size: 12px; }
  .quiet-btn, .go { font: inherit; min-height: 34px; padding: 0 10px; border: 1px solid var(--sse-rule); background: transparent; color: var(--sse-ink); cursor: pointer; display: inline-flex; align-items: center; transition: transform var(--sse-motion-quick, 120ms) var(--sse-ease, ease-out); }
  .quiet-btn:hover, .go:hover { transform: translateY(-1px); border-color: var(--sse-ink); }
  .go { background: var(--sse-ink); color: var(--sse-ground); border-color: var(--sse-ink); }
  button:disabled { opacity: 0.5; cursor: default; transform: none; }
  .file input { display: none; }
  .pop { border: 1px solid var(--sse-rule); background: var(--sse-ground); padding: 12px; display: flex; flex-direction: column; gap: 8px; }
  .preview { border-top: 1px solid var(--sse-rule); padding-top: 8px; display: flex; flex-direction: column; gap: 8px; }
  .worlds { list-style: none; margin: 0; padding: 0; max-height: 14em; overflow: auto; }
  .worlds li { padding: 2px 0; border-bottom: 1px solid var(--sse-rule); }
  .refused { opacity: 0.6; }
  .mark { color: var(--sse-accent); }
  .tick { display: inline-flex; gap: 6px; align-items: center; }
  .body { display: grid; grid-template-columns: minmax(0, 1fr); gap: 16px; }
  .body:has(.side) { grid-template-columns: minmax(0, 3fr) minmax(260px, 2fr); }
  .side { border-left: 1px solid var(--sse-rule); padding-left: 16px; min-width: 0; }
  .side h3 { margin: 0 0 4px; font-size: 15px; }
  .links { display: flex; gap: 12px; margin-bottom: 8px; }
  .links a { color: var(--sse-accent); }
  .mono { font-family: var(--sse-font-mono, monospace); }
  .lede { font-size: 14px; max-width: 46em; }
  .small { font-size: 12px; margin: 0; }
  .muted { color: var(--sse-ink-muted); }
  .warn { color: var(--sse-alert, var(--sse-accent)); }
  @media (max-width: 720px) {
    .body:has(.side) { grid-template-columns: minmax(0, 1fr); }
    .side { border-left: none; border-top: 1px solid var(--sse-rule); padding: 12px 0 0; }
  }
  @media (prefers-reduced-motion: reduce) {
    .quiet-btn, .go { transition: none; }
  }
</style>
