<!--
  The print-only rendering of the Atlas (AT4, atlas design section 9): the pages asked for, in the edition showing,
  moved to the end of the document so that print shows them and nothing else. On screen it is never seen. Printing
  happens in place, in this window: no popup a browser could block without a word.
-->
<script lang="ts">
  import type { AtlasPage as Page } from "../../../ui-contract/src/index.ts";
  import AtlasPage from "./AtlasPage.svelte";

  let { pages, look, footer, paper, bw = false }: { pages: Page[]; look: string; footer: string; paper: "A4" | "Letter"; bw?: boolean } = $props();

  // Move to the end of <body>, so print can hide everything else by one rule.
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return { destroy: () => node.remove() };
  }
  // The sheet size and margins for this print. Sheet numbers come from the browser where it can put them in the
  // margin; each Atlas page also prints its own number in its footer.
  const pageCss = $derived(
    `@page { size: ${paper === "Letter" ? "letter" : "A4"}; margin: 16mm 16mm 18mm; @bottom-right { content: counter(page) " / " counter(pages); font-size: 9pt; } }`,
  );
</script>

<svelte:head>
  {@html `<style data-atlas-page-size>${pageCss}</style>`}
</svelte:head>

<div class="atlas-print" class:ink-bw={bw} use:portal style={look} data-atlas-print={pages.length} data-atlas-ink={bw ? "bw" : "shown"}>
  {#each pages as p (p.address)}
    <div class="sheet">
      <AtlasPage page={p} onlink={() => {}} {footer} />
    </div>
  {/each}
</div>

<style>
  .atlas-print {
    display: none;
    background: var(--sse-ground-raised, var(--sse-ground));
    color: var(--sse-ink);
    container: atlas / inline-size;
  }
  .sheet {
    break-after: page;
  }
  .sheet:last-child {
    break-after: auto;
  }
  .sheet :global(.atlas-page) {
    box-shadow: none;
    border: 0;
    border-radius: 0;
    padding: 0;
    max-width: none;
  }
  .sheet :global(section),
  .sheet :global(figure),
  .sheet :global(.row),
  .sheet :global(tr) {
    break-inside: avoid;
  }
  /* On a sheet the plate follows the words at a modest size: no grid to fragment across sheets. */
  .sheet :global(.cols) {
    display: block;
  }
  .sheet :global(.plate) {
    max-width: 120mm;
    margin-top: 6mm;
  }
  .sheet :global(h2) {
    break-after: avoid;
  }
  @media print {
    :global(body > :not(.atlas-print)) {
      display: none !important;
    }
    :global(html),
    :global(body) {
      height: auto !important;
      overflow: visible !important;
      background: none !important;
    }
    .atlas-print {
      display: block;
      print-color-adjust: exact;
      -webkit-print-color-adjust: exact;
    }
  }
</style>
