<!--
  One value on a card, typed where it is shown. Commits on Enter or when focus
  leaves with a change; Escape puts the shown value back. The parent turns the
  text into an intent; this component never edits anything itself.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    label,
    display,
    editable = false,
    set = false,
    derived = false,
    flagged = false,
    changed = false,
    fictional = false,
    oncommit,
    onwhy,
    pencil = false,
    known,
    keepKnown = false,
  }: {
    label: string;
    display: string;
    editable?: boolean;
    /** The value was set by an author (shown in the accent). */
    set?: boolean;
    /** The value was derived by the kernel (shown with a small mark). */
    derived?: boolean;
    /** The kernel flagged the value (outside an atom's validity range). */
    flagged?: boolean;
    /** Published by the campaign's own physics, not the base packs. */
    changed?: boolean;
    /** Published downstream of a fictional pack. */
    fictional?: boolean;
    oncommit?: (text: string) => Promise<string | null>;
    onwhy?: () => void;
    /** The table cannot see this line (secrets design 4): quieter ink and a dotted rule under the value. */
    pencil?: boolean;
    /** The line's "Known to" eye, in the left margin: shown on hover or focus, always when `keepKnown`. */
    known?: Snippet;
    /** The line has an exception of its own, so its eye stays showing. */
    keepKnown?: boolean;
  } = $props();

  let draft = $state("");
  let error = $state<string | null>(null);
  let busy = $state(false);
  const id = $props.id();

  $effect(() => {
    draft = display;
  });

  async function commit() {
    if (!oncommit || draft === display || busy) return;
    busy = true;
    error = await oncommit(draft);
    busy = false;
    if (error) draft = display;
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      draft = display;
      error = null;
      (e.currentTarget as HTMLInputElement).select();
    }
  }
</script>

<div class="field" class:pencil data-pencil={pencil || undefined}>
  <div class="head">
    <label for={id} class="k" class:set>{label}{#if set}<span class="mark">&nbsp;· set</span>{:else if derived}<span class="drv">&nbsp;· derived</span>{/if}{#if flagged}<span class="mark">&nbsp;· out of range</span>{/if}{#if changed}<span class="mark chg">&nbsp;· changed physics</span>{/if}{#if fictional}<span class="mark">&nbsp;· fictional</span>{/if}</label>
    {#if onwhy}
      <button type="button" class="why" onclick={onwhy} aria-label="Why {label.toLowerCase()}?">why?</button>
    {/if}
  </div>
  <div class="row">
    {#if editable}
      <input
        {id}
        class="v"
        bind:value={draft}
        onchange={commit}
        {onkeydown}
        aria-invalid={error ? "true" : undefined}
        aria-describedby={error ? `${id}-err` : undefined}
        disabled={busy}
        autocomplete="off"
        spellcheck="false"
      />
    {:else}
      <!-- A new value settles into place (U12): the element is new, so its arrival animates; the text is always the true value. -->
      {#key display}<output {id} class="v arrive">{display}</output>{/key}
    {/if}
  </div>
  {#if error}<p class="err" id="{id}-err" role="alert">{error}</p>{/if}
  {#if known}<span class="margin" class:keep={keepKnown}>{@render known()}</span>{/if}
</div>

<style>
  .field {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 18px;
    min-width: 0;
  }
  .k {
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .k.set,
  .mark {
    color: var(--sse-accent-ink);
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    min-height: 24px;
  }
  .drv {
    color: var(--sse-ink-muted);
    text-transform: none;
    letter-spacing: 0;
    font-style: italic;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .arrive {
    animation: value-in var(--sse-motion) var(--sse-ease);
  }
  @keyframes value-in {
    from {
      opacity: 0.35;
      transform: translateY(4px);
    }
  }
  .v {
    flex: 1;
    min-width: 0;
    border: 0;
    border-bottom: var(--sse-line) dotted transparent;
    background: transparent;
    padding: 2px 0;
    color: var(--sse-ink);
    font-family: var(--sse-font-mono);
    font-size: 18px;
  }
  input.v:hover {
    border-bottom-color: var(--sse-ink-faint);
  }
  input.v:focus {
    outline: none;
    border-bottom: var(--sse-line-heavy) solid var(--sse-accent);
  }
  .why {
    flex: none;
    min-width: 36px;
    min-height: 24px;
    margin: -4px -6px -4px 0;
    padding: 0 6px;
    border: 0;
    background: transparent;
    color: var(--sse-ink-muted);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 14px;
    cursor: pointer;
  }
  .why:hover {
    color: var(--sse-accent-ink);
  }
  /* Pencil: what only the GM knows is drawn quieter, with a dotted rule under it. */
  .pencil .v {
    color: var(--sse-ink-muted);
    border-bottom: var(--sse-line) dotted var(--sse-ink-muted);
  }
  .margin {
    position: absolute;
    left: -6px;
    top: 30px;
    opacity: 0;
  }
  .field:hover .margin,
  .field:focus-within .margin,
  .margin.keep {
    opacity: 1;
  }
  @media (pointer: coarse) {
    .margin {
      opacity: 1;
    }
  }
  .err {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--sse-accent-ink);
  }
</style>
