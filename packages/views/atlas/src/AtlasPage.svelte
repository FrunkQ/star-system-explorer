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

  let { page, onlink, footer = "", onturnaway }: { page: AtlasPage; onlink: (address: string) => void; footer?: string; onturnaway?: (id: string) => void } = $props();

  const title = $derived(page.blocks.find((b): b is Extract<Block, { type: "title" }> => b.type === "title"));
  const plates = $derived(page.blocks.filter((b): b is Extract<Block, { type: "plate" }> => b.type === "plate"));
  const head = $derived(page.blocks.find((b): b is Extract<Block, { type: "letterhead" }> => b.type === "letterhead"));
  const margins = $derived(page.blocks.filter((b): b is Extract<Block, { type: "margin" }> => b.type === "margin"));
  const body = $derived(page.blocks.filter((b) => b.type !== "title" && b.type !== "plate" && b.type !== "letterhead" && b.type !== "margin"));

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
  {#if head}
    <div class="lh" data-deliver-item data-letterhead style={head.accent ? `--lh-accent: ${head.accent}` : undefined}>
      {#if head.logo && head.logoMono}
        <span class="logo mono" role="img" aria-label={head.name} style="--logo: url('{head.logo}')"></span>
      {:else if head.logo}
        <img class="logo" src={head.logo} alt={head.name} />
      {:else if head.art}
        <pre class="art" role="img" aria-label={head.artLabel}>{head.art.join("\n")}</pre>
      {/if}
      <div class="lhwords">
        <p class="lhname">{head.name}</p>
        {#if head.motto}<p class="lhmotto">{head.motto}</p>{/if}
        {#each head.lines.slice(head.lines[0] === head.name ? 1 : 0) as l, i (i)}<p class="lhline">{l}</p>{/each}
      </div>
      {#if head.stamp}<p class="stamp">{head.stamp}</p>{/if}
    </div>
  {/if}
  {#if title}
    <header class="head" data-deliver-item>
      <p class="sub">{title.subtitle}</p>
      <h1 id="atlas-title-{page.number}" tabindex="-1">{title.text}</h1>
      {#if title.verdict}
        <details class="verdict" data-verdict>
          <summary>{title.verdict.label ? `${title.verdict.label}: ` : ""}{title.verdict.text}</summary>
          <p class="whyhead">Read off</p>
          <ul>{#each title.verdict.why as w (w)}<li>{w}</li>{/each}</ul>
        </details>
      {/if}
    </header>
  {/if}
  <div class="cols" class:with-plate={plates.length > 0 || margins.length > 0}>
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
    {#if plates.length || margins.length}<div class="side">
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
    {#each margins as m, mi (mi)}
      {#each m.notes as n (n.id)}
        <aside class="margin" data-deliver-item data-margin-note={n.id}>
          <p class="sig">{m.signature}</p>
          <p class="said" data-deliver-text>{n.text}</p>
          {#if onturnaway}<button type="button" class="away" title={n.id} onclick={() => onturnaway(n.id)}>Not this one</button>{/if}
        </aside>
      {/each}
    {/each}
    </div>{/if}
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
    color: var(--sse-label-ink, var(--sse-ink-muted));
  }
  h1 {
    color: var(--sse-title-ink, var(--sse-ink));
    outline: none;
    margin: 4px 0 18px;
    font-family: var(--sse-font-display);
    font-style: italic;
    font-weight: 500;
    font-size: 44px;
    line-height: 1.1;
  }
  .verdict {
    display: inline-block;
    margin: -6px 0 18px;
  }
  .verdict summary {
    list-style: none;
    display: inline-block;
    cursor: pointer;
    padding: 6px 16px;
    border: 3px solid var(--sse-title-ink, var(--sse-accent));
    border-radius: 6px;
    color: var(--sse-title-ink, var(--sse-accent-ink));
    font: 800 15px/1.2 var(--sse-font-body);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    transform: rotate(-2deg);
  }
  .verdict summary::-webkit-details-marker {
    display: none;
  }
  .verdict summary:focus-visible {
    outline: 2px solid var(--sse-focus);
    outline-offset: 3px;
  }
  .verdict[open] summary {
    transform: none;
  }
  .verdict .whyhead {
    margin: 10px 0 2px;
    font-size: 12px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-label-ink, var(--sse-ink-muted));
  }
  .verdict ul {
    margin: 0;
    padding-left: 18px;
    font-family: var(--sse-font-mono);
    font-size: 13px;
    color: var(--sse-ink-muted);
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
    color: var(--sse-label-ink, var(--sse-ink-muted));
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
  .side {
    display: grid;
    gap: 18px;
    align-content: start;
  }
  .margin {
    border-left: 4px solid var(--sse-accent);
    padding: 2px 0 2px 16px;
  }
  .margin .sig {
    margin: 0 0 6px;
    font: 700 12px/1.3 var(--sse-font-body);
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--sse-label-ink, var(--sse-ink-muted));
  }
  .margin .said {
    margin: 0;
    font-style: italic;
    font-size: 19px;
    line-height: 1.5;
    color: var(--sse-margin-ink, var(--sse-ink));
  }
  .away {
    margin-top: 8px;
    font: inherit;
    font-size: 12px;
    background: none;
    border: 0;
    padding: 0;
    color: var(--sse-ink-muted);
    text-decoration: underline;
    cursor: pointer;
  }
  .away:focus-visible {
    outline: 2px solid var(--sse-focus);
    outline-offset: 2px;
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
  .lh {
    position: relative;
    display: flex;
    gap: 18px;
    align-items: flex-start;
    padding-bottom: 14px;
    margin-bottom: 18px;
    border-bottom: 2px solid var(--lh-accent, var(--sse-ink));
  }
  .logo {
    max-height: 72px;
    max-width: 160px;
    object-fit: contain;
  }
  .mono {
    display: block;
    width: 160px;
    height: 72px;
    background: var(--lh-accent, var(--sse-ink));
    mask: var(--logo) left center / contain no-repeat luminance;
  }
  .art {
    margin: 0;
    font-family: var(--sse-font-mono);
    font-size: 9px;
    line-height: 1.1;
    color: var(--sse-ink);
    overflow-x: auto;
  }
  .lhwords p {
    margin: 0;
  }
  .lhname {
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
  .lhmotto {
    font-style: italic;
    color: var(--sse-ink-muted);
  }
  .lhline {
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .stamp {
    position: absolute;
    top: 0;
    right: 0;
    margin: 0;
    padding: 3px 10px;
    border: 2px solid var(--lh-accent, var(--sse-accent));
    color: var(--lh-accent, var(--sse-accent));
    font-weight: 700;
    letter-spacing: 0.12em;
    font-size: 12px;
    transform: rotate(-4deg);
  }
  @container atlas (max-width: 760px) {
    .lh {
      flex-direction: column;
      gap: 8px;
    }
    .stamp {
      position: static;
      align-self: flex-start;
    }
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
