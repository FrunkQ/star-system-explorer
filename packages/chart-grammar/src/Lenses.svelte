<!--
  The three explain lenses over the same facts (H6.5): Curious (no background
  assumed), Science (the mechanism and its assumptions), Engine (ids,
  versions, determinism). They are cumulative: each adds to the one before.
-->
<script lang="ts">
  export type LensId = "curious" | "science" | "engine";
  let { text, lens = $bindable("curious") }: { text: { curious?: string; science?: string; engine?: string }; lens?: LensId } = $props();
  const ORDER: [LensId, string][] = [
    ["curious", "Curious"],
    ["science", "Science"],
    ["engine", "Engine"],
  ];
  const shown = $derived(ORDER.slice(0, ORDER.findIndex(([id]) => id === lens) + 1).map(([id]) => [id, text[id]] as const));
  const id = $props.id();
</script>

<div class="lenses" role="tablist" aria-label="Lens">
  {#each ORDER as [l, name] (l)}
    <button type="button" role="tab" id="{id}-{l}" aria-controls="{id}-panel" aria-selected={lens === l} tabindex={lens === l ? 0 : -1} onclick={() => (lens = l)}
      onkeydown={(e) => {
        const i = ORDER.findIndex(([x]) => x === lens);
        const by = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
        if (!by) return;
        e.preventDefault();
        lens = ORDER[(i + by + ORDER.length) % ORDER.length]![0];
        document.getElementById(`${id}-${lens}`)?.focus();
      }}>{name}</button>
  {/each}
</div>
<div class="panel" role="tabpanel" id="{id}-panel" aria-labelledby="{id}-{lens}">
  {#each shown as [l, t] (l)}
    <p class={l}>{t ?? "This lens is not written for this atom yet."}</p>
  {/each}
</div>

<style>
  .lenses {
    display: flex;
    border-bottom: var(--sse-line) solid var(--sse-rule);
    margin-top: 6px;
  }
  button {
    border: 0;
    background: transparent;
    min-height: 28px;
    padding: 0 10px;
    font-family: var(--sse-font-body);
    font-size: 12px;
    color: var(--sse-ink-muted);
    cursor: pointer;
  }
  button[aria-selected="true"] {
    color: var(--sse-ink);
    box-shadow: inset 0 -2px 0 var(--sse-accent);
  }
  .panel {
    margin: 8px 0;
  }
  p {
    margin: 0 0 6px;
  }
  .curious,
  .science {
    font-family: var(--sse-font-display);
    font-size: 15px;
    line-height: 1.45;
  }
  .science {
    padding-left: 10px;
    border-left: var(--sse-line) solid var(--sse-rule);
  }
  .engine {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    line-height: 1.5;
    color: var(--sse-ink-muted);
    overflow-wrap: anywhere;
  }
</style>
