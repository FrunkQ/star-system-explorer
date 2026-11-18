<!--
  Change requests (H13): every physics change, the GM's or Orac's, is a
  document shown as a diff with its effect before anyone accepts it. A
  request the kernel would reject never reaches the Adopt button. Adopting is
  one intent (adopt_physics) through the session; withdrawing is another.
  Orac is not connected yet (A2): its requests come from a fixture and say so.
-->
<script lang="ts">
  import {
    adoptRequest,
    benchRef,
    declineRequest,
    paramText,
    previewRequest,
    siText,
    withdrawRequest,
    type ChangeRequest,
    type Json,
    type Preview,
    type SystemScene,
    type ViewContext,
  } from "../../../ui-contract/src/index.ts";
  import { Card, Stamp } from "../../../chart-grammar/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const ws = $derived(ctx.workspace);
  const session = $derived(ctx.session);

  const open = $derived(ws.requests.filter((r) => r.status === "draft" && r.edits.length));
  const inForce = $derived(ws.requests.filter((r) => r.status === "adopted"));
  const closed = $derived(ws.requests.filter((r) => r.status === "declined" || r.status === "superseded"));

  let chosen = $state<string | null>(null);
  // The GM's own draft for the flow on the bench comes first, and comes back to the front whenever it changes.
  const gmDraft = $derived(open.find((r) => r.author === "gm" && (!ws.bench || r.flow === ws.bench.flow)));
  $effect(() => {
    void JSON.stringify(gmDraft?.edits);
    chosen = null;
  });
  const current = $derived(
    ws.requests.find((r) => r.id === chosen && r.status === "draft") ?? gmDraft ?? open.find((r) => r.flow === ws.bench?.flow) ?? open[0] ?? null,
  );
  const bench = $derived(current ? benchRef(scene, ws, current.flow) : null);

  let preview = $state<Preview | null>(null);
  let gen = 0;
  $effect(() => {
    void ws.revision;
    const r = current;
    const edits = JSON.stringify(r?.edits ?? []);
    const ref = bench?.ref ?? null;
    const g = ++gen;
    preview = null;
    if (r && edits) previewRequest(session, ws, r, ref).then((p) => g === gen && (preview = p));
  });

  let note = $state<string | null>(null);
  let busy = $state(false);
  const editable = $derived(ws.mode === "edit" && session.seat.authority === "gm");

  async function adopt(r: ChangeRequest) {
    busy = true;
    const res = await adoptRequest(session, ws, r.id);
    busy = false;
    note = res.status === "applied" ? `Adopted. The campaign pack now carries it (stack ${res.event_id.slice(6, 14)}); every value it touches is re-derived and badged.` : res.status === "refused" ? res.reason : res.status === "pending" ? "Sent to the GM." : res.message;
  }
  async function withdraw(r: ChangeRequest) {
    busy = true;
    const res = await withdrawRequest(session, ws, r.id);
    busy = false;
    note = res.status === "applied" ? `Withdrawn. ${inForce.length ? "The other adopted changes stay." : "The base physics as shipped is back in force."}` : res.status === "refused" ? res.reason : "Could not withdraw it.";
  }

  const pretty = (k: string) => k.replace(/^[a-z]+\./, "").replace(/_/g, " ");
  const num = (v: number) => (Math.abs(v) >= 1e5 || (Math.abs(v) < 1e-3 && v !== 0) ? v.toExponential(3) : String(Number(v.toPrecision(4))));
  const pct = (a: number, b: number) => (a === 0 ? "" : `${b >= a ? "+" : "−"}${Math.abs(((b - a) / a) * 100).toFixed(1)} %`);
  const who = (r: ChangeRequest) => (r.author === "orac" ? (r.fixture ? "Orac · sample" : "Orac") : "GM");
</script>

<div class="wrap">
  <Card title="Change requests" subtitle="The campaign's physics">
    {#if !open.length && !inForce.length && !closed.length}
      <p class="prose">No changes. Change a parameter on the flow and it becomes a draft here, with its effect, for you to adopt or decline.</p>
    {/if}

    {#if open.length}
      <h3>Open</h3>
      <ul class="list" aria-label="Open requests">
        {#each open as r (r.id)}
          <li>
            <button type="button" class="row" aria-pressed={current?.id === r.id} onclick={() => ((chosen = r.id), ws.bench?.flow !== r.flow && ws.openFlow(r.flow))}>
              <span class="t">{r.title}</span>
              <span class="by" class:orac={r.author === "orac"}>{who(r)}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}

    {#if current}
      <div class="detail" role="region" aria-label="Request {current.title}">
        {#if current.rationale}<p class="why">{current.rationale}</p>{/if}
        {#if current.fixture}<p class="muted">Orac is not connected yet; this request is a sample document in the shape Orac will file.</p>{/if}
        <table class="diff">
          <caption>Diff · {current.flow}</caption>
          <thead><tr><th scope="col">Parameter</th><th scope="col" class="n">In force</th><th scope="col" class="n">Proposed</th></tr></thead>
          <tbody>
            {#each current.edits as e (e.node + e.param)}
              <tr><th scope="row">{e.node}.{e.param}</th><td class="n del">{paramText(e.before)}</td><td class="n ins">{paramText(e.after)}</td></tr>
            {/each}
          </tbody>
        </table>

        {#if !preview}
          <p class="muted" aria-live="polite">Checking the stack with the kernel…</p>
        {:else if !preview.ok}
          <div class="problems" role="alert">
            <Stamp kind="rejected" tilt={false} />
            <ul>{#each preview.problems as p (p)}<li>{p}</li>{/each}</ul>
          </div>
        {:else}
          <p class="ok" aria-live="polite"><Stamp kind="validated" tilt={false} /> The kernel accepts the stack with this change.</p>
          {#if preview.values.length}
            <table class="effect">
              <caption>Effect on {bench?.label}</caption>
              <thead><tr><th scope="col">Quantity</th><th scope="col" class="n">Now</th><th scope="col" class="n">With it</th><th scope="col" class="n">Change</th></tr></thead>
              <tbody>
                {#each preview.values as v (v.kind)}
                  {@const moved = v.before && v.after && v.before.value !== v.after.value}
                  <tr class:moved>
                    <th scope="row">{pretty(v.kind)}</th>
                    <td class="n">{v.before ? `${num(v.before.value)} ${siText(v.before.dim)}` : "–"}</td>
                    <td class="n">{v.after ? `${num(v.after.value)} ${siText(v.after.dim)}` : "–"}</td>
                    <td class="n">{moved ? pct(v.before!.value, v.after!.value) : "same"}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
            <p class="muted">The change applies to every body this flow serves, not only {bench?.label}.</p>
          {/if}
        {/if}

        {#if editable}
          <div class="actions">
            <button type="button" class="primary" disabled={!preview?.ok || busy} onclick={() => adopt(current)}>Adopt</button>
            <button type="button" disabled={busy} onclick={() => declineRequest(ws, current.id)}>Decline</button>
          </div>
        {/if}
      </div>
    {/if}

    {#if inForce.length}
      <h3>In force</h3>
      <ul class="list" aria-label="Adopted requests">
        {#each inForce as r (r.id)}
          <li class="held">
            <span class="t">{r.title}</span>
            <span class="edits">{r.edits.map((e) => `${e.node}.${e.param} ${paramText(e.before)} → ${paramText(e.after)}`).join("; ")}</span>
            {#if editable}<button type="button" class="link" disabled={busy} onclick={() => withdraw(r)}>Withdraw</button>{/if}
          </li>
        {/each}
      </ul>
    {/if}

    {#if closed.length && params.show_closed !== false}
      <details>
        <summary>Closed ({closed.length})</summary>
        <ul class="list">{#each closed as r (r.id)}<li class="held"><span class="t">{r.title}</span><span class="edits">{r.status}</span></li>{/each}</ul>
      </details>
    {/if}

    {#if note}<p class="note" role="status">{note}</p>{/if}
  </Card>
</div>

<style>
  h3 {
    margin: 0;
    padding: 12px 18px 4px;
    font-size: 11px;
    font-weight: 400;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .list {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 10px;
    width: 100%;
    min-height: 40px;
    padding: 6px 18px;
    border: 0;
    border-top: var(--sse-line) solid var(--sse-rule);
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 14px;
    text-align: left;
    cursor: pointer;
  }
  .row[aria-pressed="true"] {
    box-shadow: inset 3px 0 0 var(--sse-accent);
  }
  .t {
    font-family: var(--sse-font-display);
    font-size: 15px;
  }
  .by {
    flex: none;
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .by.orac {
    color: var(--sse-accent-ink);
  }
  .detail {
    padding: 4px 18px 12px;
    border-top: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    font-size: 13px;
    line-height: 1.5;
  }
  .why {
    margin: 8px 0 4px;
    font-family: var(--sse-font-display);
    font-size: 15px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    margin: 8px 0 4px;
  }
  caption {
    text-align: left;
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    overflow-wrap: anywhere;
  }
  th,
  td {
    padding: 3px 4px 3px 0;
    border-bottom: var(--sse-line) dotted var(--sse-rule);
    text-align: left;
    font-weight: 400;
    vertical-align: baseline;
  }
  thead th {
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  tbody th {
    font-family: var(--sse-font-mono);
    font-size: 12px;
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
  .ins,
  tr.moved td {
    color: var(--sse-accent-ink);
    font-weight: 600;
  }
  .effect tbody th {
    font-family: var(--sse-font-body);
  }
  .ok {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 8px 0 0;
  }
  .problems {
    margin: 8px 0 0;
    color: var(--sse-accent-ink);
  }
  .problems ul {
    margin: 4px 0 0;
    padding-left: 16px;
    overflow-wrap: anywhere;
  }
  .muted {
    margin: 6px 0 0;
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .prose {
    margin: 0;
    padding: 12px 18px 14px;
    font-family: var(--sse-font-display);
    font-size: 15px;
    line-height: 1.5;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }
  .actions button,
  .link {
    min-height: 40px;
    padding: 0 16px;
    border: var(--sse-line) solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 14px;
    cursor: pointer;
  }
  .actions .primary {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .held {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 2px 10px;
    align-items: center;
    padding: 8px 18px;
    border-top: var(--sse-line) solid var(--sse-rule);
  }
  .edits {
    grid-column: 1;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .held .link {
    grid-column: 2;
    grid-row: 1 / span 2;
    min-height: 36px;
    padding: 0 10px;
    font-size: 13px;
  }
  details {
    border-top: var(--sse-line) solid var(--sse-rule);
  }
  summary {
    padding: 8px 18px;
    font-size: 12px;
    color: var(--sse-ink-muted);
    cursor: pointer;
  }
  .note {
    margin: 0;
    padding: 10px 18px;
    border-top: var(--sse-line) solid var(--sse-ink);
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-accent-ink);
  }
</style>
