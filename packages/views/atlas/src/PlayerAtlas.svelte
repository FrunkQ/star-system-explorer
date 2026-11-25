<!--
  The Atlas on a players' screen (track AT, AT15 part 2; atlas design 11A.4): the same book, built from the scene the
  table was sent, so a page holds only what the seat knows. There is no Edition switch and no "Seen by" (14A note 6):
  the edition comes with the layout (a kit names it), and the audience is the seat's own. The page follows what the
  screen has selected (`page: "selected"`), and steps, links and contents move the selection, so a chart beside it
  follows the book. Controls are the part's own (AT13), drawn in the kit's control set when the role wears a kit.
  The look is the screen's: the kit's look and palette are already on the page, so this view names no look.
-->
<script lang="ts">
  import { tick } from "svelte";
  import { SHIPPED_NOTE_SETS, controlOn, drawingOf, withLetterhead, withMargin, withVoice, buildPage, atlasContents, resolveEdition, DEFAULT_EDITION, type ContentsLine, type PlayerContext, type PlayerScene, type AtlasPage as Page } from "../../../ui-contract/src/index.ts";
  import AtlasPage from "./AtlasPage.svelte";
  import AtlasPrint from "./AtlasPrint.svelte";
  import AtlasDevice from "./AtlasDevice.svelte";
  import KitControl from "../../../chart-grammar/src/KitControl.svelte";

  let { ctx, scene }: { ctx: PlayerContext; scene: PlayerScene } = $props();

  const edition = $derived.by(() => {
    try {
      return resolveEdition(typeof ctx.params.edition === "string" ? ctx.params.edition : DEFAULT_EDITION);
    } catch {
      return resolveEdition(DEFAULT_EDITION);
    }
  });
  const depth = "curious" as const;
  const vocab = $derived(ctx.vocab);
  const lines = $derived<ContentsLine[]>(atlasContents(scene));
  const order = $derived(lines.map((l) => l.address));
  const titleOf = (a: string | null) => lines.find((l) => l.address === a)?.title ?? "";
  const place = $derived(titleOf(lines[0]?.address ?? null));

  // The page in front: the screen's selection when the layout says so, else the one stepped to, else the top.
  const follows = $derived(ctx.params.page !== "contents");
  let asked = $state<string | null>(null);
  // The screen's selection is state behind the context, so reading it here follows it.
  const selected = $derived(ctx.selection());
  const address = $derived.by(() => {
    const has = (a: string | null) => !!a && lines.some((l) => l.address === a);
    if (follows && has(selected)) return selected!;
    if (has(asked)) return asked!;
    return lines[0]?.address ?? null;
  });

  const clearance = $derived(scene.audience === "everyone" ? "THE TABLE" : scene.audience.toUpperCase());
  const footer = $derived(`${edition.name} · ${place}`);
  const dressed = (a: string): Page | null => {
    const p = buildPage(scene, a, edition, { vocab, depth });
    if (!p) return null;
    const voiced = withVoice(p, scene, edition, { vocab, depth });
    return withMargin(withLetterhead(voiced, edition, { clearance, date: "", place, pages: lines.length }), scene, edition, SHIPPED_NOTE_SETS, { vocab, depth, order });
  };
  const page = $derived(address ? dressed(address) : null);

  const on = (id: string) => controlOn(ctx.controls, id);
  const kitSet = $derived(ctx.control_set);
  const showsKeys = $derived(!!kitSet && drawingOf(kitSet.id, "button", [kitSet]).hotkey === true);

  let folded = $state(false);
  let pageBox = $state<HTMLElement | null>(null);
  function go(a: string | null) {
    if (!a) return;
    asked = a;
    folded = false;
    if (ctx.selection() !== a) ctx.select(a);
    queueMicrotask(() => {
      pageBox?.scrollTo?.({ top: 0 });
      pageBox?.querySelector<HTMLElement>("h1")?.focus({ preventScroll: true });
    });
  }

  // Print this page, as the screen shows it.
  let printing = $state<Page[] | null>(null);
  let said = $state<string | null>(null);
  const paper: "A4" | "Letter" = typeof navigator !== "undefined" && /^(en-US|en-CA|es-MX|fr-CA)$/i.test(navigator.language) ? "Letter" : "A4";
  async function printThisPage() {
    said = null;
    if (!page) return;
    printing = [page];
    await tick();
    try {
      if (typeof window.print !== "function") throw new Error("this browser has no print");
      window.print();
    } catch (e) {
      said = `Printing could not start: ${(e as Error).message}. Try your browser's own Print.`;
    }
    printing = null;
  }

  let root = $state<HTMLElement | null>(null);
  $effect(() => {
    const el = root;
    if (!el) return;
    el.addEventListener("keydown", onkey);
    return () => el.removeEventListener("keydown", onkey);
  });
  function onkey(e: KeyboardEvent) {
    const t = e.target as HTMLElement | null;
    if (t && (t.closest("input, textarea, select, [contenteditable]") || e.altKey || e.ctrlKey || e.metaKey)) return;
    if (showsKeys && e.key.length === 1) {
      const k = e.key.toUpperCase();
      if (k === "P" && on("print") && page) return void (e.preventDefault(), printThisPage());
      if (k === "C" && on("contents")) return void (e.preventDefault(), (folded = !folded));
      if (k === "N" && on("step") && page?.next) return void (e.preventDefault(), go(page.next));
      if (k === "B" && on("step") && page?.prev) return void (e.preventDefault(), go(page.prev));
    }
    if (!on("step")) return;
    if (e.key === "ArrowRight" && page?.next) (e.preventDefault(), go(page.next));
    else if (e.key === "ArrowLeft" && page?.prev) (e.preventDefault(), go(page.prev));
  }
</script>

{#snippet sheet()}
  <AtlasPage page={page!} onlink={on("links") ? go : () => {}} {footer} />
{/snippet}

<div class="patlas" bind:this={root} role="region" aria-label="Atlas" data-player-atlas={edition.id} data-kit-set={kitSet?.id}>
  {#if on("print") || on("contents") || on("step")}
    <div class="bar">
      {#if on("step")}
        <KitControl set={kitSet} kind="step" words="Back" hotkey="B" disabled={!page?.prev} onclick={() => go(page?.prev ?? null)} label="Previous page{page?.prev ? `: ${titleOf(page.prev)}` : ''}" />
        <KitControl set={kitSet} kind="step" words="Next" hotkey="N" disabled={!page?.next} onclick={() => go(page?.next ?? null)} label="Next page{page?.next ? `: ${titleOf(page.next)}` : ''}" />
      {/if}
      <span class="gapfill"></span>
      {#if on("contents")}<KitControl set={kitSet} kind="switch" words="Contents" hotkey="C" on={folded} onclick={() => (folded = !folded)} />{/if}
      {#if on("print")}<KitControl set={kitSet} words="Print" hotkey="P" disabled={!page} onclick={printThisPage} label="Print this page" />{/if}
    </div>
  {/if}
  {#if said}<p class="said" role="status">{said}</p>{/if}
  {#if folded && on("contents")}
    <nav class="contents" aria-label="Contents">
      <ol>
        {#each lines as l (l.address)}
          <li style="--level: {l.level}">
            <button type="button" aria-current={l.address === address ? "page" : undefined} onclick={() => go(l.address)}><span>{l.title}</span><span class="n">{l.number}</span></button>
          </li>
        {/each}
      </ol>
    </nav>
  {/if}
  <div class="pagebox" bind:this={pageBox} role="presentation">
    {#if page}
      {#if edition.device}
        <AtlasDevice
          device={edition.device}
          coverKey={edition.cover?.big ?? "Cover"}
          onhome={() => go(lines[0]?.address ?? null)}
          onback={() => go(page?.prev ?? null)}
          oncover={() => go(lines[0]?.address ?? null)}
          onnext={() => go(page?.next ?? null)}
          onprint={printThisPage}
          canBack={!!page.prev && on("step")}
          canNext={!!page.next && on("step")}
          children={sheet}
        />
      {:else}
        {@render sheet()}
      {/if}
    {:else}
      <p class="empty">Nothing here is known to the table yet.</p>
    {/if}
  </div>
</div>

{#if printing}<AtlasPrint pages={printing} look="" bw={false} {footer} {paper} />{/if}

<style>
  .patlas {
    container: atlas / inline-size;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 10px 14px 18px;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    box-sizing: border-box;
    min-width: 0;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .gapfill {
    margin-right: auto;
  }
  .said {
    margin: 0;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .contents ol {
    list-style: none;
    margin: 0;
    padding: 6px 0;
    border-block: 1px solid var(--sse-rule, var(--sse-ink-faint));
    max-height: 40vh;
    overflow: auto;
  }
  .contents button {
    font: inherit;
    width: 100%;
    display: flex;
    justify-content: space-between;
    gap: 8px;
    min-height: 32px;
    padding: 4px 8px 4px calc(8px + var(--level, 0) * 14px);
    background: transparent;
    color: var(--sse-ink);
    border: 0;
    text-align: left;
    cursor: pointer;
  }
  @media (pointer: coarse) {
    .contents button {
      min-height: 44px;
    }
  }
  .contents button[aria-current="page"] {
    color: var(--sse-accent-ink, var(--sse-accent));
    font-weight: 600;
  }
  .contents button:focus-visible {
    outline: 2px solid var(--sse-focus, var(--sse-accent));
    outline-offset: -2px;
  }
  .contents .n {
    color: var(--sse-ink-muted);
    font-family: var(--sse-font-mono);
  }
  .pagebox {
    min-width: 0;
  }
  .empty {
    color: var(--sse-ink-muted);
  }
</style>
