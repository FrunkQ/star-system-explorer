<!-- Mounts a checked player screen onto a scene: the same mounting a player's device does. -->
<script lang="ts">
  import type { CheckedScreen, PlayerContext, PlayerScene, Vocab, ViewHandle } from "../../../ui-contract/src/index.ts";

  let { screen, scene, vocab }: { screen: CheckedScreen; scene: PlayerScene; vocab: Vocab } = $props();
  let selection = $state<string | null>(null);
  const handles: ViewHandle<PlayerScene>[] = [];
  $effect(() => {
    const s = scene;
    for (const h of handles) h.update(s);
  });
  function view(node: HTMLElement, m: CheckedScreen["mounts"][number]) {
    const ctx: PlayerContext = { params: m.params, vocab, selection: () => selection, select: (a) => (selection = a) };
    const h = m.module.mount(node, ctx, scene) as ViewHandle<PlayerScene>;
    handles.push(h);
    return { destroy: () => (handles.splice(handles.indexOf(h), 1), h.dispose()) };
  }
</script>

<div class="grid">
  {#each screen.mounts as m (m.area)}
    <section class="area area-{m.area}" data-view={m.module.id} aria-label="{m.module.id} in area {m.area}"><div class="fill" use:view={m}></div></section>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    gap: 12px;
    grid-template-columns: minmax(0, 1.3fr) minmax(0, 1fr);
    grid-template-areas: "a b" "c b" "d b" "e e" "f f";
    align-items: start;
  }
  .area-a { grid-area: a; height: 300px; border: var(--sse-line) solid var(--sse-ink); }
  .area-a .fill { height: 100%; }
  .area-b { grid-area: b; }
  .area-c { grid-area: c; }
  .area-d { grid-area: d; }
  .area-e { grid-area: e; }
  .area-f { grid-area: f; }
  @media (max-width: 760px) {
    .grid {
      grid-template-columns: minmax(0, 1fr);
      grid-template-areas: "a" "b" "c" "d" "e" "f";
    }
  }
</style>
