<!--
  A holding card's ground (PC5, postcards design 8.2 and 8.3): None, A postcard, or Postcards in turn. One row under
  the card's kind on the line card and on a preset's Hold step, with a strip of the campaign's postcard stamps when one
  is to be picked. A postcard that has not been opened on the wall yet has no kept picture, so it cannot be a ground
  until it has (the row says so).
-->
<script lang="ts">
  import { holdGround, isPostcard, stampFor, withGround, type Bookmark, type HoldCard, type HoldGround } from "../../../ui-contract/src/index.ts";

  let { card, bookmarks, label, onpick }: { card: HoldCard; bookmarks: readonly Bookmark[]; label: string; onpick: (next: HoldCard) => void } = $props();

  const g = $derived(holdGround(card));
  const mode = $derived(g === "none" ? "none" : "bookmark" in g || "postcard" in g ? "one" : g.set === "campaign" ? "campaign" : "base");
  const cards = $derived(bookmarks.filter(isPostcard).map((b) => ({ b, stamp: stampFor(b.scope, b.id) })));
  const withPicture = $derived(cards.filter((c) => c.stamp));
  let picking = $state(false);
  function choose(v: string) {
    picking = v === "one";
    if (v === "one") {
      const first = withPicture[0];
      if (first && mode !== "one") onpick(withGround(card, { bookmark: first.b.id }));
      return;
    }
    const next: HoldGround = v === "none" ? "none" : { set: v };
    onpick(withGround(card, next));
  }
  const stop = (e: Event) => e.stopPropagation();
</script>

<span class="ground" data-ground-pick={mode}>
  <label
    >Ground
    <select aria-label="Ground for {label}" data-ground value={mode} onclick={stop} onkeydown={stop} onchange={(e) => choose(e.currentTarget.value)}>
      <option value="none">None</option>
      <option value="one" disabled={!withPicture.length}>A postcard</option>
      <option value="base">Postcards in turn</option>
      <option value="campaign" disabled={!withPicture.length}>This campaign's postcards in turn</option>
    </select>
  </label>
  {#if mode === "one" || picking}
    {#if withPicture.length}
      <span class="strip" role="radiogroup" aria-label="Postcard for {label}">
        {#each withPicture as c (c.b.id)}
          {@const on = typeof g === "object" && "bookmark" in g && g.bookmark === c.b.id}
          <button type="button" role="radio" aria-checked={on} class:on data-ground-card={c.b.id} title={c.b.name} onclick={(e) => (stop(e), onpick(withGround(card, { bookmark: c.b.id })))}><img src={c.stamp} alt={c.b.name} /></button>
        {/each}
      </span>
    {/if}
  {/if}
  {#if !withPicture.length && cards.length}<span class="why" data-ground-why>These postcards have no kept picture yet: open one on the wall once and it can be a ground.</span>{/if}
</span>

<style>
  .ground {
    display: flex;
    flex-wrap: wrap;
    flex-basis: 100%;
    min-width: 0;
    gap: 6px;
    align-items: center;
    font-size: 12px;
  }
  .ground label {
    display: inline-flex;
    gap: 6px;
    align-items: center;
  }
  .ground select {
    width: auto;
    max-width: 100%;
  }
  .strip {
    display: flex;
    gap: 4px;
    overflow-x: auto;
    max-width: 100%;
  }
  .strip button {
    padding: 0;
    border: var(--sse-line) solid transparent;
    border-radius: 3px;
    background: none;
    cursor: pointer;
    line-height: 0;
  }
  .strip button.on {
    border-color: var(--sse-accent);
  }
  .strip img {
    width: 48px;
    height: 32px;
    object-fit: cover;
    border-radius: 2px;
  }
  .why {
    color: var(--sse-ink-muted);
  }
</style>
