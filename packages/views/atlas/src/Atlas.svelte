<!--
  The Atlas (track AT, AT2; atlas design section 4): a book of the place, read a page at a time. Contents down the
  left (built from the audience's own knowledge: a world they have not found has no line), the page, and step by
  previous and next, the arrow keys or a swipe; a paper edition curls its pages (AT15a), the Guide redraws in bands. Seen by (GM only) is the existing "See as" under its Atlas name: the
  page is built from that audience's scene, so it can only hold what they know. On a narrow box the contents fold into
  a "Contents" button.
-->
<script lang="ts">
  import { tick, untrack } from "svelte";
  import { EVERYONE, BUILTIN_LOGOS, resolveDelivery, SHIPPED_NOTE_SETS, pickBanners, controlOn, withLetterhead, withMargin, withVoice, withSources, firstSentence, cardFieldsOf, showReference, type HowKnown, type FactRow, atlasHtml, chapterOf, pageText, buildPage, atlasContents, resolveEdition, DEFAULT_EDITION, SHIPPED_EDITIONS, type ContentsLine, type PlayerScene, type ViewContext, type SystemScene } from "../../../ui-contract/src/index.ts";
  import type { AtlasHost } from "./index.svelte.ts";
  import Segmented from "../../../chart-grammar/src/Segmented.svelte";
  import AtlasPage from "./AtlasPage.svelte";
  import AtlasPrint from "./AtlasPrint.svelte";
  import AtlasPrintView from "./AtlasPrintView.svelte";
  import AtlasDevice from "./AtlasDevice.svelte";
  import AtlasCover from "./AtlasCover.svelte";
  import Explain from "../../body-card/src/Explain.svelte";
  import { curlTurn, pageTurn, playDelivery } from "../../../chart-grammar/src/index.ts";
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
  // Reading order: margin notes are picked page by page in it, so a joke is used once in the book (14A note 8).
  const order = $derived(lines.map((l) => l.address));
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
    const margined = withMargin(withLetterhead(voiced, edition, letterhead), book, edition, noteSets, { vocab, depth: ws.depth, exclude: turnedAway, order });
    return withSources(margined, edition, howOf, { depth: ws.depth });
  };

  // ---- Every number keeps its Why (AT8; design section 8). How a figure is known starts with the audience: for the
  // GM, where the value came from; for the table, the rung it was given at (the kernel's `known_to`). The reasoning is
  // the kernel's own explanation, its lens at the reader's depth and its first reference: the method, never an input,
  // so a seat's note names only what its page already says.
  const gmFact = (a: string, field: string) => {
    const facts = scene.subjects.find((s) => s.address === a)?.facts ?? [];
    for (const c of cardFieldsOf(field)) {
      const f = facts.find((x) => x.field === c || x.kernel?.field === c);
      if (f) return f;
    }
    return undefined;
  };
  let rungs = $state<Record<string, string>>({});
  let reasons = $state<Record<string, { reason?: string; cite?: string } | null>>({});
  const reasonKey = (a: string, field: string) => `${ws.depth}|${a}|${field}`;
  const FROM: Record<string, string> = { authored: "Set by the GM", catalogued: "From a catalogue", derived: "Worked out from the physics", resolved: "Resolved for this universe", fixture: "From a sample" };
  const givenTo = $derived(shownAudience === EVERYONE ? "the table" : audienceWords);
  function howOf(a: string, field: string): HowKnown | undefined {
    const f = gmFact(a, field);
    const rung = rungs[`${shownAudience}|${a}`];
    const source = shownAudience === "gm" ? (f?.why ? FROM.derived! : (FROM[f?.source ?? "resolved"] ?? FROM.resolved!)) : rung ? `${rung}, given to ${givenTo}` : `Given to ${givenTo}`;
    const r = reasons[reasonKey(a, field)];
    return { source, ...(r?.reason ? { reason: r.reason } : {}), ...(r?.cite ? { cite: r.cite } : {}) };
  }
  // Read what the notes need for some pages: each audience's rung once a page, each explanation once a figure.
  function learn(want: readonly string[]) {
    if (!book || edition.sources === "off") return;
    const aud = shownAudience;
    for (const a of want) {
      const bare = buildPage(book, a, edition, { vocab, depth: ws.depth });
      if (!bare) continue;
      if (aud !== "gm" && !(`${aud}|${a}` in rungs)) {
        rungs[`${aud}|${a}`] = "";
        ctx.session.knownTo(a).then((r) => {
          const mine = r.ok ? r.value.audiences.find((x) => x.audience === aud) : undefined;
          if (mine?.rung_label) rungs[`${aud}|${a}`] = mine.rung_label;
        }, () => {});
      }
      for (const b of bare.blocks) {
        if (b.type !== "facts") continue;
        for (const row of b.rows) {
          const k = reasonKey(a, row.field);
          const why = row.figure ? gmFact(a, row.field)?.why : undefined;
          if (!why || k in reasons) continue;
          reasons[k] = null;
          const depth: "curious" | "science" | "engine" = ws.depth ?? "science";
          ctx.session.explain(why, why.kind).then((r) => {
            if (!r.ok) return;
            const lens = r.explanation.lenses[depth] ?? r.explanation.lenses.science ?? r.explanation.lenses.curious;
            const ref = r.explanation.references[0];
            const cite = ref ? showReference(ref).text : undefined;
            reasons[k] = { ...(lens ? { reason: firstSentence(lens) } : {}), ...(cite ? { cite } : {}) };
          }, () => {});
        }
      }
    }
  }
  // On screen, a figure opens its Why: how it is known, then (for the GM) the kernel's whole chain, as on a card.
  let whyOpen = $state<string | null>(null);
  const isGmSeat = $derived((ctx.session as { seat?: { authority?: string } }).seat?.authority === "gm");
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
  $effect(() => {
    void edition;
    void ws.depth;
    const a = address;
    if (a && book) untrack(() => learn([a]));
  });
  $effect(() => {
    void address;
    whyOpen = null;
  });
  // The part's own controls (AT13): a screen may switch each off; off means not drawn and not pressable.
  const on = (id: string) => controlOn((ctx as { controls?: Readonly<Record<string, boolean>> }).controls, id);
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

  // ---- Print under the view (AT4; 14A note 7): "Print this page" prints at once; "Print…" opens the print view of
  // every page as a small sheet with a tick box, Ink (as shown, or black and white) and Paper.
  let viewOpen = $state(false);
  let ink = $state<"shown" | "bw">("shown");
  // Letter where the device's language says North America; A4 everywhere else. Changeable.
  let paper = $state<"A4" | "Letter">(typeof navigator !== "undefined" && /^(en-US|en-CA|es-MX|fr-CA)$/i.test(navigator.language) ? "Letter" : "A4");
  let printing = $state<{ pages: import("../../../ui-contract/src/index.ts").AtlasPage[]; look: string; bw: boolean } | null>(null);
  let said = $state<string | null>(null);
  // A dark look prints its paper form when ink is saved; black and white is the toner form on a white ground.
  const dark = $derived(host.paperCss(edition.look, edition.print.save_ink_look) !== look);
  const bwLook = $derived(host.tonerCss?.() ?? host.paperCss(edition.look, edition.print.save_ink_look));
  const chapterNow = $derived(book && address ? chapterOf(lines, address).map((l) => l.address) : []);

  const pagesAt = (want: readonly string[]) => (book ? want.map(dressed).filter((p) => p !== null) : []);

  async function print(want: readonly string[], how: "shown" | "bw" = ink) {
    said = null;
    viewOpen = false;
    const pages = pagesAt(want);
    if (!pages.length) return void (said = "There is no page to print yet.");
    printing = { pages, look: how === "bw" ? bwLook : look, bw: how === "bw" };
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
    if (!started && !said) said = "Printing did not start. This browser may block printing here; try your browser's own Print, or Save as a web page.";
    printing = null;
  }
  // The one-press shortcut: this page, as shown on screen for a light edition, its paper form for a dark one.
  const printThisPage = () => address && print([address], dark ? "bw" : "shown");

  async function copyText(want: readonly string[], markdown: boolean) {
    said = null;
    viewOpen = false;
    const text = pagesAt(want).map((p) => pageText(p, { markdown, footer })).join(markdown ? "\n---\n\n" : "\n\n");
    try {
      await navigator.clipboard.writeText(text);
      said = markdown ? "Copied as Markdown." : "Copied as text.";
    } catch {
      said = "The browser would not let the Atlas copy. Use Save as a web page instead.";
    }
  }

  function savePage(want: readonly string[]) {
    said = null;
    viewOpen = false;
    const pages = pagesAt(want);
    if (!pages.length) return;
    const html = atlasHtml(pages, { title: `${edition.name}: ${pages[0]!.title}`, tokens: ink === "bw" ? bwLook : look, footer });
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

  // ---- Turning a page (AT15a; 14A note 5): the edition's look says how. Paper curls from the bottom outer corner,
  // the old page peeling over the new one; the Guide redraws in bands; a look with no delivery just changes.
  const delivery = $derived(host.lookDelivery?.(edition.look));
  const reducedMotion = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  const sheetTurns = $derived(resolveDelivery(delivery, "turn").kind === "page_turn");
  let turning = false;
  const articleNow = () => pageBox?.querySelector<HTMLElement>("article[data-atlas-page]") ?? null;
  function turn(a: string | null, dir: 1 | -1) {
    if (!a) return;
    const art = articleNow();
    if (!art || turning || covered) return go(a);
    const plan = resolveDelivery(delivery, "turn", { back: dir < 0, reducedMotion: reducedMotion() });
    if (plan.kind === "page_turn") {
      turning = true;
      curlTurn(art, dir, () => (go(a), tick()), { ms: plan.ms }).done.finally(() => (turning = false));
    } else {
      go(a);
      if (plan.kind !== "none") tick().then(() => articleNow() && playDelivery(articleNow()!, plan));
    }
  }
  // A drag by hand on a paper page carries the curl; it steps as the drag becomes a turn, and steps back if let go short.
  // On other looks a swipe (finger, pen or mouse) steps when let go, with the look's Turn; vertical drags still scroll.
  function handTurn(dir: 1 | -1) {
    if (!on("step") || covered) return;
    const to = dir > 0 ? (page?.next ?? null) : (page?.prev ?? null);
    if (sheetTurns) go(to);
    else turn(to, dir);
  }

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
    if (!on("step")) return;
    if (e.key === "ArrowRight" && page?.next) (e.preventDefault(), turn(page.next, 1));
    else if (e.key === "ArrowLeft" && page?.prev) (e.preventDefault(), turn(page.prev, -1));
  }

</script>

{#snippet whyBody(r: FactRow)}
  {@const h = page ? howOf(page.address, r.field) : undefined}
  {@const f = page ? gmFact(page.address, r.field) : undefined}
  <div class="why" role="note" data-atlas-why={r.field}>
    {#if h}<p class="how"><span class="k">How we know</span> {h.source}.{#if h.reason} {h.reason}{/if}{#if h.cite && ws.depth !== "curious"} <cite>{h.cite}</cite>{/if}</p>{/if}
    {#if isGmSeat && f?.why}
      <Explain session={ctx.session} depth={ws.depth} why={f.why} label={r.label} onclose={() => (whyOpen = null)} />
    {:else}
      <button type="button" class="link" onclick={() => (whyOpen = null)}>Close</button>
    {/if}
  </div>
{/snippet}

{#snippet stepper(where: string)}
  {#if on("step")}<div class="step {where}">
    <button type="button" disabled={!page?.prev} onclick={() => turn(page?.prev ?? null, -1)} aria-label="Previous page{page?.prev ? `: ${titleOf(page.prev)}` : ''}">‹ {titleOf(page?.prev ?? null) || "Back"}</button>
    <button type="button" disabled={!page?.next} onclick={() => turn(page?.next ?? null, 1)} aria-label="Next page{page?.next ? `: ${titleOf(page.next)}` : ''}">{titleOf(page?.next ?? null) || "Next"} ›</button>
  </div>{/if}
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
    {#if on("print")}<button type="button" class="printnow" onclick={printThisPage} disabled={!page}>Print this page</button>
    <button type="button" class="printmore" aria-haspopup="dialog" onclick={() => (learn(lines.map((l) => l.address)), (viewOpen = true))} disabled={!page}>Print…</button>{/if}
    {#if on("contents")}<button type="button" class="fold" aria-expanded={folded} onclick={() => (folded = !folded)}>Contents</button>{/if}
  </div>
  {#if said}<p class="said" role="status">{said}</p>{/if}
  <div class="spread">
    {#if on("contents") || on("step")}<nav class="contents" class:open={folded} aria-label="Contents">
      {#if on("contents")}
      <p class="book">{titleOf(lines[0]?.address ?? null)}</p>
      <ol>
        {#each lines as l (l.address)}
          <li style="--level: {l.level}">
            <button type="button" aria-current={l.address === address ? "page" : undefined} onclick={() => go(l.address)}>
              <span>{l.title}</span><span class="n">{l.number}</span>
            </button>
          </li>
        {/each}
      </ol>{/if}
      {@render stepper("side")}
      <p class="note">{lines.length} {lines.length === 1 ? "page" : "pages"}, as known to {audienceWords}.</p>
    </nav>{/if}
    <div class="pagebox" bind:this={pageBox} role="presentation">
      {#if page}
        <div class="look" style={look} data-atlas-edition={edition.id} data-atlas-audience={book?.audience}>
          {#snippet screen()}
            {#if covered && edition.cover}
              <AtlasCover cover={edition.cover} place={titleOf(lines[0]?.address ?? null)} onopen={openCover} />
            {:else}
              <div class="sheet" use:pageTurn={{ sheet: sheetTurns && on("step"), onstep: handTurn }} data-turn={sheetTurns ? "curl" : (delivery?.turn?.kind ?? "none")}>
                <AtlasPage page={page!} onlink={on("links") ? go : () => {}} {footer} onturnaway={shownAudience === "gm" ? notThisOne : undefined} onwhy={(f) => (whyOpen = whyOpen === f ? null : f)} {whyOpen} {whyBody} />
              </div>
            {/if}
          {/snippet}
          {#if edition.device}
            <AtlasDevice
              device={edition.device}
              coverKey={edition.cover?.big ?? "Cover"}
              onhome={() => ((covered = false), go(lines[0]?.address ?? null))}
              onback={() => ((covered = false), turn(page?.prev ?? null, -1))}
              oncover={() => (covered = true)}
              onnext={() => ((covered = false), turn(page?.next ?? null, 1))}
              onprint={printThisPage}
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

{#if printing}<AtlasPrint pages={printing.pages} look={printing.look} bw={printing.bw} {footer} {paper} />{/if}
{#if viewOpen && book}
  <AtlasPrintView
    {lines}
    pageOf={dressed}
    current={address}
    chapter={chapterNow}
    title={titleOf(lines[0]?.address ?? null)}
    subtitle="{edition.name}, as known to {audienceWords}, {date}"
    {look}
    {bwLook}
    {dark}
    {footer}
    bind:paper
    bind:ink
    onprint={(want) => print(want)}
    oncopy={copyText}
    onsave={savePage}
    onclose={() => (viewOpen = false)}
  />
{/if}

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
  .printnow,
  .printmore {
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
  .printmore:not(:disabled):active {
    transform: translateY(1px);
  }
  .printnow:focus-visible,
  .printmore:focus-visible {
    outline: 2px solid var(--sse-focus, var(--sse-accent));
    outline-offset: 2px;
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
  }
  .why {
    padding: 10px 12px;
    border: 1px solid var(--sse-rule);
    border-radius: 4px;
    background: var(--sse-ground-raised, var(--sse-ground));
  }
  .why .link {
    font: inherit;
    background: none;
    border: 0;
    padding: 0;
    color: var(--sse-accent);
    cursor: pointer;
  }
</style>
