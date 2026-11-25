<!--
  The LifeFlow board (U7 part 2): one world's life pipeline as a flow. Each stage (habitats, energy budget, origin,
  opportunity, catastrophes, innovations, ecosystem, traits, fluxes, signatures, story hooks) shows what the kernel's
  life_view gives for it, wired to the stages it takes from; a stage with nothing for this world is drawn faint. Pick
  a stage for its card: what it does, its results, the dated events it made with the kernel's reasons, and the dials
  it leans on as knobs. Turning a knob asks the kernel for a what-if (life_view with other dial values, read only: the
  campaign does not change) and marks every stage that moved, with what it was and what it is now. A kernel that does
  not take dials yet is found out once per world, and the knobs then stay read only and say so. Nothing here computes
  biology. On a phone the flow is a list, top to bottom.
-->
<script lang="ts">
  import { tick } from "svelte";
  import { lifeFlow, lifeWords, placeStages, whatMoved, type LifeView, type StageId, type LifeWhatIf } from "../../../ui-contract/src/index.ts";

  let { view, lens, dialWords, ask }: { view: LifeView; lens: "physics" | "ecology" | "history"; dialWords: Record<string, string>; ask?: (dials: Record<string, number>) => Promise<LifeWhatIf> } = $props();

  // The what-if: dial values the GM has turned, the kernel's answer under them, and whether this kernel takes dials.
  let turned = $state<Record<string, number>>({});
  let alt = $state.raw<LifeView | null>(null);
  let takes = $state<boolean | null>(null);
  let asking = $state(false);
  let whatIfNote = $state<string | null>(null);
  let wgen = 0;
  $effect(() => {
    const v = view;
    const q = ask;
    const g = ++wgen;
    ((turned = {}), (alt = null), (whatIfNote = null), (takes = null));
    const first = Object.entries(v.dials)[0];
    if (!q || !first) return void (takes = false);
    // Found out once per world: ask with one dial moved and see whether the answer has it in force.
    void q({ [first[0]]: first[1] + 0.5 }).then((r) => g === wgen && (takes = r.ok && r.honoured));
  });
  async function turn(id: string, value: number) {
    if (!ask) return;
    const next = { ...turned, [id]: value };
    if (Math.abs(value - (view.dials[id] ?? value)) < 1e-9) delete next[id];
    turned = next;
    if (!Object.keys(next).length) return void ((alt = null), (whatIfNote = null));
    const g = wgen;
    asking = true;
    const r = await ask({ ...view.dials, ...next });
    if (g !== wgen) return;
    asking = false;
    if (!r.ok) return void ((whatIfNote = `The kernel could not work this what-if out: ${r.message}`), (alt = null));
    if (!r.honoured) return void ((takes = false), (turned = {}), (alt = null));
    ((alt = r.view), (whatIfNote = null));
  }
  function backToPack() {
    ((turned = {}), (alt = null), (whatIfNote = null));
  }
  const moves = $derived(alt ? whatMoved(view, alt) : []);
  const moved = $derived(new Map(moves.map((m) => [m.id, m])));
  const shown = $derived(alt ?? view);

  const NW = 124;
  const NH = 60;
  const board = $derived.by(() => {
    const stages = lifeFlow(shown);
    return { stages, ...placeStages(stages, { w: NW, h: NH, gapX: 22, gapY: 16 }) };
  });
  let pick = $state<StageId | null>(null);
  $effect(() => {
    void view;
    pick = null;
  });
  const open = $derived(board.stages.find((s) => s.id === pick) ?? null);
  const near = $derived(new Set(open ? [open.id, ...open.from, ...board.stages.filter((s) => s.from.includes(open.id)).map((s) => s.id)] : []));
  const labelOf = (id: StageId) => board.stages.find((s) => s.id === id)?.label ?? id;
  const dialText = (v: number) => (v === 1 ? "Earth-like pace" : `${v} × Earth's pace`);
  let card: HTMLElement | null = $state(null);
  async function choose(id: StageId) {
    pick = pick === id ? null : id;
    await tick();
    if (pick && matchMedia("(max-width: 720px)").matches) card?.scrollIntoView({ block: "nearest" });
  }
</script>

<section class="flow" aria-labelledby="flow-h" data-lifeflow={board.stages.filter((s) => s.live).length}>
  <h3 id="flow-h">How this life was worked out</h3>
  <p class="hint">Each stage takes from the ones wired into it. Pick one for what it does and what it gave; faint stages had nothing to give on this world.</p>

  {#if alt}
    <div class="whatif" role="status" data-whatif={moves.length}>
      <p>
        <b>What if</b>
        {Object.entries(turned).map(([id, v]) => `${dialWords[id] ?? lifeWords(id)} at ${dialText(v)}`).join(", ")}:
        {moves.length ? `${moves.length} ${moves.length === 1 ? "stage moved" : "stages moved"}, marked on the board.` : "nothing on this world moves."}
        Only a look: nothing in the campaign changes.
      </p>
      <button type="button" onclick={backToPack} data-whatif-back>Back to the pack's values</button>
    </div>
  {/if}
  {#if whatIfNote}<p class="note" role="status">{whatIfNote}</p>{/if}

  <div class="board" aria-busy={asking}>
    <svg class="wires-board" width={board.width + 8} height={board.height + 8} viewBox="-4 -4 {board.width + 8} {board.height + 8}" role="group" aria-label="Life pipeline">
      {#each board.wires as w (w.from + w.to)}
        <path d={w.d} class="wire" class:dead={!w.live} class:near={open && (w.from === open.id || w.to === open.id)} data-wire="{w.from}>{w.to}" />
      {/each}
      {#each board.placed as p (p.stage.id)}
        <g
          class="node"
          class:dead={!p.stage.live}
          class:on={pick === p.stage.id}
          class:dim={open && !near.has(p.stage.id)}
          class:moved={moved.has(p.stage.id)}
          data-moved={moved.has(p.stage.id) || undefined}
          transform="translate({p.x},{p.y})"
          role="button"
          tabindex="0"
          aria-pressed={pick === p.stage.id}
          aria-label="{p.stage.label}: {p.stage.headline}"
          data-stage={p.stage.id}
          data-live={p.stage.live}
          onclick={() => choose(p.stage.id)}
          onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), choose(p.stage.id))}
        >
          <rect width={NW} height={NH} rx="3" />
          <text x="9" y="20" class="nl">{p.stage.label}</text>
          <text x="9" y="40" class="nh">{p.stage.headline.length > 20 ? `${p.stage.headline.slice(0, 19)}…` : p.stage.headline}</text>
          {#if moved.has(p.stage.id)}<text x={NW - 9} y="20" text-anchor="end" class="nm">moved</text>{/if}
          {#if p.stage.events.length || p.stage.dials.length}<text x="9" y="54" class="ne">{[p.stage.events.length ? `${p.stage.events.length} dated` : "", p.stage.dials.length ? `${p.stage.dials.length} ${p.stage.dials.length === 1 ? "dial" : "dials"}` : ""].filter((x) => x).join(" · ")}</text>{/if}
        </g>
      {/each}
    </svg>
  </div>

  <!-- A phone: the same stages as a list, each saying what it takes from. -->
  <ol class="list" aria-label="Life pipeline, in order">
    {#each board.stages as s (s.id)}
      <li class:dead={!s.live} class:moved={moved.has(s.id)}>
        <button type="button" aria-pressed={pick === s.id} data-stage-row={s.id} onclick={() => choose(s.id)}>
          <span class="nl">{s.label}</span>
          <span class="nh">{s.headline}</span>
          {#if moved.has(s.id)}<span class="from">moved in this what-if</span>{/if}
          {#if s.from.length}<span class="from">from {s.from.map(labelOf).join(", ")}</span>{/if}
        </button>
      </li>
    {/each}
  </ol>

  {#if open}
    <article class="card" aria-labelledby="stage-h" data-stage-card={open.id} bind:this={card}>
      <header>
        <h4 id="stage-h">{open.label}</h4>
        <button type="button" class="close" aria-label="Close {open.label}" onclick={() => (pick = null)}>×</button>
      </header>
      <p>{open.does}</p>
      {#if open.from.length}<p class="hint">Takes from {open.from.map(labelOf).join(", ")}.</p>{/if}
      {#if moved.get(open.id)}
        {@const m = moved.get(open.id)!}
        <div class="was" data-stage-moved>
          <p><b>In this what-if</b>: {m.was === m.now ? m.now : `was ${m.was}, now ${m.now}`}{m.wasLive !== m.nowLive ? (m.nowLive ? "; it now has something to give" : "; it now has nothing to give") : ""}.</p>
          {#if m.added.length}<p>New: {m.added.join("; ")}.</p>{/if}
          {#if m.gone.length}<p>Gone: {m.gone.join("; ")}.</p>{/if}
        </div>
      {/if}
      {#if !open.live}<p class="note" data-stage-empty>Nothing on this world reached this stage.</p>{/if}
      {#if open.id === "hooks" && view.hooks.length}
        <ul class="lines">{#each view.hooks as h (h.id)}<li data-hook={h.id}><b>{h.text}.</b> {h.why[lens]}</li>{/each}</ul>
      {:else if open.events.length}
        <ul class="lines" data-stage-events={open.events.length}>
          {#each open.events as e (e.id)}<li data-event={e.kind}>{e.why}{e.causes.length ? ` (after ${e.causes.map((c) => lifeWords(c.replace(/^[a-z]+:/, ""))).join(", ")})` : ""}</li>{/each}
        </ul>
      {:else if open.lines.length}
        <ul class="lines">{#each open.lines as l, i (i)}<li>{l}</li>{/each}</ul>
      {/if}
      {#if open.dials.length}
        <div class="dials" data-stage-dials={open.dials.length}>
          <h5>The dials it leans on</h5>
          {#each open.dials as d (d.id)}
            <label class="dial">
              <span>{dialWords[d.id] ?? lifeWords(d.id)}</span>
              <input
                type="range"
                min="0"
                max="3"
                step="0.1"
                value={turned[d.id] ?? d.value}
                disabled={!takes || asking}
                aria-label="{dialWords[d.id] ?? lifeWords(d.id)}: {dialText(turned[d.id] ?? d.value)}"
                data-dial={d.id}
                data-turned={d.id in turned || undefined}
                onchange={(e) => void turn(d.id, Number((e.currentTarget as HTMLInputElement).value))}
              />
              <span class="val">{dialText(turned[d.id] ?? d.value)}{d.id in turned ? ` (pack: ${dialText(view.dials[d.id] ?? d.value)})` : ""}</span>
            </label>
          {/each}
          {#if takes}
            <p class="hint" data-dials-live="true">Nobody knows these values; the universe pack sets them. Turn one to see what this world's life would be under it: only a look, nothing in the campaign changes.</p>
          {:else}
            <p class="hint" data-dials-live={takes === null ? "asking" : "false"}>Nobody knows these values; the universe pack sets them. {takes === null ? "Finding out whether the simulation can re-run this world's life with other dials…" : "Turning a dial here to see what changes comes when the simulation can re-run a world's life with other dials."}</p>
          {/if}
        </div>
      {/if}
    </article>
  {/if}
</section>

<style>
  .flow {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  h3,
  h4,
  h5 {
    margin: 0;
    font-family: var(--sse-font-display);
  }
  h3 {
    font-size: 16px;
  }
  h4 {
    font-size: 15px;
  }
  h5 {
    font-size: 13px;
    margin-top: 8px;
  }
  p {
    margin: 0;
    line-height: 1.5;
  }
  .hint {
    color: var(--sse-ink-muted);
  }
  .board {
    overflow-x: auto;
    padding-bottom: 4px;
  }
  /* Drawn at its own size so the words stay readable; a narrow column scrolls sideways inside the board. */
  .wires-board {
    display: block;
    max-width: none;
  }
  .wire {
    fill: none;
    stroke: var(--sse-ink-muted);
    stroke-width: 1.2;
    transition: stroke 120ms ease;
  }
  .wire.dead {
    stroke-dasharray: 3 3;
    opacity: 0.5;
  }
  .wire.near {
    stroke: var(--sse-ink);
    stroke-width: 2;
  }
  .node {
    cursor: pointer;
    outline: none;
    transition: opacity 140ms ease;
  }
  .node rect {
    fill: var(--sse-ground);
    stroke: var(--sse-ink);
    stroke-width: 1;
  }
  .node:hover rect,
  .node:focus-visible rect {
    stroke-width: 2;
  }
  .node.on rect {
    fill: var(--sse-ground-raised);
    stroke-width: 2.2;
  }
  .node.dead rect {
    stroke-dasharray: 3 3;
    stroke: var(--sse-ink-muted);
  }
  .node.dead text {
    fill: var(--sse-ink-muted);
  }
  .node.moved rect {
    stroke: var(--sse-accent, var(--sse-ink));
    stroke-width: 2.4;
  }
  .nm {
    font-size: 10px;
    font-family: var(--sse-font-mono);
    fill: var(--sse-accent, var(--sse-ink));
  }
  .list li.moved button {
    border-left-color: var(--sse-accent, var(--sse-ink));
    border-left-width: 5px;
  }
  .whatif {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 12px;
    align-items: center;
    padding: 6px 10px;
    border: var(--sse-line) solid var(--sse-accent, var(--sse-ink));
    background: var(--sse-ground-raised);
    animation: rise 160ms ease-out;
  }
  .whatif p {
    flex: 1 1 240px;
  }
  .was {
    border-left: 3px solid var(--sse-accent, var(--sse-ink));
    padding-left: 8px;
    display: grid;
    gap: 2px;
  }
  .node.dim {
    opacity: 0.45;
  }
  .node text {
    fill: var(--sse-ink);
  }
  .nl {
    font-family: var(--sse-font-display);
    font-size: 13px;
    font-weight: 600;
  }
  .nh {
    font-size: 11.5px;
  }
  .ne {
    font-size: 10px;
    fill: var(--sse-ink-muted);
  }
  .list {
    display: none;
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .list button {
    display: grid;
    width: 100%;
    height: auto;
    gap: 1px;
    padding: 6px 10px;
    border: var(--sse-line) solid var(--sse-rule);
    border-left: 3px solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .list li + li {
    margin-top: 4px;
  }
  .list li.dead button {
    border-left-style: dashed;
    color: var(--sse-ink-muted);
  }
  .list button[aria-pressed="true"] {
    background: var(--sse-ground-raised);
  }
  .from {
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .card {
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    animation: rise 160ms ease-out;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .card {
      animation: none;
    }
    .node,
    .wire {
      transition: none;
    }
  }
  .card header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .close {
    border: none;
    background: none;
    color: var(--sse-ink);
    font-size: 18px;
    cursor: pointer;
    height: auto;
    padding: 0 4px;
  }
  .lines {
    margin: 0;
    padding-left: 18px;
    display: grid;
    gap: 3px;
  }
  .note {
    color: var(--sse-ink);
  }
  .dial {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 140px auto;
    gap: 8px;
    align-items: center;
  }
  .val {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  @media (max-width: 720px) {
    .board {
      display: none;
    }
    .list {
      display: block;
    }
    .dial {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
