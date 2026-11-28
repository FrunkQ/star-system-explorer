<!--
  The cosmic web in 3D (V9w), for the foam map's 3D and True view styles. A cube of the kernel's web: every parcel
  of matter where the truncated Zel'dovich step has moved it. 3D draws the parcels as dots on the chart's ground,
  coloured by the place they started in (voids faint, sheets, filaments, knots in the accent). True shows what
  shines: the galaxies in the cube (the kernel's haloes moved by the same step, each lit by its own stars and star
  formation), so the web is traced by its galaxies as a survey sees it; until they arrive, the parcels glow as
  stand-in light. HUD on in True adds the dark matter as a faint guide. It turns slowly on its own (the depth
  reads from the motion) until it is dragged. HUD on adds the cube's edges. A device without a GPU draws the dots
  turned on a canvas.
-->
<script lang="ts">
  import { onDestroy } from "svelte";
  import { basis, cubeEdges, mixCss, SPACE_CSS, starCss, startGpu, webGlow, webLights, webMarks, webParcelLevels, WEB_DARK_GLOW, WEB_LEVEL_INK, WEB_LIGHT_K, type Gpu, type GpuDots, type Vec3, type WebKind } from "../../../renderer/src/index.ts";
  import { SEEN_ELEVATION, WEB_CUBE_FOV, sayNumber, type SeenFrom, type WebCube, type WebGalaxies } from "../../../ui-contract/src/index.ts";

  let {
    cube,
    galaxies = null,
    look = "3d",
    seen = "tilted",
    hud = true,
    shrink = 1,
    tier = "gpu-basic",
    onturned,
    reframe = 0,
    aim = null,
    inner = null,
  }: {
    cube: WebCube;
    /** The cube's galaxies (True's light), once they are read. */
    galaxies?: WebGalaxies | null;
    look?: "3d" | "true";
    seen?: SeenFrom;
    hud?: boolean;
    /** Real: the cube at its size then (the scale factor); Readable: comoving, 1. */
    shrink?: number;
    tier?: string;
    onturned?: () => void;
    /** Bumped each time a fixed "Seen from" is picked: the camera goes back to that framing. */
    reframe?: number;
    /**
     * The journey port (grand zoom GZ8): while a journey travels through the web, the camera it puts here, in the
     * cube's own units (comoving Mpc, absolute like the cube's centre). Set: the cube does not turn by itself, takes no
     * drag, and draws only the picture (no buttons, no note), so it can be cross-faded with the level below.
     */
    aim?: { eye: Vec3; target: Vec3 } | null;
    /**
     * On a journey, at the seam to the galaxies nearby: the galaxies within `radius_mpc` of the journey's target are
     * drawn at `alpha` of their light (the level below draws those, from its catalogue), the rest at full light, so
     * the web past the level below stays behind it as a backdrop. Absent: every galaxy at full light.
     */
    inner?: { radius_mpc: number; alpha: number } | null;
  } = $props();

  const isTrue = $derived(look === "true");
  const levels = $derived(webParcelLevels(cube.volume, shrink));
  const marks = $derived(!isTrue && galaxies ? webMarks(galaxies.galaxies, cube.volume, shrink) : []);
  const marked = $derived(marks.reduce((a, m) => a + m.points.length, 0));
  const half = $derived(cube.volume.extent / 2);
  const lights = $derived(isTrue && galaxies ? webLights(galaxies.galaxies, cube.volume, shrink) : null);
  // On a journey's seam to the galaxies nearby: how many of the cube's lit galaxies lie within their reach.
  const innerCount = $derived.by(() => {
    if (!inner || !aim || !lights) return undefined;
    const r2 = inner.radius_mpc ** 2;
    let k = 0;
    for (const g of lights) for (const p of g.points) if ((p[0] - lookAt[0]) ** 2 + (p[1] - lookAt[1]) ** 2 + (p[2] - lookAt[2]) ** 2 <= r2) k++;
    return k;
  });
  const lit = $derived(lights ? lights.reduce((a, g) => a + g.points.length, 0) : 0);

  // Camera orbiting the cube's centre, distances in Mpc (comoving, so the camera stays put as Real shrinks it).
  let yaw = $state(-0.6);
  let pitch = $state(0.45);
  let dist = $state(0);
  // A new cube size puts the camera back at the same view of it.
  let framed = 0;
  $effect(() => {
    if (framed !== half) ((framed = half), (dist = half * 4.2));
  });
  // Picking a fixed "Seen from" frames it afresh (height, turn and zoom), even the one already shown.
  $effect(() => {
    const s = seen;
    void reframe;
    if (s !== "free") ((pitch = SEEN_ELEVATION[s]), (yaw = -0.6), (dist = half * 4.2));
  });
  const MIN = $derived(half * 0.3);
  const MAX = $derived(half * 10);
  const zoomBy = (f: number) => (dist = Math.min(MAX, Math.max(MIN, dist * f)));
  // Without a journey the camera orbits the cube's middle; on one it is wherever the journey puts it, about home.
  const centre = $derived<Vec3>([cube.volume.centre[0], cube.volume.centre[1], cube.volume.centre[2]]);
  const lookAt = $derived<Vec3>(aim ? [aim.target[0] - centre[0], aim.target[1] - centre[1], aim.target[2] - centre[2]] : [0, 0, 0]);
  const eye = $derived<Vec3>(aim ? [aim.eye[0] - centre[0], aim.eye[1] - centre[1], aim.eye[2] - centre[2]] : [dist * Math.cos(pitch) * Math.cos(yaw), dist * Math.cos(pitch) * Math.sin(yaw), dist * Math.sin(pitch)]);
  const FOV = WEB_CUBE_FOV;
  const rel = (p: readonly [number, number, number]): Vec3 => [Math.fround(p[0] - eye[0]), Math.fround(p[1] - eye[1]), Math.fround(p[2] - eye[2])];

  // A slow turn so the depth reads, until the viewer takes hold (or asks for less motion).
  let spinning = $state(true);
  let raf = 0;
  $effect(() => {
    const still = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (!spinning || still || aim) return;
    let last = 0;
    const step = (t: number) => {
      if (last) yaw += ((t - last) / 1000) * 0.12;
      last = t;
      raf = requestAnimationFrame(step);
    };
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
  });

  let host: HTMLDivElement | undefined = $state();
  let w = $state(0);
  let h = $state(0);
  const token = (name: string, fallback?: string) => {
    if (!host) return "";
    const cs = getComputedStyle(host);
    return cs.getPropertyValue(name).trim() || (fallback ? cs.getPropertyValue(fallback).trim() : "") || cs.color;
  };

  let canvas: HTMLCanvasElement | undefined = $state();
  let canvas2d: HTMLCanvasElement | undefined = $state();
  let gpu = $state<Gpu | null>(null);
  let using = $state<"gpu" | "canvas" | "">("");
  let reason = $state("");
  $effect(() => {
    if (!canvas || using) return;
    if (tier === "canvas") return void ((using = "canvas"), (reason = "canvas tier chosen"));
    void startGpu(canvas).then((g) => {
      if (g) ((gpu = g), (using = "gpu"), (reason = g.backend === "webgpu" ? "WebGPU" : "WebGL2 (no WebGPU here)"));
      else ((using = "canvas"), (reason = "no GPU path on this device"));
    });
  });
  onDestroy(() => {
    cancelAnimationFrame(raf);
    gpu?.dispose();
  });

  const LIGHT = starCss(WEB_LIGHT_K);
  const sciMass = (m: number) => sayNumber(m, 2);
  const total = $derived(levels.reduce((a, g) => a + g.points.length, 0));

  // What is drawn, as layers of alike dots in display units: the GPU and the canvas draw the same list.
  type Layer = { points: Vec3[]; colour: string; glow?: number; size: number };
  $effect(() => {
    if (w === 0 || h === 0) return;
    const colours: Record<WebKind, string> = {
      void: token("--sse-ink-faint", "--sse-ink-muted"),
      sheet: token("--sse-ink-muted"),
      filament: token("--sse-ink"),
      knot: token("--sse-accent"),
    };
    const ground = token("--sse-ground", "--sse-ground-raised");
    const edgeColour = isTrue ? token("--sse-ink-faint", "--sse-ink-muted") : token("--sse-rule", "--sse-ink-faint");
    const hs = half * shrink;
    const layers: Layer[] = [];
    if (isTrue) {
      if (lights) {
        if (hud) for (const g of levels) layers.push({ points: g.points, colour: colours.sheet, glow: WEB_DARK_GLOW * WEB_LEVEL_INK[g.level]!, size: 1 });
        for (const g of lights) {
          if (!inner || !aim) {
            layers.push({ points: g.points, colour: starCss(g.colour_k), glow: g.glow, size: g.size });
            continue;
          }
          // Split by distance from the journey's target: the near ones fade as the level below takes them over.
          const r2 = inner.radius_mpc * inner.radius_mpc;
          const near: Vec3[] = [];
          const far: Vec3[] = [];
          for (const p of g.points) ((p[0] - lookAt[0]) ** 2 + (p[1] - lookAt[1]) ** 2 + (p[2] - lookAt[2]) ** 2 <= r2 ? near : far).push(p);
          if (far.length) layers.push({ points: far, colour: starCss(g.colour_k), glow: g.glow, size: g.size });
          if (near.length && inner.alpha > 0.002) layers.push({ points: near, colour: starCss(g.colour_k), glow: g.glow * inner.alpha, size: g.size });
        }
      } else for (const g of levels) layers.push({ points: g.points, colour: LIGHT, glow: webGlow(total) * (0.4 + 0.6 * WEB_LEVEL_INK[g.level]!), size: 1.5 });
    } else {
      // Fainter to stronger, so the dense are drawn over the sparse.
      for (const g of [...levels].sort((x, y) => x.level - y.level))
        layers.push({ points: g.points, colour: mixCss(ground, colours[g.kind], WEB_LEVEL_INK[g.level]!), size: (g.level >= 2 ? 1.5 : 1) + (g.kind === "knot" ? 1 : 0) });
      for (const m of marks) layers.push({ points: m.points, colour: m.name === "galaxy" ? colours.filament : colours.knot, size: m.size });
    }
    if (using === "gpu" && gpu) {
      const { f, u } = basis(eye, lookAt);
      const dots: GpuDots[] = layers.map((l) => ({ points: l.points.map(rel), colour: l.colour, size: l.size, ...(l.glow !== undefined ? { glow: l.glow } : {}) }));
      gpu.draw({
        items: [],
        rings: hud ? cubeEdges(hs).map((e) => ({ points: e.map(rel), colour: edgeColour })) : [],
        dots,
        space: isTrue,
        forward: f,
        up: u,
        fovY: FOV,
        width: w,
        height: h,
        ground,
      });
    } else if (using === "canvas" && canvas2d) {
      // No GPU: the same layers projected by hand.
      const dpr = typeof devicePixelRatio === "number" ? devicePixelRatio : 1;
      canvas2d.width = Math.round(w * dpr);
      canvas2d.height = Math.round(h * dpr);
      const g = canvas2d.getContext("2d")!;
      g.setTransform(dpr, 0, 0, dpr, 0, 0);
      g.fillStyle = isTrue ? SPACE_CSS : ground;
      g.fillRect(0, 0, w, h);
      const { f, u } = basis(eye, lookAt);
      const r: Vec3 = [f[1] * u[2] - f[2] * u[1], f[2] * u[0] - f[0] * u[2], f[0] * u[1] - f[1] * u[0]];
      const k = Math.min(w, h) / 2 / Math.tan(FOV / 2);
      const put = (p: Vec3) => {
        const q = rel(p);
        const d = q[0] * f[0] + q[1] * f[1] + q[2] * f[2];
        if (d <= 0) return null;
        return [w / 2 + (k * (q[0] * r[0] + q[1] * r[1] + q[2] * r[2])) / d, h / 2 - (k * (q[0] * u[0] + q[1] * u[1] + q[2] * u[2])) / d] as const;
      };
      for (const l of layers) {
        g.globalCompositeOperation = l.glow !== undefined ? "lighter" : "source-over";
        g.globalAlpha = l.glow ?? 1;
        g.fillStyle = l.colour;
        for (const p of l.points) {
          const xy = put(p);
          if (xy) g.fillRect(xy[0] - l.size / 2, xy[1] - l.size / 2, l.size, l.size);
        }
      }
      g.globalAlpha = 1;
      g.globalCompositeOperation = "source-over";
      if (hud) {
        g.strokeStyle = edgeColour;
        for (const e of cubeEdges(hs)) {
          g.beginPath();
          e.forEach((p, i) => {
            const xy = put(p);
            if (xy) (i ? g.lineTo(xy[0], xy[1]) : g.moveTo(xy[0], xy[1]));
          });
          g.stroke();
        }
      }
    }
  });

  // Turning: drag with a mouse (the choice becomes Free), arrows, wheel or the buttons to zoom.
  let dragFrom: { x: number; y: number; yaw: number; pitch: number } | null = null;
  function down(e: PointerEvent) {
    if (aim) return;
    if ((e.target as HTMLElement).closest("button")) return;
    if (e.pointerType === "touch" && seen !== "free") return;
    spinning = false;
    dragFrom = { x: e.clientX, y: e.clientY, yaw, pitch };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    if (!dragFrom) return;
    const dx = e.clientX - dragFrom.x;
    const dy = e.clientY - dragFrom.y;
    if (Math.abs(dx) + Math.abs(dy) > 3 && seen !== "free") onturned?.();
    yaw = dragFrom.yaw - dx * 0.008;
    pitch = Math.min(1.5, Math.max(-1.5, dragFrom.pitch + dy * 0.008));
  }
  const up = () => (dragFrom = null);
  function wheel(e: WheelEvent) {
    if (aim) return;
    e.preventDefault();
    zoomBy(e.deltaY > 0 ? 1.12 : 1 / 1.12);
  }
  function key(e: KeyboardEvent) {
    if (aim) return;
    const k = e.key;
    if (k.startsWith("Arrow") && seen !== "free") onturned?.();
    if (k === "ArrowLeft") yaw -= 0.12;
    else if (k === "ArrowRight") yaw += 0.12;
    else if (k === "ArrowUp") pitch = Math.min(1.5, pitch + 0.1);
    else if (k === "ArrowDown") pitch = Math.max(-1.5, pitch - 0.1);
    else if (k === "+" || k === "=") zoomBy(1 / 1.25);
    else if (k === "-") zoomBy(1.25);
    else return;
    spinning = false;
    e.preventDefault();
  }
</script>

<!-- The canvas takes drag, wheel and arrow keys to turn the cube; the zoom and turn buttons are the controls. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  class="w3d"
  class:true-look={isTrue}
  bind:this={host}
  role="application"
  aria-roledescription="3D cosmic web"
  aria-label={`The cosmic web in ${isTrue ? "True" : "3D"}, a cube ${cube.volume.extent} megaparsecs across`}
  tabindex="0"
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={up}
  onwheel={wheel}
  onkeydown={key}
  style:touch-action={seen === "free" ? "none" : "pan-y"}
  data-web-3d
  data-tier={using}
  data-look={isTrue ? "true" : "3d"}
  data-parcels={total}
  data-galaxies={lit}
  data-marks={marked}
  data-spinning={spinning && !aim ? "yes" : "no"}
  data-journey={aim ? "" : undefined}
  data-journey-inner={innerCount}
>
  <div class="stage" bind:clientWidth={w} bind:clientHeight={h}>
    <canvas bind:this={canvas} class="gpu" class:off={using !== "gpu"} aria-hidden="true"></canvas>
    <canvas bind:this={canvas2d} class="c2d" class:off={using !== "canvas"} aria-hidden="true"></canvas>
    {#if !aim}<div class="zoom" role="group" aria-label="Zoom and turn">
      <button type="button" aria-label={spinning ? "Stop turning" : "Turn slowly"} data-spin onclick={() => (spinning = !spinning)}>{spinning ? "❚❚" : "↻"}</button>
      <button type="button" aria-label="Zoom in" onclick={() => zoomBy(1 / 1.25)}>+</button>
      <button type="button" aria-label="Zoom out" onclick={() => zoomBy(1.25)}>−</button>
    </div>{/if}
  </div>
  {#if !aim}<p class="note">
    <span>Drawn with {using === "gpu" ? reason : using === "canvas" ? `Canvas 2D (${reason})` : "…"}: {sayNumber(total, 6)} parcels of matter, each the same mass, moved from a grid {cube.volume.n} a side by the field smoothed over {sayNumber(cube.volume.smoothing_mpc, 2)} Mpc.</span>
    {#if isTrue && lights}<span>Light: {sayNumber(lit, 6)} galaxies, the haloes above {sciMass(galaxies?.galaxy_floor_msun ?? 0)} solar masses, each where its matter has been carried and shining by its own stars: old stars warm, young ones blue. The gas and dark matter between them are dark{hud ? " (HUD shows the dark matter faintly)" : ""}.</span>{:else if isTrue}<span>Lighting the galaxies… for now every parcel shines alike, one colour, old starlight at about {sayNumber(WEB_LIGHT_K, 2)} K.</span>{:else}<span>Faint: voids. Grey: sheets. Ink: filaments. Accent: knots, where clusters grow. Each dot is inked by how crowded its matter is today, so the dense web stands out of the thin voids.{marked ? ` Bigger dots: ${sayNumber(marked, 6)} big galaxies, groups and clusters (haloes over 10¹² solar masses).` : ""}</span>{/if}
  </p>{/if}
</div>

<style>
  .w3d {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--sse-ground);
    outline: none;
  }
  .stage {
    position: relative;
    flex: 1;
    min-height: 320px;
    aspect-ratio: 1;
    overflow: hidden;
    cursor: grab;
    border: 1px solid var(--sse-rule);
  }
  .w3d[data-journey] .stage {
    min-height: 0;
    border: 0;
    cursor: default;
  }
  .w3d:focus-visible {
    box-shadow: inset 0 0 0 2px var(--sse-accent);
  }
  canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
  canvas.off {
    display: none;
  }
  .zoom {
    position: absolute;
    right: 8px;
    top: 8px;
    display: flex;
  }
  .zoom button {
    font: inherit;
    width: 32px;
    height: 32px;
    border: 1px solid var(--sse-rule);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    cursor: pointer;
  }
  .note {
    margin: 6px 0 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
    display: flex;
    flex-wrap: wrap;
    gap: 4px 10px;
  }
</style>
