<!-- A block's references: a DOI or URL is a link (new tab), anything else plain text. -->
<script lang="ts">
  import { showReference, type ReferenceIn } from "../../ui-contract/src/index.ts";
  let { refs, inline = false }: { refs: readonly ReferenceIn[]; inline?: boolean } = $props();
  const shown = $derived(refs.map(showReference));
</script>

{#if inline}
  <p class="refs">{#each shown as r, i (i)}{#if i}{" · "}{/if}{#if r.href}<a href={r.href} target="_blank" rel="noopener noreferrer">{r.text}</a>{:else}{r.text}{/if}{/each}</p>
{:else}
  <ul class="refs">{#each shown as r, i (i)}<li>{#if r.href}<a href={r.href} target="_blank" rel="noopener noreferrer">{r.text}</a>{:else}{r.text}{/if}</li>{/each}</ul>
{/if}

<style>
  .refs {
    margin: 6px 0 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  ul.refs {
    padding-left: 16px;
  }
  a {
    color: inherit;
    text-decoration: underline;
    text-underline-offset: 2px;
    overflow-wrap: anywhere;
  }
</style>
