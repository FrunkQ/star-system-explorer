<!--
  Packs (U5, PackSetup board): the campaign's pack stack, top to bottom,
  checked by the kernel. Adding or removing a pack shows every derived value
  it would change, with fictional labels, before you accept; accepting is one
  stack_packs intent. Done once per campaign and rarely touched after.
-->
<script lang="ts">
  import {
    checkPackDoc,
    packSummary,
    previewStack,
    stackPacks,
    type Json,
    type PackDoc,
    type StackPreview,
    type SystemScene,
    type ViewContext,
  } from "../../../ui-contract/src/index.ts";
  import { Stamp } from "../../../chart-grammar/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const ws = $derived(ctx.workspace);
  const session = $derived(ctx.session);
  const info = $derived.by(() => {
    void ws.revision;
    return session.stack();
  });
  const opening = $derived(info.base.slice(0, info.base.length - info.added.length));
  const layers = $derived([
    ...(info.campaign ? [{ p: info.campaign, role: "campaign" as const }] : []),
    ...[...info.added].reverse().map((p) => ({ p, role: "added" as const })),
    ...[...opening].reverse().map((p) => ({ p, role: "opening" as const })),
  ]);
  const editable = $derived(ws.mode === "edit" && session.seat.authority === "gm");

  let check = $state<StackPreview | null>(null);
  let checking = $state(false);
  async function checkStack() {
    checking = true;
    check = await previewStack(session, scene, info.added);
    checking = false;
  }

  // A proposed change to the added packs, with its preview.
  let proposal = $state<{ title: string; added: PackDoc[]; preview: StackPreview | null } | null>(null);
  let note = $state<string | null>(null);
  let busy = $state(false);
  async function propose(title: string, added: PackDoc[]) {
    note = null;
    proposal = { title, added, preview: null };
    const p = await previewStack(session, scene, added);
    if (proposal?.title === title) proposal = { ...proposal, preview: p };
  }
  async function accept() {
    if (!proposal) return;
    busy = true;
    const r = await stackPacks(session, ws, $state.snapshot(proposal.added) as PackDoc[]);
    busy = false;
    note = r.status === "applied" ? `${proposal.title}: done. Every value it touches is re-derived and labelled.` : r.status === "refused" ? r.reason : "Not applied.";
    if (r.status === "applied") proposal = null;
  }

  let fileError = $state<string | null>(null);
  async function addFile(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (!f) return;
    try {
      const doc = JSON.parse(await f.text());
      const bad = checkPackDoc(doc, [...info.base, ...(info.campaign ? [info.campaign] : [])].map((p) => p.id));
      fileError = bad;
      if (!bad) await propose(`Add ${doc.id}`, [...info.added, doc]);
    } catch (err) {
      fileError = `Not readable as JSON: ${(err as Error).message}`;
    }
  }
  const samples = $derived((params.samples as string[]) ?? []);
  async function addSample(url: string) {
    const doc = (await (await fetch(new URL(url, document.baseURI))).json()) as PackDoc;
    const bad = checkPackDoc(doc, info.base.map((p) => p.id));
    fileError = bad;
    if (!bad) await propose(`Add ${doc.id}`, [...info.added, doc]);
  }
  const ROLE = { campaign: "your physics changes", added: "added here", opening: "campaign start" };
</script>

<div class="packs" role="region" aria-label="Packs">
  <section class="intro">
    <h2>Campaign setup</h2>
    <p>Done once when a campaign starts, and rarely touched after. The stack decides what physics runs, what universe you are in, which game rules name things, and what looks are available. Screens and lenses then choose from what is stacked here.</p>
    <button type="button" class="btn ink" onclick={checkStack} disabled={checking}>Check stack</button>
    {#if check}
      <div class="verdict" role="status">
        {#if check.ok}<Stamp kind="validated" tilt={false} /> The kernel accepts the stack: one publisher per quantity, every override declared. <span class="m">stack {check.stack_hash?.slice(0, 10)}</span>
        {:else}<Stamp kind="rejected" tilt={false} /><ul>{#each check.problems as p (p)}<li>{p}</li>{/each}</ul>{/if}
      </div>
    {/if}
  </section>

  <section class="stack" aria-label="Pack stack">
    <div class="k">Pack stack</div>
    <p class="muted">Higher layers override lower ones. Each value remembers which layer made it.</p>
    <ol>
      {#each layers as { p, role } (p.id)}
        {@const s = packSummary(p)}
        <li class="layer" class:fictional={s.fictional} data-pack={p.id}>
          <div class="lhead">
            <span class="name">{p.id}</span>
            <span class="k">{s.kinds.join(", ")}{s.fictional ? " · fictional" : ""}</span>
          </div>
          <div class="muted">
            v{s.version} · {ROLE[role]} · {s.flows.length} flows{s.quantity_kinds ? `, ${s.quantity_kinds} quantity kinds` : ""}{s.other.length ? `, ${s.other.join(", ")}` : ""}
          </div>
          {#each s.replaces as r (r)}<div class="rep">replaces {r}</div>{/each}
          {#if role === "added" && editable}
            <button type="button" class="link" onclick={() => propose(`Remove ${p.id}`, info.added.filter((x) => x.id !== p.id))}>Remove…</button>
          {/if}
        </li>
      {/each}
    </ol>
    <p class="fic"><Stamp kind="fictional" tilt={false} /> Anything a fictional pack touches carries this stamp, so real and invented physics never get mixed up.</p>
    {#if editable}
      <div class="add">
        <label class="btn" for="pack-file">Add a pack from a file…</label>
        <input id="pack-file" type="file" accept=".json,application/json" onchange={addFile} />
        {#each samples as u (u)}<button type="button" class="btn" onclick={() => addSample(u)}>Try the sample pack</button>{/each}
      </div>
      {#if fileError}<p class="err" role="alert">{fileError}</p>{/if}
    {/if}
  </section>

  <section class="effect" aria-label="What changing it does">
    <h2>What changing it does</h2>
    {#if !proposal}
      <p>Adding or reordering a physics or universe pack can change derived values. You are shown what would change before you accept, and worlds already charted keep their recorded facts. Adding a presentation or game-system pack never changes the physics.</p>
    {:else}
      <div class="proposal" role="region" aria-label="Proposal">
        <h3>{proposal.title}</h3>
        {#if !proposal.preview}
          <p class="muted" aria-live="polite">Checking the stack with the kernel…</p>
        {:else if !proposal.preview.ok}
          <div class="problems" role="alert"><Stamp kind="rejected" tilt={false} /><ul>{#each proposal.preview.problems as p (p)}<li>{p}</li>{/each}</ul></div>
        {:else}
          <p class="ok"><Stamp kind="validated" tilt={false} /> {proposal.preview.changes.length ? `${proposal.preview.changes.length} values change:` : "No derived value changes."}</p>
          {#if proposal.preview.changes.length}
            <table>
              <thead><tr><th scope="col">Value</th><th scope="col" class="n">Now</th><th scope="col" class="n">After</th></tr></thead>
              <tbody>
                {#each proposal.preview.changes as c (c.address + c.field)}
                  <tr><th scope="row">{c.subject} · {c.label.toLowerCase()}</th><td class="n" class:del={c.before !== c.after}>{c.before}</td><td class="n ins">{c.after}{#if c.fictional}<span class="fmark"> · fictional</span>{/if}</td></tr>
                {/each}
              </tbody>
            </table>
          {/if}
        {/if}
        <div class="actions">
          <button type="button" class="btn ink" disabled={!proposal.preview?.ok || busy} onclick={accept}>Accept</button>
          <button type="button" class="btn" onclick={() => (proposal = null)}>Cancel</button>
        </div>
      </div>
    {/if}
    {#if note}<p class="note" role="status">{note}</p>{/if}
  </section>
</div>

<style>
  .packs {
    position: absolute;
    inset: 0;
    overflow: auto;
    display: grid;
    grid-template-columns: minmax(0, 300px) minmax(0, 1.4fr) minmax(0, 1fr);
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
  h3 {
    margin: 0 0 6px;
    font-family: var(--sse-font-display);
    font-size: 17px;
  }
  p {
    margin: 0 0 10px;
    line-height: 1.55;
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
  .m {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  ol {
    margin: 10px 0;
    padding: 0;
    list-style: none;
  }
  .layer {
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    padding: 9px 12px;
    margin-bottom: -1px;
  }
  .layer.fictional {
    box-shadow: inset 4px 0 0 var(--sse-accent);
  }
  .lhead {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 10px;
  }
  .name {
    font-family: var(--sse-font-display);
    font-size: 16px;
    font-weight: 600;
    overflow-wrap: anywhere;
  }
  .rep {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-accent-ink);
  }
  .fic {
    font-size: 12px;
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
  .add {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .add input {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
  }
  .add input:focus-visible + .btn,
  .add label:focus-within {
    outline: 2px solid var(--sse-focus);
  }
  .link {
    margin-top: 4px;
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
  .verdict,
  .problems,
  .err {
    margin-top: 10px;
    font-size: 13px;
    color: var(--sse-accent-ink);
    overflow-wrap: anywhere;
  }
  .verdict ul,
  .problems ul {
    margin: 4px 0 0;
    padding-left: 16px;
  }
  .ok {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    margin: 4px 0 10px;
  }
  th,
  td {
    padding: 4px 4px 4px 0;
    border-bottom: var(--sse-line) dotted var(--sse-rule);
    text-align: left;
    font-weight: 400;
    font-size: 13px;
  }
  thead th {
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .n {
    text-align: right;
    font-family: var(--sse-font-mono);
    font-size: 12px;
    white-space: nowrap;
  }
  .del {
    text-decoration: line-through;
    color: var(--sse-ink-muted);
  }
  .ins {
    color: var(--sse-accent-ink);
    font-weight: 600;
  }
  .fmark {
    font-family: var(--sse-font-body);
    font-weight: 400;
    font-size: 11px;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .note {
    margin-top: 12px;
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-accent-ink);
  }
  @media (max-width: 1100px) {
    .packs {
      grid-template-columns: minmax(0, 1fr);
      padding-left: 96px;
    }
  }
  @media (max-width: 760px) {
    .packs {
      position: relative;
      padding: 12px;
      gap: 16px;
    }
  }
</style>
