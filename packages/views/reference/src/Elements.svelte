<!--
  The Elements page of the Reference screen (matter-pack design 8.1 and 8.6, MP10 part 2): the periodic table, each
  cell's bar filled in proportion to where that element's atoms were made, in the one origin legend the whole app
  uses (theme tokens origin-*). A cell opens the element's card: its shares in words, its origin story (plain at
  Curious, the physics at Science), how sure the science is, its weight and abundance, and its sources. On a phone
  the table is a small coloured map with a strip of full cells beneath it, and the card rises from the foot.
  Everything is the sheet's (read_matter); nothing here is typed.
-->
<script lang="ts">
  import { tick, untrack } from "svelte";
  import { formatValue, originGradient, originShares, originToken, readElementsPage, readRow, shareWords, showReference, tablePosition, type Depth, type ElementRow, type ElementsPage, type RowDetail, type ViewSession } from "../../../ui-contract/src/index.ts";
  import { Stamp } from "../../../chart-grammar/src/index.ts";

  let { session, depth, pick = null }: { session: ViewSession; depth: Depth; pick?: string | null } = $props();
  const full = $derived(depth !== "curious");

  let sheet = $state.raw<ElementsPage | null>(null);
  let failed = $state<string | null>(null);
  $effect(() => {
    const s = session;
    untrack(() => {
      void readElementsPage(s).then((r) => (r.ok ? (sheet = r.page) : (failed = r.message)));
    });
  });
  // Elements past the usual 118, or any a pack marks, sit in the campaign row beneath the table.
  const extra = $derived((sheet?.elements ?? []).filter((e) => !tablePosition(e.z)));
  const parts = (e: ElementRow) => originShares(e, sheet?.legend ?? []);
  const markOf = (e: ElementRow) => e.mark ?? (e.fictional ? "fictional" : e.hypothetical ? "hypothetical" : null);

  let chosen = $state.raw<ElementRow | null>(null);
  let detail = $state.raw<RowDetail<ElementRow> | null>(null);
  let opener: HTMLElement | null = null;
  async function choose(e: ElementRow, from: HTMLElement | null) {
    opener = from;
    chosen = e;
    detail = null;
    detail = await readRow<ElementRow>(session, "elements", e.id);
  }
  async function close() {
    chosen = null;
    await tick();
    opener?.focus();
  }
  // Found from the Find box: the element's card opens as if pressed.
  $effect(() => {
    const id = pick;
    const e = id ? sheet?.elements.find((x) => x.id === id) : undefined;
    if (e) untrack(() => void choose(e, document.querySelector<HTMLElement>(`[data-el="${e.id}"]`)));
  });
  // The phone's strip: the chosen element and its neighbours by number.
  const strip = $derived.by(() => {
    const els = sheet?.elements ?? [];
    const at = chosen ? els.findIndex((e) => e.id === chosen!.id) : els.findIndex((e) => e.z === 26);
    const from = Math.max(0, Math.min(els.length - 7, at - 3));
    return els.slice(from, from + 7);
  });
  const sentence = (s: string) => (s ? s[0]!.toUpperCase() + s.slice(1) : s);
</script>

<!-- Escape closes the card first, wherever focus is; only with no card open does it reach the shell (which goes up a step). -->
<svelte:window
  onkeydowncapture={(e) => {
    if (chosen && e.key === "Escape") {
      e.stopPropagation();
      e.preventDefault();
      void close();
    }
  }}
/>

<div
  class="elements"
  data-elements={sheet ? "ready" : failed ? "failed" : "loading"}
>
  {#if failed}
    <p class="note">Could not read the elements: {failed}</p>
  {:else if !sheet}
    <p class="note" aria-busy="true">Reading the elements…</p>
  {:else}
    <div class="lens">
      <span class="k">Colour by</span><span class="on" data-lens="origin">Where it was made</span>
      <span class="muted">More ways to colour the table (how common, where in a world, where it condenses) follow.</span>
    </div>
    <div class="layout" class:with-card={chosen}>
      <div class="tablebox">
        <div class="table" style:grid-template-rows={`repeat(7, var(--cell)) 8px var(--cell) var(--cell)${extra.length ? " 14px var(--cell)" : ""}`} role="group" aria-label="The periodic table, each element coloured by where its atoms were made">
          {#each sheet.elements as e (e.id)}
            {@const p = tablePosition(e.z)}
            {#if p}
              <button
                type="button"
                class="cell"
                class:sel={chosen?.id === e.id}
                style:grid-row={p[0]}
                style:grid-column={p[1]}
                aria-label="{e.name}, {e.z}"
                aria-pressed={chosen?.id === e.id}
                data-el={e.id}
                onclick={(ev) => choose(e, ev.currentTarget)}
              >
                <span class="z">{e.z}</span><span class="sym">{e.symbol}</span>
                <span class="bar" style:background={originGradient(parts(e))}></span>
              </button>
            {/if}
          {/each}
          <span class="rowname" style:grid-row="9">Lanthanides</span>
          <span class="rowname" style:grid-row="10">Actinides</span>
          {#if extra.length}
            <span class="rowname campaign" style:grid-row="12">Campaign row</span>
            {#each extra as e, i (e.id)}
              <button type="button" class="cell" class:sel={chosen?.id === e.id} data-mark={markOf(e)} style:grid-row="12" style:grid-column={3 + i} aria-label="{e.name}" data-el={e.id} onclick={(ev) => choose(e, ev.currentTarget)}>
                <span class="z">{e.z}</span><span class="sym">{e.symbol}</span>
                <span class="bar" style:background={originGradient(parts(e))}></span>
              </button>
            {/each}
          {/if}
        </div>

        <div class="strip" aria-label="Elements near the one chosen">
          {#each strip as e (e.id)}
            <button type="button" class="cell big" class:sel={chosen?.id === e.id} aria-label="{e.name}, {e.z}" data-strip={e.id} onclick={(ev) => choose(e, ev.currentTarget)}>
              <span class="z">{e.z}</span><span class="sym">{e.symbol}</span>
              <span class="bar" style:background={originGradient(parts(e))}></span>
            </button>
          {/each}
        </div>

        <ul class="legend" aria-label="Where the atoms were made" data-origin-legend>
          {#each sheet.legend as k (k.id)}
            <li><span class="sw" style:background={`var(${originToken(k.id)})`}></span>{sentence(k.plain)}{#if depth === "engine"} <code>{k.id}</code>{/if}</li>
          {/each}
        </ul>
        <p class="muted small">Each bar is filled in proportion to where that element's atoms came from. The same colours mark origin everywhere in the app.</p>
      </div>

      {#if chosen}
        {@const e = chosen}
        {@const ps = parts(e)}
        <aside class="card" aria-label="About {e.name}" data-element-card={e.id}>
          <div class="grip" aria-hidden="true"></div>
          <div class="head">
            <div>
              <div class="zs">{e.z} · {e.symbol}</div>
              <h2>{e.name}</h2>
            </div>
            {#if markOf(e)}<Stamp kind={markOf(e)!} tilt={false} />{/if}
            <button type="button" class="close" aria-label="Close" onclick={close}>×</button>
          </div>
          <div class="share" data-shares>
            <div class="sharebar" style:background={originGradient(ps)}></div>
            <p class="small">{ps.map((p, i) => shareWords(p, i === 0)).join(". ")}.</p>
          </div>
          {#if e.origin_story}
            <p class="story" data-story>{(full ? e.origin_story.physics : e.origin_story.plain) ?? e.origin_story.plain}</p>
          {/if}
          {#if e.origin?.argued}<p class="small sure" data-argued><b>How sure is this?</b> {sentence(e.origin.argued)}.</p>{/if}
          {#if full}
            <dl class="facts">
              <dt>Atomic weight</dt><dd>{e.atomic_weight ? formatValue(e.atomic_weight.v) : e.no_standard_weight ? "none: no isotope lasts long enough to set one" : "not on the sheet"}</dd>
              {#if e.abundance_protosolar}<dt>In the Sun's birth cloud</dt><dd>{formatValue(e.abundance_protosolar.v)} atoms for every million of silicon{#if e.abundance_protosolar.sigma}{" "}(± {formatValue(e.abundance_protosolar.sigma)}){/if}</dd>{/if}
            </dl>
            {#if e.origin?.note}<p class="small muted" data-origin-note>The shares: {e.origin.note}.</p>{/if}
          {/if}
          <div class="why" data-card-why>
            <h3>Why?</h3>
            {#if !detail}
              <p class="muted small" aria-busy="true">Reading its sources…</p>
            {:else if detail.sources.length}
              <ul class="sources">
                {#each detail.sources as s (s.id)}
                  {@const ref = showReference(s)}
                  <li data-source={s.id}>{#if ref.href}<a href={ref.href} target="_blank" rel="noopener">{ref.text}</a>{:else}{ref.text}{/if}</li>
                {/each}
              </ul>
            {:else}
              <p class="muted small">No source on the sheet.</p>
            {/if}
            {#if depth === "engine" && detail}
              <dl class="facts" data-depth="engine">
                <dt>Id</dt><dd><code>{detail.id}</code></dd>
                <dt>Set by</dt><dd>{detail.packs.join(", ") || "—"}</dd>
                <dt>Read by</dt><dd>{detail.used_by.join(", ") || "no block declares it yet"}</dd>
                {#if e.origin?.check}<dt>Shares checked</dt><dd>{e.origin.check}</dd>{/if}
              </dl>
            {/if}
          </div>
        </aside>
      {/if}
    </div>
  {/if}
</div>

<style>
  .elements {
    --cell: 46px;
    min-width: 0;
  }
  .note,
  .muted {
    color: var(--sse-ink-muted);
  }
  .small {
    font-size: 12px;
    line-height: 1.45;
  }
  .lens {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 10px;
    padding: 4px 0 12px;
  }
  .lens .k {
    color: var(--sse-ink-muted);
  }
  .lens .on {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
    padding: 4px 12px;
  }
  .lens .muted {
    font-size: 12px;
  }
  .layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 24px;
  }
  .layout.with-card {
    grid-template-columns: minmax(0, 1fr) minmax(260px, 360px);
  }
  .table {
    display: grid;
    grid-template-columns: repeat(18, minmax(0, 1fr));
    gap: 3px;
  }
  .cell {
    font: inherit;
    color: var(--sse-ink);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: space-between;
    min-width: 0;
    padding: 0;
    border: 1px solid var(--sse-rule);
    background: var(--sse-ground-raised);
    cursor: pointer;
    overflow: hidden;
    text-align: left;
  }
  .cell:hover {
    border-color: var(--sse-ink-muted);
  }
  .cell.sel {
    border: 2px solid var(--sse-accent);
    box-shadow: 0 2px 0 var(--sse-accent-ink);
  }
  .cell[data-mark="fictional"] {
    border: 1.5px dashed var(--sse-accent-ink);
  }
  .cell[data-mark="hypothetical"] {
    border: 1.5px dotted var(--sse-ink);
  }
  .z {
    font-family: var(--sse-font-mono);
    font-size: 9px;
    line-height: 1;
    color: var(--sse-ink-muted);
    padding: 4px 5px 0;
  }
  .sym {
    font-size: 15px;
    font-weight: 600;
    line-height: 1.1;
    padding: 0 5px;
  }
  .bar {
    align-self: stretch;
    height: 8px;
    flex: none;
  }
  .rowname {
    grid-column: 1 / span 2;
    align-self: center;
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .rowname.campaign {
    color: var(--sse-accent-ink);
    font-weight: 600;
  }
  .strip {
    display: none;
  }
  .legend {
    list-style: none;
    display: flex;
    flex-wrap: wrap;
    gap: 6px 18px;
    margin: 14px 0 6px;
    padding: 12px 0 0;
    border-top: 1px solid var(--sse-rule);
    font-size: 12px;
  }
  .legend li {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .sw {
    display: inline-block;
    width: 22px;
    height: 10px;
    border: 1px solid var(--sse-ink);
  }
  .card {
    align-self: start;
    position: sticky;
    top: 12px;
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    box-shadow: var(--sse-card-shadow);
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    animation: rise var(--sse-motion, 180ms) var(--sse-ease, ease-out);
  }
  .card p {
    margin: 0;
  }
  .grip {
    display: none;
  }
  .head {
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }
  .head > div {
    flex: 1;
  }
  .zs {
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .card h2 {
    margin: 0;
    font-family: var(--sse-font-display);
    font-weight: 500;
    font-size: 32px;
    line-height: 1;
  }
  .close {
    font: inherit;
    font-size: 20px;
    line-height: 1;
    border: 0;
    background: none;
    color: var(--sse-ink);
    min-width: 36px;
    min-height: 36px;
    cursor: pointer;
  }
  .share {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .sharebar {
    height: 14px;
    border: 1px solid var(--sse-ink);
  }
  .story {
    font-family: var(--sse-font-display);
    font-size: 17px;
    line-height: 1.42;
  }
  .sure {
    border-top: 1px solid var(--sse-rule);
    padding-top: 10px;
    color: var(--sse-ink-muted);
  }
  .sure b {
    color: var(--sse-ink);
  }
  .facts {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 4px 12px;
    margin: 0;
    font-size: 12px;
  }
  .facts dt {
    color: var(--sse-ink-muted);
  }
  .facts dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
  .why {
    border-top: 1px solid var(--sse-rule);
    padding-top: 8px;
  }
  .why h3 {
    margin: 0 0 4px;
    font-size: 12px;
    font-weight: 600;
  }
  .sources {
    margin: 0 0 6px;
    padding-left: 18px;
    display: grid;
    gap: 4px;
    font-size: 12px;
  }
  .sources a {
    color: var(--sse-ink);
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
  }
  /* A phone: the whole table as a small coloured map (no letters), a strip of full cells beneath, the card a sheet. */
  @container (max-width: 640px) {
    .elements {
      --cell: 18px;
    }
    .table {
      gap: 2px;
    }
    .table .cell {
      border: 0;
    }
    .table .cell .z,
    .table .cell .sym {
      display: none;
    }
    .table .cell .bar {
      height: 100%;
    }
    .table .cell.sel {
      outline: 2px solid var(--sse-ink);
      outline-offset: 1px;
      box-shadow: none;
    }
    .rowname {
      display: none;
    }
    .strip {
      display: flex;
      gap: 4px;
      overflow-x: auto;
      margin-top: 12px;
    }
    .cell.big {
      flex: none;
      width: 54px;
      height: 60px;
    }
    .cell.big .z {
      font-size: 10px;
      padding: 5px 6px 0;
    }
    .cell.big .sym {
      font-size: 18px;
      padding: 0 6px;
    }
    .cell.big .bar {
      height: 10px;
    }
    .layout.with-card {
      grid-template-columns: minmax(0, 1fr);
    }
    .card {
      position: fixed;
      left: 0;
      right: 0;
      bottom: 0;
      top: auto;
      max-height: 60vh;
      overflow-y: auto;
      z-index: 20;
      border-width: var(--sse-line) 0 0;
      padding: 8px 16px 16px;
    }
    .grip {
      display: block;
      align-self: center;
      width: 40px;
      height: 3px;
      background: var(--sse-ink-muted);
    }
    .close {
      min-width: 44px;
      min-height: 44px;
    }
    .lens .muted {
      display: none;
    }
  }
</style>
