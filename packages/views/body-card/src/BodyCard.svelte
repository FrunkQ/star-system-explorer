<!--
  Body card view: the selected subject's facts, typed in place. Typed values
  become set_value intents through the session, like any other seat's;
  derived values are read-only and open the kernel's explanation.
-->
<script lang="ts">
  import { commitEdit, type Json, type SceneFact, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { Card, CardField, Stamp } from "../../../chart-grammar/src/index.ts";
  import { parseFact } from "./parse.ts";
  import Explain from "./Explain.svelte";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  const ws = $derived(ctx.workspace);
  const subject = $derived(scene.subjects.find((s) => s.address === ws.selection));
  const shown = $derived(subject ? subject.facts.filter((f) => f.field !== "name" || ws.mode === "edit") : []);
  let why = $state<string | null>(null);
  let note = $state<string | null>(null);
  $effect(() => {
    void ws.selection;
    why = null;
    note = null;
  });

  async function commit(f: SceneFact, text: string): Promise<string | null> {
    if (!subject) return null;
    const p = parseFact(f, text);
    if (!p.ok) return p.message;
    const r = await commitEdit(ctx.session, ws, { address: subject.address, field: f.field, value: p.value, display: p.display, before: { value: f.value, display: f.display } });
    switch (r.status) {
      case "applied":
        note = `Inked at ${(ws.now_yr / 1e9).toFixed(2)} Gyr as event ${r.event_id.slice(0, 8)}`;
        return null;
      case "pending":
        return "Sent to the GM to approve.";
      case "refused":
        return r.reason;
      case "error":
        return `The kernel refused this: ${r.message}`;
    }
  }

  function whyText(f: SceneFact): string {
    if (f.source === "authored") return "you set it. It feeds every value derived from it.";
    if (f.source === "fixture") return "it comes from the sample system's fixture data, not from physics. Set it to make it yours.";
    return `its source is ${f.source}.`;
  }
</script>

<div class="wrap">
  {#if subject}
    <Card title={subject.label} subtitle={subject.subtitle}>
      {#if params.stamps && subject.stamps.length}
        <div class="stamps">{#each subject.stamps as k (k)}<Stamp kind={k} />{/each}</div>
      {/if}
      <div class="grid" style="--cols: {Number(params.columns)}">
        {#each shown as f (f.field)}
          <CardField
            label={f.label}
            display={f.display}
            set={f.source === "authored"}
            derived={f.source === "derived"}
            flagged={f.validity !== undefined && f.validity !== "in-range"}
            changed={f.changed}
            editable={f.editable && ws.mode === "edit"}
            oncommit={(t) => commit(f, t)}
            onwhy={f.field === "name" ? undefined : () => (why = why === f.field ? null : f.field)}
          />
          {#if why === f.field}
            {#if f.why}
              <Explain session={ctx.session} why={f.why} label={f.label} onclose={() => (why = null)} onopenflow={ws.mode === "edit" && ctx.session.seat.authority === "gm" ? (node) => ws.openFlow(f.why!.flow, { node, ref: { flow: f.why!.flow, inputs: f.why!.inputs }, label: `${subject.label} · ${f.label.toLowerCase()}` }) : undefined} />
            {:else}
              <p class="why" role="note">Why {f.label.toLowerCase()} is {f.display}: {whyText(f)}</p>
            {/if}
          {/if}
        {/each}
      </div>
      {#if params.prose && subject.prose}<p class="prose">{subject.prose}</p>{/if}
      {#if note}<p class="note" aria-live="polite">{note}</p>{/if}
    </Card>
  {:else}
    <Card title="Nothing selected">
      <p class="prose">Pick a body on the chart, or press Tab to reach the chart and Enter to choose. The card shows its facts here; in edit mode you type new values where they are shown, and drag a body to move its orbit.</p>
    </Card>
  {/if}
</div>

<style>
  .stamps {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding: 10px 18px 2px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(var(--cols), minmax(0, 1fr));
  }
  .grid > :global(.field) {
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .why {
    grid-column: 1 / -1;
    margin: 0;
    padding: 10px 18px;
    background: var(--sse-ground);
    border-bottom: var(--sse-line) solid var(--sse-rule);
    font-size: 13px;
    line-height: 1.5;
  }
  .prose {
    margin: 0;
    padding: 12px 18px 14px;
    font-family: var(--sse-font-display);
    font-size: 15px;
    line-height: 1.5;
  }
  .note {
    margin: 0;
    padding: 10px 18px;
    border-top: var(--sse-line) solid var(--sse-ink);
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-accent-ink);
  }
  @media (max-width: 420px) {
    .grid {
      grid-template-columns: repeat(min(var(--cols), 2), minmax(0, 1fr));
    }
  }
</style>
