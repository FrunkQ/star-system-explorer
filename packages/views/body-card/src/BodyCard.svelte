<!--
  Body card view: the selected subject's facts, typed in place. Typed values
  become set_value intents through the session, like any other seat's;
  derived values are read-only and open the kernel's explanation.
-->
<script lang="ts">
  import { CURIOUS_FACTS, HONESTY_MARKS, yearsToSeconds, namesOnStack, asPlayers, curiousDisplay, curiousSentence, sayFact, subjectClass, nextReading, headline, cardFacts, cardFor, commitEdit, factEdit, gmStamps, hasTool, primaryOf, eyesShow, fieldLandmark, hasException, secrecyRelevant, tableOf, type IntentResult, type KnownChange, type KnownTo as KnownRead, type Json, type PlayerContext, type SceneFact, type SystemScene, type ViewContext, type Vocab } from "../../../ui-contract/src/index.ts";
  import PlayerCard from "../../player-stack/src/PlayerCard.svelte";
  import { Card, CardField, KnownTo, Stamp, opensFrom } from "../../../chart-grammar/src/index.ts";
  import { parseFact } from "./parse.ts";
  import Explain from "./Explain.svelte";
  import NameCard from "./NameCard.svelte";
  import Handles from "../../editing/src/Handles.svelte";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  const ws = $derived(ctx.workspace);
  const subject = $derived(scene.subjects.find((s) => s.address === ws.selection));
  // The GM's lens (U9): which fields, in what order, how dense. A field it hides is untouched.
  const L = $derived(ws.lens);
  const rule = $derived(cardFor(L, subject?.kind ?? "*"));
  const shown = $derived(subject ? cardFacts(rule, subject.facts.filter((f) => f.field !== "name" || ws.mode === "edit")) : []);
  // Depth (top-level design 2.4): Curious leads with a sentence and four headline facts in words, one tap from the rest;
  // Science is every value; Engine adds addresses and who published each value. On the Build desk the card keeps every
  // field it can edit, in the units you type, whatever the depth.
  const depth = $derived(ws.depth ?? "science");
  const curious = $derived(depth === "curious" && ws.mode !== "edit");
  let allFacts = $state(false);
  // Names (NM6): when a pack on the stack carries name schemes, "Names" opens every name the body has, by layer.
  const namesInPlay = $derived.by(() => {
    void ws.revision;
    const st = ctx.session.stack();
    return namesOnStack([...st.base, ...(st.campaign ? [st.campaign] : [])]).schemes.length > 0;
  });
  let namesOpen = $state(false);
  // Known to (SV2): who knows this body, read from the kernel on the GM's Build and Run desks. The eye shows once
  // secrecy matters here (something held back, or the campaign shared); a press reveals or hides through set_known.
  // A reread after a change keeps the eye in place (its slip carries the Undo); only another body clears it.
  let known = $state.raw<KnownRead | null>(null);
  let knownOf: string | null = null;
  $effect(() => {
    const a = subject?.address;
    void ws.revision;
    if (a !== knownOf) known = null;
    knownOf = a ?? null;
    if (!a || ctx.session.seat.authority !== "gm" || (ws.desk !== "build" && ws.desk !== "run")) return void (known = null);
    let live = true;
    void ctx.session.knownTo(a, undefined, BigInt(knownAt().at_s)).then((r) => {
      if (live && r.ok) known = r.value;
    });
    return () => (live = false);
  });
  const eye = $derived(!!known && eyesShow(ws.desk, secrecyRelevant(known, !!ws.shared)));
  const inside = $derived(!!subject && scene.subjects.some((x) => x.parent === subject.address));
  const failed = (r: IntentResult) => (r.status === "refused" ? r.reason : r.status === "pending" ? "sent to the GM" : r.status === "error" ? r.message : "");
  // Who knows what is read and changed at the campaign's now (the story's date, which players' screens follow), never
  // the sky's time: a change made at a time the read does not reach would leave the eye where it was. One exact
  // second for both, since a year in floating point is only good to about half a minute.
  function knownAt(): { at_yr: number; at_s: string } {
    const at_yr = ws.campaign_now_yr ?? ws.now_yr;
    return { at_yr, at_s: String(yearsToSeconds(at_yr)) };
  }
  async function setKnown(c: KnownChange): Promise<{ ok: true; undo?: string } | { ok: false; why: string }> {
    if (!subject) return { ok: false, why: "nothing selected" };
    const r = await ctx.session.send({ kind: "set_known", address: subject.address, ...c, ...knownAt() });
    if (r.status === "applied") return (ws.revision++, { ok: true, ...(r.event_id ? { undo: r.event_id } : {}) });
    return { ok: false, why: failed(r) };
  }
  async function undoKnown(undo: string): Promise<string | null> {
    const r = await ctx.session.send({ kind: "undo_known", undo, ...knownAt() });
    return r.status === "applied" ? (ws.revision++, null) : failed(r);
  }
  // Pencil on the card (secrets design 4): the lines the table is not sent are drawn quieter, each with its own eye.
  const tableKnows = $derived(eye && known ? tableOf(known) : null);
  const pencilled = (field: string) => !!tableKnows && fieldLandmark(tableKnows, field) !== "known";
  const listed = $derived(curious && !allFacts ? headline(shown) : shown);
  const sentence = $derived(subject && depth === "curious" ? curiousSentence(subject, scene.subjects.find((x) => x.address === subject.parent)?.label) : null);
  const published = $derived(subject && depth === "engine" ? shown.filter((f) => f.publisher) : []);
  const cols = $derived(rule.density === "full" ? Number(params.columns) : 2);
  // Evidence stamps follow the lens; honesty stamps always show on the GM view.
  const stamps = $derived(subject ? (params.stamps === false ? subject.stamps.filter((k) => HONESTY_MARKS.includes(k)) : gmStamps(L, subject.stamps)) : []);
  // "As players see it": the card turns into theirs, with the GM's honesty stamps over it.
  const through = $derived(asPlayers(L) && ws.lensScene ? ws.lensScene : null);
  const theirs = $derived(through && subject ? (subject === primaryOf(scene) ? through.scope : subject.address) : null);
  let vocab = $state<Vocab | null>(null);
  $effect(() => {
    if (!through || vocab) return;
    fetch(new URL(String(params.vocab ?? "documents/vocab.plain.json"), document.baseURI))
      .then((r) => r.json())
      .then((v) => (vocab = v));
  });
  const pctx = $derived<PlayerContext | null>(vocab && theirs ? { params: { density: rule.density === "full" ? "full" : "compact" }, vocab, selection: () => theirs, select: () => {} } : null);
  let why = $state<string | null>(null);
  let note = $state<string | null>(null);
  $effect(() => {
    void ws.selection;
    why = null;
    allFacts = false;
    note = null;
  });

  // Every figure through the units engine (ED1a): a number in a unit the sheet holds is said for this body's class
  // at the workspace depth; anything else keeps the scene's own text.
  const cls = $derived(subject ? subjectClass(subject) : "planet");
  // Typing over a figure (edit mode) shows it plainly, as Science does.
  const said = (f: SceneFact) => sayFact(f, cls, { depth: curious ? "curious" : depth === "curious" ? "science" : depth, choices: ws.unit_choices });
  // Pressing a unit steps the figure to its next reading, for every body of this class (design 3C).
  const stepUnit = (f: SceneFact) => {
    const s = said(f);
    if (s && ws.setUnitChoice) ws.setUnitChoice(s.kind, s.about, nextReading(s.kind, s.about, s.reading));
  };
  const shownOf = (f: SceneFact) => said(f)?.text ?? (curious ? curiousDisplay(f) : f.display);

  async function commit(f: SceneFact, text: string): Promise<string | null> {
    if (!subject) return null;
    const p = parseFact(f, text, said(f)?.unitId);
    if (!p.ok) return p.message;
    // A value read back from the universe goes to its kernel field, in SI; the fixture's own fields take it as typed.
    const e = factEdit(f, p.value);
    const r = await commitEdit(ctx.session, ws, { address: subject.address, field: e.field, value: e.value, display: p.display, before: { value: e.before, display: shownOf(f) } });
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
    if (f.source === "catalogued") return "it comes from a real catalogue, carried in with the import.";
    if (f.source === "fixture") return "it comes from the sample system's fixture data, not from physics. Set it to make it yours.";
    if (f.source === "resolved") return "the formation run put it here, from the cloud the system grew out of. Set it to make it yours.";
    // A value the kernel works out but has no flow to show here yet (a system read back from the universe).
    if (f.source === "derived" && f.field === "surface_k") return "the physics works it out from the star's light, this orbit and the world's air, so it follows when you change them. Open the world: the globe's Climate, Why? shows each step.";
    if (f.source === "derived") return "the physics works it out from this world's other values, so it follows when you change them.";
    return `its source is ${f.source}.`;
  }
</script>

{#snippet knownLead()}
  {#if known && subject}<KnownTo read={known} label={subject.label} hold={(ws.players_online ?? 0) > 0} children={inside} onset={setKnown} onundo={undoKnown} />{/if}
{/snippet}

<div class="wrap">
  <!-- A new subject's card opens from its body on the chart (dynamic minimalism); edits to the same subject do not. -->
  {#key subject?.address}
  <div use:opensFrom={() => (subject ? document.querySelector(`[data-address="${CSS.escape(subject.address)}"]`) : null)}>
  {#if subject}
    <Card title={subject.label} subtitle={subject.subtitle} lead={eye && known ? knownLead : undefined}>
      {#if stamps.length}
        <div class="stamps">{#each stamps as k (k)}<Stamp kind={k} />{/each}</div>
      {/if}
      {#if through}
        <p class="as" data-through={L?.id}>{L?.label}{through.subjects.some((x) => x.address === theirs) ? "" : ": not known to them"}</p>
        {#if pctx && through.subjects.some((x) => x.address === theirs)}<div class="theirs"><PlayerCard ctx={pctx} scene={through} /></div>{/if}
      {:else}
      {#if namesInPlay}
        <button type="button" class="names-open" aria-expanded={namesOpen} data-names-open onclick={() => (namesOpen = !namesOpen)}>{namesOpen ? "Hide names" : "Names ›"}</button>
        {#if namesOpen}<NameCard session={ctx.session} workspace={ws} address={subject.address} label={subject.label} {depth} onclose={() => (namesOpen = false)} />{/if}
      {/if}
      {#if ctx.describe}<button type="button" class="names-open" data-describe onclick={() => ctx.describe?.(subject.address)} title="Minerva writes a description from what one audience knows of it">Describe ›</button>{/if}
      {#if sentence}<p class="sentence" data-depth="curious">{sentence}</p>{/if}
      <!-- The handles (track ED): what you can set, one row; Build moves them, Explore shows them; Run has none. -->
      {#if ws.desk !== "run" && ctx.session.seat.authority === "gm"}<Handles {ctx} {scene} {subject} />{/if}
      {#if depth === "engine"}<p class="address" data-depth="engine" title="This body's address in the universe">{subject.address}</p>{/if}
      <div class="grid" class:line={rule.density === "line"} style="--cols: {cols}">
        {#each listed as f (f.field)}
          {#snippet lineEye()}
            {#if known && subject}<KnownTo read={known} label={subject.label} field={f.field} fieldLabel={f.label} size={11} hold={(ws.players_online ?? 0) > 0} onset={setKnown} onundo={undoKnown} />{/if}
          {/snippet}
          <CardField
            label={f.label}
            display={shownOf(f)}
            unit={said(f)?.unit || undefined}
            onunit={ws.setUnitChoice ? () => stepUnit(f) : undefined}
            set={f.source === "authored"}
            derived={f.source === "derived"}
            flagged={f.validity !== undefined && f.validity !== "in-range"}
            changed={f.changed}
            fictional={f.fictional}
            editable={f.editable && ws.mode === "edit"}
            oncommit={(t) => commit(f, t)}
            onwhy={f.field === "name" ? undefined : () => (why = why === f.field ? null : f.field)}
            pencil={pencilled(f.field)}
            known={tableKnows && f.field !== "name" ? lineEye : undefined}
            keepKnown={hasException(tableKnows, f.field)}
          />
          {#if why === f.field}
            {#if f.why}
              <Explain session={ctx.session} {depth} why={f.why} label={f.label} onclose={() => (why = null)} onopenflow={ws.mode === "edit" && hasTool(L, "flows") && ctx.session.seat.authority === "gm" ? (node) => ws.openFlow(f.why!.flow, { node, ref: { flow: f.why!.flow, inputs: f.why!.inputs }, label: `${subject.label} · ${f.label.toLowerCase()}` }) : undefined} />
            {:else}
              <p class="why" role="note">Why {f.label.toLowerCase()} is {shownOf(f)}: {whyText(f)}</p>
            {/if}
          {/if}
        {/each}
      </div>
      {#if curious && shown.length > CURIOUS_FACTS}
        <button type="button" class="all" aria-expanded={allFacts} onclick={() => (allFacts = !allFacts)}>{allFacts ? "Fewer facts" : `All ${shown.length} facts ›`}</button>
      {/if}
      {#if published.length}
        <ul class="published" data-depth="engine" aria-label="Who published each value">
          {#each published as f (f.field)}<li><span>{f.label}</span> <code>{f.publisher}</code></li>{/each}
        </ul>
      {/if}
      {/if}
      {#if params.prose && depth !== "curious" && rule.density === "full" && subject.prose && !through}<p class="prose">{subject.prose}</p>{/if}
      {#if note}<p class="note" aria-live="polite">{note}</p>{/if}
    </Card>
  {:else}
    <Card title="Nothing selected">
      <p class="prose">Pick a body on the chart, or press Tab to reach the chart and Enter to choose. The card shows its facts here; in edit mode you type new values where they are shown, and drag a body to move its orbit.</p>
    </Card>
  {/if}
  </div>
  {/key}
</div>

<style>
  .names-open {
    display: block;
    height: auto;
    margin: 6px 18px 0;
    padding: 0;
    border: none;
    background: none;
    color: var(--sse-accent-ink);
    font: inherit;
    font-size: 13px;
    text-decoration: underline;
    cursor: pointer;
  }
  .stamps {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding: 10px 18px 2px;
  }
  .as {
    margin: 0;
    padding: 8px 18px 0;
    font-size: 12px;
    color: var(--sse-accent-ink);
    letter-spacing: 0.04em;
  }
  .theirs {
    padding: 4px 6px 8px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(var(--cols), minmax(0, 1fr));
  }
  .grid.line > :global(.field) {
    padding-top: 2px;
    padding-bottom: 2px;
  }
  .grid > :global(.field) {
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .sentence {
    margin: 0;
    padding: 12px 18px 10px;
    font-family: var(--sse-font-display);
    font-size: 16px;
    line-height: 1.45;
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .address {
    margin: 0;
    padding: 8px 18px;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
    border-bottom: var(--sse-line) solid var(--sse-rule);
    overflow-wrap: anywhere;
  }
  .all {
    display: block;
    width: 100%;
    min-height: 40px;
    padding: 8px 18px;
    border: 0;
    background: transparent;
    text-align: left;
    font-family: var(--sse-font-body);
    font-size: 13px;
    color: var(--sse-accent-ink);
    cursor: pointer;
  }
  .all:hover {
    text-decoration: underline;
  }
  .published {
    list-style: none;
    margin: 0;
    padding: 8px 18px 10px;
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .published code {
    font-family: var(--sse-font-mono);
    overflow-wrap: anywhere;
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
