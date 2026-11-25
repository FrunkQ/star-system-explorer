<!--
  Fill this display (PB9, design 3.2): a window the GM opened on one of their own displays offers to go full screen,
  since a browser lets only a press in that window do it. F does the same. It goes once the window is full screen,
  and is never shown in a part of a split window (the frame around the parts asks instead) or on a phone's own link.
-->
<script lang="ts">
  import { onMount } from "svelte";

  const local = new URLSearchParams(location.search).get("local") === "1";
  const top = (() => {
    try {
      return window.top === window;
    } catch {
      return false;
    }
  })();
  let full = $state(false);
  let hidden = $state(false);
  const fill = () => void document.documentElement.requestFullscreen?.().catch(() => (hidden = true));
  onMount(() => {
    const sync = () => (full = !!document.fullscreenElement);
    const key = (e: KeyboardEvent) => {
      if ((e.key === "f" || e.key === "F") && !e.ctrlKey && !e.metaKey && !e.altKey && !(e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement)) fill();
    };
    document.addEventListener("fullscreenchange", sync);
    window.addEventListener("keydown", key);
    return () => (document.removeEventListener("fullscreenchange", sync), window.removeEventListener("keydown", key));
  });
</script>

{#if local && top && !full && !hidden}
  <div class="fill" data-fill-screen>
    <button type="button" class="go" onclick={fill}>Fill this display</button>
    <span>or press F</span>
    <button type="button" class="x" aria-label="Not now" onclick={() => (hidden = true)}>×</button>
  </div>
{/if}

<style>
  .fill {
    position: fixed;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    z-index: 50;
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 8px 12px;
    border: var(--sse-line, 1px) solid var(--sse-ink, #222);
    border-radius: var(--sse-radius, 4px);
    background: var(--sse-ground-raised, #fff);
    color: var(--sse-ink, #222);
    font: 14px var(--sse-font-body, sans-serif);
    box-shadow: 0 6px 24px rgb(0 0 0 / 0.25);
  }
  button {
    font: inherit;
    cursor: pointer;
    color: inherit;
    background: transparent;
    border: var(--sse-line, 1px) solid currentColor;
    border-radius: var(--sse-radius, 4px);
    padding: 3px 10px;
  }
  .go {
    font-weight: 600;
  }
  .x {
    border: 0;
    padding: 0 4px;
  }
</style>
