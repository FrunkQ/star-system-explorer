<!--
  Body card view: the selected subject's facts, typed in place. A typed value
  becomes a set_value intent through the session, like any other seat's.
-->
<script lang="ts">
  import type { Json, SceneFact, SystemScene, ViewContext } from "../../../ui-contract/src/index.ts";
  import { Card, CardField, Stamp } from "../../../chart-grammar/src/index.ts";
  import { parseFact } from "./parse.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  const ws = $derived(ctx.workspace);
  const subject = $derived(scene.subjects.find((s) => s.address === ws.selection));
  const applied = (f: SceneFact) => (subject ? ws.applied[`${subject.address}#${f.field}`] : undefined);
  const title = $derived.by(() => {
    if (!subject) return "";
    const n = ws.applied[`${subject.address}#name`];
    return n ? String(n.value) : subject.label;
  });
  const shown = $derived(subject ? subject.facts.filter((f) => f.field !== "name" || ws.mode === "edit") : []);
  const authoredHere = $derived(subject ? Object.keys(ws.applied).some((k) => k.startsWith(`${subject.address}#`)) : false);
  const stamps = $derived(subject ? [...new Set([...subject.stamps, ...(authoredHere ? ["authored"] : [])])] : []);
  let why = $state<string | null>(null);
  let note = $state<string | null>(null);

  async function commit(f: SceneFact, text: string): Promise<string | null> {
    if (!subject) return null;
    const p = parseFact(f, text);
    if (!p.ok) return p.message;
    const r = await ctx.session.send({ kind: "set_value", address: subject.address, field: f.field, value: p.value, at_yr: ws.now_yr });
    switch (r.status) {
      case "applied":
        ws.noteApplied(subject.address, f.field, p.value, p.display, r.event_id);
        note = `Inked at ${scene.title} ${fmt(ws.now_yr)} as event ${r.event_id.slice(0, 8)}`;
        return null;
      case "pending":
        return "Sent to the GM to approve.";
      case "refused":
        return r.reason;
      case "error":
        return `The kernel refused this: ${r.message}`;
    }
  }
  const fmt = (yr: number) => `${(yr / 1e9).toFixed(2)} Gyr`;
</script>

<div class="wrap">
  {#if subject}
    <Card title={title} subtitle={subject.subtitle}>
      {#if params.stamps && stamps.length}
        <div class="stamps">{#each stamps as k (k)}<Stamp kind={k} />{/each}</div>
      {/if}
      <div class="grid" style="--cols: {Number(params.columns)}">
        {#each shown as f (f.field)}
          {@const a = applied(f)}
          <CardField
            label={f.label}
            display={a ? a.display : f.display}
            set={!!a || f.source === "authored"}
            editable={f.editable && ws.mode === "edit"}
            oncommit={(t) => commit(f, t)}
            onwhy={f.field === "name" ? undefined : () => (why = why === f.field ? null : f.field)}
          />
          {#if why === f.field}
            <p class="why" role="note">
              Why {f.label.toLowerCase()} is {a ? a.display : f.display}: {a
                ? "you set it."
                : f.source === "fixture"
                  ? "it comes from the fixture scene, not from physics. Explanations open here once the physics library is in (U2)."
                  : `its source is ${f.source}.`}
            </p>
          {/if}
        {/each}
      </div>
      {#if params.prose && subject.prose}<p class="prose">{subject.prose}</p>{/if}
      {#if note}<p class="note" aria-live="polite">{note}</p>{/if}
    </Card>
  {:else}
    <Card title="Nothing selected">
      <p class="prose">Pick a body on the chart, or press Tab to reach the chart and Enter to choose. The card shows its facts here, and in edit mode you type new values where they are shown.</p>
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
    border-top: 0;
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
