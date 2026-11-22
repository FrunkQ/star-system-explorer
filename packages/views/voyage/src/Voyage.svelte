<!--
  Voyages (V8 parts 3 and 3b): one console, one finder, two scales. In this
  system the places are the bodies and belts and the legs NAV's transfers;
  between stars the places are star systems and the legs jumps or flights.
  The crew asks; the computer plans. Presentation only.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import type { Json, SystemScene, ViewContext } from "../../../ui-contract/src/index.ts";
  import InSystem from "./InSystem.svelte";
  import Stars from "./Stars.svelte";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  let scale = $state<"system" | "stars">(untrack(() => (params.scale === "stars" ? "stars" : "system")));
</script>

<div class="voyages" data-scale={scale}>
  <span class="seg" role="group" aria-label="Scale">
    <button type="button" data-scale-pick="system" aria-pressed={scale === "system"} onclick={() => (scale = "system")}>In this system</button>
    <button type="button" data-scale-pick="stars" aria-pressed={scale === "stars"} onclick={() => (scale = "stars")}>Between stars</button>
  </span>
  {#if scale === "system"}
    <InSystem {ctx} {scene} {params} />
  {:else}
    <Stars {ctx} {scene} {params} />
  {/if}
</div>

<style>
  .voyages {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .seg {
    display: inline-flex;
    flex-wrap: wrap;
  }
  .seg button {
    font: inherit;
    font-size: 13px;
    min-height: 34px;
    padding: 0 12px;
    border: 1px solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    margin-right: -1px;
    cursor: pointer;
  }
  .seg button[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
</style>
