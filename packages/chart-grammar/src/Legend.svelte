<!--
  The legend: a list of entries the symbology generated (legendFor), each
  drawn with the very mark the chart draws, so the two cannot disagree. It
  knows nothing about classes or colours.
-->
<script lang="ts">
  import type { LegendEntry } from "../../ui-contract/src/index.ts";
  import HatchDefs from "./HatchDefs.svelte";
  import Mark from "./Mark.svelte";
  let { entries, prefix, note = null, label = "Key" }: { entries: LegendEntry[]; prefix: string; note?: string | null; label?: string } = $props();
</script>

<figure class="legend-list" aria-label={label}>
  <svg width="0" height="0" aria-hidden="true" class="defs"><HatchDefs {prefix} /></svg>
  <ul>
    {#each entries as e (e.key)}
      <li data-legend-key={e.key} data-legend-mark="{e.mark.shape} {e.mark.fill_pattern}{e.mark.colour ? ` ${e.mark.colour}` : ''}{e.mark.ring ? ' ring' : ''}">
        <svg width="34" height="26" viewBox="0 0 34 26" aria-hidden="true"><Mark mark={{ ...e.mark, size: Math.min(6, Math.max(4, e.mark.size)) }} cx={e.mark.ring ? 15 : 12} cy={10} {prefix} /></svg>
        <span>{e.label}</span>
      </li>
    {/each}
  </ul>
  {#if note}<figcaption>{note}</figcaption>{/if}
</figure>

<style>
  .legend-list {
    margin: 0;
  }
  .defs {
    position: absolute;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 2px 12px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12.5px;
    color: var(--sse-ink);
  }
  svg {
    flex: none;
    overflow: visible;
  }
  figcaption {
    margin-top: 6px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
</style>
