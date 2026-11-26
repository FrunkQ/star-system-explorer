<!--
  The Formation screen (formation design section 7, F10, the reading half): the nine stages as a strip to jump along,
  what the system inherited from its patch and how its history was worked out on the left, the formation chart, and
  the chosen world's story on the right. "Watch it form" plays the same record as the replay. The time slice is the time ruler's
  date: moving the slice moves the ruler (and every other view) when let go, and moving the ruler moves the slice.
  Read from the system's own record (kernel `formation_record`); the history is stored, so moving the slice
  recomputes nothing.
-->
<script lang="ts">
  import { chartTime, formationChart, formationOf, howWorkedOut, noWorldsWhy, stageAt, storyFor, worldStory, type FormationSource, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import FormationReplay from "../../system-chart/src/FormationReplay.svelte";
  import FormationChart from "./FormationChart.svelte";
  import StoryCard from "./StoryCard.svelte";

  let { ctx, scene }: { ctx: ViewContext; scene: SystemScene } = $props();

  let source = $state<FormationSource | null>(null);
  let gen = 0;
  // Only a change of system reads the record again: a new scene for a new date (the slice moving the ruler) does not.
  const system = $derived(scene.system);
  $effect(() => {
    const sys = system;
    const g = ++gen;
    source = null;
    void formationOf(ctx.session, sys).then((s) => {
      if (g === gen) source = s;
    });
  });

  const noWorlds = $derived(source?.kind === "run" ? noWorldsWhy(source.run) : null);
  const names = $derived(Object.fromEntries(scene.subjects.map((x) => [x.address, x.label])));
  const chart = $derived(source?.kind === "run" ? formationChart(source.run, { names, seed: source.seed }) : null);

  // The slice follows the ruler; while it is being dragged it is the screen's own until let go.
  let held = $state<number | null>(null);
  const fromRuler = $derived(chart ? Math.max(chart.t_top, Math.min(chart.t_bottom, ctx.workspace.now_yr / 1e6)) : 1);
  const slice = $derived(held ?? fromRuler);
  const stage = $derived(chart ? stageAt(chart, slice) : null);
  function commit(t: number) {
    held = null;
    ctx.workspace.setTime(t * 1e6);
  }
  const worldsAt = $derived(chart ? chart.tracks.filter((k) => k.born.t <= slice && (k.end.how === "lasts" || k.end.t > slice)).length : 0);
  // The story shows the track pressed here (a lost body has no address, so it is the screen's own pick), else the
  // world selected anywhere in the app, else the heaviest world.
  let picked = $state<string | null>(null);
  const storyId = $derived.by(() => {
    if (!chart) return null;
    const p = picked ? chart.tracks.find((k) => k.id === picked) : undefined;
    if (p && (!p.address || p.address === ctx.workspace.selection)) return p.id;
    return storyFor(chart, ctx.workspace.selection);
  });
  const chosen = $derived(!!storyId && (storyId === picked || chart?.tracks.find((k) => k.id === storyId)?.address === ctx.workspace.selection));
  const story = $derived(chart && storyId && source?.kind === "run" ? worldStory(source.run, chart, storyId) : null);
  function pick(id: string) {
    picked = id;
    const a = chart?.tracks.find((k) => k.id === id)?.address;
    if (a) ctx.workspace.select(a);
  }
  const how = $derived(source?.kind === "run" ? howWorkedOut(source.run) : []);

  // "Watch it form": the replay of the same record, over the page.
  let watching = $state(false);
  let dlg = $state<HTMLDialogElement | undefined>();
  $effect(() => {
    if (!dlg) return;
    if (watching && !dlg.open) (dlg.showModal(), dlg.querySelector<HTMLElement>(".replay")?.focus());
    else if (!watching && dlg.open) dlg.close();
  });
</script>

<div class="formation" data-formation={source?.kind ?? "loading"}>
  {#if !source}
    <p class="quiet" data-formation-loading>Reading how {scene.title} was made…</p>
  {:else if source.kind !== "run"}
    <div class="none" data-formation-none={source.kind === "none" ? source.source : "failed"}>
      <h2>No formation history for {scene.title}</h2>
      <p>{source.why}</p>
      {#if source.kind === "none"}
        <p class="hint">A system grown with <b>Make a system › Grow a new system</b> keeps its whole history, and it shows here: every world's track from the disc to now.</p>
      {/if}
    </div>
  {:else if chart && stage}
    <nav class="stages" aria-label="The nine stages">
      <button type="button" class="play" onclick={() => (watching = true)} data-watch-form>▶ Watch it form</button>
      <span class="k">Stage</span>
      {#each chart.stages as st (st.n)}
        <button type="button" aria-current={stage.n === st.n ? "step" : undefined} onclick={() => commit(st.from)} title={st.text} data-stage={st.n}>{st.n} {st.name}</button>
      {/each}
    </nav>
    <p class="stage-text" data-stage-text>{stage.text}</p>
    {#if noWorlds}<p class="drift" data-formation-empty>{noWorlds}</p>{/if}
    {#if source.drift}
      <p class="drift" data-formation-drift>Grown again with today's formation rules, which have changed since this system was added. The history may differ in detail from the facts on the chart.</p>
    {/if}

    <div class="body">
      <aside class="panel inherited" aria-label="Inherited from its patch">
        <h2>Inherited <span class="k">from its patch</span></h2>
        <dl>
          {#each chart.inherited as r (r.key)}
            <div title={r.title} data-inherited={r.key}><dt>{r.label}</dt><dd>{r.value}</dd></div>
          {/each}
        </dl>
        <h3 class="k">Worked out at</h3>
        <dl>
          {#each chart.worked_at as r (r.key)}
            <div title={r.title} data-worked={r.key}><dt>{r.label}</dt><dd>{r.value}</dd></div>
          {/each}
        </dl>
      </aside>


      <section class="chart" aria-label="Formation chart">
        <FormationChart {chart} {slice} selected={storyId} onslice={(t) => (held = t)} onslicedone={commit} onpick={pick} />
        <p class="at" data-slice-words>At {chartTime(slice)}: stage {stage.n}, {stage.name.toLowerCase()} · {worldsAt} {worldsAt === 1 ? "body" : "bodies"}{chart.gas_until > slice ? " · gas still in the disc" : ""}</p>
        <details class="how" data-how-panel>
          <summary>How this was worked out <span class="k">one seed</span></summary>
          <dl>
            {#each how as r (r.label)}
              <div title={r.title} data-how={r.value}><dt>{r.label}</dt><dd>{r.value}</dd></div>
            {/each}
          </dl>
          <p class="foot">Nothing is worked out again when you move the slice: the history is stored.</p>
        </details>
      </section>

      {#if story}<StoryCard {story} {chosen} />{/if}
    </div>
    <dialog class="replay-dlg" bind:this={dlg} aria-label="How {scene.title} formed" onclose={() => (watching = false)}>
      {#if watching}<FormationReplay run={source.run} title={scene.title} onclose={() => (watching = false)} view={ctx.workspace.view} onview={(c) => ctx.workspace.pickView?.(c)} />{/if}
    </dialog>
  {/if}
</div>

<style>
  .formation {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
    height: 100%;
    min-height: 0;
    padding: 12px 16px;
    box-sizing: border-box;
    color: var(--sse-ink);
    background: var(--sse-ground);
    font-family: var(--sse-font-body);
    overflow: auto;
  }
  .k {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    font-weight: normal;
  }
  .stages {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }
  .stages .k {
    margin-right: 6px;
  }
  .stages button {
    font: inherit;
    font-size: 13px;
    min-height: 32px;
    padding: 0 10px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    background: var(--sse-ground);
    color: var(--sse-ink);
    cursor: pointer;
    transition: transform 120ms ease-out;
  }
  .stages button:hover {
    transform: translateY(-1px);
    border-color: var(--sse-ink);
  }
  .stages button:focus-visible {
    outline: 2px solid var(--sse-accent);
    outline-offset: 1px;
  }
  .stages button[aria-current] {
    background: var(--sse-ink);
    color: var(--sse-ground);
    border-color: var(--sse-ink);
  }
  .stage-text,
  .drift,
  .at {
    margin: 0;
    font-size: 14px;
    line-height: 1.45;
    max-width: 60em;
  }
  .stage-text {
    color: var(--sse-ink-muted);
  }
  .drift {
    padding: 6px 10px;
    border-left: 3px solid var(--sse-accent);
    background: color-mix(in srgb, var(--sse-accent) 8%, var(--sse-ground));
  }
  .body {
    display: grid;
    grid-template-columns: minmax(220px, 260px) minmax(0, 1fr) minmax(250px, 300px);
    grid-template-areas: "inh chart story";
    gap: 16px;
    flex: 1 1 0;
    min-height: 420px;
    min-width: 0;
  }
  .inherited {
    grid-area: inh;
  }
  .how {
    font-size: 13px;
  }
  .how summary {
    cursor: pointer;
    font-family: var(--sse-font-display);
    font-size: 16px;
    padding: 4px 0;
  }
  .how summary:hover {
    color: var(--sse-accent-ink);
  }
  .how dl {
    max-width: 40em;
  }
  .chart {
    grid-area: chart;
  }
  .body > :global(.story) {
    grid-area: story;
  }
  .foot {
    margin: 8px 0 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .play {
    font: inherit;
    font-size: 13px;
    min-height: 32px;
    padding: 0 12px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ink);
    color: var(--sse-ground);
    cursor: pointer;
    margin-right: 10px;
  }
  .replay-dlg {
    width: min(1400px, 100vw);
    height: min(920px, 100dvh);
    max-width: 100vw;
    max-height: 100dvh;
    padding: 0;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
  }
  .replay-dlg::backdrop {
    background: color-mix(in srgb, var(--sse-ink) 35%, transparent);
  }
  .panel {
    border: var(--sse-line) solid var(--sse-ink);
    box-shadow: 4px 4px 0 var(--sse-ink);
    padding: 12px 14px;
    align-self: start;
    max-height: calc(100% - 6px);
    overflow: auto;
    box-sizing: border-box;
    background: var(--sse-ground);
  }
  .panel h2 {
    margin: 0 0 8px;
    font-family: var(--sse-font-display);
    font-weight: normal;
    font-size: 20px;
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
  }
  .panel h3 {
    margin: 14px 0 4px;
  }
  dl {
    margin: 0;
  }
  dl div {
    display: flex;
    justify-content: space-between;
    gap: 10px;
    padding: 6px 0;
    border-bottom: var(--sse-line) solid var(--sse-ink-faint);
    font-size: 13px;
  }
  dt {
    color: var(--sse-ink);
  }
  dd {
    margin: 0;
    font-family: var(--sse-font-mono);
    text-align: right;
  }
  .chart {
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
    gap: 4px;
  }
  .chart > :global(.fchart) {
    flex: 1 1 0;
  }
  .at {
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .none {
    max-width: 38em;
    margin: 40px auto;
  }
  .none h2 {
    font-family: var(--sse-font-display);
    font-weight: normal;
  }
  .quiet,
  .hint {
    color: var(--sse-ink-muted);
  }
  @media (max-width: 1180px) {
    .body {
      grid-template-columns: minmax(220px, 260px) minmax(0, 1fr);
      grid-template-areas: "inh chart" "story story";
    }
  }
  @media (max-width: 800px) {
    .body {
      grid-template-columns: 1fr;
      grid-template-areas: "chart" "story" "inh";
    }
    .chart {
      min-height: 480px;
    }
    .stages {
      flex-wrap: nowrap;
      overflow-x: auto;
      padding-bottom: 4px;
    }
    .stages button {
      font-size: 12px;
      padding: 0 8px;
      flex: none;
    }
    .formation {
      padding: 8px 12px;
    }
  }
</style>
