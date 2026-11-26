<!--
  The selected world's story (formation design section 7, right panel): where its material came from as a bar by band,
  its events in order, and how sure each part is. From ui-contract formation-story.ts.
-->
<script lang="ts">
  import type { WorldStory } from "../../../ui-contract/src/index.ts";

  let { story, chosen }: { story: WorldStory; chosen: boolean } = $props();

  // Bands from hot to cold, then what merged in, then gas: the same ink everywhere a band is named.
  const INK: Record<string, string> = {
    vapour: "color-mix(in srgb, var(--sse-accent) 70%, var(--sse-ink))",
    rock: "var(--sse-ink)",
    carbon: "color-mix(in srgb, var(--sse-ink) 55%, var(--sse-ground))",
    "water-ice": "var(--sse-zone-ink)",
    "co2-ice": "color-mix(in srgb, var(--sse-zone-ink) 60%, var(--sse-ground))",
    "co-ice": "color-mix(in srgb, var(--sse-zone-ink) 35%, var(--sse-ground))",
  };
  const ink = (m: WorldStory["material"][number]) => (m.kind === "impactor" ? "var(--sse-accent)" : m.kind === "gas" ? "var(--sse-ink-faint)" : (INK[m.key] ?? "var(--sse-ink-muted)"));
</script>

<aside class="panel story" aria-label="{story.label}: where it came from" data-story={story.id}>
  <h2>{story.label} <span class="k">where it came from</span></h2>
  {#if !chosen}<p class="hint">The heaviest world, to start. Press any track to read its story.</p>{/if}
  {#if story.material.length}
    <h3 class="k">Material by band</h3>
    <div class="bar" role="img" aria-label={story.material_words} data-material>
      {#each story.material as m (m.key)}
        <span class:impactor={m.kind === "impactor"} style:flex-grow={m.share} style:background={ink(m)} title="{Math.round(m.share * 100)}% {m.words}"></span>
      {/each}
    </div>
    <p class="words">{story.material_words}{story.water_words ? `. ${story.water_words}` : ""}.</p>
  {/if}
  <h3 class="k">Its story</h3>
  <ol class="lines">
    {#each story.lines as l, i (i)}
      <li class:violent={l.violent} data-story-line><span class="when">{l.when}</span><span class="what">{l.text}</span></li>
    {/each}
  </ol>
  <h3 class="k">How sure</h3>
  <dl>
    {#each story.sure as r (r.label)}
      <div title={r.title}><dt>{r.label}</dt><dd>{r.value}</dd></div>
    {/each}
  </dl>
</aside>

<style>
  .panel {
    border: var(--sse-line) solid var(--sse-ink);
    box-shadow: 4px 4px 0 var(--sse-ink);
    padding: 12px 14px;
    background: var(--sse-ground);
    align-self: start;
    min-width: 0;
    max-height: calc(100% - 6px);
    overflow: auto;
    box-sizing: border-box;
  }
  h2 {
    margin: 0 0 6px;
    font-family: var(--sse-font-display);
    font-weight: normal;
    font-size: 20px;
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
  }
  h3 {
    margin: 12px 0 6px;
  }
  .k {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    font-weight: normal;
  }
  .hint {
    margin: 0 0 4px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .bar {
    display: flex;
    height: 14px;
    border: var(--sse-line) solid var(--sse-ink);
  }
  .bar span {
    flex-basis: 0;
    min-width: 2px;
  }
  .bar span.impactor {
    background-image: repeating-linear-gradient(45deg, transparent 0 3px, var(--sse-ground) 3px 4px) !important;
  }
  .words {
    margin: 6px 0 0;
    font-size: 12.5px;
    line-height: 1.45;
  }
  .lines {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .lines li {
    display: grid;
    grid-template-columns: 5.5em 1fr;
    gap: 8px;
    padding: 5px 0;
    border-bottom: var(--sse-line) solid var(--sse-ink-faint);
    font-size: 13px;
    line-height: 1.35;
  }
  .when {
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .lines li.violent .what {
    color: var(--sse-accent-ink);
    font-weight: 600;
  }
  dl {
    margin: 0;
  }
  dl div {
    display: flex;
    justify-content: space-between;
    gap: 10px;
    padding: 5px 0;
    border-bottom: var(--sse-line) solid var(--sse-ink-faint);
    font-size: 13px;
  }
  dd {
    margin: 0;
    font-family: var(--sse-font-mono);
    text-align: right;
  }
</style>
