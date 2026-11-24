<!--
  The screens runtime (H8.3, V3): mounts a checked screen document onto a
  player scene, the same on a player's device, in the composer's preview and
  in a printed report. The document decides everything: which views, where
  (its layout's areas), with which parameters, and what each item shows
  (`bind`). Views that fill a box (a chart) get one; the rest grow with what
  they show. Selection is shared by the views of one screen.
-->
<script lang="ts">
  import { boundScene, withLive, type CheckedScreen, type Mark, type PlayerContext, type PlayerScene, type PlayerSubject, type Vocab, type ViewHandle, type LandingGround, type LandingShow } from "../../../ui-contract/src/index.ts";

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
  }: { screen: CheckedScreen; scene: PlayerScene; vocab: Vocab; mark?: (s: PlayerSubject) => Mark; selection?: string | null; compact?: boolean; live?: Record<string, boolean>; landing?: () => LandingShow | null; ground?: LandingGround } = $props();

  const handles: { h: ViewHandle<PlayerScene>; m: CheckedScreen["mounts"][number] }[] = [];
  $effect(() => {
    const s = scene;
    for (const x of handles) x.h.update(boundScene(s, x.m.bind));
  });
  function view(node: HTMLElement, m: CheckedScreen["mounts"][number]) {
    const ctx: PlayerContext = { params: withLive(m.params, m.module.params ?? [], live), vocab, selection: () => selection, select: (a) => (selection = a), ...(mark ? { mark } : {}), ...(landing ? { landing: () => landing() } : {}), ...(ground ? { ground } : {}) };
    const h = m.module.mount(node, ctx, boundScene(scene, m.bind)) as ViewHandle<PlayerScene>;
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
    <section class="area area-{m.area}" class:boxed={m.module.box === "fill"} data-view={m.module.id} data-area={m.area} aria-label="{m.module.id.replace('sse.view.', '')} ({m.area})">
      <div class="fill" use:view={m}></div>
    </section>
  {/each}
  {/key}
</div>

<style>
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
  }
</style>
