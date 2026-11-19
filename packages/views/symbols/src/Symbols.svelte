<!--
  Symbols (V12, H8.4A): how classes look on the charts, chosen apart from
  the theme. Pick a grammar (diagrammatic, colour-coded or both), mix them
  channel by channel, and change one class's shape, shading, colour or code;
  every change is kept as the campaign's own symbology on this device. The
  key and the previews under every theme are generated from the symbology,
  so they always match the marks. Presentation only: nothing here reaches
  the kernel.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { BODY_SHAPES, PATTERNS, coloursFromV3Save, objectColourSummary, parseV3Palette, symbologyFromV3Palette, v3SaveText, withV3TagColours, campaignCopy, legendFor, markerFor, resolveSymbology, subjectOf, type Binding, type Channel, type Json, type MarkSubject, type SymbologyDoc, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { HatchDefs, Legend, Mark, Stamp } from "../../../chart-grammar/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const host = $derived(ctx.symbology!);
  const editable = $derived(ctx.workspace.mode === "edit");
  const CAMPAIGN = "campaign.symbology";
  const MIXABLE: Channel[] = ["shape", "fill_pattern", "colour", "glyph"];
  const CHANNEL_NAMES: Record<string, string> = { shape: "Shape", fill_pattern: "Shading", colour: "Colour", glyph: "Code", size: "Size", ring: "Ring", outline: "Outline", tone: "Ink" };

  let all = $state(untrack(() => Boolean(params.all_classes)));
  const registry = $derived(host.registry());
  const marker = $derived(host.marker());
  const chosen = $derived(registry.find((d) => d.id === marker.id)!);
  const bases = $derived(registry.filter((d) => d.id !== CAMPAIGN));
  const placeholder = (d: SymbologyDoc | undefined): string | null => {
    if (!d) return null;
    if (typeof d.placeholder === "string") return d.placeholder;
    if (d.placeholder) return "Some values are placeholders.";
    const from = [d.inherits, ...Object.values(d.mix ?? {})].filter((x): x is string => !!x);
    for (const f of from) {
      const p = placeholder(registry.find((x) => x.id === f));
      if (p) return p;
    }
    return null;
  };

  const names = $derived(host.names());
  const order = $derived.by(() => {
    for (let d: SymbologyDoc | undefined = chosen; d; d = registry.find((x) => x.id === d!.inherits)) if (Array.isArray(d.legend)) return d.legend;
    return [] as string[];
  });
  const present = $derived(scene.subjects.map(subjectOf));
  const presentClasses = $derived([...new Set(present.flatMap((s) => (s.kind === "planet" ? s.classes : [])))]);
  // A class is listed when a binding of the symbology knows it (the classification pack's base classes).
  const known = $derived([...new Set(marker.bindings.filter((b) => b.from === "class" && MIXABLE.includes(b.channel) && "table" in b.scale).flatMap((b) => Object.keys((b.scale as { table: Record<string, unknown> }).table)))]);
  const rows = $derived((all ? [...order, ...known.filter((k) => !order.includes(k))] : presentClasses.filter((c) => known.includes(c))).filter((c, i, a) => a.indexOf(c) === i));
  const legend = $derived(legendFor(marker, { present, names, order }));
  // A colour to start from when a class has none: the one another grammar gives it.
  function suggestColour(c: string): string | null {
    for (const b of bases) {
      const v = resolveSymbology(b.id, registry).find((x) => x.channel === "colour" && x.from === "class" && "table" in x.scale);
      const t = v && "table" in v.scale ? v.scale.table[c] : undefined;
      if (typeof t === "string" && t.startsWith("#")) return t;
    }
    return null;
  }
  const one = (c: string): MarkSubject => ({ kind: "planet", classes: [c], tags: [] });

  // The campaign's own symbology: created from the chosen one on first change.
  function campaignDoc(): SymbologyDoc {
    const have = registry.find((d) => d.id === CAMPAIGN);
    if (have) return structuredClone($state.snapshot(have) as SymbologyDoc);
    return { schema: "sse.symbology/1", id: CAMPAIGN, name: "Campaign symbols", description: `${chosen.name}, changed for this campaign.`, inherits: chosen.id, mix: {}, bindings: [] };
  }
  function save(d: SymbologyDoc) {
    host.keep(d);
    host.choose(CAMPAIGN);
  }
  function setMix(ch: Channel, src: string) {
    const d = campaignDoc();
    d.mix = { ...(d.mix ?? {}) };
    if (src === "") delete d.mix[ch];
    else d.mix[ch] = src;
    d.bindings = d.bindings.filter((b) => b.channel !== ch);
    save(d);
  }
  function mixOf(ch: Channel): string {
    let d: SymbologyDoc | undefined = chosen;
    while (d) {
      if (d.mix?.[ch]) return d.mix[ch]!;
      if (d.bindings.some((b) => b.channel === ch && b.from === "class")) return d.id === CAMPAIGN ? (d.inherits ?? "") : d.id;
      d = registry.find((x) => x.id === d!.inherits);
    }
    return "";
  }
  function setClass(ch: Channel, cls: string, v: string | null) {
    const d = campaignDoc();
    const resolved = resolveSymbology(chosen.id === CAMPAIGN ? CAMPAIGN : chosen.id, registry);
    const was = d.bindings.find((b) => b.applies_to === "planet" && b.channel === ch && b.from === "class") ?? resolved.find((b) => b.applies_to === "planet" && b.channel === ch && b.from === "class" && !b.when);
    const table = { ...(was && "table" in was.scale ? was.scale.table : {}) } as Record<string, string | number | boolean>;
    if (v === null) delete table[cls];
    else table[cls] = v;
    const b: Binding = { applies_to: "planet", channel: ch, from: "class", scale: { table }, fallback: was?.fallback ?? null, ...(was?.prefer ? { prefer: was.prefer } : {}) };
    d.bindings = [...d.bindings.filter((x) => !(x.applies_to === "planet" && x.channel === ch && x.from === "class")), b];
    save(d);
  }
  function reset() {
    host.keep(null);
    host.choose(bases[0]!.id);
  }
  const previews = $derived(host.previews?.() ?? []);

  // Importing SSE 3's colours, from where V3 keeps them: the viewer's palette
  // overrides (V3's localStorage, pasted or loaded) and a save's tag categories.
  let paletteText = $state("");
  let report = $state<{ title: string; lines: string[]; error?: boolean } | null>(null);
  const tagColours = $derived(Object.entries(chosen?.tag_colours ?? registry.find((d) => d.id === CAMPAIGN)?.tag_colours ?? {}));
  function applyPalette(text: string) {
    const map = host.v3Tokens?.();
    const parsed = parseV3Palette(text);
    if (!map || typeof parsed === "string") {
      report = { title: "Palette not applied", lines: [typeof parsed === "string" ? parsed : "This shell has no V3 token map."], error: true };
      return;
    }
    const r = symbologyFromV3Palette(parsed, map, chosen.id, registry);
    if (r.applied.length) save(r.doc);
    report = {
      title: r.applied.length ? `Applied ${r.applied.length} of ${r.applied.length + r.skipped.length} V3 colours` : "No V3 symbol colours in it",
      lines: [...r.applied.map((a) => `${a.token} ${a.hex}: ${a.note}`), ...r.skipped.map((x) => `${x.token} left: ${x.note}`)],
    };
  }
  async function loadPalette(f: File | undefined) {
    if (!f) return;
    paletteText = await f.text();
    applyPalette(paletteText);
  }
  async function loadSave(f: File | undefined) {
    if (!f) return;
    try {
      const text = await v3SaveText(new Uint8Array(await f.arrayBuffer()));
      const c = coloursFromV3Save(JSON.parse(text));
      const cats = Object.keys(c.tag_colours);
      if (cats.length) save(withV3TagColours(campaignCopy(chosen.id, registry, CAMPAIGN), c));
      report = {
        title: `From ${f.name}`,
        lines: [
          cats.length ? `${cats.length} tag ${cats.length === 1 ? "category" : "categories"} with the GM's own colours (${cats.join(", ")}): kept as the campaign's tag colours.` : "No changed tag categories: the save uses V3's shipped tag colours.",
          ...objectColourSummary(c.objects),
          "Palette overrides and skins are not in V3 saves; they live in the V3 browser. Paste them above.",
        ],
      };
    } catch (e) {
      report = { title: "Save not read", lines: [(e as Error).message], error: true };
    }
  }
</script>

<div class="sym" role="region" aria-label="Symbols">
  <svg width="0" height="0" class="defs" aria-hidden="true"><HatchDefs prefix="sy" /></svg>
  <section aria-labelledby="sym-grammar">
    <h2 id="sym-grammar">Symbols</h2>
    <p>How classes look on the charts. Symbols are separate from the theme: any theme can draw any of these, and a theme that cannot carry colour says what it does instead.</p>
    <div class="k">Grammar</div>
    <div class="glist" role="radiogroup" aria-label="Grammar">
      {#each registry as d (d.id)}
        <button type="button" role="radio" class="gbtn" aria-checked={marker.id === d.id} onclick={() => host.choose(d.id)}>
          <span class="gname">{d.name}</span>
          {#if d.description}<span class="muted">{d.description}</span>{/if}
          {#if placeholder(d)}<span class="ph"><Stamp kind="fixture" title={placeholder(d) ?? ""} /> Placeholder colours</span>{/if}
        </button>
      {/each}
    </div>
    {#if placeholder(chosen)}<p class="note">{placeholder(chosen)}</p>{/if}

    <div class="k">Mix by channel</div>
    <p class="muted">Take each channel from either grammar, or leave it out.</p>
    {#each MIXABLE as ch (ch)}
      <label class="row">
        <span>{CHANNEL_NAMES[ch]}</span>
        <select aria-label="Take {CHANNEL_NAMES[ch]?.toLowerCase()} from" value={mixOf(ch)} disabled={!editable} onchange={(e) => setMix(ch, e.currentTarget.value)}>
          <option value="">None</option>
          {#each bases as b (b.id)}{#if b.bindings.some((x) => x.channel === ch) || b.mix?.[ch]}<option value={b.id}>{b.name}</option>{/if}{/each}
        </select>
      </label>
    {/each}
    {#if registry.some((d) => d.id === CAMPAIGN)}
      <div class="actions"><button type="button" class="btn" disabled={!editable} onclick={reset}>Back to the shipped symbols</button></div>
    {/if}
    <p class="muted">Kept on this device. Symbols never change the universe or its state hash.</p>

    <details class="v3" aria-label="Colours from SSE 3">
      <summary>Colours from SSE 3</summary>
      <p class="muted">SSE 3 keeps custom colours in the browser, not the save. In SSE 3, open the browser console and run <code>localStorage["sse-palette-overrides"]</code>, then paste the result here, or load a file of it.</p>
      <textarea rows="3" aria-label="SSE 3 palette overrides" placeholder={'{"--body-gas-giant": "…"}'} bind:value={paletteText} disabled={!editable}></textarea>
      <div class="actions">
        <button type="button" class="btn" disabled={!editable || !paletteText.trim()} onclick={() => applyPalette(paletteText)}>Apply palette</button>
        <label class="file"><span>Load file</span><input type="file" accept=".json,.txt,application/json" aria-label="SSE 3 palette file" disabled={!editable} onchange={(e) => loadPalette(e.currentTarget.files?.[0])} /></label>
      </div>
      <p class="muted">A save (.sse.zip or its starmap.json) carries tag category colours and each object's own colours.</p>
      <label class="file"><span>Read a save</span><input type="file" accept=".zip,.json,application/zip,application/json" aria-label="SSE 3 save file" disabled={!editable} onchange={(e) => loadSave(e.currentTarget.files?.[0])} /></label>
      {#if report}
        <div class="report" role="status" aria-label="SSE 3 colour import" class:error={report.error}>
          <strong>{report.title}</strong>
          <ul>{#each report.lines as l, i (i)}<li>{l}</li>{/each}</ul>
        </div>
      {/if}
      {#if tagColours.length}
        <div class="k">Tag colours</div>
        <div class="tags" aria-label="Tag colours">
          {#each tagColours as [id, t] (id)}<span class="tchip" data-tag-category={id} style="background:{t.color ?? 'transparent'};color:{t.textColor ?? 'inherit'}">{t.label ?? id}</span>{/each}
        </div>
      {/if}
    </details>
  </section>

  <section aria-labelledby="sym-classes">
    <div class="phead">
      <h2 id="sym-classes">Classes</h2>
      <label><input type="checkbox" bind:checked={all} /> Every class ({known.length})</label>
    </div>
    <p class="muted">{all ? "Every class the classification pack knows." : "The classes on this chart."} Change one to make it the campaign's own.</p>
    <div class="ctable" role="table" aria-label="Class symbols">
      <div class="crow head" role="row">
        <span role="columnheader">Mark</span><span role="columnheader">Class</span><span role="columnheader">Shape</span><span role="columnheader">Shading</span><span role="columnheader">Colour</span><span role="columnheader">Code</span>
      </div>
      {#each rows as c (c)}
        {@const m = marker.mark(one(c))}
        <div class="crow" role="row" data-class={c}>
          <span role="cell"><svg width="30" height="26" viewBox="0 0 30 26" aria-hidden="true"><Mark mark={{ ...m, size: 6 }} cx={13} cy={11} prefix="sy" /></svg></span>
          <span role="cell" class="cname">{names[c] ?? c}<span class="m muted">{c}</span></span>
          <span role="cell"><select aria-label="Shape of {names[c] ?? c}" value={m.shape} disabled={!editable} onchange={(e) => setClass("shape", c, e.currentTarget.value)}>{#each BODY_SHAPES as s (s)}<option value={s}>{s}</option>{/each}</select></span>
          <span role="cell"><select aria-label="Shading of {names[c] ?? c}" value={m.fill_pattern} disabled={!editable} onchange={(e) => setClass("fill_pattern", c, e.currentTarget.value)}>{#each PATTERNS as p (p)}<option value={p}>{p}</option>{/each}</select></span>
          <span role="cell" class="colour">
            {#if m.colour}
              <input type="color" aria-label="Colour of {names[c] ?? c}" value={m.colour} disabled={!editable} onchange={(e) => setClass("colour", c, e.currentTarget.value.toUpperCase())} />
              <button type="button" class="x" aria-label="No colour for {names[c] ?? c}" disabled={!editable} onclick={() => setClass("colour", c, null)}>×</button>
            {:else}
              <button type="button" class="link" aria-label="Give {names[c] ?? c} a colour" disabled={!editable || host.theme().supports.colours !== "full" || !suggestColour(c)} onclick={() => setClass("colour", c, suggestColour(c))}>add</button>
            {/if}
          </span>
          <span role="cell"><input class="code" aria-label="Code of {names[c] ?? c}" value={m.glyph ?? ""} maxlength="3" disabled={!editable} onchange={(e) => setClass("glyph", c, e.currentTarget.value.trim() || null)} /></span>
        </div>
      {/each}
    </div>
    {#if marker.collapse}<p class="note">{marker.collapse}</p>{/if}
  </section>

  <section aria-labelledby="sym-preview">
    <h2 id="sym-preview">Key and themes</h2>
    <div class="k">Key for this chart</div>
    <Legend entries={legend} prefix="syk" note={marker.collapse} label="Key for this chart" />
    {#if previews.length && params.previews !== false}
      <div class="k">Under every theme</div>
      <p class="muted">The same symbols drawn by each theme, with its own rule where it cannot carry a channel.</p>
      {#each previews as t (t.id)}
        {@const pm = markerFor(marker.id, registry, t.capability)}
        <div class="pv" style={t.style} data-preview-theme={t.id}>
          <div class="pvname">{t.name}</div>
          <Legend entries={legendFor(pm, { present, names, order })} prefix="sy-{t.id}" note={pm.collapse} label="Key under {t.name}" />
        </div>
      {/each}
    {/if}
  </section>
</div>

<style>
  .sym {
    position: absolute;
    inset: 0;
    overflow: auto;
    display: grid;
    grid-template-columns: minmax(0, 300px) minmax(0, 1.3fr) minmax(0, 1fr);
    gap: 28px;
    align-items: start;
    padding: 24px 24px 24px 96px;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 14px;
  }
  .defs {
    position: absolute;
  }
  section {
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    box-shadow: var(--sse-card-shadow);
    padding: 14px 18px 16px;
    min-width: 0;
  }
  h2 {
    margin: 0 0 8px;
    font-family: var(--sse-font-display);
    font-size: 20px;
    font-weight: 600;
  }
  p {
    margin: 0 0 10px;
    line-height: 1.55;
  }
  .k {
    margin: 12px 0 6px;
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .m {
    font-family: var(--sse-font-mono);
    font-size: 11px;
  }
  .muted {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .glist {
    display: flex;
    flex-direction: column;
  }
  .gbtn {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    min-height: 44px;
    margin-bottom: -1px;
    padding: 8px 12px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    text-align: left;
    font: inherit;
    cursor: pointer;
  }
  .gbtn[aria-checked="true"] {
    box-shadow: inset 4px 0 0 var(--sse-accent);
    background: var(--sse-ground-raised);
  }
  .gname {
    font-family: var(--sse-font-display);
    font-size: 16px;
    font-weight: 600;
  }
  .ph {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 12px;
    color: var(--sse-accent-ink);
  }
  .row {
    display: grid;
    grid-template-columns: 80px minmax(0, 1fr);
    align-items: center;
    gap: 8px;
    margin: 6px 0;
    font-size: 13px;
  }
  select,
  .code {
    min-height: 34px;
    min-width: 0;
    padding: 0 6px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
  }
  .code {
    width: 4.5em;
    font-family: var(--sse-font-mono);
  }
  .phead {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    flex-wrap: wrap;
    gap: 8px;
  }
  .phead label {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 13px;
  }
  .v3 {
    margin-top: 14px;
    border-top: 1px solid var(--sse-rule, currentColor);
    padding-top: 10px;
  }
  .v3 summary {
    cursor: pointer;
    font-weight: 600;
  }
  .v3 textarea {
    width: 100%;
    box-sizing: border-box;
    font: inherit;
    font-size: 12px;
  }
  .file {
    display: inline-flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    font-size: 13px;
  }
  .file span {
    white-space: nowrap;
  }
  .file input {
    max-width: 100%;
    font-size: 12px;
  }
  .report {
    margin-top: 8px;
    font-size: 12px;
    overflow-wrap: anywhere;
  }
  .report.error strong {
    color: var(--sse-alert, currentColor);
  }
  .report ul {
    margin: 4px 0 0;
    padding-left: 16px;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .tchip {
    font-size: 12px;
    padding: 1px 6px;
    border-radius: 3px;
    border: 1px solid var(--sse-rule, currentColor);
  }
  .ctable {
    display: flex;
    flex-direction: column;
  }
  .crow {
    display: grid;
    grid-template-columns: 34px minmax(0, 1.4fr) minmax(0, 0.9fr) minmax(0, 0.9fr) 76px 60px;
    gap: 6px;
    align-items: center;
    padding: 3px 0;
    border-bottom: var(--sse-line) dotted var(--sse-rule);
  }
  .crow.head {
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .cname {
    display: flex;
    flex-direction: column;
    overflow-wrap: anywhere;
  }
  .colour {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  input[type="color"] {
    width: 40px;
    height: 30px;
    padding: 0;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
  }
  .x {
    min-width: 28px;
    min-height: 30px;
    border: 0;
    background: none;
    color: var(--sse-ink-muted);
    font-size: 16px;
    cursor: pointer;
  }
  .actions {
    margin: 12px 0 8px;
  }
  .btn {
    display: inline-flex;
    align-items: center;
    min-height: 40px;
    padding: 0 14px;
    border: var(--sse-line) solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 14px;
    cursor: pointer;
  }
  .btn:disabled,
  select:disabled,
  .code:disabled,
  .link:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .link {
    min-height: 30px;
    padding: 0;
    border: 0;
    background: none;
    color: var(--sse-accent-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 14px;
    cursor: pointer;
  }
  .note {
    font-size: 13px;
    color: var(--sse-accent-ink);
  }
  .pv {
    margin: 8px 0;
    padding: 10px 12px;
    background: var(--sse-panel-ground);
    color: var(--sse-panel-ink);
    border: var(--sse-frame-width) solid var(--sse-frame-line);
    border-radius: var(--sse-frame-radius);
  }
  .pvname {
    margin-bottom: 6px;
    font-family: var(--sse-title-font);
    font-size: 14px;
  }
  @media (max-width: 1100px) {
    .sym {
      grid-template-columns: minmax(0, 1fr);
      padding: 16px 16px 24px;
    }
  }
  @media (max-width: 560px) {
    .crow {
      grid-template-columns: 34px minmax(0, 1fr) minmax(0, 1fr);
    }
    .crow.head span:nth-child(n + 4) {
      display: none;
    }
  }
</style>
