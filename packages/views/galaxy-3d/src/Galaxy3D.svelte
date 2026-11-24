<!--
  A galaxy in 3D (V9t), for the galaxy chart's 3D and True view styles. The disc is the kernel's: its rings' stars
  and gas, its arm crests, its bulge and bar, bent by the warp and ripples its collisions left (galaxy-marks.ts).
  True draws it in light (galaxy-true.ts) in the chosen band against black space, as layers through the disc's
  thickness with the bulge as a glow; 3D draws the same disc as a diagram on the chart's ground: arm crests, the
  rings' edges and the Sun's orbit. HUD on adds the Sun's orbit and the disc's edge over True. A device without a GPU
  draws the face-on picture tilted on a canvas.
-->
<script lang="ts">
  import { onDestroy } from "svelte";
  import { basis, galaxyFace, glowPicture, heightAt, scarsAt, sheetWeights, startGpu, type GalaxyBand, type Gpu, type GpuGlow, type GpuSheet, type Vec3 } from "../../../renderer/src/index.ts";
  import { SEEN_ELEVATION, type GalaxyChart, type SeenFrom } from "../../../ui-contract/src/index.ts";

  let {
    chart,
    look = "true",
    band = "visible",
    seen = "tilted",
    hud = true,
    tier = "gpu-basic",
    onturned,
    reframe = 0,
  }: {
    chart: GalaxyChart;
    /** "3d": the chart's diagram turned in 3D; "true": the galaxy in light. */
    look?: "3d" | "true";
    band?: string;
    seen?: SeenFrom;
    hud?: boolean;
    tier?: string;
    onturned?: () => void;
    /** Bumped each time a fixed "Seen from" is picked: the camera goes back to that framing. */
    reframe?: number;
  } = $props();

  const isTrue = $derived(look === "true");
  // Mag has no galaxy-wide field drawing yet: True draws Visible and says so.
  const shownBand = $derived<GalaxyBand>(band === "infrared" || band === "ultraviolet" || band === "xray" || band === "radio" ? band : "visible");
  const outer = $derived((chart.rings.at(-1)?.outer_kpc ?? 10) * 1.15);
  const scars = $derived(scarsAt(chart.assembly?.marks ?? [], chart.at_gyr, chart.disc_kpc));

  // The picture, made once per chart, time and band (a few hundred ms), never per frame.
  const PICTURE_PX = 384;
  let pictureVersion = 0;
  const picture = $derived.by(() => {
    if (!isTrue) return null;
    const p = galaxyFace(chart, chart.at_gyr, shownBand, PICTURE_PX);
    pictureVersion++;
    return { p, version: pictureVersion, bulge: p.bulge.level > 0 ? glowPicture(64, p.bulge.rgb, p.bulge.level) : null };
  });
  // The disc's bend from the collisions' marks: heights on a grid over the picture, rebuilt with the marks.
  const GRID = 48;
  let heightVersion = 0;
  const heights = $derived.by(() => {
    const z = new Float32Array((GRID + 1) * (GRID + 1));
    for (let j = 0; j <= GRID; j++) {
      const y = outer - (2 * outer * j) / GRID;
      for (let i = 0; i <= GRID; i++) {
        const x = -outer + (2 * outer * i) / GRID;
        z[j * (GRID + 1) + i] = heightAt(scars, Math.hypot(x, y), Math.atan2(y, x));
      }
    }
    heightVersion++;
    return { z, version: heightVersion };
  });
  const warped = $derived(scars.warp.rise > 0.005 || scars.ripples.length > 0 || scars.tilt_deg > 0.5);
  const LAYERS = sheetWeights(7);

  // Camera orbiting the galaxy's centre, distances in kpc.
  let yaw = $state(-0.5);
  let pitch = $state(0.55);
  let dist = $state(0);
  $effect(() => {
    if (dist === 0) dist = outer * 2;
  });
  // Picking a fixed "Seen from" frames it afresh (height, turn and zoom), even the one already shown.
  $effect(() => {
    const s = seen;
    void reframe;
    if (s !== "free") ((pitch = SEEN_ELEVATION[s]), (yaw = -0.5), (dist = outer * 2));
  });
  const MIN = $derived(outer * 0.12);
  const MAX = $derived(outer * 8);
  const zoomBy = (f: number) => (dist = Math.min(MAX, Math.max(MIN, dist * f)));
  const eye = $derived<Vec3>([dist * Math.cos(pitch) * Math.cos(yaw), dist * Math.cos(pitch) * Math.sin(yaw), dist * Math.sin(pitch)]);
  const FOV = (45 * Math.PI) / 180;
  const rel = (p: readonly [number, number, number]): Vec3 => [Math.fround(p[0] - eye[0]), Math.fround(p[1] - eye[1]), Math.fround(p[2] - eye[2])];

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
  onDestroy(() => gpu?.dispose());

  /** A circle on the disc, following its bend. */
  function circle(r: number, lift = 0, seg = 128): Vec3[] {
    const out: Vec3[] = [];
    for (let i = 0; i <= seg; i++) {
      const a = (i / seg) * 2 * Math.PI;
      out.push(rel([r * Math.cos(a), r * Math.sin(a), heightAt(scars, r, a) + lift]));
    }
    return out;
  }
  const armLines = () => chart.arms.map((arm) => arm.map(([r, phi]) => rel([r * Math.cos(phi), r * Math.sin(phi), heightAt(scars, r, phi)])));

  $effect(() => {
    if (w === 0 || h === 0) return;
    const ink = token("--sse-ink");
    const faint = token("--sse-ink-faint", "--sse-ink-muted");
    const accent = token("--sse-accent");
    const ground = token("--sse-ground", "--sse-ground-raised");
    if (using === "gpu" && gpu) {
      const { f, u } = basis(eye, [0, 0, 0]);
      const sun = chart.sun_radius_kpc > 0 ? [{ points: circle(chart.sun_radius_kpc), colour: accent }] : [];
      if (isTrue && picture) {
        const pic = picture;
        const sheets: GpuSheet[] = LAYERS.map((l, i) => ({
          key: `galaxy:${shownBand}`,
          version: pic.version,
          texture: { width: pic.p.width, data: pic.p.data },
          rel: rel([0, 0, l.z_kpc]),
          half: outer,
          // The layers' light adds up to the face-on picture's.
          weight: Math.min(1, l.weight * 1.2),
          heights: { n: GRID, z: heights.z, version: heights.version * 16 + i },
        }));
        const glows: GpuGlow[] = pic.bulge ? [{ key: `bulge:${shownBand}`, version: pic.version, texture: { width: 64, data: pic.bulge }, rel: rel([0, 0, 0]), radius: 4 * pic.p.bulge.scale_kpc }] : [];
        gpu.draw({
          items: [],
          rings: hud ? [...sun, { points: circle(chart.rings.at(-1)?.outer_kpc ?? outer), colour: faint }] : [],
          sheets,
          glows,
          space: true,
          forward: f,
          up: u,
          fovY: FOV,
          width: w,
          height: h,
          ground,
        });
        return;
      }
      // 3D: the chart's diagram turned, on the theme's ground.
      const edges = chart.rings.filter((_, i) => i % 4 === 3).map((g) => ({ points: circle(g.outer_kpc), colour: faint }));
      gpu.draw({
        items: [],
        rings: [...armLines().map((points) => ({ points, colour: ink })), ...(hud ? [...edges, ...sun] : [])],
        forward: f,
        up: u,
        fovY: FOV,
        width: w,
        height: h,
        ground,
      });
    } else if (using === "canvas" && canvas2d) {
      // No GPU: the face-on picture (or the arm crests) tilted on a canvas; the warp is not drawn.
      const dpr = typeof devicePixelRatio === "number" ? devicePixelRatio : 1;
      canvas2d.width = Math.round(w * dpr);
      canvas2d.height = Math.round(h * dpr);
      const g = canvas2d.getContext("2d")!;
      g.setTransform(dpr, 0, 0, dpr, 0, 0);
      g.fillStyle = ground;
      g.fillRect(0, 0, w, h);
      const k = (Math.min(w, h) / 2 / Math.tan(FOV / 2)) / dist;
      g.translate(w / 2, h / 2);
      g.scale(1, Math.max(0.02, Math.sin(pitch)));
      g.rotate(-yaw - Math.PI / 2);
      if (isTrue && picture) {
        const off = document.createElement("canvas");
        off.width = off.height = picture.p.width;
        off.getContext("2d")!.putImageData(new ImageData(new Uint8ClampedArray(picture.p.data), picture.p.width, picture.p.width), 0, 0);
        g.globalCompositeOperation = "lighter";
        g.drawImage(off, -outer * k, -outer * k, 2 * outer * k, 2 * outer * k);
      } else {
        g.strokeStyle = ink;
        g.lineWidth = 1;
        for (const arm of chart.arms) {
          g.beginPath();
          arm.forEach(([r, phi], i) => (i ? g.lineTo(r * k * Math.cos(phi), -r * k * Math.sin(phi)) : g.moveTo(r * k * Math.cos(phi), -r * k * Math.sin(phi))));
          g.stroke();
        }
      }
      if (hud && chart.sun_radius_kpc > 0) {
        g.strokeStyle = accent;
        g.beginPath();
        g.arc(0, 0, chart.sun_radius_kpc * k, 0, 2 * Math.PI);
        g.stroke();
      }
    }
  });

  // Turning: drag with a mouse (the choice becomes Free), arrows, wheel or the buttons to zoom.
  let dragFrom: { x: number; y: number; yaw: number; pitch: number } | null = null;
  function down(e: PointerEvent) {
    if ((e.target as HTMLElement).closest("button")) return;
    if (e.pointerType === "touch" && seen !== "free") return;
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
    e.preventDefault();
    zoomBy(e.deltaY > 0 ? 1.12 : 1 / 1.12);
  }
  function key(e: KeyboardEvent) {
    const k = e.key;
    if (k.startsWith("Arrow") && seen !== "free") onturned?.();
    if (k === "ArrowLeft") yaw -= 0.12;
    else if (k === "ArrowRight") yaw += 0.12;
    else if (k === "ArrowUp") pitch = Math.min(1.5, pitch + 0.1);
    else if (k === "ArrowDown") pitch = Math.max(-1.5, pitch - 0.1);
    else if (k === "+" || k === "=") zoomBy(1 / 1.25);
    else if (k === "-") zoomBy(1.25);
    else return;
    e.preventDefault();
  }

  const bandWords = $derived(
    !isTrue
      ? ""
      : band === "mag"
        ? "The galaxy's magnetic field is not drawn yet: this is Visible light."
        : (picture?.p.words ?? ""),
  );
  const marksWords = $derived(warped ? "The disc is bent by the marks its collisions left (warp, ripples and tilt), drawn to the kernel's record." : "");
</script>

<!-- The canvas takes drag, wheel and arrow keys to turn the galaxy; the zoom buttons are the controls. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  class="g3d"
  class:true-look={isTrue}
  bind:this={host}
  role="application"
  aria-roledescription="3D galaxy"
  aria-label={`The galaxy in ${isTrue ? "True" : "3D"}, ${shownBand === "visible" ? "visible light" : shownBand}`}
  tabindex="0"
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={up}
  onwheel={wheel}
  onkeydown={key}
  style:touch-action={seen === "free" ? "none" : "pan-y"}
  data-galaxy-3d
  data-tier={using}
  data-look={isTrue ? "true" : "3d"}
  data-band={isTrue ? shownBand : "chart"}
  data-seen={seen}
  data-hud={hud ? "on" : "off"}
  data-warped={warped ? "yes" : "no"}
>
  <div class="stage" bind:clientWidth={w} bind:clientHeight={h}>
    <canvas bind:this={canvas} class="gpu" class:off={using !== "gpu"} aria-hidden="true"></canvas>
    <canvas bind:this={canvas2d} class="c2d" class:off={using !== "canvas"} aria-hidden="true"></canvas>
    <div class="zoom" role="group" aria-label="Zoom">
      <button type="button" aria-label="Zoom in" onclick={() => zoomBy(1 / 1.25)}>+</button>
      <button type="button" aria-label="Zoom out" onclick={() => zoomBy(1.25)}>−</button>
    </div>
  </div>
  <p class="note">
    <span>Drawn with {using === "gpu" ? reason : using === "canvas" ? `Canvas 2D (${reason}); the disc is drawn flat, without its bend` : "…"}.</span>
    {#if bandWords}<span data-galaxy-band={shownBand}>{bandWords}</span>{/if}
    {#if isTrue}<span>Its stars and gas ring by ring, arms, bulge and bar from the kernel at {chart.at_gyr.toFixed(1)} Gyr; arm widths, clouds, the bar's shape and the bulge's size are stand-ins (fudge register).</span>{/if}
    {#if marksWords}<span data-galaxy-warp>{marksWords}</span>{/if}
    {#if hud && chart.sun_radius_kpc > 0}<span>The coloured circle is the Sun's distance from the centre, {chart.sun_radius_kpc.toFixed(1)} kpc.</span>{/if}
  </p>
</div>

<style>
  .g3d {
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
    min-height: 280px;
    overflow: hidden;
    cursor: grab;
  }
  .g3d:focus-visible {
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
    gap: 0;
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
  /* Under the picture, so HUD off leaves a photograph. */
  .note {
    margin: 6px 0 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
    display: flex;
    flex-wrap: wrap;
    gap: 4px 10px;
  }
</style>
