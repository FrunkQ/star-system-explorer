<!-- On air (Switchboard PB2): one chip that says how many screens are on, how many held, and what the table sees, and
     opens the Switchboard. In the top bar on the Run desk (chart room, 24 Nov); on the status line on other desks. -->
<script lang="ts">
  import type { Share } from "../share.svelte.ts";

  let { share, onswitchboard, bar = false }: { share: Share; onswitchboard: () => void; bar?: boolean } = $props();
  const shown = $derived(share.state.shown);
  const drv = $derived(share.state.board.driving);
  const drvName = $derived(!drv ? "" : drv === "everyone" ? "Everyone" : drv.startsWith("screen:") ? (share.state.board.screens[drv.slice(7)]?.name ?? "a screen") : drv.slice(5));
  const held = $derived(Object.values(share.state.board.lines).filter((l) => l?.held).length + (share.state.board.all ? 1 : 0));
</script>

{#if share.state.link}
  <button type="button" class="chip" class:bar data-on-air data-players={share.state.players} onclick={onswitchboard} title="Open the Switchboard: what every screen shows">
    {#if drv}<b data-chip-driving>You are driving {drvName}</b> ·{/if}
    {#if share.state.waiting.length}<b class="asking" data-chip-asking={share.state.waiting.length}>{share.state.waiting.length} asking</b> ·{/if}
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
  .asking {
    color: var(--sse-accent);
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
