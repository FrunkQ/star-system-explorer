<!--
  A theme's filter chain drawn over a screen (V2): overlays that never take
  a click and are hidden from assistive technology. The host applies `root`
  (see filterLayers) to its own container.
-->
<script lang="ts">
  let { layers }: { layers: readonly { kind: string; overlay: string }[] } = $props();
</script>

{#each layers.filter((l) => l.overlay) as l, i (i)}
  <div class="fx" data-filter={l.kind} style={l.overlay} aria-hidden="true"></div>
{/each}

<style>
  .fx {
    position: fixed;
    inset: 0;
    z-index: 50;
    pointer-events: none;
  }
  @keyframes -global-sse-flicker {
    0% {
      opacity: 1;
    }
    100% {
      opacity: 0.4;
    }
  }
</style>
