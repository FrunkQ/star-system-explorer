<!--
  Player data (U5, PlayerData board): what players can learn, and when. The
  campaign's data views (which tiers each shows, which tags, how unknowns
  read), the disclosure map (which tier each fact sits at) and the campaign's
  before and after a scan. Showing the table one thing is the eye on a card (SV2). Edits are a draft until Apply, which stacks them as the
  campaign's own player-data pack (one chronicled intent). The preview is the
  kernel's live scene for an audience, seen through any view: what that
  table would see right now.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import {
    applySetup,
    checkSetup,
    ladderOf,
    observeOf,
    setupChanges,
    subjectName,
    type DataView,
    type Json,
    type ObserveSetup,
    type PlayerContext,
    type PlayerScene,
    type SystemScene,
    type ViewContext,
    type ViewHandle,
    type Vocab,
  } from "../../../ui-contract/src/index.ts";
  import { Stamp } from "../../../chart-grammar/src/index.ts";
  import { PLAYER_STACK } from "../../player-stack/src/index.ts";

  let { ctx, scene: system, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const P = untrack(() => params);
  const ws = $derived(ctx.workspace);
  const session = $derived(ctx.session);
  const editable = $derived(ws.mode === "edit" && session.seat.authority === "gm");
  const audience = String(P.audience ?? "everyone");

  const applied = $derived.by(() => {
    void ws.revision;
    return observeOf(session.stack().base);
  });
  let draft = $state<ObserveSetup | null>(null);
  $effect(() => {
    const a = applied;
    if (a && !draft) draft = structuredClone(a.setup);
  });
  const changes = $derived(applied && draft ? setupChanges(applied.setup, draft) : []);
  const problem = $derived(draft ? checkSetup(draft) : null);
  const tiers = $derived(draft ? ladderOf(draft) : []);
  const playerTiers = $derived(tiers.slice(0, -1));
  const secretTier = $derived(tiers[tiers.length - 1]?.[0] ?? "secret");

  let selected = $state<string | null>(null);
  const view = $derived(draft?.views.find((v) => v.id === (selected ?? draft?.campaign.default_view)) ?? draft?.views[0]);
  function role(id: string): string[] {
    if (!draft) return [];
    const c = draft.campaign;
    return [c.default_view === id ? "default" : "", c.before === id ? "before a scan" : "", c.after === id ? "after a scan" : ""].filter(Boolean);
  }
  function toggleTier(v: DataView, t: string, on: boolean) {
    v.show_tiers = on ? playerTiers.map((r) => r[0]).filter((x) => x === t || v.show_tiers.includes(x)) : v.show_tiers.filter((x) => x !== t);
  }
  function newView() {
    if (!draft || !view) return;
    let n = 1;
    while (draft.views.some((v) => v.id === `custom-${n}`)) n++;
    draft.views.push({ ...structuredClone($state.snapshot(view) as DataView), id: `custom-${n}`, label: `${view.label} (copy)` });
    selected = `custom-${n}`;
  }
  function removeView(id: string) {
    if (!draft) return;
    draft.views = draft.views.filter((v) => v.id !== id);
    selected = null;
  }
  const disclosureRows = $derived(
    draft
      ? [
          ...Object.entries(draft.disclosure.fields).map(([k, t]) => ({ kind: "fields" as const, key: k, tier: t })),
          ...Object.entries(draft.disclosure.tags).map(([k, t]) => ({ kind: "tags" as const, key: k, tier: t })),
        ]
      : [],
  );
  let newField = $state("");
  function addField() {
    const f = newField.trim();
    if (!draft || !/^[a-z][a-z0-9_.-]*\*?$/.test(f)) return;
    draft.disclosure.fields[f] = secretTier;
    newField = "";
  }

  let busy = $state(false);
  let note = $state<string | null>(null);
  let history = $state<ObserveSetup[]>([]);
  async function apply() {
    if (!draft || !applied) return;
    busy = true;
    const before = structuredClone($state.snapshot(applied.setup) as ObserveSetup);
    const n = changes.length;
    const r = await applySetup(session, ws, $state.snapshot(draft) as ObserveSetup);
    busy = false;
    if (r.status === "applied") {
      history = [...history, before];
      note = `Applied: ${n} change${n === 1 ? "" : "s"}. Players see it now; the change is in the chronicle.`;
    } else note = r.status === "refused" ? r.reason : `Not applied (${r.status}).`;
  }
  async function putBack() {
    const last = history[history.length - 1];
    if (!last) return;
    const prev = $state.snapshot(last) as ObserveSetup;
    busy = true;
    const r = await applySetup(session, ws, prev);
    busy = false;
    if (r.status === "applied") {
      history = history.slice(0, -1);
      draft = structuredClone(prev);
      note = "Put back as it was.";
    }
  }

  // Preview: the live scene for the audience, through one view.
  let vocab = $state<Vocab | null>(null);
  fetch(new URL(String(P.vocab ?? "documents/vocab.plain.json"), document.baseURI))
    .then((r) => r.json())
    .then((v) => (vocab = v));
  let through = $state<string>("");
  let preview = $state<PlayerScene | null>(null);
  let previewError = $state<string | null>(null);
  // With unapplied changes, the preview is built from the draft (the kernel records nothing).
  const draftSetup = $derived(draft && changes.length && !problem ? ($state.snapshot(draft) as ObserveSetup) : null);
  $effect(() => {
    void ws.revision;
    const v = through;
    const observe = draftSetup;
    session.scene(system.system, { audience, ...(v ? { data_view: v } : {}), ...(observe ? { observe } : {}) }).then((r) => {
      previewError = r.ok ? null : r.message;
      if (r.ok) preview = r.scene;
    });
  });
  let focus = $state<string | null>(null);
  const handles: ViewHandle<PlayerScene>[] = [];
  $effect(() => {
    const s = preview;
    if (s) for (const h of handles) h.update(s);
  });
  function mountView(node: HTMLElement, m: { id: string; params: Record<string, Json> }) {
    const module = PLAYER_STACK.find((x) => x.id === m.id)!;
    const pctx: PlayerContext = { params: m.params, vocab: vocab!, selection: () => focus, select: (a) => (focus = a) };
    const h = module.mount(node, pctx, preview!) as ViewHandle<PlayerScene>;
    handles.push(h);
    return { destroy: () => (handles.splice(handles.indexOf(h), 1), h.dispose()) };
  }
  const CARD = { id: "sse.view.card", params: { fields: [] as string[] } };
  const TABLE = { id: "sse.view.table", params: { columns: ["kind", "mass_kg", "surface.temp_mean_k"] } };
  const subjects = $derived(preview ? preview.subjects.filter((s) => s.address !== preview!.scope) : []);

  const TAGS = { none: "no tags", listed: "only listed tags", all: "every tag it may see" };
</script>

<div class="dv" role="region" aria-label="Player data">
  {#if !draft || !applied}
    <p class="muted">No pack in the stack says what players can learn yet.</p>
  {:else}
    <section class="views" aria-label="Data views">
      <h2>What players can learn</h2>
      <p>Every fact sits at a tier. A data view says which tiers a table sees. To show the table something, press the eye by its name on a card (Build and Run desks): one press reveals it; hold the eye for who knows what.</p>
      <div class="k">Data views · from {applied.pack}</div>
      <ul class="vlist">
        {#each draft.views as v (v.id)}
          <li>
            <button type="button" class="vbtn" aria-pressed={view?.id === v.id} onclick={() => (selected = v.id)}>
              <span class="vlabel">{v.label}</span>
              <span class="muted">{v.show_tiers.length ? v.show_tiers.join(" · ") : "names only"}{role(v.id).length ? ` · ${role(v.id).join(", ")}` : ""}</span>
            </button>
          </li>
        {/each}
      </ul>
      {#if editable}<button type="button" class="link" onclick={newView}>New view from this one…</button>{/if}
      <div class="campaign" aria-label="Campaign views">
        <div class="k">Campaign views</div>
        <label>Starts with <select bind:value={draft.campaign.default_view} disabled={!editable}>{#each draft.views as v (v.id)}<option value={v.id}>{v.label}</option>{/each}</select></label>
        <label title="What a new fog-of-war seat sees of a thing once you mark it discovered">Before a scan <select bind:value={draft.campaign.before} disabled={!editable}>{#each draft.views as v (v.id)}<option value={v.id}>{v.label}</option>{/each}</select></label>
        <label title="What a scan shows">After a scan <select bind:value={draft.campaign.after} disabled={!editable}>{#each draft.views as v (v.id)}<option value={v.id}>{v.label}</option>{/each}</select></label>
      </div>
    </section>

    <section class="edit" aria-label="Edit view">
      {#if view}
        <div class="k">Data view <span class="m">{view.id}</span></div>
        <label class="row">Label <input type="text" bind:value={view.label} disabled={!editable} /></label>
        <label class="row">Description <input type="text" bind:value={view.description} disabled={!editable} /></label>
        <fieldset>
          <legend>Shows facts at</legend>
          {#each playerTiers as [t, placeholder] (t)}
            <label class="chk"><input type="checkbox" checked={view.show_tiers.includes(t)} disabled={!editable} onchange={(e) => toggleTier(view, t, e.currentTarget.checked)} /> {t} <span class="muted">(unknown reads "{placeholder}")</span></label>
          {/each}
          <label class="chk"><input type="checkbox" disabled /> {secretTier} <span class="muted">(GM only, never shown)</span></label>
        </fieldset>
        <label class="row">Tags
          <select bind:value={view.tags.mode} disabled={!editable}>{#each Object.entries(TAGS) as [m, t] (m)}<option value={m}>{t}</option>{/each}</select>
        </label>
        {#if view.tags.mode === "listed"}
          <label class="row">Listed tags <input type="text" value={(view.tags.list ?? []).join(", ")} disabled={!editable} onchange={(e) => view.tags.list = e.currentTarget.value.split(",").map((x) => x.trim()).filter(Boolean)} /></label>
        {/if}
        <label class="row">Unknown facts
          <select bind:value={view.unknown_style} disabled={!editable}><option value="placeholder">say what it takes to learn them</option><option value="omit">leave them out</option></select>
        </label>
        {#if view.overrides?.length}
          <div class="k">Exceptions</div>
          <ul class="ov">
            {#each view.overrides as o, i (i)}
              <li>{o.target.id}: {o.show ? "shown" : "hidden"}{o.precision ? `, ${o.precision.as === "band" ? `as a band of ±${o.precision.half_width}` : o.precision.as}` : ""}</li>
            {/each}
          </ul>
        {/if}
        {#if editable && draft.views.length > 1 && !role(view.id).length}<button type="button" class="link" onclick={() => removeView(view.id)}>Remove this view</button>{/if}
      {/if}

      <div class="apply" aria-label="Changes">
        {#if problem}<p class="err" role="alert"><Stamp kind="rejected" tilt={false} /> {problem}</p>
        {:else if changes.length}<p><Stamp kind="draft" tilt={false} /> {changes.length} unapplied: {changes.join("; ")}.</p>
        {:else}<p class="muted">No unapplied changes.</p>{/if}
        {#if editable}
          <div class="actions">
            <button type="button" class="btn ink" disabled={busy || !!problem || !changes.length} onclick={apply}>Apply to the campaign</button>
            <button type="button" class="btn" disabled={busy || !changes.length} onclick={() => (draft = structuredClone($state.snapshot(applied.setup) as ObserveSetup))}>Discard</button>
            {#if history.length}<button type="button" class="btn" disabled={busy} onclick={putBack}>Put back</button>{/if}
          </div>
        {/if}
        {#if note}<p class="note" role="status">{note}</p>{/if}
      </div>
      <details class="disc">
        <summary>Which tier each fact sits at ({disclosureRows.length})</summary>
        <table>
          <tbody>
            {#each disclosureRows as r (r.kind + r.key)}
              <tr>
                <th scope="row">{r.kind === "tags" ? `tag ${r.key}` : r.key}</th>
                <td><select aria-label="Tier of {r.key}" value={r.tier} disabled={!editable} onchange={(e) => (draft!.disclosure[r.kind][r.key] = e.currentTarget.value)}>{#each tiers as [t] (t)}<option value={t}>{t}</option>{/each}</select></td>
              </tr>
            {/each}
          </tbody>
        </table>
        {#if editable}
          <div class="addf"><input type="text" placeholder="field or prefix.*" aria-label="New field" bind:value={newField} /><button type="button" class="btn" onclick={addField}>Add</button></div>
          <p class="muted">Anything not listed is GM only, so a new fact never leaks by omission.</p>
        {/if}
      </details>

    </section>

    <section class="preview" aria-label="Preview">
      <div class="phead">
        <div class="k">{draftSetup ? `What ${audience} would see` : `What ${audience} sees now`}</div>
        <label>Through
          <select bind:value={through}><option value="">their own grants</option>{#each applied.setup.views as v (v.id)}<option value={v.id}>{v.label}</option>{/each}</select>
        </label>
      </div>
      {#if previewError}<p class="err" role="alert">{previewError}</p>
      {:else if preview && vocab}
        <div class="pcard" data-view="sse.view.card">{#key through}<div use:mountView={CARD}></div>{/key}</div>
        <div class="ptable" data-view="sse.view.table">{#key through}<div use:mountView={TABLE}></div>{/key}</div>
        <p class="muted">Live from the kernel. {draftSetup ? "Showing your unapplied draft: players do not see it until you apply it." : "Showing the campaign as applied."}</p>
      {:else}<p class="muted" aria-busy="true">Asking the kernel…</p>{/if}
    </section>
  {/if}
</div>

<style>
  .dv {
    position: absolute;
    inset: 0;
    overflow: auto;
    display: grid;
    grid-template-columns: minmax(0, 300px) minmax(0, 1fr) minmax(0, 1.1fr);
    gap: 28px;
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
    line-height: 1.55;
  }
  .k {
    margin: 10px 0 6px;
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .m {
    font-family: var(--sse-font-mono);
    letter-spacing: 0;
    text-transform: none;
  }
  .muted {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .vlist,
  .ov {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .vbtn {
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
  .vbtn[aria-pressed="true"] {
    box-shadow: inset 4px 0 0 var(--sse-accent);
    background: var(--sse-ground-raised);
  }
  .vlabel {
    font-family: var(--sse-font-display);
    font-size: 16px;
    font-weight: 600;
  }
  .campaign label,
  .row {
    display: grid;
    grid-template-columns: 110px minmax(0, 1fr);
    align-items: center;
    gap: 8px;
    margin: 6px 0;
    font-size: 13px;
  }
  .campaign label {
    grid-template-columns: 96px minmax(0, 1fr);
  }
  input[type="text"],
  select {
    min-height: 36px;
    min-width: 0;
    padding: 0 8px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
  }
  fieldset {
    margin: 10px 0;
    padding: 6px 10px 8px;
    border: var(--sse-line) solid var(--sse-rule);
  }
  legend {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .chk {
    display: flex;
    gap: 8px;
    align-items: center;
    min-height: 32px;
    font-size: 13px;
  }
  .ov li {
    font-family: var(--sse-font-mono);
    font-size: 12px;
  }
  .disc {
    margin-top: 14px;
    padding-top: 6px;
    border-top: var(--sse-line) dotted var(--sse-rule);
  }
  .disc summary {
    min-height: 36px;
    display: flex;
    align-items: center;
    cursor: pointer;
    font-family: var(--sse-font-display);
    font-size: 15px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th,
  td {
    padding: 3px 4px 3px 0;
    border-bottom: var(--sse-line) dotted var(--sse-rule);
    text-align: left;
    font-weight: 400;
    font-size: 13px;
  }
  th {
    font-family: var(--sse-font-mono);
    font-size: 12px;
    overflow-wrap: anywhere;
  }
  td {
    text-align: right;
  }
  .addf {
    display: flex;
    gap: 8px;
    margin: 8px 0;
  }
  .addf input {
    flex: 1;
  }
  .apply {
    margin-top: 14px;
    padding-top: 10px;
    border-top: var(--sse-line) solid var(--sse-ink);
  }
  .actions,
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
  .link {
    margin-top: 6px;
    min-height: 32px;
    padding: 0;
    border: 0;
    background: none;
    color: var(--sse-accent-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 14px;
    cursor: pointer;
  }
  .err,
  .note {
    font-size: 13px;
    color: var(--sse-accent-ink);
    overflow-wrap: anywhere;
  }
  .note {
    font-family: var(--sse-font-mono);
    font-size: 12px;
  }
  .phead {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 10px;
  }
  .phead .k {
    margin: 0;
  }
  .phead label {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 13px;
  }
  .pcard,
  .ptable {
    margin-bottom: 14px;
  }
  @media (max-width: 1100px) {
    .dv {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  @media (max-width: 760px) {
    .dv {
      position: relative;
      padding: 12px;
      gap: 16px;
    }
  }
</style>
