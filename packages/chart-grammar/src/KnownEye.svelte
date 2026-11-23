<!-- The "Known to" eye (secrets and visibility design 4): open for known to the table, half open for heard of, closed
     for GM only. A symbol, never words; drawn in ordinary ink (closed in muted ink), its states told apart by shape, so
     it reads in the Night chart and without colour. 14 px beside a name, 11 px on a card line or in the popup. -->
<script lang="ts">
  import type { KnownLandmark } from "../../ui-contract/src/index.ts";
  let { landmark, size = 14 }: { landmark: KnownLandmark; size?: number } = $props();
</script>

<svg class="eye" class:closed={landmark === "gm_only"} width={size} height={size} viewBox="0 0 14 14" aria-hidden="true" data-eye={landmark}>
  {#if landmark === "known"}
    <path d="M1 7Q7 0 13 7Q7 14 1 7Z" />
    <circle class="fill" cx="7" cy="7" r="2.5" />
  {:else if landmark === "heard_of"}
    <path d="M1 7Q7 0 13 7Q7 14 1 7Z" />
    <path class="fill" d="M1 7Q7 0 13 7Q7 4.2 1 7Z" />
    <circle class="fill" cx="7" cy="8.6" r="1.6" />
  {:else}
    <path d="M1 5.5Q7 12 13 5.5" />
    <path d="M3.8 8.8 3 11.4M7 10V13M10.2 8.8l.8 2.6" />
  {/if}
</svg>

<style>
  .eye {
    display: block;
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
    overflow: visible;
  }
  .fill {
    fill: currentColor;
    stroke: none;
  }
  .closed {
    color: var(--sse-ink-muted);
  }
</style>
