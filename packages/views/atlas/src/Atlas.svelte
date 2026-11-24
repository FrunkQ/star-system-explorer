<!--
  The Atlas (track AT, AT2; atlas design section 4): a book of the place, read a page at a time. Contents down the
  left (built from the audience's own knowledge: a world they have not found has no line), the page, and step by
  previous and next, the arrow keys or a swipe. Seen by (GM only) is the existing "See as" under its Atlas name: the
  page is built from that audience's scene, so it can only hold what they know. On a narrow box the contents fold into
  a "Contents" button.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { EVERYONE, buildPage, atlasContents, resolveEdition, DEFAULT_EDITION, type ContentsLine, type PlayerScene, type Vocab, type ViewContext, type SystemScene } from "../../../ui-contract/src/index.ts";
  import Segmented from "../../../chart-grammar/src/Segmented.svelte";
  import AtlasPage from "./AtlasPage.svelte";

  let { ctx, scene, vocab }: { ctx: ViewContext; scene: SystemScene; vocab: Vocab } = $props();
  const ws = $derived(ctx.workspace);

  // Who the book is seen by: the GM or everyone at the table. Each audience of the seat plan joins this list when the
  // shell hands views the plan's audiences (the table port lists seats, which are not audiences).
  const audiences = [
    { value: "gm", label: "GM" },
    { value: EVERYONE, label: "Everyone" },
  ];
  let audience = $state("gm");
  const audienceWords = $derived(audience === "gm" ? "the GM" : audiences.find((a) => a.value === audience)?.label ?? audience);

  const edition = resolveEdition(DEFAULT_EDITION);

  // The audience's own scene for this system, read again when the record moves or the audience changes.
  let book = $state<PlayerScene | null>(null);
  let failed = $state<string | null>(null);
  let gen = 0;
  $effect(() => {
    void ws.revision;
    const sys = scene.system;
    const aud = audience;
    const at = ws.now_yr;
    const g = ++gen;
    ctx.session.scene(sys, { audience: aud, at_yr: at }).then((r) => {
      if (g !== gen) return;
      book = r.ok ? r.scene : null;
      failed = r.ok ? null : r.message;
    });
  });

  const lines = $derived<ContentsLine[]>(book ? atlasContents(book) : []);
  // The page in front: the one asked for, else the selected object's, else the top of the book.
  let asked = $state<string | null>(null);
  const address = $derived.by(() => {
    const has = (a: string | null) => !!a && lines.some((l) => l.address === a);
    if (has(asked)) return asked!;
    if (has(ws.selection)) return ws.selection!;
    return lines[0]?.address ?? null;
  });
  const page = $derived(book && address ? buildPage(book, address, edition, { vocab, depth: ws.depth }) : null);
  const titleOf = (a: string | null) => lines.find((l) => l.address === a)?.title ?? "";

  let folded = $state(false);
  let pageBox = $state<HTMLElement | null>(null);
  let root = $state<HTMLElement | null>(null);
  // Left and right step the book while focus is anywhere in the Atlas.
  $effect(() => {
    const el = root;
    if (!el) return;
    el.addEventListener("keydown", onkey);
    return () => el.removeEventListener("keydown", onkey);
  });

  function go(a: string | null) {
    if (!a) return;
    asked = a;
    folded = false;
    untrack(() => {
      if (ws.selection !== a) ws.select(a);
    });
    // The new page starts at its top, and keyboard focus follows it.
    queueMicrotask(() => {
      pageBox?.scrollTo?.({ top: 0 });
      pageBox?.querySelector<HTMLElement>("h1")?.focus({ preventScroll: true });
    });
  }

  function onkey(e: KeyboardEvent) {
    const t = e.target as HTMLElement | null;
    if (t && (t.closest("input, textarea, select, [contenteditable]") || e.altKey || e.ctrlKey || e.metaKey)) return;
    if (e.key === "ArrowRight" && page?.next) (e.preventDefault(), go(page.next));
    else if (e.key === "ArrowLeft" && page?.prev) (e.preventDefault(), go(page.prev));
  }

  // A swipe on touch steps the book; a vertical drag still scrolls.
  let downX = 0;
  let downY = 0;
  function pdown(e: PointerEvent) {
    if (e.pointerType !== "touch") return;
    downX = e.clientX;
    downY = e.clientY;
  }
  function pup(e: PointerEvent) {
    if (e.pointerType !== "touch") return;
    const dx = e.clientX - downX;
    const dy = e.clientY - downY;
    if (Math.abs(dx) > 60 && Math.abs(dx) > 2 * Math.abs(dy)) go(dx < 0 ? page?.next ?? null : page?.prev ?? null);
  }
</script>

{#snippet stepper(where: string)}
  <div class="step {where}">
    <button type="button" disabled={!page?.prev} onclick={() => go(page?.prev ?? null)} aria-label="Previous page{page?.prev ? `: ${titleOf(page.prev)}` : ''}">‹ {titleOf(page?.prev ?? null) || "Back"}</button>
    <button type="button" disabled={!page?.next} onclick={() => go(page?.next ?? null)} aria-label="Next page{page?.next ? `: ${titleOf(page.next)}` : ''}">{titleOf(page?.next ?? null) || "Next"} ›</button>
  </div>
{/snippet}

<div class="atlas" bind:this={root} role="region" aria-label="Atlas">
  <div class="bar">
    <span class="what">{edition.name}</span>
    <Segmented label="Seen by" options={audiences} value={audience} onchange={(v) => (audience = v)} slide />
    <button type="button" class="fold" aria-expanded={folded} onclick={() => (folded = !folded)}>Contents</button>
  </div>
  <div class="spread">
    <nav class="contents" class:open={folded} aria-label="Contents">
      <p class="book">{titleOf(lines[0]?.address ?? null)}</p>
      <ol>
        {#each lines as l (l.address)}
          <li style="--level: {l.level}">
            <button type="button" aria-current={l.address === address ? "page" : undefined} onclick={() => go(l.address)}>
              <span>{l.title}</span><span class="n">{l.number}</span>
            </button>
          </li>
        {/each}
      </ol>
      {@render stepper("side")}
      <p class="note">{lines.length} {lines.length === 1 ? "page" : "pages"}, as known to {audienceWords}.</p>
    </nav>
    <div class="pagebox" bind:this={pageBox} onpointerdown={pdown} onpointerup={pup} role="presentation">
      {#if page}
        <AtlasPage {page} onlink={go} footer="{edition.name} · {titleOf(lines[0]?.address ?? null)} · as known to {audienceWords}" />
        {@render stepper("below")}
      {:else if failed}
        <p class="empty">The Atlas could not be read: {failed}</p>
      {:else if book}
        <p class="empty">Nothing here is known to {audienceWords} yet.</p>
      {:else}
        <p class="empty">Opening the Atlas…</p>
      {/if}
    </div>
  </div>
</div>

<style>
  .atlas {
    container: atlas / inline-size;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 14px 18px 24px;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    min-height: 100%;
    box-sizing: border-box;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 14px;
    flex-wrap: wrap;
  }
  .what {
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 17px;
    margin-right: auto;
  }
  .fold {
    display: none;
  }
  .spread {
    display: grid;
    grid-template-columns: 240px minmax(0, 1fr);
    gap: 24px;
    align-items: start;
  }
  .contents {
    position: sticky;
    top: 0;
    background: var(--sse-ground-raised);
    border: 1px solid var(--sse-line);
    border-radius: var(--sse-radius);
    padding: 14px;
  }
  .book {
    margin: 0 0 8px;
    font-size: 12px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li button {
    all: unset;
    box-sizing: border-box;
    display: flex;
    justify-content: space-between;
    width: 100%;
    padding: 6px 8px 6px calc(8px + var(--level) * 14px);
    border-radius: var(--sse-radius);
    cursor: pointer;
    transition: transform var(--sse-motion-quick, 120ms) var(--sse-ease, ease-out);
  }
  li button:hover {
    transform: translateX(1px);
    box-shadow: inset 2px 0 0 var(--sse-accent);
  }
  li button:focus-visible,
  .step button:focus-visible,
  .fold:focus-visible {
    outline: 2px solid var(--sse-focus, var(--sse-accent));
    outline-offset: 1px;
  }
  li button[aria-current="page"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .n {
    color: var(--sse-ink-muted);
    font-size: 13px;
  }
  li button[aria-current="page"] .n {
    color: inherit;
  }
  .step {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }
  .step button,
  .fold {
    flex: 1;
    font: inherit;
    padding: 6px 8px;
    background: var(--sse-ground);
    color: var(--sse-ink);
    border: 1px solid var(--sse-ink);
    border-radius: var(--sse-radius);
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .step.below {
    display: none;
  }
  .step button:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .step button:not(:disabled):active,
  .fold:active {
    transform: translateY(1px);
  }
  .note {
    margin: 10px 0 0;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .pagebox {
    min-width: 0;
    touch-action: pan-y;
  }
  .empty {
    color: var(--sse-ink-muted);
    font-style: italic;
  }
  @container atlas (max-width: 760px) {
    .spread {
      grid-template-columns: minmax(0, 1fr);
    }
    .fold {
      display: inline-block;
      flex: 0 0 auto;
    }
    .contents {
      display: none;
      position: static;
    }
    .contents.open {
      display: block;
    }
    .step.below {
      display: flex;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    li button {
      transition: none;
    }
  }
</style>
