<!-- The standard view controls (interface style rule 12): View style (Chart, 3D, True, Wireframe) and Seen from (Above,
     Tilted, Edge-on, Free), the same words in the same order on every diagram, for every seat. Options a view or this
     device cannot do stay in place, quiet, with the reason as their title. Free shows how it is worked. -->
<script lang="ts">
  import Segmented from "./Segmented.svelte";
  import { FREE_HINT, pickSeen, pickStyle, viewOptions, type SeenFrom, type ViewChoice, type ViewControlsSpec, type ViewStyle } from "../../ui-contract/src/index.ts";

  let { spec, choice, onchange, compact = false }: { spec: ViewControlsSpec; choice: ViewChoice; onchange: (c: ViewChoice) => void; compact?: boolean } = $props();
  const o = $derived(viewOptions(spec, choice));
</script>

<div class="vc" class:compact data-view-controls data-style={o.shown.style} data-seen={o.shown.seen}>
  <div class="field">
    <span class="k" aria-hidden="true">View style</span>
    <Segmented label="View style" value={o.shown.style} options={o.styles} onchange={(s: ViewStyle) => onchange(pickStyle(choice, s))} />
  </div>
  <div class="field">
    <span class="k" aria-hidden="true">Seen from</span>
    <Segmented label="Seen from" value={o.shown.seen} options={o.seen} onchange={(s: SeenFrom) => onchange(pickSeen(spec, choice, s))} />
  </div>
  {#if o.shown.seen === "free" && o.turnable}<p class="hint" data-free-hint>{FREE_HINT}</p>{/if}
</div>

<style>
  .vc {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 6px 14px;
    max-width: 100%;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .k {
    font-family: var(--sse-font-body);
    font-size: 10px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .field :global(.seg) {
    background: var(--sse-ground-raised);
  }
  .field :global(.seg button) {
    height: 30px;
    padding: 0 10px;
    font-size: 12.5px;
  }
  .hint {
    flex-basis: 100%;
    margin: 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  /* A phone: the labels go, the rows stay, and the two switches scroll sideways rather than wrap mid-word. */
  .compact .k {
    display: none;
  }
  @media (max-width: 760px) {
    .k {
      display: none;
    }
    .field :global(.seg) {
      overflow-x: auto;
      max-width: 100%;
    }
    .field :global(.seg button) {
      padding: 0 8px;
      flex: none;
    }
  }
</style>
