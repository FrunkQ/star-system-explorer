<!--
  Card: one subject's key facts as the audience has them. Unknown facts keep
  their placeholder ("not yet scanned"), so players can see what a scan would
  add. Fields listed in `fields` come first, in that order.
-->
<script lang="ts">
  import { controlOn, describeFact, subjectName, type PlayerContext, type PlayerScene } from "../../../ui-contract/src/index.ts";
  import { childrenOf, focusOf } from "./pick.ts";

  let { ctx, scene }: { ctx: PlayerContext; scene: PlayerScene } = $props();
  const subject = $derived(focusOf(scene, ctx.selection()));
  const order = $derived((ctx.params.fields as string[]) ?? []);
  const shown = $derived.by(() => {
    if (!subject) return [];
    const only = ctx.params.only_fields === true && order.length > 0;
    const facts = subject.facts.filter((f) => f.field !== "name" && (ctx.params.unknown !== false || f.placeholder === undefined) && (!only || order.includes(f.field)));
    const rank = (f: string) => (order.includes(f) ? order.indexOf(f) : order.length);
    // Facts not given that read the same ("Air: not yet scanned" for each withheld gas) are said once.
    const said = new Set<string>();
    return [...facts]
      .sort((a, b) => rank(a.field) - rank(b.field))
      .map((f) => ({ f, d: describeFact(ctx.vocab, f) }))
      .filter(({ d }) => d.state !== "unknown" || (!said.has(`${d.label}|${d.text}`) && (said.add(`${d.label}|${d.text}`), true)));
  });
  const moons = $derived(subject ? childrenOf(scene, subject.address) : []);
</script>

<article class="pcard" class:compact={ctx.params.density === "compact"} aria-label={subject ? subjectName(subject) : "No subject"}>
  {#if subject}
    <header>
      <h2>{subjectName(subject)}</h2>
      {#if ctx.params.show_view !== false}<span class="view">{subject.view.replace(/-/g, " ")}</span>{/if}
    </header>
    {#if subject.tags.length && ctx.params.show_tags !== false}
      <ul class="tags" aria-label="Tags">{#each subject.tags as t (t)}<li>{ctx.vocab.tags?.[t] ?? t}</li>{/each}</ul>
    {/if}
    <dl>
      {#each shown as { f, d } (f.field)}
        <div class="row" class:unknown={d.state === "unknown"} data-field={f.field}>
          <dt>{d.label}</dt>
          <dd>{d.text}{#if f.marks?.length}<span class="marks"> · {f.marks.join(", ")}</span>{/if}</dd>
        </div>
      {/each}
    </dl>
    {#if moons.length && ctx.params.show_moons !== false}
      <p class="moons">Moons: {#each moons as m, i (m.address)}{i ? ", " : ""}{#if controlOn(ctx.controls, "open")}<button type="button" onclick={() => ctx.select(m.address)}>{subjectName(m)}</button>{:else}{subjectName(m)}{/if}{/each}</p>
    {/if}
  {:else}
    <p class="empty">Nothing in view.</p>
  {/if}
</article>

<style>
  .pcard {
    background: var(--sse-panel-ground);
    border: var(--sse-frame-width) solid var(--sse-frame-line);
    border-radius: var(--sse-frame-radius);
    box-shadow: var(--sse-frame-shadow);
    color: var(--sse-panel-ink);
    text-shadow: var(--sse-text-glow);
    overflow: hidden;
    font-family: var(--sse-font-body);
  }
  /* The cap: the frame part that carries a panel's title. */
  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 10px;
    padding: 14px 18px 8px;
    background: var(--sse-cap-ground);
    color: var(--sse-cap-ink);
    border-bottom: var(--sse-line) solid var(--sse-cap-rule);
  }
  header::before {
    content: "";
    flex: 0 0 var(--sse-cap-end);
    align-self: stretch;
    margin-right: calc(var(--sse-cap-end) * 0.2 - 10px);
    border-radius: var(--sse-cap-end-radius);
    background: var(--sse-cap-end-colour);
  }
  h2 {
    flex: 1;
    margin: 0;
    font-family: var(--sse-title-font);
    font-size: var(--sse-title-size);
    font-style: var(--sse-title-style);
    text-transform: var(--sse-title-case);
    font-weight: 600;
  }
  .view {
    font-size: 11px;
    text-transform: var(--sse-label-case);
    letter-spacing: var(--sse-label-tracking);
  }
  dt {
    font-size: 11px;
    text-transform: var(--sse-label-case);
    letter-spacing: var(--sse-label-tracking);
    color: var(--sse-panel-muted);
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin: 0;
    padding: 10px 18px 0;
    list-style: none;
  }
  .tags li {
    border: var(--sse-line) solid var(--sse-tag-ink);
    border-radius: var(--sse-tag-radius);
    background: var(--sse-tag-ground);
    color: var(--sse-tag-ink);
    font-size: 11px;
    letter-spacing: var(--sse-label-tracking);
    text-transform: var(--sse-label-case);
    padding: 0 8px;
  }
  .compact dl {
    padding: 2px 0;
  }
  .compact .row {
    padding-top: 1px;
    padding-bottom: 1px;
  }
  dl {
    margin: 0;
    padding: 6px 0 4px;
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 12px;
    padding: 6px 18px;
    border-bottom: var(--sse-line) var(--sse-row-rule-style) var(--sse-row-rule);
  }
  dd {
    margin: 0;
    text-align: right;
    font-family: var(--sse-value-font);
    color: var(--sse-value-ink);
    font-size: 14px;
  }
  .unknown dd {
    font-family: var(--sse-title-font);
    font-style: var(--sse-unknown-style);
    color: var(--sse-panel-muted);
  }
  .marks {
    color: var(--sse-accent-ink);
    font-size: 11px;
  }
  .moons,
  .empty {
    margin: 0;
    padding: 10px 18px 14px;
    font-size: 13px;
  }
  .moons button {
    border: 0;
    background: none;
    padding: 4px 0;
    color: var(--sse-accent-ink);
    font: inherit;
    text-decoration: underline;
    cursor: pointer;
  }
</style>
