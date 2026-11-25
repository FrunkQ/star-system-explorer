<!--
  In orbit (landing run-through LR4): the world the table's ship is circling, turning below, on a player's screen.
  The GM sends it from a world's own screen (Send this view on the world rail, not on Land): the screen selects the
  world it names, and this seat's own kernel reads that one world's six map tiles (granted for it alone, as for a
  landing; Players.svelte). Drawn on Body's globe path (renderer mountGlobe) with the true-colour recipe and the slow
  presentation spin; no cutaway, no controls. Players watch; nothing here asks the kernel for more than the map.
  The world's air comes with the send (LR4b): a player seat does not read resolved systems, so the GM's kernel reads
  the air's facts and optics and they ride in what is shown (seat.ts `Shown.air`).
-->
<script module lang="ts">
  import type { SurfaceTile } from "../../../ui-contract/src/index.ts";
  // The world's map tiles, kept per world for the page's life (a second or so to build).
  const globes = new Map<string, SurfaceTile[]>();
</script>

<script lang="ts">
  import { untrack } from "svelte";
  import { subjectName, type Globe, type Json, type PlayerContext, type PlayerScene } from "../../../ui-contract/src/index.ts";
  import { LIBRARY, mountGlobe, type AirFacts, type GlobeRun, type GlobeRunOptions } from "../../../renderer/src/index.ts";
  import { SHIPPED_RECIPES } from "../../../renderer/src/recipes.ts";

  let { ctx, scene, params }: { ctx: PlayerContext; scene: PlayerScene; params: Record<string, Json> } = $props();

  // The world in view: the one the GM's send selected, if the table knows it.
  const subject = $derived.by(() => {
    const a = ctx.selection();
    return a ? (scene.subjects.find((s) => s.address === a) ?? null) : null;
  });
  const address = $derived(subject?.address ?? "");
  const name = $derived(subject ? subjectName(subject) : "");

  let tiles = $state.raw<SurfaceTile[] | null>(null);
  let failed = $state<string | null>(null);
  let gen = 0;
  $effect(() => {
    const sys = scene.scope;
    const body = address;
    const g = ++gen;
    tiles = null;
    failed = null;
    if (!body || !ctx.ground) return;
    const key = `${sys}|${body}`;
    const hit = globes.get(key);
    if (hit) return void (tiles = hit);
    // The grant for this world arrives with the send, a moment after the selection: ask again while it is refused.
    let tries = 0;
    const ask = async () => {
      const r = await ctx.ground!.surfaceTiles(sys, body);
      if (g !== gen) return;
      if (r.ok) {
        globes.set(key, r.tiles);
        tiles = r.tiles;
      } else if (++tries < 20) setTimeout(() => void ask(), 250);
      else failed = r.message;
    };
    void ask();
  });
  const globe = $derived<Globe>(tiles ? [0, 1, 2, 3, 4, 5].map((f) => tiles!.find((t) => t.key.face === f && t.key.level === 0)) : []);
  const has = $derived(globe.some((t) => t));

  // The air the GM's kernel read for this world, sent with the orbit; kept as text so a new send of the same air does not stir the globe.
  const airText = $derived.by(() => {
    const sh = ctx.shown?.();
    return sh && "scope" in sh && sh.focus === address && sh.air ? JSON.stringify(sh.air) : "";
  });
  const air = $derived(airText ? (JSON.parse(airText) as AirFacts) : null);
  let w = $state(0);
  let h = $state(0);
  const size = $derived(Math.round(Math.min(640, Math.max(160, Math.min(w || 320, (h || 360) - 48) * 0.9))));
  const runOpts = $derived<GlobeRunOptions>({
    registry: SHIPPED_RECIPES,
    library: LIBRARY,
    recipe: "true-colour",
    tier: String(params.tier ?? "auto") as GlobeRunOptions["tier"],
    knobs: { relief: 0.5, detail: 0.5 },
    budget_ms: 33.3,
    governor: true,
    spin: params.spin !== false,
    size_px: size,
    label: name,
    air,
  });
  let host = $state<HTMLElement | undefined>();
  let run: GlobeRun | null = null;
  let backend = $state("");
  $effect(() => {
    const el = host;
    if (!el) return;
    const r = mountGlobe(el, untrack(() => runOpts), (st) => (backend = st.backend));
    run = r;
    return () => {
      r.dispose();
      run = null;
    };
  });
  $effect(() => {
    const o = runOpts;
    untrack(() => run)?.update(o);
  });
  $effect(() => {
    const g = globe;
    if (has) untrack(() => run)?.setGlobe(g);
  });

  // The party's ship in orbit (LR4c): a marker going round the world on a tilted ring, named, passing behind the
  // world on the far side. A presentation drift like the globe's spin: one turn in ORBIT_S, not its real period, and
  // the ring drawn clear of the limb, not at its real height (fudge register, "The ship in orbit on the table").
  const ORBIT_S = 24;
  const TILT = 0.28;
  const ship = $derived.by(() => {
    const sh = ctx.shown?.();
    return sh && "scope" in sh && sh.focus === address && sh.ship ? sh.ship : "";
  });
  let phase = $state(0);
  $effect(() => {
    if (!ship) return;
    let raf = 0;
    const t0 = performance.now();
    const tick = (t: number) => {
      phase = (((t - t0) / 1000 / ORBIT_S) % 1) * 2 * Math.PI;
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  });
  const ring = $derived({ rx: size * 0.62, ry: size * 0.62 * TILT });
  const shipAt = $derived.by(() => {
    const x = Math.cos(phase) * ring.rx;
    const y = Math.sin(phase) * ring.ry;
    // The far half of the ring (y < 0, above the centre) is behind the world where it crosses the disc.
    const behind = y < 0 && Math.abs(x) < size * 0.48;
    return { x, y, behind };
  });
</script>

<section class="orbit" data-orbit={address || undefined} data-orbit-drawn={has ? backend || "yes" : undefined} data-orbit-air={air ? "yes" : undefined} bind:clientWidth={w} bind:clientHeight={h} aria-label={name ? `In orbit of ${name}` : "In orbit"}>
  {#if !subject}
    <p class="quiet">Waiting for the game master to name the world below.</p>
  {:else}
    <div class="sky" style:width="{size}px" style:height="{size}px">
      <div class="globe" style:width="{size}px" style:height="{size}px" bind:this={host} role="img" aria-label="{name}, turning below"></div>
      {#if ship && has}
        {@const pad = size * 0.2}
        <svg class="ring" width={size + 2 * pad} height={size + 2 * pad} viewBox="{-size / 2 - pad} {-size / 2 - pad} {size + 2 * pad} {size + 2 * pad}" data-orbit-ship={ship} data-orbit-ship-behind={shipAt.behind || undefined} aria-label="{ship}, in orbit">
          <path class="near" d="M {-ring.rx} 0 A {ring.rx} {ring.ry} 0 0 0 {ring.rx} 0" />
          {#if !shipAt.behind}
            <circle class="ship" cx={shipAt.x} cy={shipAt.y} r={Math.max(3, size * 0.012)} />
            <text class="name" x={shipAt.x + 8} y={shipAt.y - 8}>{ship}</text>
          {/if}
        </svg>
      {/if}
    </div>
    <p class="caption" data-orbit-caption>
      {#if has}{name}, turning below{ship ? `, ${ship} in orbit` : ""}.{:else if failed}{name}: the map could not be read on this screen ({failed}).{:else}Reading {name}'s map…{/if}
    </p>
  {/if}
</section>

<style>
  .orbit {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    width: 100%;
    height: 100%;
    min-height: 240px;
  }
  .globe {
    max-width: 100%;
  }
  .sky {
    position: relative;
    max-width: 100%;
  }
  .ring {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
    overflow: visible;
  }
  .ring .near {
    fill: none;
    stroke: var(--sse-ink-muted, var(--sse-ink));
    stroke-width: 1;
    stroke-dasharray: 2 4;
    opacity: 0.6;
  }
  .ring .ship {
    fill: var(--sse-accent);
  }
  .ring .name {
    fill: var(--sse-ink);
    font-size: 0.8rem;
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 3px;
  }
  .caption,
  .quiet {
    margin: 0;
    color: var(--sse-ink-muted, var(--sse-ink));
    font-size: 0.9rem;
  }
</style>
