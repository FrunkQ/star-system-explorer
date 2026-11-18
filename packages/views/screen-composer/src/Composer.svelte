<!--
  Screens composer (U5, Compose board): a player screen is a document that
  combines views from the stack. Edit its items (view, area, parameters),
  see it drawn live on a player scene, read the document, take or decline a
  screen change request from Orac (a labelled sample until A2), and see which
  of the campaign's packs it draws on. Nothing here writes code.
-->
<script lang="ts">
  import {
    LAYOUT_AREAS,
    checkPlayerScene,
    checkScreen,
    observeOf,
    packSummary,
    type Json,
    type PlayerScene,
    type ScreenDoc,
    type SystemScene,
    type ViewContext,
    type Vocab,
  } from "../../../ui-contract/src/index.ts";
  import { Stamp } from "../../../chart-grammar/src/index.ts";
  import { PLAYER_STACK } from "../../player-stack/src/index.ts";
  import Preview from "./Preview.svelte";

  import { untrack } from "svelte";

  let { ctx, scene: system, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  // Read once: the documents a composer opens are fixed when it mounts.
  const P = untrack(() => params);
  const ws = $derived(ctx.workspace);
  const editable = $derived(ws.mode === "edit" && ctx.session.seat.authority === "gm");

  const DESCRIBE: Record<string, string> = {
    "sse.view.chart": "2D orbits",
    "sse.view.card": "one subject",
    "sse.view.table": "sortable facts",
    "sse.view.text": "prose",
    "sse.view.readout": "one value, gauge or light",
  };
  // Views of the finite stack that are designed but not built yet (H8.2).
  const LATER = ["System 3D", "Body", "Cutaway", "Sky", "Map", "Timeline", "Route", "Graph", "Media", "Console"];

  type Orac = { id: string; asked: string; rationale: string; fixture?: boolean; screen: ScreenDoc };
  let doc = $state<ScreenDoc | null>(null);
  let vocab = $state<Vocab | null>(null);
  let scenes = $state<Record<string, PlayerScene>>({});
  let orac = $state<{ r: Orac; status: "open" | "accepted" | "declined" } | null>(null);
  let failure = $state<string | null>(null);
  const get = async (u: string) => (await fetch(new URL(u, document.baseURI))).json();
  Promise.all([get(String(P.screen)), get(String(P.vocab)), P.orac ? get(String(P.orac)) : null])
    .then(([d, v, o]) => {
      doc = d;
      vocab = v;
      if (o) orac = { r: o.request, status: "open" };
    })
    .catch((e) => (failure = String(e)));

  // Live scenes from the kernel: the audience seen through the campaign's
  // before and after views (the two sides of the Scanned button). Read only.
  const audience = String(P.audience ?? "everyone");
  const pair = $derived.by(() => {
    void ws.revision;
    return observeOf(ctx.session.stack().base)?.setup.campaign ?? null;
  });
  $effect(() => {
    const p = pair;
    const scope = system.system;
    if (!p) return void (failure = "No pack in the stack says what players can learn (provides.other.observe).");
    Promise.all([p.before, p.after].map((v) => ctx.session.scene(scope, { audience, data_view: v }))).then((rs) => {
      const bad = rs.find((r) => !r.ok);
      if (bad && !bad.ok) return void (failure = bad.message);
      const [b, a] = rs.map((r) => (r.ok ? r.scene : null)) as PlayerScene[];
      const wrong = [b, a].map((x) => checkPlayerScene(x!)).find(Boolean);
      if (wrong) failure = wrong;
      scenes = { before: b!, after: a! };
    });
  });

  let sensor = $state<"before" | "after">("after");
  const checked = $derived(doc ? checkScreen(doc, PLAYER_STACK) : null);
  const oracChecked = $derived(orac ? checkScreen(orac.r.screen, PLAYER_STACK) : null);
  const areas = $derived(doc ? (LAYOUT_AREAS[doc.layout] ?? []) : []);
  const freeArea = $derived(areas.find((a) => !doc?.items.some((i) => i.area === a)));
  let showDoc = $state(false);
  let paramErrors = $state<Record<number, string>>({});

  function addView(id: string) {
    if (!doc || !freeArea) return;
    doc.items.push({ view: id, area: freeArea });
  }
  function setParams(i: number, text: string) {
    try {
      const v = text.trim() ? JSON.parse(text) : {};
      if (typeof v !== "object" || Array.isArray(v) || v === null) throw new Error("parameters are a JSON object");
      doc!.items[i]!.params = v;
      const { [i]: _, ...rest } = paramErrors;
      paramErrors = rest;
    } catch (e) {
      paramErrors = { ...paramErrors, [i]: (e as Error).message };
    }
  }
  function acceptOrac() {
    if (!orac) return;
    doc = JSON.parse(JSON.stringify(orac.r.screen)) as ScreenDoc;
    orac = { ...orac, status: "accepted" };
  }
  function download() {
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([JSON.stringify(doc, null, 2)], { type: "application/json" }));
    a.download = `${doc!.id}.json`;
    a.click();
    URL.revokeObjectURL(a.href);
  }
  const packs = $derived.by(() => {
    void ws.revision;
    const s = ctx.session.stack();
    return [...s.base, ...(s.campaign ? [s.campaign] : [])].map(packSummary);
  });
  // Previews remount whenever the document changes, so a new parameter shows at once.
  const docKey = $derived(JSON.stringify(doc));
</script>

<div class="composer" role="region" aria-label="Screens composer">
  <section class="stack" aria-label="The view stack">
    <div class="k">The view stack</div>
    <p class="muted">Every player screen and report is a combination of these.</p>
    <ul>
      {#each PLAYER_STACK as m (m.id)}
        <li>
          <span class="vname">{m.id.replace("sse.view.", "")}</span>
          <span class="muted">{DESCRIBE[m.id] ?? ""}</span>
          {#if editable}<button type="button" class="link" disabled={!freeArea} onclick={() => addView(m.id)} aria-label="Add {m.id.replace('sse.view.', '')} to the screen">Add</button>{/if}
        </li>
      {/each}
      {#each LATER as n (n)}<li class="later"><span class="vname">{n}</span><span class="muted">later</span></li>{/each}
    </ul>
  </section>

  <section class="main" aria-label="Screen">
    {#if failure}
      <p class="err" role="alert">{failure}</p>
    {:else if !doc || !vocab || !scenes.before}
      <p class="muted">Loading the screen…</p>
    {:else}
      <div class="mhead">
        <h2>{doc.label}</h2>
        <span class="k">{doc.audience} · {doc.layout} · {doc.vocab} words</span>
        <span class="spacer"></span>
        <div class="seg" role="group" aria-label="Preview scene">
          <button type="button" aria-pressed={sensor === "before"} onclick={() => (sensor = "before")}>Long range</button>
          <button type="button" aria-pressed={sensor === "after"} onclick={() => (sensor = "after")}>Scanned</button>
        </div>
      </div>
      {#if checked && !checked.ok}
        <div class="problems" role="alert"><Stamp kind="rejected" tilt={false} /><ul>{#each checked.errors as e (e)}<li>{e}</li>{/each}</ul></div>
      {:else if checked?.ok}
        <div class="preview">
          {#key docKey}<Preview screen={checked.screen} scene={scenes[sensor]!} {vocab} />{/key}
        </div>
      {/if}
      <p class="muted">Preview scene: the kernel's verified scene for Kestrel (until the scene op is wired). Players on the screen see only what their data view gives them.</p>

      <h3>Items</h3>
      <ol class="items">
        {#each doc.items as it, i (i)}
          <li>
            <label>View
              <select value={it.view} disabled={!editable} onchange={(e) => (doc!.items[i]!.view = e.currentTarget.value)}>
                {#each PLAYER_STACK as m (m.id)}<option value={m.id}>{m.id.replace("sse.view.", "")}</option>{/each}
              </select>
            </label>
            <label>Area
              <select value={it.area} disabled={!editable} onchange={(e) => (doc!.items[i]!.area = e.currentTarget.value)}>
                {#each areas as a (a)}<option value={a}>{a}</option>{/each}
              </select>
            </label>
            <label class="pl">Parameters
              <input value={JSON.stringify(it.params ?? {})} disabled={!editable} onchange={(e) => setParams(i, e.currentTarget.value)} spellcheck="false" autocomplete="off" aria-invalid={paramErrors[i] ? "true" : undefined} />
            </label>
            {#if editable}<button type="button" class="link" onclick={() => doc!.items.splice(i, 1)} aria-label="Remove item {i + 1}">Remove</button>{/if}
            {#if paramErrors[i]}<span class="err" role="alert">{paramErrors[i]}</span>{/if}
          </li>
        {/each}
      </ol>
      <div class="actions">
        <button type="button" class="btn" onclick={() => (showDoc = !showDoc)} aria-expanded={showDoc}>{showDoc ? "Hide" : "Show"} document</button>
        <button type="button" class="btn" disabled={!checked?.ok} onclick={download}>Download</button>
      </div>
      {#if showDoc}<pre class="doc" aria-label="Screen document">{JSON.stringify(doc, null, 2)}</pre>{/if}
    {/if}
  </section>

  <aside class="side">
    {#if orac}
      <section class="orac" aria-label="Orac change request">
        <div class="ohead"><h2>Orac</h2><span class="k">screen "{orac.r.screen.label}"</span></div>
        {#if orac.r.fixture}<p class="muted">Orac is not connected yet; this is a sample request in the shape Orac will file.</p>{/if}
        <p class="muted">You asked:</p>
        <p class="asked">"{orac.r.asked}"</p>
        <p>{orac.r.rationale}</p>
        {#if oracChecked && !oracChecked.ok}
          <div class="problems" role="alert"><Stamp kind="rejected" tilt={false} /><ul>{#each oracChecked.errors as e (e)}<li>{e}</li>{/each}</ul></div>
        {:else}
          <p class="ok"><Stamp kind="validated" tilt={false} /> Every view is in the stack and every parameter fits its schema.</p>
        {/if}
        {#if orac.status === "open" && editable}
          <div class="actions">
            <button type="button" class="btn ink" disabled={!oracChecked?.ok} onclick={acceptOrac}>Accept</button>
            <button type="button" class="btn" onclick={() => (orac = { ...orac!, status: "declined" })}>Decline</button>
          </div>
        {:else if orac.status !== "open"}
          <p class="note" role="status">{orac.status === "accepted" ? "Accepted: the screen now uses Orac's layout. Edit it like any other." : "Declined."}</p>
        {/if}
      </section>
    {/if}
    <section class="draws" aria-label="This screen draws on">
      <div class="k">This screen draws on</div>
      <ul>
        {#each packs as p (p.id)}
          <li><span class="vname">{p.id}</span><span class="k">{p.kinds.join(", ")}</span></li>
        {/each}
      </ul>
      <p class="muted">The physics behind the numbers is the campaign's and is not chosen per screen. Pack order, overrides and conflicts are set once on the Packs screen. Themes, codes and names come with presentation, game-system and universe packs.</p>
    </section>
  </aside>
</div>

<style>
  .composer {
    position: absolute;
    inset: 0;
    overflow: auto;
    display: grid;
    grid-template-columns: minmax(0, 240px) minmax(0, 1fr) minmax(0, 320px);
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
  .side {
    display: flex;
    flex-direction: column;
    gap: 24px;
    min-width: 0;
  }
  h2 {
    margin: 0;
    font-family: var(--sse-font-display);
    font-size: 20px;
    font-weight: 600;
  }
  h3 {
    margin: 14px 0 4px;
    font-size: 11px;
    font-weight: 400;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  p {
    margin: 6px 0;
    line-height: 1.5;
  }
  .k {
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .muted {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  ul,
  ol {
    margin: 6px 0 0;
    padding: 0;
    list-style: none;
  }
  .stack li,
  .draws li {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 6px 0;
    border-bottom: var(--sse-line) dotted var(--sse-rule);
  }
  .stack li .muted,
  .draws li .k {
    flex: 1;
    text-align: right;
  }
  .vname {
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 16px;
    text-transform: capitalize;
    overflow-wrap: anywhere;
  }
  .draws .vname {
    text-transform: none;
    font-style: normal;
    font-size: 14px;
  }
  .later {
    opacity: 0.55;
  }
  .link {
    min-height: 32px;
    border: 0;
    background: none;
    padding: 0 0 0 6px;
    color: var(--sse-accent-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 14px;
    cursor: pointer;
  }
  .link:disabled {
    opacity: 0.4;
  }
  .mhead {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 8px 14px;
    margin-bottom: 12px;
  }
  .spacer {
    flex: 1;
  }
  .seg {
    display: flex;
    border: var(--sse-line) solid var(--sse-ink);
  }
  .seg button {
    min-height: 34px;
    padding: 0 12px;
    border: 0;
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
    cursor: pointer;
  }
  .seg button[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .preview {
    padding: 12px;
    border: var(--sse-line) dashed var(--sse-ink-muted);
    background: var(--sse-ground);
  }
  .items li {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    gap: 6px 12px;
    padding: 8px 0;
    border-bottom: var(--sse-line) dotted var(--sse-rule);
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .pl {
    flex: 1;
    min-width: 200px;
  }
  select,
  input {
    min-height: 34px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-mono);
    font-size: 12px;
    padding: 0 6px;
    text-transform: none;
    letter-spacing: 0;
  }
  input[aria-invalid="true"] {
    border-color: var(--sse-accent);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 12px;
  }
  .btn {
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
  .doc {
    max-height: 320px;
    overflow: auto;
    margin: 10px 0 0;
    padding: 10px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
    font-family: var(--sse-font-mono);
    font-size: 11px;
  }
  .ohead {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
  }
  .asked {
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 16px;
  }
  .ok {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 13px;
  }
  .problems,
  .err {
    color: var(--sse-accent-ink);
    font-size: 13px;
    overflow-wrap: anywhere;
  }
  .problems ul {
    padding-left: 16px;
    list-style: disc;
  }
  .note {
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-accent-ink);
  }
  @media (max-width: 1200px) {
    .composer {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  @media (max-width: 760px) {
    .composer {
      position: relative;
      padding: 12px;
      gap: 16px;
    }
  }
</style>
