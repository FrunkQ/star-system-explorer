<!--
  System 3D (V6): one frame at a time, in perspective. The centre body sits
  at the origin and the bodies orbiting it lie on log-scaled rings. Every
  position is composed in f64 relative to the camera and only then narrowed
  for the GPU (handover/rendering-rules.md). Zooming in on a body with moons
  hands over to its frame; zooming out of a planet's frame hands back. Tiers:
  GPU (three.js WebGPU, WebGL2 when the device has no WebGPU), else Canvas
  2D drawing the same projection. The overlay buttons are the keyboard path.
  True (V10a): the same frame against black space, each world wrapped in its
  own surface (given in `surfaces`, read from its tiles) and lit by its star,
  the star in its own colour. HUD (every style, Alex 23 Nov): orbit lines,
  belts, names and locator rings; off, names show on hover only and True
  looks like a photograph. Scale Real: distances and sizes in proportion.
-->
<script lang="ts">
  import { onDestroy } from "svelte";
  import { basis, project, startGpu, tree, type Camera, type Gpu, type GpuLight, type TrueTexture, type Vec3 } from "../../../renderer/src/index.ts";
  import { SEEN_ELEVATION, type SeenFrom } from "../../../ui-contract/src/index.ts";
  import { bandLines, hasChildren, layout, layoutReal, shellDots, type Band3D, type Body3D } from "./bodies.ts";

  let {
    bodies,
    selection,
    onselect,
    title,
    tier = "auto",
    labels = true,
    distanceNote = "",
    bands = [],
    seen = "free",
    onturned,
    trueLook = false,
    surfaces = new Map(),
    starColour = "",
    trueNote = "",
    realScale = false,
    hud = true,
  }: {
    bands?: Band3D[];
    bodies: Body3D[];
    selection: string | null;
    onselect: (id: string) => void;
    title: string;
    tier?: string;
    labels?: boolean;
    distanceNote?: string;
    seen?: SeenFrom;
    onturned?: () => void;
    /** The True view style (V10a); the GPU tier draws it, the canvas tier keeps the plain look. */
    trueLook?: boolean;
    /** True look: each world's surface by body id, as far as it has been read. */
    surfaces?: ReadonlyMap<string, TrueTexture>;
    /** True look: the star's colour (its surface temperature's, as a CSS colour). */
    starColour?: string;
    /** True look: what is still being read, said under the chart. */
    trueNote?: string;
    /** Scale: Real (Alex, 23 Nov 2026): distances in proportion and every body at its true size on that scale. */
    realScale?: boolean;
    /** HUD (True only): orbit lines, names and locator rings over the picture; off, it looks like a photograph. */
    hud?: boolean;
  } = $props();
  const overlay = $derived(hud);
  const isTrue = $derived(trueLook);

  const root = $derived(bodies.find((b) => b.parent === null)?.id ?? bodies[0]?.id ?? "");
  let frame = $state<string | null>(null);
  const centreId = $derived(frame && bodies.some((b) => b.id === frame) ? frame : root);
  const lay = $derived(realScale ? layoutReal(bodies, centreId) : layout(bodies, centreId));
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
  // Slide (rule 11): where the camera looks, in this frame's display units.
  let pan = $state<[number, number]>([0, 0]);
  // "Seen from" (rule 12): a fixed view sets the camera's height and puts the slide back; Free keeps what the hands did.
  $effect(() => {
    const s = seen;
    if (s === "free") return;
    pitch = SEEN_ELEVATION[s];
    pan = [0, 0];
  });
  // Changing the scale changes what a distance means: start the camera again from its usual place.
  let lastScale: boolean | null = null;
  $effect(() => {
    const r = realScale;
    if (lastScale !== null && r !== lastScale) ((dist = 26), (pan = [0, 0]));
    lastScale = r;
  });
  // Real scale lets the camera come much closer: the worlds are small.
  const MIN = $derived(realScale ? 0.3 : 4);
  const MAX = 60;
  let w = $state(0);
  let h = $state(0);
  const cam = $derived<Camera>({
    frame: centreId,
    eye: [pan[0] + dist * Math.cos(pitch) * Math.cos(yaw), pan[1] + dist * Math.cos(pitch) * Math.sin(yaw), dist * Math.sin(pitch)],
    look: [pan[0], pan[1], 0],
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
  // Readable: sizes for reading. Real: the body's own radius on the frame's scale, but never under a pixel or two at
  // the camera's distance, so a world too small to show as a disc still shows as a point of light (fudge register).
  const sizeOf = (b: Body3D, centre: boolean) => {
    if (!realScale) return centre ? 0.55 : 0.13 * Math.max(0.6, (b.mark?.size ?? 5) / 5);
    const n = lay?.nodes.find((x) => x.id === b.id);
    const far = n ? Math.hypot(n.local[0] - cam.eye[0], n.local[1] - cam.eye[1], n.local[2] - cam.eye[2]) : Math.hypot(...cam.eye);
    const px = (2 * Math.tan(cam.fovY / 2) * far) / Math.max(1, h);
    return Math.max((b.parent === null ? 1.5 : 0.75) * px, lay ? lay.dist(b.radius_m ?? 0) : 0);
  };

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

  // The star's light: from the star itself when it is in the frame, else from where it lies as seen from this frame's
  // centre (opposite the centre's own place on its orbit round the star).
  function lightFor(): GpuLight {
    const c = lay!.centre;
    if (c.parent === null) return { at: relOf(c.id) };
    const a = (c.phase_deg * Math.PI) / 180;
    return { from: [-Math.cos(a), -Math.sin(a), 0.15] };
  }

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
      if (isTrue) {
        // True: worlds in their own surfaces, a world not read yet plain and faint; only the selected orbit drawn.
        const tex = (b: Body3D) => surfaces.get(b.id);
        gpu.draw({
          items: shown.map((b) => {
            const t = tex(b);
            const star = b.parent === null;
            return { id: b.id, rel: relOf(b.id), radius: sizeOf(b, b === lay!.centre), colour: star ? starColour || colour(b) : faint, ring: b.mark?.ring ?? false, selected: b.id === selection, shines: star, ...(t ? { surface: { key: b.id, texture: t } } : {}) };
          }),
          rings: hud ? lay.placed.map((b) => ({ points: ringPoints(lay!.ring.get(b.id)!), colour: b.id === selection ? accent : faint })) : [],
          forward: f,
          up: u,
          fovY: cam.fovY,
          width: w,
          height: h,
          ground,
          light: lightFor(),
        });
        return;
      }
      gpu.draw({
        items: shown.map((b) => ({ id: b.id, rel: relOf(b.id), radius: sizeOf(b, b === lay!.centre), colour: colour(b), ring: b.mark?.ring ?? false, selected: b.id === selection })),
        rings: !hud ? [] : [
          ...bandLocal.map(({ b, pts }) => ({ points: pts.map(rel), colour: b.id === selection ? accent : faint })),
          ...lay.placed.map((b) => ({ points: ringPoints(lay!.ring.get(b.id)!), colour: b.id === selection ? accent : faint })),
        ],
        dots: hud ? shellLocal.map(({ b, pts }) => ({ points: pts.map(rel), colour: b.id === selection ? accent : haze })) : [],
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
      // Belts and the Oort cloud first, dotted and faint, under the orbits (all HUD).
      g.setLineDash([2, 4]);
      for (const { b, pts } of hud ? bandLocal : []) {
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
      for (const { b, pts } of hud ? shellLocal : []) {
        g.fillStyle = b.id === selection ? accent : haze;
        for (const q of pts) {
          const p = project(t, cam, lay.centre.id, q);
          if (p) g.fillRect(p.x - 0.75, p.y - 0.75, 1.5, 1.5);
        }
      }
      // Orbits as projected polylines, in the same camera-relative arithmetic.
      for (const b of hud ? lay.placed : []) {
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
  // With a mouse, dragging turns it and says so (the choice becomes Free); Shift-drag slides. On touch, one finger
  // scrolls the page until Free is chosen, so the chart never traps a scroll; in Free, two fingers slide.
  let dragFrom: { x: number; y: number; yaw: number; pitch: number; pan: [number, number]; slide: boolean } | null = null;
  const touches = new Map<number, { x: number; y: number }>();
  function down(e: PointerEvent) {
    if ((e.target as HTMLElement).closest("button")) return;
    if (e.pointerType === "touch") {
      if (seen !== "free") return;
      touches.set(e.pointerId, { x: e.clientX, y: e.clientY });
    }
    dragFrom = { x: e.clientX, y: e.clientY, yaw, pitch, pan: [...pan], slide: e.shiftKey || touches.size >= 2 };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    if (!dragFrom) return;
    const dx = e.clientX - dragFrom.x;
    const dy = e.clientY - dragFrom.y;
    if (Math.abs(dx) + Math.abs(dy) > 3 && seen !== "free") onturned?.();
    if (dragFrom.slide || touches.size >= 2) {
      // Slide along the plane, in the direction the screen moved.
      const k = dist / Math.max(1, h);
      const c = Math.cos(yaw);
      const s2 = Math.sin(yaw);
      pan = [dragFrom.pan[0] + (dx * s2 + dy * c) * k, dragFrom.pan[1] + (-dx * c + dy * s2) * k];
      return;
    }
    yaw = dragFrom.yaw - dx * 0.008;
    pitch = Math.min(1.45, Math.max(-1.45, dragFrom.pitch + dy * 0.008));
  }
  function up(e: PointerEvent) {
    touches.delete(e.pointerId);
    dragFrom = null;
  }
  function wheel(e: WheelEvent) {
    e.preventDefault();
    zoomBy(e.deltaY > 0 ? 1.12 : 1 / 1.12);
  }
  function key(e: KeyboardEvent) {
    const k = e.key;
    if (k.startsWith("Arrow") && seen !== "free") onturned?.();
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
  onpointerup={up}
  onpointercancel={up}
  onwheel={wheel}
  data-seen={seen}
  style:touch-action={seen === "free" ? "none" : "pan-y"}
  onkeydown={key}
  data-tier={using}
  data-frame={centreId}
  data-look={isTrue && using === "gpu" ? "true" : "plain"}
  data-scale={realScale ? "real" : "readable"}
  data-hud={overlay ? "on" : "off"}
  class:true-look={isTrue && using === "gpu"}
>
  <canvas bind:this={canvas} class="gpu" class:off={using !== "gpu"} aria-hidden="true"></canvas>
  <canvas bind:this={canvas2d} class="c2d" class:off={using !== "canvas"} aria-hidden="true"></canvas>
  {#each pins as { b, p } (b.id)}
    {#if p && p.x > -40 && p.x < w + 40 && p.y > -40 && p.y < h + 40}
      <button
        type="button"
        class="pin"
        class:sel={b.id === selection}
        class:locate={realScale && overlay}
        style="left: {p.x}px; top: {p.y}px; --r: {Math.max(7, (sizeOf(b, b === lay?.centre) * (b.mark?.ring ? 1.9 : 1) * (h / 2 / Math.tan(cam.fovY / 2))) / p.depth)}px"
        aria-pressed={b.id === selection}
        data-body={b.id}
        data-x={p.x.toFixed(2)}
        data-y={p.y.toFixed(2)}
        onclick={() => onselect(b.id)}
        ondblclick={() => hasChildren(bodies, b.id) && b.id !== centreId && enter(b.id)}
      ><span class="dot"></span>{#if labels}<span class="name" class:hover={!hud}>{b.name}</span>{/if}</button>
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
    <span>{realScale ? "Real scale: distances and sizes in proportion (a world smaller than a pixel shows as a point)" : "Distances on a log scale, sizes not to scale"}{distanceNote}. Drawn with {using === "gpu" ? reason : using === "canvas" ? `Canvas 2D (${reason})` : "…"}.</span>
    {#if isTrue && (using === "canvas" || trueNote)}<span data-true-note>{using === "canvas" ? "True needs a GPU: this device draws the chart's look." : trueNote}</span>{/if}
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
  /* Real scale: a faint ring finds a world too small to see (its name on hover in True). */
  .pin.locate .dot {
    box-shadow: 0 0 0 1px var(--sse-ink-faint, var(--sse-ink-muted));
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
  /* HUD off: names only on hover or focus. True: text sits on a chip over black space. */
  .pin .name.hover {
    visibility: hidden;
  }
  .pin:hover .name.hover,
  .pin:focus-visible .name.hover {
    visibility: visible;
  }
  .true-look .pin .name,
  .true-look .note span {
    background: var(--sse-ground);
    padding: 0 4px;
    text-shadow: none;
  }
  /* Low in the stage: the view controls run along the top. */
  .bar {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    bottom: 64px;
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
