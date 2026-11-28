<!-- The folded rail (three desks design 3.1, question 6): on the Run desk the rail is one button at the head of the view
     controls that names the instrument in view ("Globe ▾"). It opens the rail as a list in words, with the Run desk's
     instruments only (the rail's own entries, filtered by desk), and the doors in and out. The picture gets the width.
     A press on anything in the list closes it; so do Esc and a press elsewhere. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { opensFrom } from "../../../chart-grammar/src/index.ts";

  let { label, children }: { label: string; children: Snippet } = $props();

  let open = $state(false);
  let anchor: HTMLElement | undefined = $state();
  let pop: HTMLElement | undefined = $state();

  $effect(() => {
    if (!open) return;
    const away = (e: PointerEvent) => {
      const t = e.target as Node;
      if (!pop?.contains(t) && !anchor?.contains(t)) open = false;
    };
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") (e.stopPropagation(), (open = false), anchor?.querySelector("button")?.focus());
    };
    document.addEventListener("pointerdown", away, true);
    document.addEventListener("keydown", key, true);
    return () => (document.removeEventListener("pointerdown", away, true), document.removeEventListener("keydown", key, true));
  });
</script>

<span class="fold" bind:this={anchor}>
  <button type="button" class="fold-btn" aria-expanded={open} aria-haspopup="true" data-rail-fold title="Instruments for the table: what to look at here" onclick={() => (open = !open)}>{label} <span aria-hidden="true">▾</span></button>
  {#if open}
    <!-- Any button pressed inside goes somewhere, so the list closes behind it. -->
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="pop" bind:this={pop} use:opensFrom={() => anchor} onclick={(e) => (e.target as HTMLElement).closest("button") && (open = false)} data-rail-list>
      {@render children()}
    </div>
  {/if}
</span>

<style>
  .fold {
    position: relative;
    display: inline-flex;
    align-self: flex-end;
  }
  .fold-btn {
    height: 32px;
    padding: 0 12px;
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    cursor: pointer;
    white-space: nowrap;
  }
  .fold-btn[aria-expanded="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .pop {
    position: absolute;
    left: 0;
    top: calc(100% + 6px);
    z-index: 20;
    width: 240px;
    max-height: 70vh;
    overflow: auto;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    box-shadow: 0 6px 18px rgb(0 0 0 / 0.14);
  }
  /* The rail inside the list keeps its words and groups, without its own rule. */
  .pop :global(.rail) {
    border-right: 0;
  }
  /* On a phone the button is thumb-sized and the list takes most of the width (design section 8). */
  @media (max-width: 760px) {
    .fold {
      align-self: flex-start;
    }
    .fold-btn {
      height: 44px;
    }
    .pop {
      width: min(320px, calc(100vw - 32px));
    }
  }
</style>
