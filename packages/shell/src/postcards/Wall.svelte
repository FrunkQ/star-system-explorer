<script lang="ts">
  // The wall (PC3, design section 6): this campaign's postcards, then postcards opened from outside (visits), then the
  // sets from packs. A row of stamps; press one and it opens as a card. Presentation only.
  import { envelope, isPostcard, type Bookmark, type Postcard, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import Card from "./Card.svelte";
  import { keepStamp, stampFor } from "../../../ui-contract/src/index.ts";
  import { visits, GO_BOOKMARK_EVENT, PASTE_CLIP_EVENT } from "./visits.svelte.ts";
  import { packForSending } from "./pack.ts";

  let { ctx, scene }: { ctx: ViewContext; scene: SystemScene } = $props();
  const ws = $derived(ctx.workspace);

  const own = $derived((ws.bookmarks ?? []).filter(isPostcard) as (Bookmark & { at_yr: number })[]);
  const ownCards = $derived(own.map((b) => envelope(b, { how: "here" }, { id: b.id, name: b.name })));
  let openId = $state<string | null>(null);
  $effect(() => {
    if (visits.open) {
      openId = `visit:${visits.open}`;
      visits.open = null;
    }
  });
  const opened = $derived.by((): { card: Postcard; visiting: boolean } | null => {
    if (!openId) return null;
    if (openId.startsWith("visit:")) {
      const v = visits.list.find((x) => `visit:${x.id}` === openId);
      return v ? { card: v, visiting: true } : null;
    }
    const c = ownCards.find((x) => x.id === openId);
    return c ? { card: c, visiting: false } : null;
  });
  let stamps = $state<Record<string, string>>({});
  const stampOf = (c: Postcard, visiting: boolean) => (visiting ? (c.stamp ?? null) : (stamps[c.id] ?? stampFor(scene.system, c.id)));

  let kernel = $state("");
  $effect(() => {
    void ctx.session.kernelVersion().then((v) => (kernel = v));
  });
  const packs = $derived(ctx.session.stack().base.map((p) => ({ id: p.id, version: p.version })));

  function go(c: Postcard) {
    window.dispatchEvent(new CustomEvent(GO_BOOKMARK_EVENT, { detail: c.bookmark }));
  }
  function keep(c: Postcard) {
    if (c.world.how !== "clip") return;
    window.dispatchEvent(new CustomEvent(PASTE_CLIP_EVENT, { detail: JSON.stringify(c.world.clip) }));
  }
  const reachable = (c: Postcard) => c.bookmark.scope === scene.system;
  const newer = (c: Postcard) => (c.world.how !== "here" && c.world.made_with && kernel && c.world.made_with !== kernel ? kernel : null);
</script>

<section class="wall" data-postcard-wall aria-label="Postcards">
  {#if opened}
    <div class="opened">
      <button type="button" class="back" data-postcard-close onclick={() => (openId = null)}>← All postcards</button>
      {#key opened.card.id}
        <Card
          {ctx}
          {scene}
          card={opened.card}
          visiting={opened.visiting}
          stamp={stampOf(opened.card, opened.visiting)}
          reachable={reachable(opened.card)}
          newer={newer(opened.card)}
          {packs}
          ongo={() => go(opened.card)}
          {...opened.visiting && opened.card.world.how === "clip" ? { onkeep: () => keep(opened.card) } : {}}
          onstamp={(uri) => {
            if (opened.visiting) return;
            stamps = { ...stamps, [opened.card.id]: uri };
            keepStamp(scene.system, opened.card.id, uri);
          }}
          onpack={() => packForSending(ctx, scene, opened.card.bookmark, { packs, kernel, credit: [], asTable: !!ws.shared })}
        />
      {/key}
    </div>
  {:else}
    <h2>Postcards</h2>
    <p class="lede">A postcard is a bookmark that travels: the place, the date and the exact camera. Every screen draws its own picture from it, and the back says why it looks that way.</p>
    <h3>This campaign</h3>
    {#if ownCards.length}
      <ul class="row" data-postcard-row="campaign">
        {#each ownCards as c (c.id)}
          {@const s = stampOf(c, false)}
          <li><button type="button" class="tile" data-postcard-tile={c.id} onclick={() => (openId = c.id)}>{#if s}<img src={s} alt="" />{:else}<span class="blank"></span>{/if}<span class="name">{c.name}</span></button></li>
        {/each}
      </ul>
    {:else}
      <p class="quiet" data-postcard-none>None yet. On a world's globe, Take photograph then Keep as postcard; or open a bookmark with a camera and make it one.</p>
    {/if}
    {#if visits.list.length}
      <h3>Opened here</h3>
      <ul class="row" data-postcard-row="visits">
        {#each visits.list as c (c.id)}
          <li><button type="button" class="tile" data-postcard-tile={c.id} onclick={() => (openId = `visit:${c.id}`)}>{#if c.stamp}<img src={c.stamp} alt="" />{:else}<span class="blank"></span>{/if}<span class="name">{c.name}</span></button></li>
        {/each}
      </ul>
    {/if}
    <p class="quiet">Paste a postcard, open a postcard link or a .ssepostcard.json file, and it opens here as a visit: read-only, never changing this campaign.</p>
  {/if}
</section>

<style>
  .wall {
    padding: 12px 16px;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  h2 {
    margin: 0 0 4px;
    font-family: var(--sse-font-display, var(--sse-font-body));
  }
  h3 {
    margin: 16px 0 6px;
    font-size: 14px;
  }
  .lede {
    max-width: 60ch;
    margin: 0;
    font-size: 13px;
  }
  .quiet {
    opacity: 0.7;
    font-size: 12px;
  }
  .row {
    list-style: none;
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin: 0;
    padding: 0;
  }
  .tile {
    display: grid;
    gap: 4px;
    width: 160px;
    padding: 0 0 6px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
    color: inherit;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    overflow: hidden;
    transition: transform 120ms ease-out;
  }
  .tile:hover {
    transform: translateY(-2px);
  }
  .tile img,
  .blank {
    display: block;
    width: 100%;
    aspect-ratio: 1 / 1;
    object-fit: cover;
    background: #000;
  }
  .name {
    padding: 0 6px;
    text-align: left;
  }
  .back {
    font: inherit;
    font-size: 13px;
    margin-bottom: 8px;
    padding: 4px 10px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  @media (prefers-reduced-motion: reduce) {
    .tile {
      transition: none;
    }
  }
</style>
