<!--
  Styles (V11, H8.9): configuration built for Orac first, people second.
  Every view publishes its settings as a schema (name, type, range, default,
  unit, meaning, cost); that full list is here for Orac and the raw
  document. A style document names settings for one view and may extend
  another. People get the few promoted settings as plain controls; the
  document is checked (types, ranges, cost against a device tier) before it
  can be kept, and the change is shown as a diff. Presentation only: styles
  never reach the kernel.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { checkScreen, checkStyle, observeOf, promotedOf, resolveStyle, settingsDiff, COSTS, TIER_RUNS_UP_TO, type CheckedScreen, type Json, type PlayerScene, type ScreenDoc, type StyleDoc, type SystemScene, type ViewContext, type Vocab } from "../../../ui-contract/src/index.ts";
  import { Settings, Stamp } from "../../../chart-grammar/src/index.ts";
  import { ScreenRuntime } from "../../player-stack/src/index.ts";

  let { ctx, scene: system, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const P = untrack(() => params);
  const host = $derived(ctx.styles!);
  const editable = $derived(ctx.workspace.mode === "edit");
  const views = $derived(host.views());
  let viewId = $state(String(P.view));
  const view = $derived(views.find((v) => v.id === viewId) ?? views[0]!);
  const isPlayer = $derived(view.scene === "player");
  const forView = $derived(host.registry().filter((d) => d.view === view.id));
  let styleId = $state<string | null>(null);
  const style = $derived(forView.find((d) => d.id === styleId) ?? null);

  // The draft: a copy of the chosen style (or a new one), edited by controls or as raw text.
  let draft = $state<StyleDoc | null>(null);
  let raw = $state("");
  let rawError = $state<string | null>(null);
  function startFrom(d: StyleDoc | null) {
    const base: StyleDoc = d ? (JSON.parse(JSON.stringify(d)) as StyleDoc) : { schema: "sse.style/1", id: `campaign.style.${view.id.replace(/^sse\.view\./, "")}`, name: "Campaign style", view: view.id, settings: {} };
    draft = base;
    raw = JSON.stringify(base, null, 2);
    rawError = null;
  }
  $effect(() => {
    void view.id;
    untrack(() => {
      styleId = forView[0]?.id ?? null;
      startFrom(forView[0] ?? null);
    });
  });
  function choose(id: string) {
    styleId = id;
    startFrom(forView.find((d) => d.id === id) ?? null);
  }
  function derive() {
    if (!style) return;
    let n = 1;
    while (host.registry().some((d) => d.id === `campaign.style.${view.id.replace(/^sse\.view\./, "")}-${n}`)) n++;
    draft = { schema: "sse.style/1", id: `campaign.style.${view.id.replace(/^sse\.view\./, "")}-${n}`, name: `${style.name} (mine)`, view: view.id, extends: style.id, settings: {} };
    raw = JSON.stringify(draft, null, 2);
  }
  function setSetting(name: string, value: Json) {
    if (!draft) return;
    draft.settings = { ...draft.settings, [name]: value };
    raw = JSON.stringify(draft, null, 2);
  }
  function readRaw(text: string) {
    raw = text;
    try {
      const d = JSON.parse(text) as StyleDoc;
      if (!d || typeof d !== "object" || !d.settings) throw new Error("a style document has settings");
      draft = d;
      rawError = null;
    } catch (e) {
      rawError = (e as Error).message;
    }
  }

  const TIERS = Object.keys(TIER_RUNS_UP_TO);
  let tier = $state(untrack(() => (TIERS.includes(ctx.styles!.tier()) ? ctx.styles!.tier() : "standard")));
  const withDraft = $derived.by((): StyleDoc[] => {
    const all = [...host.registry()];
    const d = draft;
    return d ? [...all.filter((x) => x.id !== d.id), d as StyleDoc] : all;
  });
  const verdict = $derived(draft ? (draft.view !== view.id ? { ok: false as const, errors: [`this document is for ${draft.view}`] } : checkStyle(draft.id, withDraft, views, tier)) : null);
  const effective = $derived.by(() => {
    if (!draft) return {} as Record<string, Json>;
    try {
      return resolveStyle(draft.id, withDraft).settings;
    } catch {
      return draft.settings;
    }
  });
  const promoted = $derived(promotedOf(view, draft ?? undefined));
  const before = $derived.by(() => {
    if (!style) return {} as Record<string, Json>;
    try {
      return resolveStyle(style.id, host.registry()).settings;
    } catch {
      return {};
    }
  });
  const diff = $derived(settingsDiff(before, effective));
  let note = $state<string | null>(null);
  function keep() {
    if (!draft || !verdict?.ok) return;
    host.keep(JSON.parse(JSON.stringify(draft)) as StyleDoc);
    styleId = draft.id;
    note = `Kept "${draft.name}" on this device. Screens can name it as ${draft.id}.`;
  }

  // Preview of a player view: one item on a page screen, through the runtime players use, on the live scene.
  let vocab = $state<Vocab | null>(null);
  let pscene = $state<PlayerScene | null>(null);
  fetch(new URL(String(P.vocab), document.baseURI))
    .then((r) => r.json())
    .then((v) => (vocab = v as Vocab));
  $effect(() => {
    void ctx.workspace.revision;
    // Seen through the campaign's "after Scanned" view, so the preview has something to draw.
    const after = observeOf(ctx.session.stack().base)?.setup.campaign.after;
    ctx.session.scene(system.system, { audience: String(P.audience), ...(after ? { data_view: after } : {}) }).then((r) => {
      if (r.ok) pscene = r.scene;
    });
  });
  const preview = $derived.by((): CheckedScreen | null => {
    if (!isPlayer || !draft || !verdict?.ok) return null;
    const doc: ScreenDoc = { schema: "sse.screen/1", id: "styles.preview", label: "Preview", audience: "players", layout: "page", theme: "", symbology: "", vocab: "plain", items: [{ view: view.id, area: "1", style: draft.id }] };
    const c = checkScreen(doc, [view], undefined, { registry: withDraft, tier });
    return c.ok ? c.screen : null;
  });
  const previewKey = $derived(JSON.stringify(effective));
  const groups = $derived([
    { name: "GM", list: views.filter((v) => v.scene !== "player") },
    { name: "Players", list: views.filter((v) => v.scene === "player") },
  ]);
  const show = (v: Json | undefined) => (v === undefined ? "–" : JSON.stringify(v));
</script>

<div class="sty" role="region" aria-label="Styles">
  <section aria-labelledby="sty-views">
    <h2 id="sty-views">Styles</h2>
    <p class="muted">Every view describes all its settings for Orac. You get the few that matter as plain controls; a style document keeps a set of them under a name.</p>
    {#each groups as { name: group, list } (group)}
      <div class="k">{group}</div>
      <div class="vlist" role="radiogroup" aria-label="{group} views">
        {#each list as v (v.id)}
          <button type="button" role="radio" class="vbtn" aria-checked={v.id === view.id} onclick={() => (viewId = v.id)}>
            <span class="vname">{v.id.replace("sse.view.", "")}</span>
            <span class="muted">{v.params.length} settings · {v.params.filter((s) => s.promoted).length} promoted</span>
          </button>
        {/each}
      </div>
    {/each}
  </section>

  <section aria-labelledby="sty-schema" class="schema">
    <h2 id="sty-schema">{view.id.replace("sse.view.", "")} <span class="k m">{view.id} {view.version}</span></h2>
    {#if params.schema_open !== false}
      <p class="muted">The view's whole schema: what Orac reads and writes. Promoted settings are marked.</p>
      <div class="tbl" role="table" aria-label="Settings schema of {view.id}">
        <div class="tr head" role="row"><span role="columnheader">Setting</span><span role="columnheader">Type and range</span><span role="columnheader">Default</span><span role="columnheader">Meaning</span></div>
        {#each view.params as s (s.name)}
          <div class="tr" role="row" data-schema={s.name}>
            <span role="cell" class="m">{s.name}{#if s.promoted}<span class="pro" title="Promoted: a plain control"> ●</span>{/if}</span>
            <span role="cell" class="m">{s.type}{s.range ? ` ${s.range[0]}–${s.range[1]}` : ""}{s.options ? ` ${s.options.join("|")}` : ""}{s.unit ? ` ${s.unit}` : ""}</span>
            <span role="cell" class="m">{JSON.stringify(s.default)}</span>
            <span role="cell">{s.meaning}{#if s.cost}<span class="cost">&nbsp;Costs {s.cost} when on.</span>{/if}{#if s.cost_options}<span class="cost">&nbsp;Costs: {Object.entries(s.cost_options).map(([k, c]) => `${k} ${c}`).join(", ")}.</span>{/if}</span>
          </div>
        {:else}
          <p class="muted">This view has no settings.</p>
        {/each}
      </div>
    {/if}
  </section>

  <section aria-labelledby="sty-doc" class="doc">
    <h2 id="sty-doc">Style</h2>
    {#if forView.length}
      <div class="slist" role="radiogroup" aria-label="Styles for {view.id}">
        {#each forView as d (d.id)}
          <button type="button" role="radio" class="sbtn" aria-checked={d.id === styleId} onclick={() => choose(d.id)}>
            <span class="vname">{d.name}</span>
            <span class="muted">{d.description ?? d.id}{d.extends ? ` · extends ${d.extends}` : ""}</span>
          </button>
        {/each}
      </div>
    {:else}
      <p class="muted">No styles for this view yet. Start one: it begins from the view's defaults.</p>
    {/if}
    <div class="actions">
      {#if style}<button type="button" class="btn" disabled={!editable} onclick={derive}>New style from this</button>{/if}
      {#if !forView.length}<button type="button" class="btn" disabled={!editable} onclick={() => startFrom(null)}>Start a style</button>{/if}
    </div>

    {#if draft}
      <div class="k">Promoted settings</div>
      {#if promoted.length}
        <Settings specs={promoted} values={effective} disabled={!editable} onchange={setSetting} idPrefix="sty" />
      {:else}
        <p class="muted">This view promotes no settings; Orac and the document reach all of them.</p>
      {/if}

      <div class="check">
        <label>Check for a
          <select bind:value={tier} aria-label="Device tier to check for">{#each TIERS as t (t)}<option value={t}>{t}</option>{/each}</select>
          device <span class="muted">(this one is {host.tier()}; runs up to {TIER_RUNS_UP_TO[tier]})</span>
        </label>
        {#if verdict?.ok}
          <p class="ok" role="status"><Stamp kind="validated" tilt={false} /> Every setting fits the schema. Cost: {verdict.cost}{verdict.by.length ? ` (${verdict.by.join(", ")})` : ""}, within {COSTS.indexOf(verdict.cost) <= COSTS.indexOf(TIER_RUNS_UP_TO[tier]!) ? "budget" : "?"}.</p>
        {:else if verdict}
          <div class="problems" role="alert"><Stamp kind="rejected" tilt={false} /><ul>{#each verdict.errors as e (e)}<li>{e}</li>{/each}</ul></div>
        {/if}
      </div>

      {#if diff.length}
        <div class="k">Change</div>
        <table class="diff" aria-label="Change from {style?.name ?? 'the defaults'}">
          <tbody>{#each diff as c (c.name)}<tr><th scope="row" class="m">{c.name}</th><td class="m del">{show(c.before)}</td><td class="m ins">{show(c.after)}</td></tr>{/each}</tbody>
        </table>
      {/if}

      <details class="raw">
        <summary>The document</summary>
        <textarea rows="10" spellcheck="false" aria-label="Style document" value={raw} disabled={!editable} oninput={(e) => readRaw(e.currentTarget.value)}></textarea>
        {#if rawError}<p class="err" role="alert">{rawError}</p>{/if}
      </details>
      <div class="actions">
        <button type="button" class="btn ink" disabled={!editable || !verdict?.ok} onclick={keep}>Keep on this device</button>
        <button type="button" class="btn" disabled={!editable} onclick={() => startFrom(style)}>Put back</button>
      </div>
      {#if note}<p class="note" role="status">{note}</p>{/if}
      <p class="muted">Styles change how a view looks and behaves, never the universe: the state hash does not move.</p>

      {#if isPlayer}
        <div class="k">Preview</div>
        {#if preview && pscene && vocab}
          <div class="pv">{#key previewKey}<ScreenRuntime screen={preview} scene={pscene} {vocab} compact />{/key}</div>
        {:else if !verdict?.ok}
          <p class="muted">Nothing to preview until the document passes its check.</p>
        {:else}
          <p class="muted">Opening the scene…</p>
        {/if}
      {/if}
    {/if}
  </section>
</div>

<style>
  .sty {
    position: absolute;
    inset: 0;
    overflow: auto;
    display: grid;
    grid-template-columns: minmax(0, 230px) minmax(0, 1.2fr) minmax(0, 1fr);
    gap: 24px;
    align-items: start;
    padding: 24px 24px 24px 96px;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 14px;
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
    line-height: 1.5;
  }
  .k {
    margin: 12px 0 6px;
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  h2 .k {
    margin: 0;
    font-size: 11px;
    letter-spacing: 0;
    text-transform: none;
    font-weight: 400;
  }
  .m {
    font-family: var(--sse-font-mono);
    font-size: 12px;
    overflow-wrap: anywhere;
  }
  .muted {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .vlist,
  .slist {
    display: flex;
    flex-direction: column;
  }
  .vbtn,
  .sbtn {
    display: flex;
    flex-direction: column;
    gap: 1px;
    width: 100%;
    min-height: 44px;
    margin-bottom: -1px;
    padding: 6px 10px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    text-align: left;
    font: inherit;
    cursor: pointer;
  }
  .vbtn[aria-checked="true"],
  .sbtn[aria-checked="true"] {
    box-shadow: inset 4px 0 0 var(--sse-accent);
    background: var(--sse-ground-raised);
  }
  .vname {
    font-family: var(--sse-font-display);
    font-size: 15px;
    font-weight: 600;
  }
  .tbl {
    display: flex;
    flex-direction: column;
  }
  .tr {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) minmax(0, 0.8fr) minmax(0, 2fr);
    gap: 8px;
    padding: 5px 0;
    border-bottom: var(--sse-line) dotted var(--sse-rule);
    font-size: 12.5px;
  }
  .tr.head {
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .pro,
  .cost {
    color: var(--sse-accent-ink);
  }
  .check {
    margin: 14px 0 6px;
    padding-top: 10px;
    border-top: var(--sse-line) solid var(--sse-rule);
  }
  .check label {
    display: block;
    margin-bottom: 8px;
    font-size: 13px;
  }
  select,
  textarea {
    min-height: 32px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
  }
  textarea {
    display: block;
    width: 100%;
    box-sizing: border-box;
    margin-top: 6px;
    font-family: var(--sse-font-mono);
    font-size: 12px;
  }
  .ok {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    font-size: 13px;
  }
  .problems {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    color: var(--sse-accent-ink);
    font-size: 13px;
  }
  .problems ul {
    margin: 0;
    padding-left: 16px;
  }
  .diff {
    border-collapse: collapse;
    width: 100%;
  }
  .diff th,
  .diff td {
    padding: 3px 6px 3px 0;
    text-align: left;
    font-weight: 400;
    border-bottom: var(--sse-line) dotted var(--sse-rule);
  }
  .del {
    text-decoration: line-through;
    color: var(--sse-ink-muted);
  }
  .ins {
    color: var(--sse-accent-ink);
  }
  .raw summary {
    min-height: 36px;
    display: flex;
    align-items: center;
    cursor: pointer;
    font-family: var(--sse-font-display);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin: 10px 0;
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
  .btn.ink {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .err,
  .note {
    font-size: 13px;
    color: var(--sse-accent-ink);
  }
  .pv {
    padding: 10px;
    border: var(--sse-line) dashed var(--sse-rule);
  }
  @media (max-width: 1100px) {
    .sty {
      grid-template-columns: minmax(0, 1fr);
      padding: 16px 16px 24px;
    }
    .tr {
      grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    }
  }
</style>
