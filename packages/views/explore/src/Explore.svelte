<!--
  Explore: switches between this track's sections (Route, Galaxy, ...), each
  an ordinary view module mounted here with the screen's context and scene.
  The section chosen is remembered on this device.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import type { SystemScene, ViewContext, ViewHandle, ViewModule } from "../../../ui-contract/src/index.ts";

  let { ctx, scene, sections }: { ctx: ViewContext; scene: SystemScene; sections: readonly { id: string; label: string; view: ViewModule }[] } = $props();
  const KEY = "sse.explore.section";
  const stored = (() => {
    try {
      return localStorage.getItem(KEY);
    } catch {
      return null;
    }
  })();
  const start = untrack(() => {
    const first = String(ctx.params.section ?? "route");
    return sections.some((s) => s.id === stored) ? stored! : sections.some((s) => s.id === first) ? first : sections[0]!.id;
  });
  let current = $state(start);
  function choose(id: string) {
    current = id;
    try {
      localStorage.setItem(KEY, id);
    } catch {
      /* private window: not remembered */
    }
  }

  // A section can send the GM to another one (the cluster map's "Chart this
  // galaxy"): it dispatches a bubbling "sse:explore" event naming the section.
  let root = $state<HTMLDivElement | null>(null);
  $effect(() => {
    const el = root;
    if (!el) return;
    const go = (e: Event) => {
      const id = (e as CustomEvent<{ section?: string }>).detail?.section;
      if (id && sections.some((s) => s.id === id)) choose(id);
    };
    el.addEventListener("sse:explore", go);
    return () => el.removeEventListener("sse:explore", go);
  });

  let host = $state<HTMLDivElement | null>(null);
  let handle: ViewHandle | null = null;
  $effect(() => {
    const el = host;
    const sec = sections.find((s) => s.id === current);
    if (!el || !sec) return;
    const h = untrack(() => sec.view.mount(el, { ...ctx, params: Object.fromEntries(sec.view.params.map((p) => [p.name, p.default])) }, scene));
    handle = h;
    return () => {
      h.dispose();
      if (handle === h) handle = null;
    };
  });
  $effect(() => {
    const s = scene;
    untrack(() => handle?.update(s));
  });
</script>

<div class="explore" data-section={current} bind:this={root}>
  <nav class="tabs" aria-label="Explore">
    {#each sections as s (s.id)}<button type="button" class:on={current === s.id} aria-pressed={current === s.id} onclick={() => choose(s.id)} data-section-tab={s.id}>{s.label}</button>{/each}
  </nav>
  {#key current}<div class="section" bind:this={host}></div>{/key}
</div>

<style>
  .explore {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .tabs {
    display: inline-flex;
    flex-wrap: wrap;
  }
  .tabs button {
    font: inherit;
    font-size: 13px;
    min-height: 34px;
    padding: 0 14px;
    border: 1px solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    margin-right: -1px;
    cursor: pointer;
  }
  .tabs button.on {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
</style>
