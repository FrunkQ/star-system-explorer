<!--
  The Atlas (track AT, AT2; atlas design section 4): a book of the place, read a page at a time. Contents down the
  left (built from the audience's own knowledge: a world they have not found has no line), the page, and step by
  previous and next, the arrow keys or a swipe. Seen by (GM only) is the existing "See as" under its Atlas name: the
  page is built from that audience's scene, so it can only hold what they know. On a narrow box the contents fold into
  a "Contents" button.
-->
<script lang="ts">
  import { tick, untrack } from "svelte";
  import { EVERYONE, BUILTIN_LOGOS, SHIPPED_NOTE_SETS, pickBanners, withLetterhead, withMargin, withVoice, atlasHtml, chapterOf, pageText, buildPage, atlasContents, resolveEdition, DEFAULT_EDITION, SHIPPED_EDITIONS, type ContentsLine, type PlayerScene, type ViewContext, type SystemScene } from "../../../ui-contract/src/index.ts";
  import type { AtlasHost } from "./index.svelte.ts";
  import Segmented from "../../../chart-grammar/src/Segmented.svelte";
  import AtlasPage from "./AtlasPage.svelte";
  import AtlasPrint from "./AtlasPrint.svelte";
  import AtlasDevice from "./AtlasDevice.svelte";
  import AtlasCover from "./AtlasCover.svelte";
  import { dateParts, dateWords, defaultCalendar, fromPackCalendar } from "../../../chart-grammar/src/calendar.ts";

  let { ctx, scene, host }: { ctx: ViewContext; scene: SystemScene; host: AtlasHost } = $props();
  const ws = $derived(ctx.workspace);

  // Who the book is seen by: the GM or everyone at the table. Each audience of the seat plan joins this list when the
  // shell hands views the plan's audiences (the table port lists seats, which are not audiences).
  const audiences = [
    { value: "gm", label: "GM" },
    { value: EVERYONE, label: "Everyone" },
  ];
  let audience = $state("gm");

  // The edition is the GM's own choice for browsing (design 14A note 6): a player's screen has no Edition switch; an
  // Atlas sent to the table carries its edition with it. Switching re-dresses the same page in place.
  const editions = SHIPPED_EDITIONS.map((d) => ({ value: d.id, label: d.name, hint: d.description }));
  let editionId = $state(DEFAULT_EDITION);
  const edition = $derived(resolveEdition(editionId));
  const vocab = $derived(host.vocabs[edition.vocab] ?? host.vocabs.plain!);
  const look = $derived(host.lookCss(edition.look));

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

  // Words for the audience of the book on screen (the one read, not the one just asked for).
  const shownAudience = $derived(book?.audience ?? audience);
  const audienceWords = $derived(shownAudience === "gm" ? "the GM" : audiences.find((a) => a.value === shownAudience)?.label ?? shownAudience);
  const lines = $derived<ContentsLine[]>(book ? atlasContents(book) : []);
  // The page in front: the one asked for, else the selected object's, else the top of the book.
  let asked = $state<string | null>(null);
  const address = $derived.by(() => {
    const has = (a: string | null) => !!a && lines.some((l) => l.address === a);
    if (has(asked)) return asked!;
    if (has(ws.selection)) return ws.selection!;
    return lines[0]?.address ?? null;
  });
  const titleOf = (a: string | null) => lines.find((l) => l.address === a)?.title ?? "";

  // The date the page speaks for: the ruler's time, in the campaign's calendar. A printed page says it in its footer.
  const calendar = $derived(ws.pack_calendar ? fromPackCalendar(ws.pack_calendar) : defaultCalendar(ws.campaign_epoch_yr ?? ws.campaign_now_yr ?? ws.now_yr));
  const date = $derived(dateWords(ws.now_yr, calendar));
  const footer = $derived(`${edition.name} · ${titleOf(lines[0]?.address ?? null)} · as known to ${audienceWords}, ${date}`);

  // Letterhead (AT6): whose paper the page is on. On a player's screen it is the role's brand (AT9); the GM browsing
  // may try a built-in logo, else the placeholder logo drawn in text from the edition's fallback name.
  let paperOf = $state("");
  const brand = $derived.by(() => {
    const b = BUILTIN_LOGOS.find((l) => l.id === paperOf);
    return b ? { name: b.name, logo: b.logo, ...(b.mono ? { mono: true } : {}) } : host.brand?.(shownAudience);
  });
  const letterhead = $derived({
    ...(brand ? { brand } : {}),
    clearance: shownAudience === "gm" ? "GM EYES ONLY" : audienceWords.toUpperCase(),
    date,
    place: titleOf(lines[0]?.address ?? null),
    est: String(dateParts(ws.campaign_epoch_yr ?? ws.now_yr, calendar).year),
    pages: lines.length,
  });
  // Margin notes (AT7): the edition's sets, picked by page and edition from what the audience knows. "Not this one"
  // turns a note away on this page for this sitting; keeping it with the campaign comes with the GM's notes.
  let turnedAway = $state<string[]>([]);
  const noteSets = $derived([...SHIPPED_NOTE_SETS, ...(host.noteSets ?? [])]);
  const dressed = (a: string) => {
    const p = book ? buildPage(book, a, edition, { vocab, depth: ws.depth }) : null;
    if (!p || !book) return null;
    const voiced = withVoice(p, book, edition, { vocab, depth: ws.depth });
    return withMargin(withLetterhead(voiced, edition, letterhead), book, edition, noteSets, { vocab, depth: ws.depth, exclude: turnedAway });
  };
  const notThisOne = (id: string) => (turnedAway = [...turnedAway, id]);

  // The cover (AT7): once each time an edition with a cover is opened; its key brings it back.
  let covered = $state(false);
  $effect(() => {
    void editionId;
    untrack(() => (covered = !!edition.cover));
  });
  function openCover() {
    covered = false;
    tick().then(() => pageBox?.querySelector<HTMLElement>("h1")?.focus({ preventScroll: true }));
  }
  const page = $derived(book && address ? dressed(address) : null);
  // The one-liners at the top and bottom of a device's screen (SSE3's guide tips, Alex 25 Nov): by the page's use,
  // from the edition's own sets; the cover is a holding screen. Seeded by the edition, so they repeat like the page.
  const SITUATION_OF: Record<string, string> = { sector: "sector", system: "system", star: "system", world: "world", moon: "world", station: "world" };
  const banners = $derived.by(() => {
    const mode = edition.device?.banners;
    if (!mode || !book || !edition.margin) return {};
    const sets = noteSets.filter((x) => edition.margin!.sets.includes(x.id));
    const b = covered || !page ? pickBanners(book, null, "holding", sets, { vocab, depth: ws.depth, seed: edition.id }) : pickBanners(book, page.address, SITUATION_OF[page.kind] ?? "system", sets, { vocab, depth: ws.depth, seed: edition.id, exclude: turnedAway });
    return { top: mode === "bottom" ? undefined : b.top?.text, bottom: mode === "top" ? undefined : b.bottom?.text };
  });

  // ---- Print under the view (AT4): this page, this chapter or the whole Atlas, in the edition showing, in place.
  let printOpen = $state(false);
  let scope = $state<"page" | "chapter" | "atlas">("page");
  // Letter where the device's language says North America; A4 everywhere else. Changeable.
  let paper = $state<"A4" | "Letter">(typeof navigator !== "undefined" && /^(en-US|en-CA|es-MX|fr-CA)$/i.test(navigator.language) ? "Letter" : "A4");
  let saveInk = $state(true);
  let printing = $state<{ pages: import("../../../ui-contract/src/index.ts").AtlasPage[]; look: string } | null>(null);
  let said = $state<string | null>(null);

  function pagesFor(which: "page" | "chapter" | "atlas") {
    if (!book || !address) return [];
    const want = which === "page" ? [address] : which === "chapter" ? chapterOf(lines, address).map((l) => l.address) : lines.map((l) => l.address);
    return want.map(dressed).filter((p) => p !== null);
  }

  async function print(which: "page" | "chapter" | "atlas") {
    said = null;
    printOpen = false;
    const pages = pagesFor(which);
    if (!pages.length) return void (said = "There is no page to print yet.");
    printing = { pages, look: saveInk ? host.paperCss(edition.look, edition.print.save_ink_look) : look };
    await tick();
    let started = false;
    const mark = () => (started = true);
    window.addEventListener("beforeprint", mark, { once: true });
    try {
      if (typeof window.print !== "function") throw new Error("this browser has no print");
      window.print();
    } catch (e) {
      said = `Printing could not start: ${(e as Error).message}. Try your browser's own Print, or Copy as text.`;
    }
    window.removeEventListener("beforeprint", mark);
    if (!started && !said) said = "Printing did not start. This browser may block printing here; try your browser's own Print, or Save page.";
    printing = null;
  }

  async function copyText(markdown: boolean) {
    said = null;
    printOpen = false;
    const text = pagesFor(scope).map((p) => pageText(p, { markdown, footer })).join(markdown ? "\n---\n\n" : "\n\n");
    try {
      await navigator.clipboard.writeText(text);
      said = markdown ? "Copied as Markdown." : "Copied as text.";
    } catch {
      said = "The browser would not let the Atlas copy. Use Save page instead.";
    }
  }

  function savePage() {
    said = null;
    printOpen = false;
    const pages = pagesFor(scope);
    if (!pages.length) return;
    const html = atlasHtml(pages, { title: `${edition.name}: ${pages[0]!.title}`, tokens: saveInk ? host.paperCss(edition.look, edition.print.save_ink_look) : look, footer });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([html], { type: "text/html" }));
    a.download = `${edition.name} - ${pages[0]!.title}.html`.replace(/[\\/:*?"<>|]/g, "-");
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
    said = "Saved as one page you can open anywhere.";
  }

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
    if (covered) {
      if (e.key === "ArrowRight" || e.key === "ArrowLeft") (e.preventDefault(), openCover());
      return;
    }
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
    <Segmented label="Edition" options={editions} value={editionId} onchange={(v) => (editionId = v)} slide />
    <span class="gapfill"></span>
    <Segmented label="Seen by" options={audiences} value={audience} onchange={(v) => (audience = v)} slide />
    {#if edition.letterhead.use !== "none"}
      <label class="paper">Letterhead
        <select bind:value={paperOf}>
          <option value="">{brand && !paperOf ? brand.name : "Drawn in text"}</option>
          {#each BUILTIN_LOGOS as l (l.id)}<option value={l.id}>{l.name}</option>{/each}
        </select>
      </label>
    {/if}
    <button type="button" class="printnow" onclick={() => print("page")} disabled={!page}>Print this page</button>
    <span class="printwrap">
      <button type="button" class="printmore" aria-expanded={printOpen} aria-controls="atlas-print-panel" onclick={() => (printOpen = !printOpen)} disabled={!page}>Print…</button>
      {#if printOpen}
        <div class="panel" id="atlas-print-panel" role="group" aria-label="Print">
          <Segmented label="What to print" options={[{ value: "page", label: "This page" }, { value: "chapter", label: "This chapter" }, { value: "atlas", label: "Whole atlas" }]} value={scope} onchange={(v) => (scope = v)} />
          <Segmented label="Paper" options={[{ value: "A4", label: "A4" }, { value: "Letter", label: "Letter" }]} value={paper} onchange={(v) => (paper = v)} />
          <label class="ink"><input type="checkbox" bind:checked={saveInk} /> Save ink</label>
          <button type="button" class="printnow" onclick={() => print(scope)}>Print</button>
          <p class="others">Or, as {edition.name} words:</p>
          <div class="row2">
            <button type="button" onclick={() => copyText(false)}>Copy as text</button>
            <button type="button" onclick={() => copyText(true)}>Copy as Markdown</button>
            <button type="button" onclick={savePage}>Save page</button>
          </div>
        </div>
      {/if}
    </span>
    <button type="button" class="fold" aria-expanded={folded} onclick={() => (folded = !folded)}>Contents</button>
  </div>
  {#if said}<p class="said" role="status">{said}</p>{/if}
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
        <div class="look" style={look} data-atlas-edition={edition.id} data-atlas-audience={book?.audience}>
          {#snippet screen()}
            {#if covered && edition.cover}
              <AtlasCover cover={edition.cover} place={titleOf(lines[0]?.address ?? null)} onopen={openCover} />
            {:else}
              {#key page!.address}
                <div class="sheet" class:wipe={edition.device?.change === "wipe"}>
                  <AtlasPage page={page!} onlink={go} {footer} onturnaway={shownAudience === "gm" ? notThisOne : undefined} />
                </div>
              {/key}
            {/if}
          {/snippet}
          {#if edition.device}
            <AtlasDevice
              device={edition.device}
              coverKey={edition.cover?.big ?? "Cover"}
              onhome={() => ((covered = false), go(lines[0]?.address ?? null))}
              onback={() => ((covered = false), go(page?.prev ?? null))}
              oncover={() => (covered = true)}
              onnext={() => ((covered = false), go(page?.next ?? null))}
              onprint={() => print("page")}
              canBack={!!page.prev}
              canNext={!!page.next}
              bannerTop={banners.top}
              bannerBottom={banners.bottom}
              children={screen}
            />
          {:else}
            {@render screen()}
          {/if}
        </div>
        {#if !edition.device}{@render stepper("below")}{/if}
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

{#if printing}<AtlasPrint pages={printing.pages} look={printing.look} {footer} {paper} />{/if}

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
  .gapfill {
    margin-right: auto;
  }
  .look {
    background: transparent;
    color: var(--sse-ink);
  }
  .fold {
    display: none;
  }
  .paper {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .paper select {
    font: inherit;
    color: var(--sse-ink);
    background: var(--sse-ground);
    border: 1px solid var(--sse-line);
    border-radius: var(--sse-radius);
    padding: 4px 6px;
  }
  .sheet.wipe {
    animation: wipe 260ms steps(7, end);
  }
  @keyframes wipe {
    from {
      clip-path: inset(0 0 100% 0);
    }
    to {
      clip-path: inset(0 0 0 0);
    }
  }
  .printwrap {
    position: relative;
  }
  .printnow,
  .printmore,
  .row2 button {
    font: inherit;
    padding: 6px 12px;
    border-radius: var(--sse-radius);
    border: 1px solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    cursor: pointer;
  }
  .printnow {
    background: var(--sse-accent);
    border-color: var(--sse-accent);
    color: var(--sse-on-ink);
  }
  .printnow:not(:disabled):active,
  .printmore:not(:disabled):active,
  .row2 button:active {
    transform: translateY(1px);
  }
  .printnow:focus-visible,
  .printmore:focus-visible,
  .row2 button:focus-visible {
    outline: 2px solid var(--sse-focus, var(--sse-accent));
    outline-offset: 2px;
  }
  .panel {
    position: absolute;
    right: 0;
    top: calc(100% + 6px);
    z-index: 5;
    display: grid;
    gap: 10px;
    width: max-content;
    max-width: min(92vw, 380px);
    padding: 14px;
    background: var(--sse-ground-raised);
    border: 1px solid var(--sse-line);
    border-radius: var(--sse-radius);
    box-shadow: var(--sse-card-shadow);
    transform-origin: top right;
    justify-items: start;
    animation: open var(--sse-motion-quick, 120ms) var(--sse-ease, ease-out);
  }
  .panel > .printnow {
    justify-self: stretch;
  }
  @keyframes open {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.98);
    }
  }
  .ink {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .others {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .row2 {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .said {
    margin: 0;
    color: var(--sse-ink-muted);
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
    .panel,
    .sheet.wipe {
      animation: none;
    }
  }
</style>
