<!--
  An edition that is a device, not paper (track AT, AT7; atlas design 14A note 4): a case with its maker's and status
  lines, a screen with a bright rule, and big hardware keys under it (Home, Back, the cover key, Next, Print). Every
  colour is the look's (`--sse-case-*`, `--sse-key-*`); the words are the edition's. The keys do what the Atlas's own
  controls do, so nothing here is reachable only by a key.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { EditionDevice } from "../../../ui-contract/src/index.ts";

  let {
    device,
    coverKey,
    onhome,
    onback,
    oncover,
    onnext,
    onprint,
    canBack,
    canNext,
    bannerTop,
    bannerBottom,
    children,
  }: {
    device: EditionDevice;
    coverKey: string;
    onhome: () => void;
    onback: () => void;
    oncover: () => void;
    onnext: () => void;
    onprint: () => void;
    canBack: boolean;
    canNext: boolean;
    bannerTop?: string;
    bannerBottom?: string;
    children: Snippet;
  } = $props();
</script>

<div class="case" data-atlas-device>
  <p class="top">
    <span class="lamp" aria-hidden="true"></span><span>{device.label ?? ""}</span>
    <span class="status">{device.status ?? ""}</span>
  </p>
  <div class="screen">
    <div class="rule" aria-hidden="true"></div>
    {#if bannerTop}<p class="banner" data-atlas-banner="top">{bannerTop}</p>{/if}
    {@render children()}
    {#if bannerBottom}<p class="banner bottom" data-atlas-banner="bottom">{bannerBottom}</p>{/if}
  </div>
  {#if device.keys}
    <div class="keys" role="group" aria-label="Keys">
      <span class="grille" aria-hidden="true"></span>
      <button type="button" class="key home" onclick={onhome}>Home</button>
      <button type="button" class="key back" onclick={onback} disabled={!canBack}>‹ Back</button>
      <button type="button" class="key main" onclick={oncover}>{coverKey}</button>
      <button type="button" class="key next" onclick={onnext} disabled={!canNext}>Next ›</button>
      <button type="button" class="key print" onclick={onprint}>Print</button>
      <span class="grille" aria-hidden="true"></span>
    </div>
  {/if}
</div>

<style>
  /* A one-liner strip: the case's own small capitals on a hairline, never louder than the page. */
  .banner {
    margin: 0;
    padding: 6px 14px;
    font: italic 500 12.5px/1.4 var(--sse-font-body);
    color: var(--sse-margin-ink, var(--sse-ink-muted));
    border-bottom: 1px dashed var(--sse-rule, currentColor);
  }
  .banner.bottom {
    border-bottom: 0;
    border-top: 1px dashed var(--sse-rule, currentColor);
  }
  .case {
    background: var(--sse-case-ground, transparent);
    border: 1px solid var(--sse-case-edge, transparent);
    border-radius: var(--sse-case-radius, 0);
    padding: 10px 18px 18px;
    box-shadow: inset 0 -6px 0 var(--sse-case-edge, transparent);
  }
  .top {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 4px 8px;
    font: 700 11px/1.4 var(--sse-font-body);
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--sse-case-ink, var(--sse-ink-muted));
  }
  .lamp {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--sse-case-light, var(--sse-accent));
  }
  .status {
    margin-left: auto;
  }
  .screen {
    position: relative;
    overflow: hidden;
    border-radius: 12px;
    background: var(--sse-ground);
    box-shadow: inset 0 0 0 6px var(--sse-case-edge, transparent);
    padding: 6px;
  }
  .rule {
    height: 5px;
    border-radius: 3px;
    background: var(--sse-screen-rule, var(--sse-accent));
    margin: 2px 2px 0;
  }
  .keys {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 14px;
    margin-top: 16px;
    flex-wrap: wrap;
  }
  .grille {
    width: 64px;
    height: 26px;
    background: repeating-linear-gradient(90deg, var(--sse-case-edge, transparent) 0 3px, transparent 3px 8px);
  }
  .key {
    font: 800 15px/1 var(--sse-font-body);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-key-ink, var(--sse-on-ink));
    border: 0;
    border-radius: 12px;
    padding: 13px 22px;
    box-shadow: var(--sse-key-shadow, none);
    cursor: pointer;
    transition: transform 80ms ease-out, box-shadow 80ms ease-out;
  }
  .key:active:not(:disabled) {
    transform: translateY(4px);
    box-shadow: none;
  }
  .key:disabled {
    filter: saturate(0.3);
    cursor: default;
  }
  .key:focus-visible {
    outline: 3px solid var(--sse-focus);
    outline-offset: 3px;
  }
  .home {
    background: var(--sse-key-home, var(--sse-accent));
  }
  .back {
    background: var(--sse-key-back, var(--sse-accent));
  }
  .next {
    background: var(--sse-key-next, var(--sse-key-back, var(--sse-accent)));
  }
  .main {
    background: var(--sse-key-main, var(--sse-accent));
    padding: 15px 44px;
    font-size: 17px;
  }
  .print {
    background: var(--sse-key-print, var(--sse-accent));
  }
  @container atlas (max-width: 760px) {
    .case {
      padding: 8px 8px 12px;
    }
    .grille,
    .status {
      display: none;
    }
    .key {
      padding: 11px 12px;
      font-size: 13px;
    }
    .main {
      padding: 12px 16px;
      font-size: 14px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .key {
      transition: none;
    }
  }
</style>
