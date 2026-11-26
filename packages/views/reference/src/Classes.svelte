<!--
  The Classes page of the Reference screen (V3 gaps item 35, under "Names and classes"): the class gallery. Every
  class of every class atlas on the stack, scheme by scheme (each scheme answers one question: what kind of object,
  what its surface is like, what its air is, what people need to live there), as tiles drawn with the mark the charts
  give it under this screen's symbology, so a tile always looks like the chart. A class opens to its bands in words
  with their units, what it needs beyond numbers, and the named worlds the atlas is checked against (and the near
  misses that are not in it). A class is a label read off a world's results: it never decides anything about a world,
  and the page says so. When no pack on the stack carries classes, the shipped atlases are shown, stamped as not in
  this campaign, and the Build desk offers to add one. The still and turning pictures from a sample world per class
  follow when the kernel picks the sample (class_sample).
-->
<script lang="ts">
  import { tick, untrack } from "svelte";
  import { classBandWords, className, classTree, classesOnStack, findClass, findClasses, needsWords, stackPacks, type AtlasClass, type ClassAtlas, type Depth, type PackDoc, type SymbologyHost, type ViewSession, type Workspace } from "../../../ui-contract/src/index.ts";
  import { HatchDefs, Mark, Stamp } from "../../../chart-grammar/src/index.ts";
  import { drawClassPictures, pictureOf } from "./class-picture.svelte.ts";

  let { session, workspace, depth, symbology, offer = [], pick = null }: { session: ViewSession; workspace: Workspace; depth: Depth; symbology?: SymbologyHost; offer?: readonly string[]; pick?: string | null } = $props();
  const full = $derived(depth !== "curious");
  const engine = $derived(depth === "engine");

  const info = $derived.by(() => {
    void workspace.revision;
    return session.stack();
  });
  const onStack = $derived(classesOnStack([...info.base, ...(info.campaign ? [info.campaign] : [])]));

  // No classes on the stack: the shipped atlases, read from the screen's addresses, shown until one is added.
  let offered = $state.raw<PackDoc[]>([]);
  let offerFailed = $state<string | null>(null);
  $effect(() => {
    if (onStack.schemes.length || offered.length) return;
    const where = [...offer];
    if (!where.length) return void (offerFailed = "this screen offers no class atlas");
    untrack(() => {
      void Promise.all(where.map((a) => fetch(new URL(a, document.baseURI)).then((r) => (r.ok ? (r.json() as Promise<PackDoc>) : Promise.reject(new Error(`${a}: HTTP ${r.status}`))))))
        .then((ps) => (offered = ps))
        .catch((e: Error) => (offerFailed = e.message));
    });
  });
  const trying = $derived(!onStack.schemes.length && offered.length > 0);
  // Each offered atlas apart, so SSE4's own and SSE 3's classic list are never mixed into one scheme.
  const atlases = $derived<{ pack: string; label: string; atlas: ClassAtlas }[]>(
    trying ? offered.map((p) => ({ pack: p.id, label: atlasLabel(p), atlas: classesOnStack([p]) })) : onStack.schemes.length ? [{ pack: "stack", label: "This campaign", atlas: onStack }] : [],
  );
  function atlasLabel(p: PackDoc): string {
    return p.id === "sse.base-atlas" ? "SSE4 class atlas" : p.id === "sse.sse3-classic" ? "SSE 3 classic" : typeof p.name === "string" ? p.name : p.id;
  }
  let atlasChosen = $state<string | null>(null);
  const shownAtlas = $derived(atlases.find((a) => a.pack === atlasChosen) ?? atlases[0] ?? null);
  const atlas = $derived(shownAtlas?.atlas ?? null);

  let schemeChosen = $state<string | null>(null);
  const scheme = $derived(atlas?.schemes.find((s) => s.id === schemeChosen) ?? atlas?.schemes[0] ?? null);
  const tree = $derived(scheme ? classTree(scheme) : { base: [], modifiers: [] });

  let query = $state("");
  const hits = $derived(atlas ? findClasses(atlas, query) : []);

  let openId = $state<string | null>(null);
  const open = $derived(atlas && openId ? findClass(atlas, openId) : null);
  const parentOf = (c: AtlasClass) => (atlas && c.parent ? findClass(atlas, c.parent) : null);
  const kidsOf = (c: AtlasClass) => scheme?.classes.filter((x) => x.parent === c.id) ?? [];
  let detail: HTMLElement | null = $state(null);
  let opener: HTMLElement | null = null;
  async function show(c: AtlasClass, from: HTMLElement | null) {
    opener = from;
    if (atlas && c.scheme !== scheme?.id) schemeChosen = c.scheme;
    openId = c.id;
    query = "";
    await tick();
    detail?.focus({ preventScroll: false });
  }
  async function close() {
    openId = null;
    await tick();
    opener?.focus();
  }
  // Opened from elsewhere (find on the Reference screen, a body card): the class by id.
  $effect(() => {
    const p = pick;
    if (!p || !atlas) return;
    untrack(() => {
      const c = findClass(atlas!, p) ?? atlases.map((a) => findClass(a.atlas, p)).find((x) => x) ?? null;
      if (!c) return;
      const home = atlases.find((a) => findClass(a.atlas, p));
      if (home) atlasChosen = home.pack;
      void show(c, null);
    });
  });

  // The tile's mark: what the charts draw for a world of this class under this screen's symbology.
  const marker = $derived(symbology?.marker() ?? null);
  // Class looks are bound to planets (a small body on a chart is drawn by the same tables), so every tile asks as one.
  const markOf = (c: AtlasClass) => marker?.mark({ kind: "planet", classes: [c.id], tags: [] }) ?? null;

  // Pictures (V14i): each class drawn as a turning True globe from a sample world of it, the page's tiles in order and a
  // class opened first. The shipped atlases' globes come pre-drawn (class-picture.svelte.ts); others are drawn live on
  // the game master's seat. The chart mark stands until a globe is drawn, and wherever none can be (a small body, a
  // thick air, an edited atlas on a player's seat).
  const pictureAtlas = $derived(shownAtlas?.pack ?? "");
  const pictureExtra = $derived(trying ? offered.filter((p) => p.id === shownAtlas?.pack) : []);
  const pictureDocs = $derived<PackDoc[]>(trying ? offered : [...info.base, ...(info.campaign ? [info.campaign] : [])]);
  const docOf = (cls: string): PackDoc | null => {
    const from = atlas ? findClass(atlas, cls)?.pack : undefined;
    return pictureDocs.find((p) => p.id === from) ?? null;
  };
  // The gallery a page at a time (Alex, 24 Nov: 12, 24 or 48 at once, paged).
  const PAGE_SIZES = [12, 24, 48] as const;
  let pageSize = $state<number>(12);
  let pageAt = $state(0);
  const pages = $derived(Math.max(1, Math.ceil(tree.base.length / pageSize)));
  const pageNow = $derived(Math.min(pageAt, pages - 1));
  const paged = $derived(tree.base.slice(pageNow * pageSize, (pageNow + 1) * pageSize));
  $effect(() => {
    void scheme?.id;
    void atlas;
    untrack(() => (pageAt = 0));
  });
  $effect(() => {
    const a = pictureAtlas;
    const ids = paged.map((n) => n.cls.id);
    const extra = pictureExtra;
    if (a && ids.length) untrack(() => drawClassPictures(session, a, ids, extra, { docOf, page: true }));
  });
  $effect(() => {
    const c = open?.id;
    const a = pictureAtlas;
    const extra = pictureExtra;
    if (a && c) untrack(() => drawClassPictures(session, a, [c], extra, { docOf, first: true }));
  });
  // A class opened from search or a link shows its page.
  $effect(() => {
    const id = openId;
    if (!id) return;
    const i = untrack(() => tree.base.findIndex((n) => n.cls.id === id));
    if (i >= 0) untrack(() => (pageAt = Math.floor(i / pageSize)));
  });

  const editable = $derived(workspace.mode === "edit" && session.seat.authority === "gm");
  let note = $state<string | null>(null);
  let adding = $state(false);
  async function addAtlas() {
    const p = offered.find((x) => x.id === shownAtlas?.pack);
    if (!p) return;
    adding = true;
    const r = await stackPacks(session, workspace, [...info.added, p]);
    adding = false;
    note = r.status === "applied" ? `${atlasLabel(p)} is in this campaign. Worlds now carry its classes as labels; remove it on the Packs screen.` : r.status === "refused" ? r.reason : "Not added.";
  }
</script>

<svelte:window
  onkeydowncapture={(e) => {
    if (open && e.key === "Escape") {
      e.stopPropagation();
      e.preventDefault();
      void close();
    }
  }}
/>

<div class="classes" data-classes={trying ? "offered" : atlas ? "stack" : offerFailed ? "none" : "loading"} data-atlas={shownAtlas?.pack ?? ""}>
  <section class="side" aria-labelledby="classes-schemes">
    <h2 id="classes-schemes">Classes</h2>
    {#if trying}
      <p class="note" data-classes-offer>No pack in this campaign carries classes yet. These are the shipped atlases, shown so you can see what each class means.</p>
      {#if editable}
        <button type="button" class="btn ink" data-classes-add onclick={addAtlas} disabled={adding}>Add {shownAtlas?.label ?? "this atlas"}</button>
      {:else}
        <p class="muted small">Choose the Build desk to add one.</p>
      {/if}
    {:else if !atlas}
      <p class="note">{offerFailed ? `No pack in this campaign carries classes, and the shipped atlases could not be read (${offerFailed}).` : "Reading the class atlases…"}</p>
    {/if}
    {#if note}<p class="note" role="status">{note}</p>{/if}
    {#if atlases.length > 1}
      <div class="atlases" role="group" aria-label="Atlas">
        {#each atlases as a (a.pack)}
          <button type="button" class="chip" aria-pressed={shownAtlas?.pack === a.pack} data-atlas-pick={a.pack} onclick={() => ((atlasChosen = a.pack), (schemeChosen = null), (openId = null))}>{a.label}</button>
        {/each}
      </div>
    {/if}
    {#if atlas}
      <ul class="schemes" aria-label="Schemes">
        {#each atlas.schemes as s (s.id)}
          <li>
            <button type="button" class="scheme" aria-pressed={scheme?.id === s.id} data-scheme={s.id} onclick={() => ((schemeChosen = s.id), (openId = null))}>
              <span class="label">{s.question || s.id}</span>
              <span class="k">{s.classes.length} classes{full ? ` · ${s.id}` : ""}</span>
            </button>
          </li>
        {/each}
      </ul>
      <label class="find">
        <span class="k">Find a class</span>
        <input type="search" bind:value={query} placeholder="ocean, Jupiter, rings…" aria-label="Find a class" data-classes-find />
      </label>
      {#if query.trim()}
        <ul class="hits" aria-label="Classes found" data-classes-hits={hits.length}>
          {#each hits.slice(0, 12) as c (c.id)}
            <li><button type="button" class="hit" data-hit={c.id} onclick={(e) => show(c, e.currentTarget)}>{className(c)}<span class="muted small">{` · ${c.scheme}`}</span></button></li>
          {:else}
            <li class="muted small">No class mentions “{query.trim()}”.</li>
          {/each}
        </ul>
      {/if}
      <p class="muted small">A class is a label read off a world's results. It never decides anything about the world; change the world and its class follows.</p>
    {/if}
  </section>

  {#if atlas && scheme}
    <section class="gallery" aria-labelledby="classes-gallery" data-gallery={scheme.id}>
      <h2 id="classes-gallery">{scheme.question || scheme.id}</h2>
      <svg width="0" height="0" class="defs" aria-hidden="true"><HatchDefs prefix="cl" /></svg>
      <ul class="tiles" aria-label="Classes in {scheme.id}">
        {#each paged as n (n.cls.id)}
          {@const m = markOf(n.cls)}
          {@const pic = pictureOf(pictureAtlas, n.cls.id)}
          <li class:under={n.depth > 0} style:--depth={n.depth}>
            <button type="button" class="tile" aria-pressed={openId === n.cls.id} data-class={n.cls.id} data-picture-state={pic?.state ?? "pending"} data-picture-why={pic?.state === "none" ? pic.why : undefined} onclick={(e) => show(n.cls, e.currentTarget)}>
              {#if pic?.state === "drawn"}
                <span class="turn" style:--n={pic.frames} style:--w="64px" style:background-image="url({pic.sheet})" data-tile-picture data-baked={pic.baked} data-caption={pic.caption} aria-hidden="true"></span>
              {:else}
                <svg class="icon" width="84" height="40" viewBox="0 0 84 40" aria-hidden="true" data-drawing={pic?.state === "drawing" || undefined}>
                  <!-- The mark centred, with room on its right for its class code (up to three letters). -->
                  {#if m}<Mark mark={{ ...m, size: Math.max(9, Math.min(14, m.size * 1.4)) }} cx={42} cy={20} prefix="cl" />{:else}<circle cx="42" cy="20" r="11" fill="none" stroke="currentColor" />{/if}
                </svg>
              {/if}
              <span class="tname">{className(n.cls)}</span>
              {#if n.depth > 0 && parentOf(n.cls)}<span class="muted small">a kind of {parentOf(n.cls)!.names.plain}</span>{/if}
              {#if n.cls.evidence === "fictional"}<Stamp kind="fictional" />{/if}
            </button>
          </li>
        {/each}
      </ul>
      {#if tree.base.length > PAGE_SIZES[0] || session.seat.authority === "gm"}
        <nav class="pager" aria-label="Pages of classes" data-pager>
          <button type="button" class="chip" disabled={pageNow === 0} onclick={() => (pageAt = pageNow - 1)} data-page-prev>‹ Earlier</button>
          <span class="muted small" data-page-shown>{pageNow * pageSize + 1} to {Math.min(tree.base.length, (pageNow + 1) * pageSize)} of {tree.base.length}</span>
          <button type="button" class="chip" disabled={pageNow >= pages - 1} onclick={() => (pageAt = pageNow + 1)} data-page-next>Later ›</button>
          <label class="muted small">At once
            <select bind:value={pageSize} onchange={() => (pageAt = 0)} data-page-size>
              {#each PAGE_SIZES as n (n)}<option value={n}>{n}</option>{/each}
            </select>
          </label>
          {#if session.seat.authority === "gm" && session.classSample}
            <!-- The page's globes drawn again on this device with today's renderer, past the pre-drawn set (to see a new look at once). -->
            <button type="button" class="chip" onclick={() => drawClassPictures(session, pictureAtlas, paged.map((n) => n.cls.id), pictureExtra, { docOf, page: true, live: true })} data-redraw title="Draw this page's globes again here, with this build's renderer">Draw again</button>
          {/if}
        </nav>
      {/if}
      {#if tree.modifiers.length}
        <h3>Marks that add to a class</h3>
        <ul class="mods" aria-label="Modifiers in {scheme.id}">
          {#each tree.modifiers as c (c.id)}
            <li><button type="button" class="chip" aria-pressed={openId === c.id} data-class={c.id} onclick={(e) => show(c, e.currentTarget)}>{className(c)}</button></li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}

  {#if open && atlas}
    {@const m = markOf(open)}
    {@const parent = parentOf(open)}
    {@const kids = kidsOf(open)}
    {@const ins = open.examples.filter((e) => e.in)}
    {@const outs = open.examples.filter((e) => !e.in)}
    {@const pic = pictureOf(pictureAtlas, open.id)}
    <section class="card" aria-labelledby="class-title" data-class-card={open.id} tabindex="-1" bind:this={detail}>
      <div class="head">
        <svg class="icon big" width="84" height="64" viewBox="0 0 84 64" aria-hidden="true">
          {#if m}<Mark mark={{ ...m, size: 18 }} cx={32} cy={28} prefix="cl" />{:else}<circle cx="32" cy="28" r="16" fill="none" stroke="currentColor" />{/if}
        </svg>
        <div>
          <h2 id="class-title">{className(open)}</h2>
          {#if full && open.names.physics !== open.names.plain}<p class="muted small">In physics: {open.names.physics}</p>{/if}
          {#if open.names.story}<p class="muted small">In a story: {open.names.story}</p>{/if}
        </div>
        <button type="button" class="close" onclick={close} aria-label="Close {className(open)}">×</button>
      </div>
      {#if open.summary}<p class="summary">{open.summary}</p>{/if}
      {#if open.evidence === "fictional"}<p class="note"><Stamp kind="fictional" /> Only a story makes this: no physics produces it on its own.</p>{/if}
      {#if parent}<p class="small">A kind of <button type="button" class="link" data-parent={parent.id} onclick={(e) => show(parent, e.currentTarget)}>{parent.names.plain}</button>{open.kind === "modifier" ? "; it adds to a class, never replaces one" : ""}.</p>{:else if open.kind === "modifier"}<p class="small">A mark that adds to a world's class, never replaces it.</p>{/if}

      <h3>A world is {open.names.plain} when</h3>
      <ul class="bands" data-bands={open.region.length}>
        {#each open.region as b, i (i)}
          {@const ax = atlas.axes.get(b.axis)}
          <li>{classBandWords(b, atlas.axes, full)}{#if engine}<code class="muted">{` ${ax?.field ?? b.axis}`}</code>{/if}{#if full && ax?.provisional}<span class="muted small"> (provisional axis)</span>{/if}</li>
        {/each}
        {#each open.needs as n (n)}<li>and there is {needsWords(n)}</li>{/each}
      </ul>
      {#if full && open.gate.length}
        <p class="muted small">Only for worlds with {open.gate.map((b) => classBandWords(b, atlas!.axes, false)).join(" and ")}.</p>
      {/if}
      {#if full}<p class="muted small">Inside the bands it is {open.names.plain} for certain; past them it fades out, so a world near an edge can be two things at once.</p>{/if}

      {#if ins.length}
        <h3>Worlds like this</h3>
        <ul class="worlds" data-examples={ins.length}>{#each ins as e (e.name)}<li>{e.name}</li>{/each}</ul>
      {/if}
      {#if full && outs.length}
        <h3>Near misses</h3>
        <ul class="worlds misses" data-misses={outs.length}>{#each outs as e (e.name)}<li>{e.name.replace(/^illustrative: /, "")}{#if e.name.startsWith("illustrative: ")}<span class="muted small"> (made up to test the edge)</span>{/if}</li>{/each}</ul>
      {/if}
      {#if kids.length}
        <h3>Kinds of {open.names.plain}</h3>
        <ul class="mods">{#each kids as k (k.id)}<li><button type="button" class="chip" data-kid={k.id} onclick={(e) => show(k, e.currentTarget)}>{className(k)}</button></li>{/each}</ul>
      {/if}
      {#if pic?.state === "drawn"}
        <figure class="picture" data-picture="drawn"><span class="turn big" role="img" aria-label="A sample world of the class {open.names.plain}, turning" style:--n={pic.frames} style:--w="{pic.size}px" style:background-image="url({pic.sheet})"></span><figcaption class="muted small">{pic.caption}{pic.baked ? " Drawn ahead of time for this atlas." : ""}</figcaption></figure>
      {:else if pic?.state === "drawing"}
        <p class="muted small picture" data-picture="drawing">Drawing a sample world of this class…</p>
      {:else if pic?.state === "none"}
        <p class="muted small picture" data-picture="none">No picture: {pic.why}.</p>
      {:else}
        <p class="muted small picture" data-picture="pending">Its picture is drawn from a sample world of this class on the game master's seat.</p>
      {/if}
      {#if engine}<p class="muted small"><code>{open.id}</code> · scheme <code>{open.scheme}</code> · from {open.pack}</p>{/if}
    </section>
  {/if}
</div>

<style>
  .classes {
    display: grid;
    grid-template-columns: minmax(0, 250px) minmax(0, 1fr) minmax(0, 340px);
    gap: 20px;
    align-items: start;
  }
  .side,
  .gallery,
  .card {
    min-width: 0;
  }
  .defs {
    position: absolute;
  }
  h2 {
    margin: 0 0 8px;
    font-family: var(--sse-font-display);
    font-size: 17px;
    font-weight: 600;
  }
  h3 {
    margin: 14px 0 6px;
    font-size: 13px;
    font-weight: 600;
  }
  p {
    margin: 0 0 10px;
    line-height: 1.5;
  }
  .note {
    color: var(--sse-ink);
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .small {
    font-size: 13px;
  }
  .k {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .schemes {
    display: grid;
    gap: 4px;
    margin-bottom: 12px;
  }
  .scheme,
  .hit {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    width: 100%;
    height: auto;
    padding: 6px 10px;
    border: var(--sse-line) solid transparent;
    background: none;
    color: var(--sse-ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .hit {
    flex-direction: row;
    padding: 4px 8px;
  }
  .scheme:hover,
  .hit:hover {
    border-color: var(--sse-rule);
  }
  .scheme[aria-pressed="true"] {
    border-color: var(--sse-ink);
    background: var(--sse-ground-raised);
  }
  .find {
    display: grid;
    gap: 4px;
    margin-bottom: 8px;
  }
  .find input {
    font: inherit;
    padding: 6px 8px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
    color: var(--sse-ink);
    min-width: 0;
  }
  .hits {
    margin-bottom: 10px;
  }
  .atlases {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin: 0 0 10px;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 8px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    width: 100%;
    height: 100%;
    padding: 10px 8px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: inherit;
    text-align: center;
    cursor: pointer;
    transition:
      border-color 120ms ease,
      transform 120ms ease;
  }
  .tile:hover {
    border-color: var(--sse-ink);
    transform: translateY(-1px);
  }
  .tile[aria-pressed="true"] {
    border-color: var(--sse-ink);
    background: var(--sse-ground-raised);
  }
  .under .tile {
    border-style: dashed;
  }
  .icon {
    color: var(--sse-ink);
    flex: none;
  }
  .tname {
    font-family: var(--sse-font-display);
    font-size: 15px;
    overflow-wrap: anywhere;
  }
  .mods,
  .worlds {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .worlds li {
    padding: 2px 8px;
    border: var(--sse-line) solid var(--sse-rule);
    font-size: 13px;
  }
  .misses li {
    border-style: dashed;
    color: var(--sse-ink-muted);
  }
  .chip {
    height: auto;
    padding: 4px 10px;
    border: var(--sse-line) solid var(--sse-rule);
    background: none;
    color: var(--sse-ink);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .chip[aria-pressed="true"] {
    border-color: var(--sse-ink);
    background: var(--sse-ground-raised);
  }
  .card {
    position: sticky;
    top: 0;
    padding: 12px 14px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    outline: none;
    animation: rise 160ms ease-out;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .card {
      animation: none;
    }
    .tile {
      transition: none;
    }
  }
  .head {
    display: flex;
    gap: 10px;
    align-items: flex-start;
  }
  .head > div {
    flex: 1;
    min-width: 0;
  }
  .close {
    border: none;
    background: none;
    color: var(--sse-ink);
    font-size: 20px;
    line-height: 1;
    cursor: pointer;
    height: auto;
    padding: 2px 6px;
  }
  .summary {
    font-size: 15px;
  }
  .bands {
    display: grid;
    gap: 4px;
  }
  .bands li::before {
    content: "· ";
    color: var(--sse-ink-muted);
  }
  .link {
    border: none;
    background: none;
    padding: 0;
    color: var(--sse-ink);
    font: inherit;
    text-decoration: underline;
    cursor: pointer;
    height: auto;
  }
  .picture {
    margin-top: 14px;
  }
  figure.picture {
    margin: 14px 0 0;
    display: grid;
    gap: 6px;
    justify-items: start;
  }
  /* A turning globe: one turn as a strip of frames, stepped through on the compositor; no GPU or kernel of its own.
     Asked for by Alex (24 Nov), so it loops; reduced motion holds it still on its first frame. */
  .turn {
    display: block;
    flex: none;
    width: var(--w);
    height: var(--w);
    border-radius: 50%;
    background-repeat: no-repeat;
    background-size: calc(var(--w) * var(--n)) var(--w);
    background-position: 0 0;
    animation: turn 24s steps(var(--n)) infinite;
  }
  @keyframes turn {
    to {
      background-position: calc(var(--w) * var(--n) * -1) 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .turn {
      animation: none;
    }
  }
  .pager {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin: 10px 0 0;
  }
  .btn {
    height: auto;
    padding: 6px 12px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    cursor: pointer;
    margin-bottom: 10px;
  }
  .btn.ink {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  /* A narrower desk: the card goes under the gallery. */
  @media (max-width: 1100px) {
    .classes {
      grid-template-columns: minmax(0, 230px) minmax(0, 1fr);
    }
    .card {
      grid-column: 1 / -1;
      position: static;
    }
  }
  /* A phone: one column, the schemes as a sideways strip, the card first when open. */
  @media (max-width: 720px) {
    .classes {
      grid-template-columns: minmax(0, 1fr);
    }
    .schemes {
      grid-auto-flow: column;
      grid-auto-columns: minmax(160px, max-content);
      overflow-x: auto;
    }
    .tiles {
      grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    }
    .card {
      order: -1;
    }
  }
</style>
