<!--
  In memory (SH6, design section 8.3): one still page of names, in three parts, in date order. Standing rules (design
  8.1; Alex: "one place we don't joke"): the pack's words only, plain, no pictures, no accent colour, no motion beyond
  the panel's own fade, never a backdrop or a joke. The page is the same whatever date is in view.
-->
<script lang="ts">
  import { carriedLine, sharedSinceWhy, correctionUrl, inMemoryLine, recordDate, sayNumber, sinceWords, type Calendar, type CarriedName, type InMemory } from "../../../ui-contract/src/index.ts";

  let { memory, carried = [], calendar, onday, onback }: { memory: InMemory; carried?: CarriedName[]; calendar: Calendar; onday: (at_s: number) => void; onback: () => void } = $props();
  const line = $derived(inMemoryLine(memory, (n) => sayNumber(n, 6)));
  const date = (d: string) => recordDate(d, calendar.months);
  const carriedWhy = $derived(sharedSinceWhy(carried));
  const all = $derived(memory.parts.flatMap((p) => p.losses));
  const tollWords = (t: [number, number], why?: string) => `The sources give between ${sayNumber(t[0], 6)} and ${sayNumber(t[1], 6)} dead${why ? `: ${why}` : ""}.`;
</script>

<section class="memory" data-in-memory aria-label="In memory">
  <button class="back" onclick={onback} data-in-memory-back>‹ Roll call</button>
  <h2>In memory</h2>
  <p class="first" data-in-memory-line>{line}</p>

  {#each memory.parts as part (part.key)}
    <h3 data-in-memory-part={part.key}>{part.heading}</h3>
    {#each part.losses as l (l.id + part.key)}
      <article class="loss" data-in-memory-loss={l.id}>
        <p class="when"><button class="day" onclick={() => onday(l.at_s)} title="This day in Roll call">{date(l.date)}</button>, {l.where}</p>
        <p class="sentence">{l.sentence}</p>
        <ul class="names">
          {#each l.names as n (n.person ?? n.name)}<li>{n.name}</li>{/each}
          {#if l.unconfirmed}<li class="unconfirmed">{l.unconfirmed}</li>{/if}
        </ul>
        {#if Array.isArray(l.toll)}<p class="aside">{tollWords(l.toll, l.toll_why)}</p>{/if}
        {#if l.note}<p class="aside">{l.note}</p>{/if}
      </article>
    {/each}
  {/each}

  {#if memory.earth.length}
    <h3>Memorials on Earth</h3>
    <ul class="earth">
      {#each memory.earth as e (e.name)}<li>{e.name}{e.where ? `, ${e.where}` : ""}</li>{/each}
    </ul>
  {/if}

  {#if carried.length}
    <!-- Names carried (SH7, design 8.5): the real memorials in space, as text; the same whatever date is in view. -->
    <h3 data-in-memory-carried>Names carried</h3>
    <p class="aside lead" data-in-memory-carried-lead>Minor planets named for them, on their real orbits.{#if carriedWhy}{" "}Their naming dates are not given here yet: {carriedWhy}.{/if}</p>
    <ul class="carried">
      {#each carried as c (c.id)}
        <li data-in-memory-carried-row={c.id}>{carriedLine(c)}{#if !carriedWhy} <span class="since">{sinceWords(c, date)}</span>{/if}</li>
      {/each}
    </ul>
  {/if}

  <details class="sources" data-in-memory-sources>
    <summary>Sources</summary>
    {#each all as l, i (l.id + i)}
      <p class="src-head">{date(l.date)}, {l.where}</p>
      <ul>
        {#each l.sources as s, j (j)}<li>{#if s.url}<a href={s.url} target="_blank" rel="noopener noreferrer">{s.text}</a>{:else}{s.text}{/if}</li>{/each}
      </ul>
    {/each}
    {#each carried as c (c.id)}
      <p class="src-head">{c.name}</p>
      <ul>
        {#each c.sources as s, j (j)}<li>{#if s.url}<a href={s.url} target="_blank" rel="noopener noreferrer">{s.text}</a>{:else}{s.text}{/if}</li>{/each}
      </ul>
    {/each}
  </details>
  <p class="foot"><a href={correctionUrl(null)} target="_blank" rel="noopener noreferrer" data-in-memory-correct>Something wrong or missing?</a> A correction opens a prepared report; nothing is sent until you send it.</p>
</section>

<style>
  .memory {
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  .back,
  .day {
    font: inherit;
    background: none;
    border: 0;
    padding: 0;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .back {
    color: var(--sse-ink-muted);
    font-size: 0.9rem;
  }
  .day {
    text-decoration: underline;
    text-decoration-color: var(--sse-line);
    text-underline-offset: 3px;
  }
  h2 {
    font-family: var(--sse-font-display);
    font-size: 2rem;
    font-weight: 600;
    margin: 6px 0 8px;
  }
  .first {
    line-height: 1.5;
    margin: 0 0 8px;
  }
  h3 {
    font-family: var(--sse-font-display);
    font-weight: 600;
    font-size: 1.15rem;
    margin: 22px 0 4px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--sse-rule);
  }
  .loss {
    margin: 14px 0;
  }
  .when {
    margin: 0;
    color: var(--sse-ink-muted);
    font-size: 0.9rem;
  }
  .sentence {
    margin: 4px 0 6px;
    line-height: 1.45;
  }
  .names {
    list-style: none;
    margin: 0;
    padding: 0 0 0 12px;
    font-family: var(--sse-font-display);
    font-size: 1.05rem;
    line-height: 1.6;
  }
  .unconfirmed {
    font-family: var(--sse-font-body);
    font-size: 0.9rem;
    color: var(--sse-ink-muted);
  }
  .aside {
    margin: 4px 0 0 12px;
    font-size: 0.85rem;
    color: var(--sse-ink-muted);
  }
  .earth,
  .carried {
    padding-left: 18px;
  }
  .carried li {
    margin: 4px 0;
    line-height: 1.45;
  }
  .lead {
    margin-left: 0;
  }
  .since {
    color: var(--sse-ink-muted);
    font-size: 0.85rem;
  }
  .sources {
    margin-top: 22px;
    font-size: 0.85rem;
  }
  .sources summary {
    cursor: pointer;
    color: var(--sse-ink-muted);
  }
  .src-head {
    margin: 10px 0 2px;
    color: var(--sse-ink-muted);
  }
  .sources ul {
    margin: 0;
    padding-left: 18px;
  }
  .sources a,
  .foot a {
    color: var(--sse-ink);
  }
  .foot {
    margin-top: 16px;
    font-size: 0.85rem;
    color: var(--sse-ink-muted);
  }
</style>
