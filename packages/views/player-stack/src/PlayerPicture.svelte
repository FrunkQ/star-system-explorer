<!--
  Picture (PB13 part 2b): a picture a layout's box holds, from a web address, a pack's file or the GM's own. It reads
  nothing of the scene: what the table knows is not in it. An address it may not show is said plainly.
-->
<script lang="ts">
  import type { PlayerContext, PlayerScene } from "../../../ui-contract/src/index.ts";
  import { pictureOk } from "./schemas.ts";

  let { ctx }: { ctx: PlayerContext; scene: PlayerScene } = $props();
  const src = $derived(String(ctx.params.src ?? ""));
  const caption = $derived(String(ctx.params.caption ?? ""));
</script>

<figure class="ppic" data-picture-src={src ? (src.startsWith("data:") ? "own" : src) : ""}>
  {#if src && pictureOk(src)}
    <img src={src.startsWith("data:") || /^https?:/i.test(src) ? src : new URL(src, document.baseURI).href} alt={caption || "A picture from the game master"} style="object-fit: {ctx.params.fit === 'cover' ? 'cover' : 'contain'}" />
  {:else}
    <p class="none">{src ? "This picture's address cannot be shown here." : "No picture yet."}</p>
  {/if}
  {#if caption}<figcaption>{caption}</figcaption>{/if}
</figure>

<style>
  .ppic {
    margin: 0;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--sse-panel-ground);
  }
  img {
    flex: 1;
    min-height: 0;
    width: 100%;
    height: 100%;
    display: block;
  }
  figcaption,
  .none {
    padding: 6px 10px;
    color: var(--sse-ink-muted);
    font-size: 13px;
    margin: 0;
  }
</style>
