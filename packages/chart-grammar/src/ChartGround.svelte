<!-- Paper ground of the chart: the surface everything else is inked onto. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  let { children }: { children: Snippet } = $props();
</script>

<div class="ground">{@render children()}</div>

<style>
  .ground {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: var(--sse-ground);
    color: var(--sse-ink);
  }
</style>
