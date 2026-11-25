<!--
  A split display (PB9, design 3.2): Two, Three or Four players' screens in one full-screen window on a display of the
  GM's computer. Each part is a players' screen of its own (its own id, box, role, line, dial and hold on the board), so
  this frame does nothing but lay them out: Two side by side, Three one large and two small, Four two by two.
-->
<script lang="ts">
  import { SPLITS, isSplit, partLinks } from "../../ui-contract/src/index.ts";
  import FillScreen from "./FillScreen.svelte";

  const split = (() => {
    const s = new URLSearchParams(location.search).get("split");
    return isSplit(s) ? s : "whole";
  })();
  const parts = partLinks(location.href);
  const def = SPLITS[split];
  document.title = `${new URLSearchParams(location.search).get("name") ?? "Screens"}: ${def.words.toLowerCase()} screens`;
</script>

<div class="split" data-split={split} style:grid-template-columns={def.columns} style:grid-template-rows={def.rows} style:grid-template-areas={def.areas.join(" ")}>
  {#each parts as src, i (src)}
    <iframe {src} title="Part {i + 1}" style:grid-area="p{i + 1}" data-split-part={i + 1} allow="fullscreen"></iframe>
  {/each}
</div>
<FillScreen />

<style>
  /* Only while a split is on the page: the shell's one bundle carries this sheet to every players' screen too. */
  :global(html:has(.split)),
  :global(body:has(.split)) {
    margin: 0;
    height: 100%;
    background: #000;
  }
  .split {
    display: grid;
    gap: 2px;
    height: 100vh;
    width: 100vw;
  }
  iframe {
    border: 0;
    width: 100%;
    height: 100%;
    display: block;
    background: #000;
  }
</style>
