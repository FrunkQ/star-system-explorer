<!--
  The star class gallery (SC1, Alex 26 Nov): every star class as a tile, each drawn live in True by the renderer's own
  star face (churning cells sized by the kernel's convection, spots, flares), a whole page at once through one GPU
  canvas laid over the tiles. "Sizes" switches between every star filling its tile alike and each at its actual size
  against the largest on the page. A tile opens to a large view with what the kernel says of it: its class, where it is
  in its life, and its cells (how many across, how big, how long they live, and whether this size shows them). Also
  the star picker: with `onpick`, the card offers the star for a new system.
-->
<script lang="ts">
  import { tick, untrack } from "svelte";
  import { sayNumber, type Depth } from "../../../ui-contract/src/index.ts";
  import { cellsAcross, granulesShow, limbLumps, narrateLumps, startGpu, type Gpu } from "../../../renderer/src/index.ts";
  import { readGalleryStar, radiiOnPage, shareWords, span, type GalleryStar, type SizeMode, type StarTile } from "./star-gallery.ts";
  import { drawStage, type StageStar } from "./star-stage.ts";
  import StarLife from "./StarLife.svelte";

  let {
    session,
    tiles,
    depth = "science",
    onpick,
    pickWords = "Use this star",
    stamp = "",
  }: {
    session: Parameters<typeof readGalleryStar>[0];
    tiles: readonly StarTile[];
    depth?: Depth;
    onpick?: (g: GalleryStar) => void;
    pickWords?: string;
    /** A line under the heading (the stand-in list says it is one). */
    stamp?: string;
  } = $props();
  const full = $derived(depth !== "curious");

  // Two ways to look: every kind side by side, or one star through its life (Alex 26 Nov: "order by 'same star'").
  let view = $state<"kinds" | "life">("kinds");
  let lifeMass = $state(1);
  function follow(g: GalleryStar) {
    lifeMass = g.ask.mass_msun;
    openId = null;
    view = "life";
  }

  // Sizes, remembered on this device (aids are per device).
  const SIZE_KEY = "sse.star-gallery.size";
  let size = $state<SizeMode>(
    (() => {
      try {
        return localStorage.getItem(SIZE_KEY) === "actual" ? "actual" : "same";
      } catch {
        return "same";
      }
    })(),
  );
  function setSize(m: SizeMode) {
    size = m;
    try {
      localStorage.setItem(SIZE_KEY, m);
    } catch {}
  }

  // Families as chips: all, or one.
  const families = $derived([...new Set(tiles.map((t) => t.family))]);
  let family = $state<string | null>(null);
  const shown = $derived(family ? tiles.filter((t) => t.family === family) : tiles);

  // A page at a time, 12, 24 or 48 (as the world gallery).
  const PAGE_SIZES = [12, 24, 48] as const;
  let pageSize = $state<number>(24);
  let pageAt = $state(0);
  const pages = $derived(Math.max(1, Math.ceil(shown.length / pageSize)));
  const pageNow = $derived(Math.min(pageAt, pages - 1));
  const paged = $derived(shown.slice(pageNow * pageSize, (pageNow + 1) * pageSize));

  // Each tile's star, read once (the life line, then its activity), the page's in order.
  let read = $state(new Map<string, GalleryStar | string>());
  let reading = false;
  $effect(() => {
    const want = paged.filter((t) => !untrack(() => read.has(t.id)));
    if (!want.length) return;
    untrack(() => void readAll(want));
  });
  async function readAll(want: readonly StarTile[]) {
    if (reading) return;
    reading = true;
    try {
      for (const t of want) {
        if (read.has(t.id)) continue;
        const g = await readGalleryStar(session, t).catch((e: Error) => e.message);
        read = new Map(read).set(t.id, g);
      }
    } finally {
      reading = false;
    }
    // Asked for more while reading (a page turned): read those too.
    const more = paged.filter((t) => !read.has(t.id));
    if (more.length) void readAll(more);
  }
  const starOf = (id: string): GalleryStar | null => {
    const g = read.get(id);
    return g && typeof g !== "string" ? g : null;
  };

  // The tile's picture square and the disc's share of it.
  const SKY = 132;
  const FULL_R = SKY * 0.44;
  const radii = $derived(radiiOnPage(paged.map((t) => starOf(t.id)?.desc.now.radius_m ?? 0), FULL_R, size));
  const biggest = $derived(Math.max(0, ...paged.map((t) => starOf(t.id)?.desc.now.radius_rsun ?? 0)));

  // ---- The stage: one GPU canvas over the page ----
  let host: HTMLElement | null = $state(null);
  let canvas: HTMLCanvasElement | null = $state(null);
  let gpu: Gpu | null = null;
  let gpuState = $state<"starting" | "ready" | "none">("starting");
  $effect(() => {
    const c = canvas;
    if (!c) return;
    let gone = false;
    void startGpu(c).then((g) => {
      if (gone) return g?.dispose();
      if (!g) return void (gpuState = "none");
      gpu = g;
      gpuState = "ready";
    });
    return () => {
      gone = true;
      gpu?.dispose();
      gpu = null;
      gpuState = "starting";
    };
  });
  const calm = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  let box = $state({ w: 0, h: 0 });
  let drawnCount = $state(0);
  $effect(() => {
    if (gpuState !== "ready" || !host || !canvas) return;
    const h = host;
    const cv = canvas;
    const ro = new ResizeObserver(() => (box = { w: h.clientWidth, h: h.clientHeight }));
    ro.observe(h);
    let raf = 0;
    const t0 = performance.now();
    const frame = (now: number) => {
      raf = requestAnimationFrame(frame);
      if (!gpu || document.hidden) return;
      const dpr = Math.min(2, typeof devicePixelRatio === "number" ? devicePixelRatio : 1);
      const at = h.getBoundingClientRect();
      const stars: StageStar[] = [];
      for (const slot of h.querySelectorAll<HTMLElement>("[data-sky]")) {
        const id = slot.dataset.sky!;
        const g = starOf(id);
        const r = Number(slot.dataset.r);
        if (!g?.live || !(r > 0)) continue;
        const s = slot.getBoundingClientRect();
        stars.push({ key: id, live: g.live, x: s.left - at.left + s.width / 2, y: s.top - at.top + s.height / 2, r, clock: g.clock });
      }
      const w = Math.max(1, Math.round(at.width));
      const hh = Math.max(1, Math.round(at.height));
      if (cv.width !== Math.round(w * dpr) || cv.height !== Math.round(hh * dpr)) ((cv.width = Math.round(w * dpr)), (cv.height = Math.round(hh * dpr)));
      drawStage(gpu, { width: w, height: hh, dpr, ground: "black", t_s: (now - t0) / 1000, calm, stars });
      if (stars.length !== drawnCount) drawnCount = stars.length;
    };
    raf = requestAnimationFrame(frame);
    return () => (cancelAnimationFrame(raf), ro.disconnect());
  });

  // ---- The card: one star large ----
  let openId = $state<string | null>(null);
  const open = $derived(openId ? (tiles.find((t) => t.id === openId) ?? null) : null);
  const openStar = $derived(open ? starOf(open.id) : null);
  const openWhy = $derived(open ? (read.get(open.id) as string | GalleryStar | undefined) : undefined);
  let card: HTMLElement | null = $state(null);
  let opener: HTMLElement | null = null;
  async function show(t: StarTile, from: HTMLElement | null) {
    opener = from;
    openId = t.id;
    if (!read.has(t.id)) void readAll([t]);
    await tick();
    card?.focus();
  }
  async function close() {
    openId = null;
    await tick();
    opener?.focus();
  }
  const BIG = 300;
  // Close up: the disc drawn two or four times larger, its centre moved down and left so its upper right limb stays in
  // view (a giant's cells and its lumpy edge at a size that shows them).
  const ZOOMS = [1, 2] as const;
  let zoom = $state<number>(1);
  $effect(() => {
    void openId;
    untrack(() => (zoom = 1));
  });
  let bigCanvas: HTMLCanvasElement | null = $state(null);
  $effect(() => {
    const c = bigCanvas;
    const g = openStar;
    const z = zoom;
    if (!c || !g?.live) return;
    void z;
    let gone = false;
    let raf = 0;
    let big: Gpu | null = null;
    void startGpu(c).then((x) => {
      if (gone) return x?.dispose();
      big = x;
      const t0 = performance.now();
      const frame = (now: number) => {
        raf = requestAnimationFrame(frame);
        if (!big || document.hidden) return;
        const dpr = Math.min(2, typeof devicePixelRatio === "number" ? devicePixelRatio : 1);
        if (c.width !== BIG * dpr) c.width = c.height = BIG * dpr;
        const r = BIG * 0.46 * zoom;
        const off = zoom > 1 ? r * 0.62 : 0;
        drawStage(big, { width: BIG, height: BIG, dpr, ground: "black", t_s: (now - t0) / 1000, calm, stars: [{ key: `${g.tile.id}|big`, live: g.live!, x: BIG / 2 - off, y: BIG / 2 + off, r, clock: g.clock }] });
      };
      raf = requestAnimationFrame(frame);
    });
    return () => {
      gone = true;
      cancelAnimationFrame(raf);
      big?.dispose();
    };
  });

  /** The cells in words: how many, how big, how long they live, and whether a disc this many pixels across shows them. */
  function cellWords(g: GalleryStar, disc_px: number): string[] {
    if (!g.cells) return [g.desc.now.teff_k >= 6900 ? "No convection cells: its surface is hot enough (above about 6,900 K) that heat leaves by radiation alone." : "The kernel gives it no surface convection."];
    const c = g.cells;
    const out = [`About ${sayNumber(Math.round(c.across), 3)} cells across its face, each about ${sayNumber(c.size_m / 1000, 2)} km wide (ten of the photosphere's pressure scale heights, ${sayNumber(c.scale_height_m / 1000, 2)} km, from its gravity and heat), each living about ${span(c.life_s)}.`];
    out.push(granulesShow(c.across, disc_px) ? `At this size each cell is ${sayNumber(disc_px / c.across, 2)} pixels wide, so they show.` : `Too small to see at this size: each would be ${sayNumber(disc_px / c.across, 2)} pixels wide, and a cell is drawn from 6.`);
    if (c.big) {
      const b = c.big;
      const wide = b.size_m >= 2 * g.desc.now.radius_m;
      out.push(`Under them, larger cells: the kernel's next scale up is ${sayNumber(b.size_m / 1000, 2)} km${wide ? ", wider than the star itself, so only a few" : `, about ${sayNumber(Math.round(b.count), 2)} over the star`}, each living about ${span(b.life_s)}, about ${sayNumber(b.dt * 100, 2)}% hotter or cooler than the surface around them (from how fast they flow).${granulesShow(cellsAcross(Math.max(4, b.count)), disc_px) ? "" : " Too many to see at this size."}`);
    }
    const lumps = narrateLumps(limbLumps({ pressure_scale_height_m: c.scale_height_m }, g.desc.now.radius_m), disc_px);
    if (lumps) out.push(lumps);
    return out;
  }
  const lifeWords = (g: GalleryStar) => {
    const yr = g.ask.age_yr;
    const age = yr >= 1e9 ? `${sayNumber(yr / 1e9, 3)} billion` : yr >= 1e6 ? `${sayNumber(yr / 1e6, 3)} million` : sayNumber(yr, 3);
    return `Born with ${sayNumber(g.ask.mass_msun, 3)} times the Sun's mass, ${age} years ago${g.ask.feh !== undefined ? `, [Fe/H] ${sayNumber(g.ask.feh, 2, { decimals: 2 })}` : ""}: now ${g.desc.now.label}.`;
  };
  const codeOf = (g: GalleryStar | null) => ((g?.desc.now as { class?: { code?: string } } | undefined)?.class?.code ?? null);
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

<div class="views chips" role="group" aria-label="View">
  <button type="button" class="chip" aria-pressed={view === "kinds"} data-star-view="kinds" onclick={() => (view = "kinds")}>Kinds</button>
  <button type="button" class="chip" aria-pressed={view === "life"} data-star-view="life" onclick={() => (view = "life")}>One star's life</button>
</div>
{#if view === "life"}
  <StarLife {session} mass={lifeMass} {depth} />
{:else}
<div class="stars" class:opened={!!open} data-star-gallery data-gpu={gpuState} data-size={size} data-drawn={drawnCount}>
  <section class="gallery" aria-label="Star classes">
    {#if stamp}<p class="muted small" data-star-stamp>{stamp}</p>{/if}
    <div class="bar">
      <div class="chips" role="group" aria-label="Kinds">
        <button type="button" class="chip" aria-pressed={family === null} onclick={() => ((family = null), (pageAt = 0))}>All</button>
        {#each families as f (f)}
          <button type="button" class="chip" aria-pressed={family === f} data-family={f} onclick={() => ((family = f), (pageAt = 0))}>{(f.charAt(0).toUpperCase() + f.slice(1)).replace(/-/g, " ")}</button>
        {/each}
      </div>
      <div class="chips" role="group" aria-label="Sizes">
        <button type="button" class="chip" aria-pressed={size === "same"} data-size-pick="same" onclick={() => setSize("same")} title="Every star fills its tile">Same size</button>
        <button type="button" class="chip" aria-pressed={size === "actual"} data-size-pick="actual" onclick={() => setSize("actual")} title="Each star at its real size against the largest on this page">Actual size</button>
      </div>
    </div>
    {#if size === "actual" && biggest > 0}<p class="muted small" data-scale>Actual sizes: the largest on this page is {sayNumber(biggest, 3)} times the Sun's radius and fills its tile; the rest are to the same scale.{family ? "" : " Choose one kind to compare stars of a like size."}</p>{/if}
    {#if gpuState === "none"}<p class="note">This device has no GPU to draw True with, so the stars are not drawn; their facts are still read.</p>{/if}

    <div class="stage" bind:this={host}>
      <ul class="tiles" aria-label="Star classes">
        {#each paged as t, i (t.id)}
          {@const g = read.get(t.id)}
          {@const s = typeof g === "object" ? g : null}
          {@const r = radii[i]}
          <li>
            <button type="button" class="tile" aria-pressed={openId === t.id} data-star-class={t.id} data-star-state={!g ? "reading" : s?.live ? "drawn" : "none"} data-star-why={typeof g === "string" ? g : s && !s.live ? s.why : undefined} onclick={(e) => show(t, e.currentTarget)}>
              <span class="sky" style:width="{SKY}px" style:height="{SKY}px" data-sky={s?.live ? t.id : undefined} data-r={s?.live && r ? r.r_px : undefined}>
                {#if !g}<span class="muted small">Reading…</span>{:else if !s?.live}<span class="not small" data-not-drawn>Not drawn yet</span>{/if}
              </span>
              <span class="tname">{t.name}</span>
              <span class="muted small">{s ? `${codeOf(s) ?? s.desc.now.label}${full ? ` · ${sayNumber(s.desc.now.radius_rsun, 3)} R☉` : ""}` : ""}</span>
              {#if size === "actual" && s?.live && r}<span class="muted small" data-share>{r.point ? `${shareWords(r.share)}: a point at this scale` : shareWords(r.share)}</span>{/if}
            </button>
          </li>
        {/each}
      </ul>
      <canvas bind:this={canvas} class="overlay" style:width="{box.w}px" style:height="{box.h}px" aria-hidden="true"></canvas>
    </div>

    {#if shown.length > PAGE_SIZES[0]}
      <nav class="pager" aria-label="Pages of star classes">
        <button type="button" class="chip" disabled={pageNow === 0} onclick={() => (pageAt = pageNow - 1)}>‹ Earlier</button>
        <span class="muted small">{pageNow * pageSize + 1} to {Math.min(shown.length, (pageNow + 1) * pageSize)} of {shown.length}</span>
        <button type="button" class="chip" disabled={pageNow >= pages - 1} onclick={() => (pageAt = pageNow + 1)}>Later ›</button>
        <label class="muted small">At once
          <select bind:value={pageSize} onchange={() => (pageAt = 0)}>
            {#each PAGE_SIZES as n (n)}<option value={n}>{n}</option>{/each}
          </select>
        </label>
      </nav>
    {/if}
  </section>

  {#if open}
    <section class="card" aria-labelledby="star-class-title" data-star-card={open.id} tabindex="-1" bind:this={card}>
      <div class="head">
        <div>
          <h2 id="star-class-title">{open.name}</h2>
          {#if openStar}<p class="muted small">{codeOf(openStar) ? `${codeOf(openStar)} · ` : ""}{openStar.desc.now.label}</p>{/if}
        </div>
        <button type="button" class="close" onclick={close} aria-label="Close {open.name}">×</button>
      </div>
      {#if openStar?.live}
        <figure class="big">
          <canvas bind:this={bigCanvas} style:width="{BIG}px" style:height="{BIG}px" aria-label="{open.name}, drawn live in True"></canvas>
          <figcaption class="muted small">Drawn live in True from the kernel's numbers, at its own clock: {openStar.clockWhy}.{openStar.without ? ` ${openStar.without}` : ""}</figcaption>
        </figure>
        <div class="chips" role="group" aria-label="Close up">
          {#each ZOOMS as z (z)}<button type="button" class="chip" aria-pressed={zoom === z} data-zoom={z} onclick={() => (zoom = z)}>{z === 1 ? "Whole star" : `× ${z}`}</button>{/each}
        </div>
      {:else if openStar}
        <p class="note" data-not-drawn>Not drawn yet: {openStar.why}.</p>
      {:else if typeof openWhy === "string"}
        <p class="note">No star to draw: {openWhy}.</p>
      {:else}
        <p class="muted small">Reading its star from the life line…</p>
      {/if}
      {#if open.summary}<p>{open.summary}</p>{/if}
      {#if openStar}
        {@const n = openStar.desc.now}
        <p>{lifeWords(openStar)}</p>
        <ul class="facts">
          <li>{sayNumber(n.teff_k, 3)} K at its surface</li>
          <li>{sayNumber(n.radius_rsun, 3)} times the Sun's radius</li>
          <li>{sayNumber(n.luminosity_lsun, 3)} times the Sun's light</li>
          <li>{sayNumber(n.mass_msun, 3)} times the Sun's mass now</li>
        </ul>
        {#if openStar.live}
          <h3>Its surface</h3>
          {#each cellWords(openStar, BIG * 0.92 * zoom * Math.min(2, typeof devicePixelRatio === "number" ? devicePixelRatio : 1)) as w (w)}<p class="small">{w}</p>{/each}
        {/if}
        {#if full && openStar.desc.notes.length}<p class="muted small">{openStar.desc.notes.join(" ")}</p>{/if}
        <div class="acts">
          {#if onpick}<button type="button" class="btn ink" data-star-pick={open.id} onclick={() => onpick?.(openStar!)}>{pickWords}</button>{/if}
          <button type="button" class="chip" data-star-follow={open.id} onclick={() => follow(openStar!)}>Follow its life</button>
        </div>
      {/if}
    </section>
  {/if}
</div>
{/if}

<style>
  .views {
    margin-bottom: 10px;
  }
  .acts {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }
  .stars {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 20px;
    align-items: start;
  }
  .stars.opened {
    grid-template-columns: minmax(0, 1fr) minmax(0, 340px);
    gap: 20px;
    align-items: start;
  }
  @media (max-width: 860px) {
    .stars,
    .stars.opened {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .gallery,
  .card {
    min-width: 0;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 8px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  p {
    margin: 0 0 10px;
    line-height: 1.5;
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .small {
    font-size: 13px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .stage {
    position: relative;
  }
  /* The stars are light added over their black sky: the canvas is black wherever there is no star, so screen leaves the page alone. */
  .overlay {
    position: absolute;
    inset: 0 auto auto 0;
    pointer-events: none;
    mix-blend-mode: screen;
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
    padding: 8px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: inherit;
    text-align: center;
    cursor: pointer;
    transition: border-color 120ms ease;
  }
  .tile:hover,
  .tile[aria-pressed="true"] {
    border-color: var(--sse-ink);
  }
  .sky {
    display: grid;
    place-items: center;
    /* True's space: black whatever the theme, as every True view draws it. */
    background: black;
    flex: none;
    max-width: 100%;
  }
  .not {
    color: gray;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .tname {
    font-family: var(--sse-font-display);
    font-size: 15px;
    overflow-wrap: anywhere;
  }
  .pager {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-top: 10px;
  }
  .card {
    border: var(--sse-line) solid var(--sse-rule);
    padding: 14px;
    background: var(--sse-ground);
  }
  .card:focus {
    outline: 2px solid var(--sse-accent, var(--sse-ink));
  }
  .head {
    display: flex;
    justify-content: space-between;
    gap: 10px;
  }
  h2 {
    margin: 0 0 4px;
    font-family: var(--sse-font-display);
    font-size: 17px;
    font-weight: 600;
  }
  h3 {
    margin: 14px 0 6px;
    font-size: 13px;
    font-weight: 600;
  }
  .close {
    border: none;
    background: none;
    color: var(--sse-ink);
    font-size: 20px;
    cursor: pointer;
  }
  .big {
    margin: 8px 0 12px;
  }
  .big canvas {
    display: block;
    max-width: 100%;
    /* True's space: black whatever the theme, as every True view draws it. */
    background: black;
  }
  .facts {
    display: grid;
    gap: 2px;
    margin-bottom: 10px;
    font-size: 13px;
  }
</style>
