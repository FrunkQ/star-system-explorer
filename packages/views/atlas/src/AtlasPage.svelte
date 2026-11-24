<!--
  One Atlas page, drawn from its block list (track AT, AT2; atlas design sections 4.2 and 10). It formats nothing:
  every word and figure arrives in the blocks, built by ui-contract's page builder from the audience's own scene.
  Names of other things are links to their pages. A gap is drawn as the edition says: quiet words, or a bar of fixed
  length that says nothing about what is behind it.

  Seam with the look transitions thread (AT15a): the page is `[data-atlas-page]`, each block a direct child marked
  `data-deliver-item`, and prose and figures are marked `data-deliver-text`.
-->
<script lang="ts">
  import type { AtlasPage, Block } from "../../../ui-contract/src/index.ts";

  let { page, onlink, footer = "" }: { page: AtlasPage; onlink: (address: string) => void; footer?: string } = $props();

  const title = $derived(page.blocks.find((b): b is Extract<Block, { type: "title" }> => b.type === "title"));
  const plates = $derived(page.blocks.filter((b): b is Extract<Block, { type: "plate" }> => b.type === "plate"));
  const body = $derived(page.blocks.filter((b) => b.type !== "title" && b.type !== "plate"));

  // The plate's schematic: the subject's children on one line, in book order, each a link to its page.
  const PLATE_W = 300;
  function plateDots(n: number): number[] {
    if (n <= 1) return [PLATE_W / 2];
    const left = 70;
    const span = PLATE_W - left - 24;
    return Array.from({ length: n }, (_, i) => left + (span * i) / (n - 1));
  }
</script>

<article class="atlas-page" data-atlas-page aria-labelledby="atlas-title-{page.number}">
  {#if title}
    <header class="head" data-deliver-item>
      <p class="sub">{title.subtitle}</p>
      <h1 id="atlas-title-{page.number}" tabindex="-1">{title.text}</h1>
    </header>
  {/if}
  <div class="cols" class:with-plate={plates.length > 0}>
    <div class="main">
      {#each body as b, i (i)}
        {#if b.type === "prose"}
          <section class="prose" data-deliver-item aria-label={b.title}>
            <p data-deliver-text>{b.text}</p>
          </section>
        {:else if b.type === "facts"}
          <section data-deliver-item aria-label={b.title}>
            <h2>{b.title}</h2>
            <dl class="facts">
              {#each b.rows as r (r.field)}
                <div class="row" class:gap={r.state === "gap"}>
                  <dt>{r.label}</dt>
                  {#if r.state === "gap" && r.bar}
                    <dd><span class="bar" role="img" aria-label="withheld from this document"></span></dd>
                  {:else if r.state === "gap"}
                    <dd class="gapwords">{r.text}</dd>
                  {:else}
                    <dd class="fig" data-deliver-text>{r.text}</dd>
                  {/if}
                </div>
              {/each}
            </dl>
          </section>
        {:else if b.type === "table"}
          <section data-deliver-item aria-label={b.title}>
            <h2>{b.title}</h2>
            <table class="tab">
              <thead>
                <tr>{#each b.columns as c, j (j)}<th scope="col">{c}</th>{/each}</tr>
              </thead>
              <tbody>
                {#each b.rows as r (r.link)}
                  <tr>
                    {#each r.cells as c, j (j)}
                      <td class:gap={c.state === "gap"} class:fig={j > 0 && c.state !== "gap"}>
                        {#if j === 0}
                          <button type="button" class="link" onclick={() => onlink(r.link)}>{c.text}</button>
                        {:else if c.bar}
                          <span class="bar short" role="img" aria-label="withheld"></span>
                        {:else}
                          {c.text}
                        {/if}
                      </td>
                    {/each}
                  </tr>
                {/each}
              </tbody>
            </table>
          </section>
        {:else if b.type === "story"}
          <section data-deliver-item aria-label={b.title}>
            <h2>{b.title}</h2>
            {#each b.lines as l, j (j)}
              <blockquote data-deliver-text><span class="said">{l.label}:</span> {l.text}</blockquote>
            {/each}
          </section>
        {:else if b.type === "note"}
          <p class="note" data-deliver-item>{b.text}</p>
        {:else if b.type === "rule"}
          <hr data-deliver-item />
        {/if}
      {/each}
    </div>
    {#each plates as p (p.key)}
      <figure class="plate" data-deliver-item aria-label={p.title}>
        <svg viewBox="0 0 {PLATE_W} 120" role="img" aria-label="{p.title}: {p.names.join(', ')}">
          <line x1="20" y1="56" x2={PLATE_W - 12} y2="56" class="axis" />
          <circle cx="24" cy="56" r="14" class="star" />
          {#each plateDots(p.addresses.length) as x, i (p.addresses[i])}
            <g class="dot" role="link" tabindex="0" aria-label={p.names[i]} onclick={() => onlink(p.addresses[i]!)} onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), onlink(p.addresses[i]!))}>
              <circle cx={x} cy="56" r="6" />
              <text x={x} y="84" text-anchor="middle">{p.names[i]}</text>
            </g>
          {/each}
        </svg>
        <figcaption>{p.title}, chart style. Each mark is a page.</figcaption>
      </figure>
    {/each}
  </div>
  {#if footer}<footer class="foot" data-deliver-item><span>{footer}</span><span>{page.number}</span></footer>{/if}
</article>

<style>
  .atlas-page {
    background: var(--sse-paper, var(--sse-ground-raised));
    color: var(--sse-ink);
    padding: 40px 56px 28px;
    border-radius: var(--sse-radius);
    box-shadow: var(--sse-card-shadow);
    max-width: 980px;
    font-family: var(--sse-font-body);
  }
  .sub {
    margin: 0;
    font-size: 13px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  h1 {
    outline: none;
    margin: 4px 0 18px;
    font-family: var(--sse-font-display);
    font-style: italic;
    font-weight: 500;
    font-size: 44px;
    line-height: 1.1;
  }
  .cols {
    display: grid;
    gap: 32px;
  }
  .cols.with-plate {
    grid-template-columns: minmax(0, 1fr) 280px;
  }
  .prose p {
    font-family: var(--sse-font-display);
    font-size: 19px;
    line-height: 1.6;
    margin: 0 0 22px;
    max-width: 62ch;
  }
  h2 {
    margin: 22px 0 6px;
    font-size: 13px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .facts {
    margin: 0;
  }
  .row {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(0, 3fr);
    gap: 12px;
    padding: 7px 0;
    border-bottom: 1px solid var(--sse-rule);
  }
  dt {
    color: var(--sse-ink-muted);
  }
  dd {
    margin: 0;
  }
  .fig {
    font-family: var(--sse-font-mono);
  }
  .gapwords,
  td.gap {
    font-family: var(--sse-font-display);
    font-style: italic;
    color: var(--sse-ink-faint, var(--sse-ink-muted));
  }
  .bar {
    display: inline-block;
    width: 9em;
    height: 0.9em;
    vertical-align: middle;
    background: var(--sse-ink);
  }
  .bar.short {
    width: 4em;
  }
  .tab {
    width: 100%;
    border-collapse: collapse;
  }
  .tab th {
    text-align: left;
    font-weight: 500;
    color: var(--sse-ink-muted);
    padding: 6px 8px 6px 0;
    border-bottom: 1px solid var(--sse-line);
  }
  .tab td {
    padding: 6px 8px 6px 0;
    border-bottom: 1px solid var(--sse-rule);
  }
  .link {
    all: unset;
    cursor: pointer;
    color: var(--sse-ink);
    border-bottom: 1px solid var(--sse-accent);
  }
  .link:hover {
    color: var(--sse-accent-ink, var(--sse-accent));
  }
  .link:focus-visible,
  .dot:focus-visible {
    outline: 2px solid var(--sse-focus, var(--sse-accent));
    outline-offset: 2px;
  }
  .note {
    margin: 18px 0 0;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  blockquote {
    margin: 6px 0;
    padding-left: 12px;
    border-left: 2px solid var(--sse-accent);
    font-family: var(--sse-font-display);
    font-style: italic;
  }
  .said {
    font-style: normal;
    color: var(--sse-ink-muted);
  }
  .plate {
    margin: 0;
    padding: 12px;
    border: 1px solid var(--sse-line);
    background: var(--sse-ground);
    align-self: start;
  }
  .plate svg {
    width: 100%;
    height: auto;
    display: block;
  }
  .axis {
    stroke: var(--sse-line);
  }
  .star {
    fill: var(--sse-accent);
  }
  .dot {
    cursor: pointer;
  }
  .dot circle {
    fill: var(--sse-ground-raised);
    stroke: var(--sse-ink);
    stroke-width: 1.5;
  }
  .dot:hover circle {
    fill: var(--sse-accent);
  }
  .dot text {
    font-size: 11px;
    fill: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  figcaption {
    margin-top: 8px;
    font-size: 12px;
    font-style: italic;
    color: var(--sse-ink-muted);
    text-align: center;
  }
  .foot {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    margin-top: 32px;
    padding-top: 12px;
    border-top: 1px solid var(--sse-line);
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  @container atlas (max-width: 760px) {
    .atlas-page {
      padding: 22px 18px 18px;
    }
    h1 {
      font-size: 34px;
    }
    .cols.with-plate {
      grid-template-columns: minmax(0, 1fr);
    }
    .row {
      grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    }
  }
</style>
