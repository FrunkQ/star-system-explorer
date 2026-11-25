<!--
  The screens runtime (H8.3, V3): mounts a checked screen document onto a
  player scene, the same on a player's device, in the composer's preview and
  in a printed report. The document decides everything: which views, where
  (its layout's areas), with which parameters, and what each item shows
  (`bind`). Views that fill a box (a chart) get one; the rest grow with what
  they show. Selection is shared by the views of one screen. Each item's
  `controls` (AT13) reach its view as `ctx.controls`; a part with every
  control off is a picture, made inert so nothing in it is reachable.
  `still` makes every part a picture (a clean output for a stream, PB8).
-->
<script lang="ts">
  import { boundScene, withLive, type CheckedScreen, type Mark, type PlayerContext, type PlayerScene, type PlayerSubject, type Vocab, type ViewHandle, type LandingGround, type LandingShow, type Shown } from "../../../ui-contract/src/index.ts";

  let {
    screen,
    scene,
    vocab,
    mark,
    selection = $bindable(null),
    compact = false,
    live,
    landing,
    ground,
    shown,
    still = false,
    places = {},
    controlSet,
  }: { controlSet?: import("../../../ui-contract/src/index.ts").ControlSet; still?: boolean; places?: Record<string, PlayerScene>; screen: CheckedScreen; scene: PlayerScene; vocab: Vocab; mark?: (s: PlayerSubject) => Mark; selection?: string | null; compact?: boolean; live?: Record<string, boolean>; landing?: () => LandingShow | null; ground?: LandingGround; shown?: () => Shown | null } = $props();

  const handles: { h: ViewHandle<PlayerScene>; m: CheckedScreen["mounts"][number] }[] = [];
  // A box that holds a place (PB13 part 2) draws that place's scene; the rest draw what was sent.
  const sceneFor = (m: CheckedScreen["mounts"][number]) => (m.place ? (places[m.place.scope] ?? null) : scene);
  $effect(() => {
    void scene;
    void places;
    for (const x of handles) {
      const s = sceneFor(x.m);
      if (s) x.h.update(boundScene(s, x.m.bind));
    }
  });
  function view(node: HTMLElement, m: CheckedScreen["mounts"][number]) {
    // A placed box is about its own thing (the bookmark's selection), not the screen's shared selection.
    const own = m.place;
    const ctx: PlayerContext = { params: withLive(m.params, m.module.params ?? [], live), vocab, selection: own ? () => own.focus ?? null : () => selection, select: own ? () => {} : (a) => (selection = a), ...(mark ? { mark } : {}), ...(landing ? { landing: () => landing() } : {}), ...(ground ? { ground } : {}), ...(shown ? { shown: () => shown() } : {}), controls: still ? Object.fromEntries(Object.keys(m.controls).map((k) => [k, false])) : m.controls, ...(controlSet ? { control_set: controlSet } : {}) };
    const h = m.module.mount(node, ctx, boundScene(sceneFor(m)!, m.bind)) as ViewHandle<PlayerScene>;
    const entry = { h, m };
    handles.push(entry);
    return { destroy: () => (handles.splice(handles.indexOf(entry), 1), h.dispose()) };
  }
  const layout = $derived(screen.doc.layout);
</script>

<div class="screen layout-{layout}" class:compact data-screen-id={screen.doc.id} data-layout={layout}>
  {#key JSON.stringify(live ?? {})}
  <!-- Keyed by the screen and the view as well as the area: another screen document (a landing sent to the table) remounts its views. -->
  {#each screen.mounts as m (`${screen.doc.id}:${m.area}:${m.module.id}`)}
    <section class="area area-{m.area}" class:boxed={m.module.box === "fill"} class:solo={screen.mounts.length === 1 && !compact} data-view={m.module.id} data-area={m.area} aria-label="{m.module.id.replace('sse.view.', '')} ({m.area})" inert={still || m.picture} data-picture={still || m.picture || undefined} data-placed={m.place?.scope}>
      {#if m.place?.title}<p class="placed" data-place-title>{m.place.title}</p>{/if}
      {#if sceneFor(m)}<div class="fill" use:view={m}></div>{:else}<p class="placed">Reading {m.place?.title ?? "that place"}…</p>{/if}
    </section>
  {/each}
  {/key}
</div>

<style>
  .area {
    position: relative;
  }
  .boxed .placed {
    position: absolute;
    top: 6px;
    left: 10px;
    z-index: 1;
    pointer-events: none;
  }
  .placed {
    margin: 0 0 4px;
    font-size: 12px;
    letter-spacing: var(--sse-label-tracking);
    text-transform: var(--sse-label-case);
    color: var(--sse-ink-muted);
  }
  .screen {
    display: grid;
    gap: 20px;
    align-items: start;
  }
  .compact {
    gap: 12px;
  }
  .boxed {
    height: 460px;
    overflow: hidden;
    border: var(--sse-frame-width) solid var(--sse-frame-line);
    border-radius: var(--sse-frame-radius);
    box-shadow: var(--sse-frame-shadow);
  }
  .compact .boxed {
    height: 300px;
  }
  /* A screen of one filling view (the Landing screen) takes the page's height, less the bars above and below. */
  .boxed.solo {
    height: max(460px, calc(100dvh - 190px));
  }
  .boxed .fill {
    height: 100%;
  }
  /* grid: a large first area, a side column, then full-width rows. */
  .layout-grid {
    grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr) minmax(0, 1fr);
    grid-template-areas:
      "a a b"
      "a a b"
      "c d b"
      "e e e"
      "f f f";
  }
  .compact.layout-grid {
    grid-template-columns: minmax(0, 1.3fr) minmax(0, 1fr);
    grid-template-areas: "a b" "c b" "d b" "e e" "f f";
  }
  .layout-grid .area-a { grid-area: a; }
  .layout-grid .area-b { grid-area: b; }
  .layout-grid .area-c { grid-area: c; }
  .layout-grid .area-d { grid-area: d; }
  .layout-grid .area-e { grid-area: e; }
  .layout-grid .area-f { grid-area: f; }
  /* stack: one column, in area order. */
  .layout-stack,
  .layout-page {
    grid-template-columns: minmax(0, 1fr);
  }
  /* flow: tiles that wrap. */
  .layout-flow {
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 320px), 1fr));
  }
  .layout-stack .area,
  .layout-flow .area,
  .layout-page .area {
    order: var(--o, 0);
  }
  .area-1 { --o: 1; }
  .area-2 { --o: 2; }
  .area-3 { --o: 3; }
  .area-4 { --o: 4; }
  .area-5 { --o: 5; }
  .area-6 { --o: 6; }
  .area-7 { --o: 7; }
  .area-8 { --o: 8; }
  /* page: a printable report, one column at reading width. */
  .layout-page {
    max-width: 760px;
    margin: 0 auto;
  }
  @media print {
    .layout-page .area {
      break-inside: avoid;
    }
    .boxed {
      box-shadow: none;
    }
  }
  @media (max-width: 760px) {
    .layout-grid,
    .compact.layout-grid {
      grid-template-columns: minmax(0, 1fr);
      grid-template-areas: "a" "b" "c" "d" "e" "f";
      gap: 14px;
    }
    .boxed {
      height: 320px;
    }
    .boxed.solo {
      height: max(320px, calc(100dvh - 170px));
    }
  }
</style>
