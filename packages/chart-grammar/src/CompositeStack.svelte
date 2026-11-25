<!-- Composite (exotic objects design 6A, XO17): any view's picture drawn once per band and laid into one, for every
     view at every level (Alex, 24 Nov: "the composite view should be everywhere!"). The view gives one snippet that
     draws itself in a band; this draws it once per layer, tints each layer but Visible in its stated colour and adds
     them (light on a dark ground, ink on a light one). Nothing here knows what is drawn, so a view gets Composite by
     wrapping its picture, with no layering code of its own.
     - The bottom layer is Visible when it is in the mix, else the first; only it draws the HUD and takes the hands.
       Presses, drags and the wheel on it are passed to the same point of every other layer, so Free view turns
       them all together.
     - Each band arrives already scaled to its own brightest (every band view is), so a faint radio jet is not
       drowned by a bright disc. The pixel maths for views that hold their own frames is renderer/src/composite.ts. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { CompositeLayer, ViewBand } from "../../ui-contract/src/index.ts";
  import { GREY, PAPER_LIFT, inkMatrix, tintMatrix } from "../../renderer/src/composite.ts";

  // `dark`: the picture is light on a dark ground (True, Night chart). Left out: read from the page's colour scheme.
  let { layers, dark, children }: { layers: CompositeLayer[]; dark?: boolean; children: Snippet<[ViewBand, boolean]> } = $props();
  const night = $derived(dark ?? (typeof document !== "undefined" && document.documentElement.style.colorScheme === "dark"));
  const uid = $props.id();
  // The bottom layer first, then the rest in the viewer's order.
  const order = $derived.by(() => {
    const base = layers.find((l) => l.band === "visible") ?? layers[0];
    return base ? [base, ...layers.filter((l) => l !== base)] : [];
  });
  let baseEl = $state<HTMLDivElement | null>(null);
  let upper: (HTMLDivElement | null)[] = $state([]);

  // Hands on the bottom layer reach every layer at the same point.
  $effect(() => {
    const el = baseEl;
    if (!el) return;
    const kinds = ["pointerdown", "pointermove", "pointerup", "pointercancel", "wheel", "dblclick"] as const;
    const pass = (e: Event) => {
      if (!e.isTrusted) return;
      const src = e.target as Element | null;
      if (!src || !(src instanceof HTMLCanvasElement)) return;
      for (const layer of upper) {
        const to = layer?.querySelector("canvas");
        if (!to) continue;
        const copy =
          e instanceof WheelEvent
            ? new WheelEvent(e.type, e)
            : e instanceof PointerEvent
              ? new PointerEvent(e.type, e)
              : new MouseEvent(e.type, e as MouseEvent);
        to.dispatchEvent(copy);
      }
    };
    for (const k of kinds) el.addEventListener(k, pass, { capture: true, passive: true });
    return () => {
      for (const k of kinds) el.removeEventListener(k, pass, { capture: true });
    };
  });
</script>

<div class="stack" class:dark={night} data-composite-stack data-layers={order.map((l) => l.band).join(" ")}>
  <svg class="defs" aria-hidden="true" width="0" height="0">
    {#each order as l (l.band)}
      {#if l.band !== "visible"}
        <filter id="{uid}-{l.band}" color-interpolation-filters="sRGB">
          {#if night}
            <feColorMatrix type="matrix" values={tintMatrix(l.colour, true).join(" ")} />
          {:else}
            <!-- Ink on paper: brightness first, the paper pushed to white so it adds nothing, then the ink coloured. -->
            <feColorMatrix type="matrix" values={GREY.join(" ")} />
            <feComponentTransfer>
              <feFuncR type="linear" slope={PAPER_LIFT} />
              <feFuncG type="linear" slope={PAPER_LIFT} />
              <feFuncB type="linear" slope={PAPER_LIFT} />
            </feComponentTransfer>
            <feColorMatrix type="matrix" values={inkMatrix(l.colour).join(" ")} />
          {/if}
        </filter>
      {/if}
    {/each}
  </svg>
  {#each order as l, i (l.band)}
    {#if i === 0}
      <div class="layer base" bind:this={baseEl} data-layer={l.band} style:--tint={l.band === "visible" ? "none" : `url(#${uid}-${l.band})`}>
        {@render children(l.band, true)}
      </div>
    {:else}
      <div class="layer over" bind:this={upper[i - 1]} data-layer={l.band} style:--tint={l.band === "visible" ? "none" : `url(#${uid}-${l.band})`} inert aria-hidden="true">
        {@render children(l.band, false)}
      </div>
    {/if}
  {/each}
</div>

<style>
  .stack {
    position: relative;
    width: 100%;
    height: 100%;
    isolation: isolate;
  }
  .defs {
    position: absolute;
    width: 0;
    height: 0;
  }
  .layer {
    width: 100%;
    height: 100%;
  }
  /* The tint falls on the picture only, never on the bottom layer's words and buttons. */
  .layer :global(canvas) {
    filter: var(--tint);
  }
  .over :global(svg) {
    filter: var(--tint);
  }
  .over {
    position: absolute;
    inset: 0;
    pointer-events: none;
    mix-blend-mode: multiply;
  }
  .dark .over {
    mix-blend-mode: screen;
  }
  /* A layer above the bottom one gives only its picture: its words, buttons and notes are the bottom layer's. */
  .over :global(*) {
    visibility: hidden;
  }
  .over :global(canvas),
  .over :global(svg) {
    visibility: visible;
  }
  @supports (mix-blend-mode: plus-lighter) {
    .dark .over {
      mix-blend-mode: plus-lighter;
    }
  }
</style>
