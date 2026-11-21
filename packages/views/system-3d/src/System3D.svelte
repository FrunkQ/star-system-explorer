<!--
  System 3D (V6): one frame at a time, in perspective. The centre body sits
  at the origin and the bodies orbiting it lie on log-scaled rings. Every
  position is composed in f64 relative to the camera and only then narrowed
  for the GPU (handover/rendering-rules.md). Zooming in on a body with moons
  hands over to its frame; zooming out of a planet's frame hands back. Tiers:
  GPU (three.js WebGPU, WebGL2 when the device has no WebGPU), else Canvas
  2D drawing the same projection. The overlay buttons are the keyboard path.
-->
<script lang="ts">
  import { onDestroy } from "svelte";
  import { basis, project, startGpu, tree, type Camera, type Gpu, type Vec3 } from "../../../renderer/src/index.ts";
  import { bandLines, hasChildren, layout, shellDots, type Band3D, type Body3D } from "./bodies.ts";

  let {
    bodies,
    selection,
    onselect,
    title,
    tier = "auto",
    labels = true,
    distanceNote = "",
    bands = [],
  }: { bands?: Band3D[]; bodies: Body3D[]; selection: string | null; onselect: (id: string) => void; title: string; tier?: string; labels?: boolean; distanceNote?: string } = $props();

  const root = $derived(bodies.find((b) => b.parent === null)?.id ?? bodies[0]?.id ?? "");
  let frame = $state<string | null>(null);
  const centreId = $derived(frame && bodies.some((b) => b.id === frame) ? frame : root);
  const lay = $derived(layout(bodies, centreId));
  const t = $derived(lay ? tree(lay.nodes) : tree([]));
  const byId = (id: string | null) => bodies.find((b) => b.id === id);

  // A body chosen elsewhere opens the frame it is drawn in (a moon: its planet's).
  $effect(() => {
    const s = byId(selection);
    if (!s || s.id === centreId) return;
    const p = s.parent;
    if (p && p !== root && p !== centreId) frame = p;
    else if (p === root && centreId !== root && s.id !== centreId) frame = null;
  });

  // Camera: orbiting the centre, in display units of this frame.
  let yaw = $state(-0.6);
  let pitch = $state(0.55);
  let dist = $state(26);
  const MIN = 4;
  const MAX = 60;
  let w = $state(0);
  let h = $state(0);
  const cam = $derived<Camera>({
    frame: centreId,
    eye: [dist * Math.cos(pitch) * Math.cos(yaw), dist * Math.cos(pitch) * Math.sin(yaw), dist * Math.sin(pitch)],
    look: [0, 0, 0],
    fovY: (45 * Math.PI) / 180,
    width: Math.max(1, w),
    height: Math.max(1, h),
  });

  function enter(id: string) {
    frame = id;
    dist = 26;
    onselect(id);
  }
  function leave() {
    const back = centreId;
    frame = byId(centreId)?.parent && byId(centreId)!.parent !== root ? byId(centreId)!.parent : null;
    dist = 26;
    onselect(back);
  }
  // Rule 4: zoom past what the frame carries hands over to the chosen body's frame, or back up.
  function zoomBy(f: number) {
    const next = dist * f;
    const sel = byId(selection);
    if (next < MIN && sel && sel.id !== centreId && hasChildren(bodies, sel.id)) return enter(sel.id);
    if (next > MAX && centreId !== root) return leave();
    dist = Math.min(MAX, Math.max(MIN, next));
  }

  // Ink from the theme: marks without a colour draw in it.
  let host: HTMLDivElement | undefined = $state();
  // Theme tokens, read at draw time; the computed text colour stands in for a token a theme leaves out.
  const token = (name: string, fallback?: string) => {
    if (!host) return "";
    const cs = getComputedStyle(host);
    return cs.getPropertyValue(name).trim() || (fallback ? cs.getPropertyValue(fallback).trim() : "") || cs.color;
  };
  const sizeOf = (b: Body3D, centre: boolean) => (centre ? 0.55 : 0.13 * Math.max(0.6, (b.mark?.size ?? 5) / 5));

  // Camera-relative positions, f64 until the last step (frames.ts).
  const SEG = 96;
  function ringPoints(r: number): Vec3[] {
    const out: Vec3[] = [];
    for (let i = 0; i <= SEG; i++) {
      const a = (i / SEG) * 2 * Math.PI;
      out.push([Math.fround(r * Math.cos(a) - cam.eye[0]), Math.fround(r * Math.sin(a) - cam.eye[1]), Math.fround(-cam.eye[2])]);
    }
    return out;
  }
  const relOf = (id: string): Vec3 => {
    const n = lay!.nodes.find((x) => x.id === id)!;
    return [Math.fround(n.local[0] - cam.eye[0]), Math.fround(n.local[1] - cam.eye[1]), Math.fround(n.local[2] - cam.eye[2])];
  };
  const shown = $derived(lay ? [lay.centre, ...lay.placed] : []);
  // Belts and the Oort cloud belong to the star's frame; a sphere far out is drawn just past the outermost orbit.
  const bandLocal = $derived(lay && centreId === root ? bands.flatMap((b) => bandLines(b, lay!.dist, 11.5, SEG).map((pts) => ({ b, pts }))) : []);
  const shellLocal = $derived(lay && centreId === root ? bands.map((b) => ({ b, pts: shellDots(b, lay!.dist, 14) })).filter((x) => x.pts.length) : []);
  const rel = (p: [number, number, number]): Vec3 => [Math.fround(p[0] - cam.eye[0]), Math.fround(p[1] - cam.eye[1]), Math.fround(p[2] - cam.eye[2])];
  const pins = $derived(shown.map((b) => ({ b, p: project(t, cam, b.id) })));

  let canvas: HTMLCanvasElement | undefined = $state();
  let gpu = $state<Gpu | null>(null);
  let using = $state<"gpu" | "canvas" | "">("");
  let reason = $state("");
  $effect(() => {
    if (!canvas || using) return;
    const want = tier;
    if (want === "canvas") {
      using = "canvas";
      reason = "canvas tier chosen";
      return;
    }
    void startGpu(canvas).then((g) => {
      if (g) {
        gpu = g;
        using = "gpu";
        reason = g.backend === "webgpu" ? "WebGPU" : "WebGL2 (no WebGPU here)";
      } else {
        using = "canvas";
        reason = "no GPU path on this device";
      }
    });
  });
  onDestroy(() => gpu?.dispose());

  let canvas2d: HTMLCanvasElement | undefined = $state();
  $effect(() => {
    if (!lay || w === 0 || h === 0) return;
    const ink = token("--sse-ink");
    const faint = token("--sse-ink-faint", "--sse-ink-muted");
    const accent = token("--sse-accent");
    // A shell's haze is faint ink: it surrounds everything and must not crowd the orbits.
    const haze = token("--sse-ink-faint", "--sse-ink-muted");
    const ground = token("--sse-ground", "--sse-ground-raised");
    const colour = (b: Body3D) => b.mark?.colour ?? ink;
    if (using === "gpu" && gpu) {
      const { f, u } = basis(cam.eye, cam.look);
      gpu.draw({
        items: shown.map((b) => ({ id: b.id, rel: relOf(b.id), radius: sizeOf(b, b === lay!.centre), colour: colour(b), ring: b.mark?.ring ?? false, selected: b.id === selection })),
        rings: [
          ...bandLocal.map(({ b, pts }) => ({ points: pts.map(rel), colour: b.id === selection ? accent : faint })),
          ...lay.placed.map((b) => ({ points: ringPoints(lay!.ring.get(b.id)!), colour: b.id === selection ? accent : faint })),
        ],
        dots: shellLocal.map(({ b, pts }) => ({ points: pts.map(rel), colour: b.id === selection ? accent : haze })),
        forward: f,
        up: u,
        fovY: cam.fovY,
        width: w,
        height: h,
        ground,
      });
    } else if (using === "canvas" && canvas2d) {
      const dpr = typeof devicePixelRatio === "number" ? devicePixelRatio : 1;
      canvas2d.width = Math.round(w * dpr);
      canvas2d.height = Math.round(h * dpr);
      const g = canvas2d.getContext("2d")!;
      g.setTransform(dpr, 0, 0, dpr, 0, 0);
      g.clearRect(0, 0, w, h);
      // Belts and the Oort cloud first, dotted and faint, under the orbits.
      g.setLineDash([2, 4]);
      for (const { b, pts } of bandLocal) {
        g.beginPath();
        let started = false;
        for (const q of pts) {
          const p = project(t, cam, lay.centre.id, q);
          if (!p) continue;
          if (started) g.lineTo(p.x, p.y);
          else (g.moveTo(p.x, p.y), (started = true));
        }
        g.strokeStyle = b.id === selection ? accent : faint;
        g.lineWidth = 1;
        g.stroke();
      }
      g.setLineDash([]);
      for (const { b, pts } of shellLocal) {
        g.fillStyle = b.id === selection ? accent : haze;
        for (const q of pts) {
          const p = project(t, cam, lay.centre.id, q);
          if (p) g.fillRect(p.x - 0.75, p.y - 0.75, 1.5, 1.5);
        }
      }
      // Orbits as projected polylines, in the same camera-relative arithmetic.
      for (const b of lay.placed) {
        const r = lay.ring.get(b.id)!;
        g.beginPath();
        let started = false;
        for (let i = 0; i <= SEG; i++) {
          const a = (i / SEG) * 2 * Math.PI;
          const p = project(t, cam, lay.centre.id, [r * Math.cos(a), r * Math.sin(a), 0]);
          if (!p) continue;
          if (started) g.lineTo(p.x, p.y);
          else (g.moveTo(p.x, p.y), (started = true));
        }
        g.strokeStyle = b.id === selection ? accent : faint;
        g.lineWidth = b.id === selection ? 2 : 1;
        g.stroke();
      }
      const k = h / 2 / Math.tan(cam.fovY / 2);
      for (const { b, p } of [...pins].sort((x, y) => (y.p?.depth ?? 0) - (x.p?.depth ?? 0))) {
        if (!p) continue;
        g.beginPath();
        g.arc(p.x, p.y, Math.max(2, (sizeOf(b, b === lay.centre) * k) / p.depth), 0, 2 * Math.PI);
        g.fillStyle = colour(b);
        g.fill();
      }
    }
  });

  // Dragging turns the camera about the centre.
  let dragFrom: { x: number; y: number; yaw: number; pitch: number } | null = null;
  function down(e: PointerEvent) {
    if ((e.target as HTMLElement).closest("button")) return;
    dragFrom = { x: e.clientX, y: e.clientY, yaw, pitch };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    if (!dragFrom) return;
    yaw = dragFrom.yaw - (e.clientX - dragFrom.x) * 0.008;
    pitch = Math.min(1.45, Math.max(-1.45, dragFrom.pitch + (e.clientY - dragFrom.y) * 0.008));
  }
  function wheel(e: WheelEvent) {
    e.preventDefault();
    zoomBy(e.deltaY > 0 ? 1.12 : 1 / 1.12);
  }
  function key(e: KeyboardEvent) {
    const k = e.key;
    if (k === "ArrowLeft") yaw -= 0.12;
    else if (k === "ArrowRight") yaw += 0.12;
    else if (k === "ArrowUp") pitch = Math.min(1.45, pitch + 0.1);
    else if (k === "ArrowDown") pitch = Math.max(-1.45, pitch - 0.1);
    else if (k === "+" || k === "=") zoomBy(1 / 1.25);
    else if (k === "-") zoomBy(1.25);
    else if (k === "Escape" && centreId !== root) leave();
    else return;
    e.preventDefault();
  }
  const sel = $derived(byId(selection));
  const canEnter = $derived(sel && sel.id !== centreId && sel.parent === centreId && hasChildren(bodies, sel.id) ? sel : undefined);
</script>

<!-- The canvas area takes drag, wheel and arrow keys to turn the camera; the body buttons inside are the controls. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  class="s3d"
  bind:this={host}
  bind:clientWidth={w}
  bind:clientHeight={h}
  role="application"
  aria-roledescription="3D chart"
  aria-label={centreId === root ? `3D chart of ${title}` : `3D chart of ${byId(centreId)?.name} and its moons`}
  tabindex="0"
  onpointerdown={down}
  onpointermove={move}
  onpointerup={() => (dragFrom = null)}
  onpointercancel={() => (dragFrom = null)}
  onwheel={wheel}
  onkeydown={key}
  data-tier={using}
  data-frame={centreId}
>
  <canvas bind:this={canvas} class="gpu" class:off={using !== "gpu"} aria-hidden="true"></canvas>
  <canvas bind:this={canvas2d} class="c2d" class:off={using !== "canvas"} aria-hidden="true"></canvas>
  {#each pins as { b, p } (b.id)}
    {#if p && p.x > -40 && p.x < w + 40 && p.y > -40 && p.y < h + 40}
      <button
        type="button"
        class="pin"
        class:sel={b.id === selection}
        style="left: {p.x}px; top: {p.y}px; --r: {Math.max(7, (sizeOf(b, b === lay?.centre) * (b.mark?.ring ? 1.9 : 1) * (h / 2 / Math.tan(cam.fovY / 2))) / p.depth)}px"
        aria-pressed={b.id === selection}
        data-body={b.id}
        data-x={p.x.toFixed(2)}
        data-y={p.y.toFixed(2)}
        onclick={() => onselect(b.id)}
        ondblclick={() => hasChildren(bodies, b.id) && b.id !== centreId && enter(b.id)}
      ><span class="dot"></span>{#if labels}<span class="name">{b.name}</span>{/if}</button>
    {/if}
  {/each}
  <div class="bar">
    {#if centreId !== root}<button type="button" onclick={leave}>‹ {byId(root)?.name ?? title}</button><span>Moons of <b>{byId(centreId)?.name}</b></span>{/if}
    {#if canEnter}<button type="button" onclick={() => enter(canEnter.id)}>Moons of {canEnter.name} ›</button>{/if}
  </div>
  <div class="zoom" role="group" aria-label="Zoom">
    <button type="button" aria-label="Zoom in" onclick={() => zoomBy(1 / 1.25)}>+</button>
    <button type="button" aria-label="Zoom out" onclick={() => zoomBy(1.25)}>−</button>
  </div>
  <p class="note">
    {#if lay?.unplaced.length}<span>Position not known: {lay.unplaced.map((b) => b.name).join(", ")}.</span>{/if}
    <span>Distances on a log scale, sizes not to scale{distanceNote}. Drawn with {using === "gpu" ? reason : using === "canvas" ? `Canvas 2D (${reason})` : "…"}.</span>
  </p>
</div>

<style>
  .s3d {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 240px;
    overflow: hidden;
    touch-action: none;
    background: var(--sse-ground);
    outline: none;
    cursor: grab;
  }
  .s3d:focus-visible {
    box-shadow: inset 0 0 0 2px var(--sse-focus);
  }
  canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
  .off {
    visibility: hidden;
  }
  .pin {
    position: absolute;
    transform: translate(-50%, -50%);
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px;
    border: 0;
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 14px;
    cursor: pointer;
  }
  .pin .dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
  }
  .pin.sel .dot {
    outline: 1.5px dashed var(--sse-accent);
    outline-offset: 3px;
  }
  .pin:focus-visible .dot {
    outline: 2px dashed var(--sse-focus);
    outline-offset: 3px;
  }
  .pin .name {
    position: absolute;
    left: calc(50% + var(--r) + 6px);
    white-space: nowrap;
    text-shadow: 0 0 3px var(--sse-ground), 0 0 3px var(--sse-ground);
  }
  .bar {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    top: 12px;
    display: flex;
    gap: 10px;
    align-items: center;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .bar b {
    color: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-weight: normal;
  }
  .bar button,
  .zoom button {
    min-height: 36px;
    min-width: 40px;
    padding: 0 10px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    cursor: pointer;
  }
  .zoom {
    position: absolute;
    right: 12px;
    bottom: 12px;
    display: flex;
  }
  .zoom button + button {
    border-left: 0;
  }
  .note {
    position: absolute;
    left: 12px;
    bottom: 8px;
    right: 110px;
    margin: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
</style>
