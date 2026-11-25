<!--
  The Parts page of the Reference screen (AT13, atlas design 11A.4): the parts player screens are made of, each with
  the controls it can draw and whether each is on unless a layout says otherwise. A layout item's `controls` switches
  them ("none" makes the part a picture, for a wall screen or a stream). Orac reads the same list (orac-looks.ts).
  Settings are listed at Science depth and deeper; the controls are listed at every depth, because they are what a GM
  decides when making a screen for the table.
-->
<script lang="ts">
  import type { Depth, PartControl, ViewModule } from "../../../ui-contract/src/index.ts";

  let { views, depth }: { views: readonly Pick<ViewModule<unknown, unknown>, "id" | "params" | "switchable" | "scene" | "tiers">[]; depth: Depth } = $props();
  const parts = $derived([...views].filter((v) => v.scene === "player").sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0)));
  const name = (id: string) => id.replace(/^sse\.view\./, "").replace(/-/g, " ").replace(/^./, (c) => c.toUpperCase());
  const KIND: Record<PartControl["kind"], string> = { button: "Button", switch: "Switch", choice: "Choice", step: "Step", dial: "Dial", tab: "Tab", link: "Link" };
</script>

<div class="parts" data-parts>
  <p class="intro">The parts a player screen is made of. A screen switches each part's controls on or off; a part with all of them off is a picture, which nobody at the table can press, and the game master can still steer it from the Switchboard.</p>
  {#each parts as v (v.id)}
    <article class="part" data-part={v.id}>
      <h3>{name(v.id)} <small>{v.id}</small></h3>
      {#if v.switchable?.length}
        <ul class="controls" aria-label="Controls of {name(v.id)}">
          {#each v.switchable as c (c.id)}
            <li data-part-control={c.id}><b>{c.words}</b> <span class="meta">{KIND[c.kind]} · <code>{c.id}</code> · {c.default ? "on" : "off"} unless the screen says</span></li>
          {/each}
        </ul>
      {:else}
        <p class="none">No controls: always a picture.</p>
      {/if}
      {#if depth !== "curious"}<p class="meta">{v.params.length} settings · runs on {v.tiers.join(", ")}</p>{/if}
    </article>
  {/each}
</div>

<style>
  .parts {
    display: grid;
    gap: 14px;
    max-width: 760px;
  }
  .intro {
    margin: 0;
    color: var(--sse-ink-muted);
  }
  .part {
    border: var(--sse-line) solid var(--sse-rule);
    border-radius: var(--sse-radius);
    padding: 10px 14px;
    background: var(--sse-ground-raised);
  }
  h3 {
    margin: 0 0 6px;
    font-size: 16px;
  }
  h3 small,
  .meta {
    color: var(--sse-ink-muted);
    font-size: 12px;
    font-weight: normal;
  }
  .controls {
    margin: 0;
    padding-left: 18px;
    display: grid;
    gap: 4px;
  }
  .none {
    margin: 0;
    color: var(--sse-ink-muted);
  }
  p.meta {
    margin: 6px 0 0;
  }
</style>
