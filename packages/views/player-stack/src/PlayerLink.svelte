<!--
  Web page (PB13 part 2b): another program on a layout's box, by its address (an OBS or camera page on this computer,
  a tabletop, a site). It reads nothing of the scene. "use" (AT13) lets the player press and scroll in it; off, as by
  default, it is a picture. The page runs apart from SSE (sandboxed, no access to this one).
-->
<script lang="ts">
  import { controlOn, type PlayerContext, type PlayerScene } from "../../../ui-contract/src/index.ts";
  import { addressOk } from "./schemas.ts";

  let { ctx }: { ctx: PlayerContext; scene: PlayerScene } = $props();
  const address = $derived(String(ctx.params.address ?? ""));
  const use = $derived(controlOn(ctx.controls, "use") && ctx.controls?.use === true);
</script>

<div class="plink" data-link-address={address}>
  {#if address && addressOk(address)}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex (out of the tab order unless "use" is on) -->
    <iframe src={address} title={String(ctx.params.title ?? "") || "A page from the game master"} sandbox="allow-scripts allow-same-origin allow-forms" referrerpolicy="no-referrer" loading="lazy" class:still={!use} tabindex={use ? 0 : -1}></iframe>
  {:else}
    <p class="none">{address ? "This page's address cannot be shown here (https, or http on this computer)." : "No page yet."}</p>
  {/if}
</div>

<style>
  .plink {
    height: 100%;
    background: var(--sse-panel-ground);
  }
  iframe {
    width: 100%;
    height: 100%;
    border: 0;
    display: block;
  }
  iframe.still {
    pointer-events: none;
  }
  .none {
    padding: 10px;
    color: var(--sse-ink-muted);
    margin: 0;
  }
</style>
