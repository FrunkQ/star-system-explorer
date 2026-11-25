<!--
  An edition's cover (track AT, AT7; atlas design 14A note 4): shown once each time the edition is opened; one press
  anywhere, or Enter, goes past it. Its words are the edition's; its colours the look's.
-->
<script lang="ts">
  import type { EditionCover } from "../../../ui-contract/src/index.ts";

  let { cover, place, onopen }: { cover: EditionCover; place: string; onopen: () => void } = $props();
  let me = $state<HTMLButtonElement | null>(null);
  $effect(() => {
    me?.focus({ preventScroll: true });
  });
</script>

<button type="button" class="cover" bind:this={me} onclick={onopen} data-atlas-cover aria-label="{cover.big ?? 'Cover'}. Press to open.">
  {#if cover.big}<span class="big">{cover.big}</span>{/if}
  <span class="line" aria-hidden="true"></span>
  {#if cover.title}<span class="title">{cover.title.replace("{place}", place)}</span>{/if}
  {#if cover.strap}<span class="strap">{cover.strap}</span>{/if}
  {#if cover.publisher}<span class="pub">{cover.publisher}</span>{/if}
  <span class="press">press anywhere to open</span>
</button>

<style>
  .cover {
    all: unset;
    box-sizing: border-box;
    width: 100%;
    min-height: min(70vh, 620px);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    padding: 40px 18px;
    cursor: pointer;
    text-align: center;
    background: radial-gradient(ellipse at 50% 40%, var(--sse-ground-raised), var(--sse-ground) 70%);
    color: var(--sse-ink);
    animation: on 420ms steps(5, end);
  }
  .cover:focus-visible {
    outline: 3px solid var(--sse-focus);
    outline-offset: -6px;
  }
  .big {
    font: 800 clamp(48px, 11cqi, 132px) / 0.95 var(--sse-font-display);
    color: var(--sse-cover-big, var(--sse-accent));
    text-shadow: 0 5px 0 var(--sse-cover-shade, var(--sse-accent-ink));
    transform: rotate(-3deg);
    max-width: 8ch;
  }
  .line {
    width: min(60%, 520px);
    height: 6px;
    border-radius: 3px;
    background: var(--sse-screen-rule, var(--sse-accent));
    margin-top: 12px;
  }
  .title {
    font: italic 400 22px/1.3 var(--sse-font-body);
  }
  .strap {
    font-size: 13px;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--sse-label-ink, var(--sse-ink-muted));
  }
  .pub {
    font-size: 12px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .press {
    margin-top: 28px;
    font-size: 13px;
    color: var(--sse-ink-faint);
  }
  @keyframes on {
    from {
      clip-path: inset(48% 0 48% 0);
      filter: brightness(2.5);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .cover {
      animation: none;
    }
  }
</style>
