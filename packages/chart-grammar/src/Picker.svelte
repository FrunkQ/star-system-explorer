<!-- The one picker (Alex, 24 Nov 2026): find a thing at this level of the place line and go to it. It sits after the
     place's name and reads what is there ("12 bodies ▾"); a press opens a list from it with a search box, kind chips
     with counts, the level above, this level and what is directly in it, one level further folded behind a count
     that opens in place. The same component at every scale, with other content (ui-contract picker.ts holds the
     rule). Keyboard: type to search, arrows move, Enter picks, Escape closes and gives the keyboard back. -->
<script lang="ts">
  import { opensFrom } from "./motion.ts";
  import { childWordsFor, elsewhereRows, kindChips, pickerRows, type ElsewhereItem, type PickItem } from "../../ui-contract/src/index.ts";

  interface Entry {
    id: string;
    label: string;
    caption?: string;
  }
  let {
    summary,
    label,
    items,
    selected = null,
    kindOrder = [],
    above = [],
    top,
    sections = [],
    placeholder = "Find…",
    heading,
    elsewhere = [],
    onpick,
  }: {
    /** What the chip says while closed ("12 bodies"). */
    summary: string;
    /** The list's name, for a screen reader ("Go to a body"). */
    label: string;
    items: readonly PickItem[];
    selected?: string | null;
    kindOrder?: readonly string[];
    /** The level above (and further out): first in the list, so going up is as near as going down. */
    above?: readonly Entry[];
    /** This level itself as a choice ("The whole system"). */
    top?: Entry;
    /** Other groups after the list (other systems in the campaign, recent places). */
    sections?: readonly { label: string; entries: readonly Entry[] }[];
    placeholder?: string;
    /** A line naming what the list is in ("In Cell 3 · 4 · 1"), above it. */
    heading?: string;
    /** Places elsewhere in the campaign, found while typing after this level's own (the picker doubles as Find). */
    elsewhere?: readonly ElsewhereItem[];
    onpick: (id: string) => void;
  } = $props();

  const uid = $props.id();
  let open = $state(false);
  let chip = $state<HTMLButtonElement | null>(null);
  let box = $state<HTMLDivElement | null>(null);
  let input = $state<HTMLInputElement | null>(null);
  let query = $state("");
  let kinds = $state<string[]>([]);
  let opened = $state<string[]>([]);
  // Fixed to the chip's place, so a bar that clips its own overflow never clips the list.
  let at = $state({ top: 0, left: 0 });

  const chips = $derived(kindChips(items, kindOrder));
  const list = $derived(pickerRows(items, { query, kinds, open: opened, childWords: childWordsFor }));
  const searching = $derived(list.mode === "search");
  const away = $derived(elsewhereRows(elsewhere, query, items));

  function toggleKind(k: string) {
    kinds = kinds.includes(k) ? kinds.filter((x) => x !== k) : [...kinds, k];
  }
  function toggleOpen(id: string) {
    opened = opened.includes(id) ? opened.filter((x) => x !== id) : [...opened, id];
  }
  function pick(id: string) {
    open = false;
    query = "";
    onpick(id);
    chip?.focus();
  }
  function show() {
    open = !open;
    if (open && chip) {
      const r = chip.getBoundingClientRect();
      at = { top: r.bottom + 8, left: Math.max(16, Math.min(r.left, window.innerWidth - 396)) };
    }
    // The thing in focus starts open, so its moons are a glance away.
    if (open && selected && !opened.includes(selected)) {
      const host = items.find((i) => i.id === selected)?.parent;
      opened = [...opened, selected, ...(host ? [host] : [])];
    }
  }
  // Typing finds at once on a keyboard; a touch screen waits for a tap on the box, so its keyboard does not cover the list.
  $effect(() => {
    if (open && !matchMedia("(pointer: coarse)").matches) queueMicrotask(() => input?.focus());
  });
  // A press off it closes it, as a menu does; Escape clears a search first, then closes and gives the chip the keyboard.
  $effect(() => {
    if (!open) return;
    const away = (e: PointerEvent) => {
      if (box && !box.contains(e.target as Node) && !chip?.contains(e.target as Node)) open = false;
    };
    const keys = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        e.preventDefault();
        if (query) return void ((query = ""), input?.focus());
        open = false;
        chip?.focus();
        return;
      }
      if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
      const rows = [...(box?.querySelectorAll<HTMLButtonElement>("[data-pick]:not([disabled])") ?? [])];
      if (!rows.length) return;
      e.preventDefault();
      const at = rows.indexOf(document.activeElement as HTMLButtonElement);
      const next = e.key === "ArrowDown" ? (at < 0 ? 0 : Math.min(rows.length - 1, at + 1)) : at <= 0 ? -1 : at - 1;
      if (next < 0) input?.focus();
      else rows[next]!.focus();
    };
    document.addEventListener("pointerdown", away, true);
    document.addEventListener("keydown", keys, true);
    return () => {
      document.removeEventListener("pointerdown", away, true);
      document.removeEventListener("keydown", keys, true);
    };
  });
  // Typing on a row goes to the search box.
  function typeAhead(e: KeyboardEvent) {
    if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey && e.target !== input) {
      input?.focus();
    }
  }
</script>

<span class="picker" data-picker>
  <button type="button" class="chip" bind:this={chip} aria-expanded={open} aria-controls="{uid}-list" aria-haspopup="dialog" onclick={show} title={label} data-picker-chip>
    <span class="sum">{summary}</span><span class="caret" aria-hidden="true">▾</span>
  </button>
  {#if open}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="pop" id="{uid}-list" role="dialog" tabindex="-1" aria-label={label} style:top="{at.top}px" style:left="{at.left}px" bind:this={box} use:opensFrom={() => chip} onkeydown={typeAhead} data-picker-pop>
      <div class="find">
        <input type="search" bind:this={input} bind:value={query} {placeholder} aria-label="Find by name" autocomplete="off" spellcheck="false" data-picker-find />
      </div>
      {#if chips.length > 1}
        <div class="kinds" role="group" aria-label="Show only">
          {#each chips as c (c.kind)}
            <button type="button" class="kind" aria-pressed={kinds.includes(c.kind)} onclick={() => toggleKind(c.kind)} data-picker-kind={c.kind}>{c.kind} <span class="n">{c.count}</span></button>
          {/each}
          {#if kinds.length}<button type="button" class="kind clear" onclick={() => (kinds = [])}>Clear</button>{/if}
        </div>
      {/if}
      <div class="rows">
        {#if !searching && above.length}
          <ul class="above" aria-label="Further out">
            {#each above as a (a.id)}<li><button type="button" class="row up" data-pick={a.id} onclick={() => pick(a.id)}><span class="arrow" aria-hidden="true">‹</span> {a.label}{#if a.caption}<span class="cap">{a.caption}</span>{/if}</button></li>{/each}
          </ul>
        {/if}
        {#if searching}<p class="head">Results for “{query.trim()}”{list.more ? `, first ${list.rows.length}` : ""}</p>{/if}
        {#if !searching && top}
          <button type="button" class="row whole" class:sel={!selected} data-pick={top.id} onclick={() => pick(top.id)}>{top.label}{#if top.caption}<span class="cap">{top.caption}</span>{/if}</button>
        {/if}
        {#if !searching && heading}<p class="head" data-picker-head>{heading}</p>{/if}
        <ul class="tree" id="{uid}-tree">
          {#each list.rows as r (r.id)}
            <li style:--depth={r.depth}>
              <button type="button" class="row" class:sel={r.id === selected} class:context={r.context} disabled={r.context} data-pick={r.context ? undefined : r.id} data-pick-row={r.id} data-address={r.context ? undefined : r.id} aria-current={r.id === selected ? "true" : undefined} onclick={() => pick(r.id)}>
                <span class="name">{r.label}</span>{#if r.caption}<span class="cap">{r.caption}</span>{/if}
              </button>
              {#if r.folded || r.opened}
                <button type="button" class="fold" aria-expanded={!!r.opened} aria-controls="{uid}-tree" onclick={() => toggleOpen(r.id)} data-picker-fold={r.id}>{r.folded ? `${r.folded.count} ${r.folded.words} ›` : "‹ fold"}</button>
              {/if}
            </li>
          {:else}
            {#if !away.rows.length}<li class="none">{searching ? (query.trim().startsWith("#") ? "Nothing here carries that tag." : "Nothing here by that name.") : "Nothing to show: try clearing the kinds."}</li>{/if}
          {/each}
        </ul>
        {#if searching && away.rows.length}
          <p class="head" data-picker-elsewhere>Elsewhere in the campaign{away.more ? `, first ${away.rows.length}` : ""}</p>
          <ul>{#each away.rows as e (e.id)}<li><button type="button" class="row" data-pick={e.id} onclick={() => pick(e.id)}><span class="name">{e.label}</span>{#if e.caption}<span class="cap">{e.caption}</span>{/if}</button></li>{/each}</ul>
        {/if}
        {#if !searching}
          {#each sections.filter((s) => s.entries.length) as s (s.label)}
            <p class="head">{s.label}</p>
            <ul>{#each s.entries as e (e.id)}<li><button type="button" class="row" data-pick={e.id} onclick={() => pick(e.id)}>{e.label}{#if e.caption}<span class="cap">{e.caption}</span>{/if}</button></li>{/each}</ul>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</span>

<style>
  .picker {
    position: relative;
    display: inline-flex;
    min-width: 0;
  }
  .chip {
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
    min-width: 0;
    padding: 2px 2px;
    border: 0;
    border-bottom: 1px dotted var(--sse-rule);
    background: transparent;
    font: inherit;
    font-size: 12px;
    color: var(--sse-ink-muted);
    cursor: pointer;
    white-space: nowrap;
  }
  .chip:hover,
  .chip[aria-expanded="true"] {
    color: var(--sse-ink);
    border-bottom-color: var(--sse-ink);
  }
  .sum {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .pop {
    position: fixed;
    z-index: 40;
    box-sizing: border-box;
    width: min(380px, calc(100vw - 32px));
    max-height: min(70vh, 560px);
    display: flex;
    flex-direction: column;
    background: var(--sse-ground);
    border: var(--sse-line) solid var(--sse-ink);
    box-shadow: 4px 4px 0 var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 14px;
    color: var(--sse-ink);
  }
  .find {
    padding: 10px 10px 6px;
  }
  .find input {
    box-sizing: border-box;
    width: 100%;
    padding: 6px 8px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
  }
  .find input:focus-visible {
    outline: 2px solid var(--sse-focus);
    outline-offset: 1px;
  }
  .kinds {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 0 10px 8px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .kind {
    padding: 2px 8px;
    border: var(--sse-line) solid var(--sse-rule);
    border-radius: 999px;
    background: transparent;
    color: var(--sse-ink-muted);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .kind[aria-pressed="true"] {
    background: var(--sse-ink);
    border-color: var(--sse-ink);
    color: var(--sse-ground);
  }
  .kind .n {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    opacity: 0.75;
  }
  .kind.clear {
    border-style: dashed;
  }
  .rows {
    overflow: auto;
    padding: 4px 0 8px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li {
    display: flex;
    align-items: center;
  }
  .row {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 5px 10px 5px calc(10px + var(--depth, 0) * 16px);
    border: 0;
    background: transparent;
    color: var(--sse-ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .row .name {
    white-space: nowrap;
  }
  .tree .row .name {
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 15px;
  }
  .row:hover,
  .row:focus-visible {
    background: var(--sse-ground-raised);
    outline: none;
  }
  .row:focus-visible {
    box-shadow: inset 3px 0 0 var(--sse-focus);
  }
  .row.sel {
    box-shadow: inset 3px 0 0 var(--sse-accent);
  }
  .row.context {
    color: var(--sse-ink-muted);
    cursor: default;
  }
  .row.context .name {
    opacity: 0.7;
  }
  .row.up,
  .row.whole {
    width: 100%;
    color: var(--sse-ink-muted);
    font-size: 13px;
  }
  .above {
    border-bottom: var(--sse-line) dotted var(--sse-rule);
    margin-bottom: 4px;
  }
  .cap {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .fold {
    flex: 0 0 auto;
    margin-right: 8px;
    padding: 2px 6px;
    border: 0;
    background: transparent;
    color: var(--sse-accent);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
  }
  .head {
    margin: 8px 10px 2px;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .none {
    padding: 8px 10px;
    color: var(--sse-ink-muted);
    font-size: 13px;
  }
  @media (max-width: 760px) {
    /* A phone: the chip is a small ▾ beside the name, and the list runs the width of the screen. */
    .sum {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip-path: inset(50%);
    }
    .chip {
      min-width: 24px;
      min-height: 32px;
      justify-content: center;
      align-items: center;
      border-bottom: 0;
    }
    .caret {
      color: var(--sse-accent);
      font-size: 16px;
    }
    .pop {
      left: 16px !important;
      right: 16px;
      width: auto;
    }
    .row {
      min-height: 40px;
    }
  }
</style>
