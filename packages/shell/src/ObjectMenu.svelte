<script lang="ts">
  // The menu on any object (V3 gap 38): right-click, or press and hold on a touch screen. Its items come from the one
  // action list (ui-contract actions.ts) for the desk in front; an item that cannot be done now stays, greyed, with the
  // reason underneath in words (there is no hover on a phone to carry a title). Other desks' extras are one quiet line
  // each at the foot, which switches desk. Opens from the object it describes; Escape, a press outside or a pick closes
  // it. Arrow keys move, Home and End jump, Enter or Space picks. On a phone it is a sheet along the bottom.
  import { opensFrom } from "../../chart-grammar/src/motion.ts";
  import { keyWords, type ActionTarget, type Desk, type ObjectMenu } from "../../ui-contract/src/index.ts";
  import { placeMenu } from "./object-menu.ts";

  let {
    target,
    menu,
    at,
    from = null,
    kicker = "",
    onpick,
    ondesk,
    onclose,
  }: {
    target: ActionTarget;
    menu: ObjectMenu;
    /** The point it was asked for at, in viewport pixels. */
    at: { x: number; y: number };
    /** The element it opens from. */
    from?: Element | null;
    /** A line under the name: what it is, in words ("planet · orbits Kestrel"). */
    kicker?: string;
    onpick: (id: string) => void;
    ondesk: (d: Desk) => void;
    onclose: () => void;
  } = $props();

  let root = $state<HTMLElement | null>(null);
  let pos = $state<{ left: number; top: number } | null>(null);
  const sheet = typeof matchMedia === "function" && matchMedia("(max-width: 600px) and (pointer: coarse)").matches;
  const returnTo = typeof document !== "undefined" ? (document.activeElement as HTMLElement | null) : null;

  $effect(() => {
    if (!root) return;
    const r = root.getBoundingClientRect();
    pos = sheet ? null : placeMenu(at.x, at.y, r.width, r.height, innerWidth, innerHeight);
    // The first item that can be done takes the focus, so the keyboard is ready.
    queueMicrotask(() => root?.querySelector<HTMLElement>('[role="menuitem"]:not([aria-disabled="true"])')?.focus());
  });

  function close() {
    onclose();
    if (returnTo && document.contains(returnTo)) returnTo.focus({ preventScroll: true });
  }
  function items(): HTMLElement[] {
    return [...(root?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])];
  }
  function onkey(e: KeyboardEvent) {
    const all = items();
    const i = all.indexOf(document.activeElement as HTMLElement);
    const to = (j: number) => (e.preventDefault(), all[(j + all.length) % all.length]?.focus());
    if (e.key === "ArrowDown") to(i + 1);
    else if (e.key === "ArrowUp") to(i < 0 ? all.length - 1 : i - 1);
    else if (e.key === "Home") to(0);
    else if (e.key === "End") to(all.length - 1);
    else if (e.key === "Escape") (e.preventDefault(), e.stopPropagation(), close());
    else if (e.key === "Tab") close();
  }
  function pick(id: string, why?: string) {
    if (why) return;
    // The pick first, while the menu still knows its object; then it closes.
    onpick(id);
    onclose();
  }
</script>

<svelte:window
  onpointerdowncapture={(e) => root && !root.contains(e.target as Node) && onclose()}
  onkeydowncapture={(e) => e.key === "Escape" && (e.stopPropagation(), close())}
  onresize={onclose}
  onscrollcapture={(e) => root && !root.contains(e.target as Node) && onclose()} />

{#if sheet}<div class="scrim" aria-hidden="true"></div>{/if}
<div
  class="omenu"
  class:sheet
  bind:this={root}
  role="menu"
  tabindex="-1"
  aria-label="{target.label}: what to do"
  data-object-menu={target.address}
  style:left={pos ? `${pos.left}px` : undefined}
  style:top={pos ? `${pos.top}px` : undefined}
  style:visibility={pos || sheet ? "visible" : "hidden"}
  use:opensFrom={() => from ?? new DOMRect(at.x, at.y, 0, 0)}
  onkeydown={onkey}>
  <div class="head">
    <span class="name">{target.label}</span>
    {#if kicker}<span class="kicker">{kicker}</span>{/if}
  </div>
  {#each menu.items as it, i (it.id)}
    {#if i > 0 && menu.items[i - 1]!.group !== it.group}<div class="rule" role="separator"></div>{/if}
    <button
      type="button"
      role="menuitem"
      class="item"
      tabindex="-1"
      aria-disabled={it.why ? "true" : undefined}
      aria-keyshortcuts={it.key}
      aria-describedby={it.why ? `om-why-${it.id}` : undefined}
      data-action={it.id}
      onclick={() => pick(it.id, it.why)}>
      <span class="words">{it.label}</span>
      {#if it.key}<kbd>{keyWords(it.key)}</kbd>{/if}
      {#if it.why}<span class="why" id="om-why-{it.id}">{it.why}</span>{/if}
    </button>
  {/each}
  {#if menu.elsewhere.length}
    <div class="rule" role="separator"></div>
    {#each menu.elsewhere as d (d.desk)}
      <button type="button" role="menuitem" class="item other" tabindex="-1" data-other-desk={d.desk} onclick={() => (onclose(), ondesk(d.desk))} title="Switch to the {d.desk} desk">
        <span class="words">{d.words}</span><span class="go" aria-hidden="true">›</span>
      </button>
    {/each}
  {/if}
</div>

<style>
  .omenu {
    position: fixed;
    z-index: 60;
    min-width: 200px;
    max-width: min(320px, calc(100vw - 16px));
    max-height: calc(100vh - 16px);
    overflow-y: auto;
    padding: 4px 0;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    box-shadow: var(--sse-card-shadow);
    font: 13px/1.3 var(--sse-font-body);
    outline: none;
    transform-origin: top left;
  }
  .omenu.sheet {
    left: 0;
    right: 0;
    bottom: 0;
    top: auto;
    max-width: none;
    max-height: 70vh;
    border-width: var(--sse-line) 0 0;
    padding: 6px 0 calc(8px + env(safe-area-inset-bottom));
    font-size: 15px;
  }
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 59;
    background: var(--sse-ink);
    opacity: 0.12;
  }
  .head {
    display: grid;
    padding: 4px 12px 6px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
    margin-bottom: 2px;
  }
  .name {
    font: 600 13px var(--sse-font-body);
  }
  .sheet .name {
    font-size: 15px;
  }
  .kicker {
    color: var(--sse-ink-muted);
    font-size: 11.5px;
  }
  .item {
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: baseline;
    align-content: center;
    column-gap: 16px;
    width: 100%;
    min-height: 30px;
    padding: 5px 12px;
    border: none;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .sheet .item {
    min-height: 44px;
  }
  /* Every touch answers: a small change of tone and an inked edge, within a frame. */
  .item:hover,
  .item:focus-visible {
    background: var(--sse-ground);
    box-shadow: inset 2px 0 0 var(--sse-accent);
    outline: none;
  }
  .item:active:not([aria-disabled="true"]) {
    transform: translateY(1px);
  }
  .item[aria-disabled="true"] {
    cursor: default;
    color: var(--sse-ink-muted);
  }
  .item[aria-disabled="true"]:hover {
    box-shadow: none;
  }
  .why {
    grid-column: 1 / -1;
    font-size: 11.5px;
    color: var(--sse-ink-muted);
  }
  kbd {
    font: 11px var(--sse-font-mono);
    color: var(--sse-ink-muted);
  }
  .other {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .go {
    color: var(--sse-ink-muted);
  }
  .rule {
    height: 0;
    margin: 3px 0;
    border-top: var(--sse-line) solid var(--sse-rule);
    opacity: 0.6;
  }
  @media (prefers-reduced-motion: reduce) {
    .item:active:not([aria-disabled="true"]) {
      transform: none;
    }
  }
</style>
