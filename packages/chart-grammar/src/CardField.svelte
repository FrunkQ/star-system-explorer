<!--
  One value on a card, typed where it is shown. Commits on Enter or when focus
  leaves with a change; Escape puts the shown value back. The parent turns the
  text into an intent; this component never edits anything itself.
-->
<script lang="ts">
  let {
    label,
    display,
    editable = false,
    set = false,
    oncommit,
    onwhy,
  }: {
    label: string;
    display: string;
    editable?: boolean;
    /** The value was set by an author (shown in the accent). */
    set?: boolean;
    oncommit?: (text: string) => Promise<string | null>;
    onwhy?: () => void;
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

<div class="field">
  <div class="head">
    <label for={id} class="k" class:set>{label}{#if set}<span class="mark">&nbsp;· set</span>{/if}</label>
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
      <output {id} class="v">{display}</output>
    {/if}
  </div>
  {#if error}<p class="err" id="{id}-err" role="alert">{error}</p>{/if}
</div>

<style>
  .field {
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
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
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
  .err {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--sse-accent-ink);
  }
</style>
