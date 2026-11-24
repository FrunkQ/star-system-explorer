<!-- On air (Switchboard PB2): one chip that says how many screens are on, how many held, and what the table sees, and
     opens the Switchboard. In the top bar on the Run desk (chart room, 24 Nov); on the status line on other desks. -->
<script lang="ts">
  import type { Share } from "../share.svelte.ts";

  let { share, onswitchboard, bar = false }: { share: Share; onswitchboard: () => void; bar?: boolean } = $props();
  const shown = $derived(share.state.shown);
  const held = $derived(Object.values(share.state.board.lines).filter((l) => l?.held).length + (share.state.board.all ? 1 : 0));
</script>

{#if share.state.link}
  <button type="button" class="chip" class:bar data-on-air data-players={share.state.players} onclick={onswitchboard} title="Open the Switchboard: what every screen shows">
    On air · {share.state.players} {share.state.players === 1 ? "screen" : "screens"}{held ? ` · ${held} held` : ""} ·
    <span class="table-sees" data-table-sees={"off" in shown ? "off" : shown.scope}>{"off" in shown ? (shown.card ? `held: ${shown.card}` : "screens dark") : (shown.title ?? "the opening place")}</span>
  </button>
{/if}

<style>
  .chip {
    font: inherit;
    color: var(--sse-ink);
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    padding: 0 8px;
    cursor: pointer;
    white-space: nowrap;
    max-width: 28em;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: transform var(--sse-motion-quick, 120ms) var(--sse-ease, ease-out);
  }
  .chip:hover {
    transform: translateY(-1px);
  }
  .chip.bar {
    height: 32px;
    font-family: var(--sse-font-body);
    font-size: 13px;
    padding: 0 12px;
    max-width: 210px;
  }
  @media (max-width: 1279px) {
    .chip.bar {
      max-width: 120px;
    }
  }
  /* A phone's bar has no room: the Switchboard is on the rail. */
  @media (max-width: 760px) {
    .chip.bar {
      display: none;
    }
  }
  .chip::before {
    content: "●";
    color: var(--sse-accent);
    margin-right: 6px;
  }
  @media (prefers-reduced-motion: reduce) {
    .chip {
      transition: none;
    }
  }
</style>
