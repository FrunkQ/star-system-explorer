<!--
  System chart view: the star at the centre, orbits on a log-radial scale,
  zones and belts as bands, unsurveyed territory hatched, and every body a
  keyboard-reachable control. Marks come from the symbology; the view only
  places them.
-->
<script lang="ts">
  import { commitEdit, sig, type Json, type SceneSubject, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { ChartGround, HatchDefs, Hatch, Ink, Mark, Stamp, SurveyGrid, distanceAt, fitScale, polar, radius } from "../../../chart-grammar/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  const prefix = "sc";
  let w = $state(0);
  let h = $state(0);
  let zoom = $state(1);
  $effect(() => {
    zoom = Number(params.zoom);
  });

  const narrow = $derived(w < 700);
  const cx = $derived(w * (narrow ? 0.5 : Number(params.centre_x)));
  const cy = $derived(h * 0.5);
  const orbiting = $derived(scene.subjects.filter((s) => s.orbit).sort((a, b) => a.orbit!.a_au - b.orbit!.a_au));
  const bands = $derived(scene.subjects.filter((s) => s.band_au));
  const star = $derived(scene.subjects.find((s) => s.kind === "star" && !s.orbit));
  const scale = $derived(
    fitScale(
      [...orbiting.map((s) => s.orbit!.a_au), ...bands.flatMap((b) => b.band_au!)],
      narrow ? 22 : 34,
      Math.max(60, Math.min(w, h) * (narrow ? 0.47 : 0.44) * zoom),
    ),
  );
  const r = (au: number) => radius(au, drag ? drag.scale : scale);
  const surveyR = $derived(r(scene.surveyed_to_au));
  const unsurveyed = $derived(
    `M0 0H${w}V${h}H0Z M${cx - surveyR} ${cy}a${surveyR} ${surveyR} 0 1 0 ${surveyR * 2} 0a${surveyR} ${surveyR} 0 1 0 ${-surveyR * 2} 0Z`,
  );
  const decades = $derived([0.1, 1, 10, 100].filter((a) => a > scale.inner_au && a < scene.surveyed_to_au));

  const label = (s: SceneSubject) => s.label;

  // Dragging a body along its radius moves its orbit; the edit is an intent
  // sent when the drag ends. The scale is frozen during the drag.
  let drag = $state<{ address: string; a_au: number; moved: boolean; scale: typeof scale } | null>(null);
  let problem = $state<string | null>(null);
  const editable = (s: SceneSubject) => ctx.workspace.mode === "edit" && !!s.orbit && !!s.facts.find((f) => f.field === "a_au" && f.editable);
  const aOf = (s: SceneSubject) => (drag && drag.address === s.address ? drag.a_au : s.orbit!.a_au);
  const rr = (au: number) => radius(au, drag ? drag.scale : scale);
  const pos = (s: SceneSubject): [number, number] => polar(cx, cy, rr(aOf(s)), s.orbit!.phase_deg);
  let svgEl: SVGSVGElement | undefined = $state();

  function round(a: number): number {
    return Number(sig(a, 3));
  }

  function startDrag(e: PointerEvent, s: SceneSubject) {
    if (!editable(s) || e.button !== 0) return;
    svgEl!.setPointerCapture(e.pointerId);
    drag = { address: s.address, a_au: s.orbit!.a_au, moved: false, scale };
  }

  function moveDrag(e: PointerEvent) {
    if (!drag) return;
    const box = svgEl!.getBoundingClientRect();
    const d = Math.hypot(e.clientX - box.left - cx, e.clientY - box.top - cy);
    const a = round(Math.min(scene.surveyed_to_au, Math.max(drag.scale.inner_au, distanceAt(d, drag.scale))));
    if (a !== drag.a_au) drag = { ...drag, a_au: a, moved: true };
  }

  async function endDrag() {
    const d = drag;
    if (!d) return;
    const s = scene.subjects.find((x) => x.address === d.address)!;
    if (!d.moved || d.a_au === s.orbit!.a_au) {
      drag = null;
      ctx.workspace.select(d.address);
      return;
    }
    await moveOrbit(s, d.a_au);
    drag = null;
  }

  async function moveOrbit(s: SceneSubject, a: number) {
    const f = s.facts.find((x) => x.field === "a_au")!;
    const r = await commitEdit(ctx.session, ctx.workspace, {
      address: s.address,
      field: "a_au",
      value: a,
      display: `${a} AU`,
      before: { value: f.value, display: f.display },
    });
    problem = r.status === "applied" ? null : r.status === "pending" ? "Sent to the GM to approve." : r.status === "refused" ? r.reason : r.message;
    ctx.workspace.select(s.address);
  }
  const showLabel = (s: SceneSubject) =>
    params.labels === "all" || (params.labels === "selected" && ctx.workspace.selection === s.address);

  const focusables: SVGGElement[] = $state([]);
  function onkey(e: KeyboardEvent, i: number, s: SceneSubject) {
    if ((e.key === "PageUp" || e.key === "PageDown") && editable(s)) {
      e.preventDefault();
      void moveOrbit(s, round(s.orbit!.a_au * (e.key === "PageUp" ? 1.05 : 1 / 1.05)));
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      ctx.workspace.select(s.address);
    } else if (["ArrowRight", "ArrowDown", "ArrowLeft", "ArrowUp"].includes(e.key)) {
      e.preventDefault();
      const n = focusables.length;
      const next = e.key === "ArrowRight" || e.key === "ArrowDown" ? (i + 1) % n : (i - 1 + n) % n;
      focusables[next]?.focus();
    }
  }
  const all = $derived(star ? [star, ...orbiting] : orbiting);
  const zoomBy = (f: number) => (zoom = Math.min(6, Math.max(0.5, zoom * f)));
</script>

<div class="chart" bind:clientWidth={w} bind:clientHeight={h}>
  <ChartGround>
    {#if w > 0 && h > 0}
      <svg
        bind:this={svgEl}
        width={w}
        height={h}
        viewBox="0 0 {w} {h}"
        role="group"
        aria-label="System chart of {scene.title}"
        onpointermove={moveDrag}
        onpointerup={endDrag}
        onpointercancel={() => (drag = null)}
        class:dragging={drag !== null}
      >
        <HatchDefs {prefix} />
        <SurveyGrid width={w} height={h} step={Number(params.grid_step)} labels={params.grid_labels === true && !narrow} />
        {#if params.unsurveyed_hatch}
          <Hatch {prefix} d={unsurveyed} pattern="hatch" opacity={0.5} />
          <Ink circle={{ cx, cy, r: surveyR }} tone="muted" dash="dashed" />
        {/if}
        {#each bands as b (b.address)}
          {@const r0 = r(b.band_au![0])}
          {@const r1 = r(b.band_au![1])}
          {#if b.kind === "zone"}
            <Ink circle={{ cx, cy, r: (r0 + r1) / 2 }} tone="zone" width={r1 - r0} />
          {:else}
            <Ink circle={{ cx, cy, r: (r0 + r1) / 2 }} tone="muted" dash="belt" />
          {/if}
        {/each}
        {#each decades as a (a)}
          <text class="dist" x={cx + 4} y={cy - r(a) - 4} aria-hidden="true">{a} AU</text>
        {/each}
        {#each orbiting as s (s.address)}
          {@const sel = ctx.workspace.selection === s.address}
          {#if drag?.address === s.address}
            <Ink circle={{ cx, cy, r: rr(s.orbit!.a_au) }} tone="faint" dash="dashed" />
            <Ink circle={{ cx, cy, r: rr(drag.a_au) }} tone="accent" weight="heavy" dash="dashed" />
          {:else}
            <Ink circle={{ cx, cy, r: rr(s.orbit!.a_au) }} tone={sel ? "accent" : "faint"} weight={sel ? "heavy" : "line"} />
          {/if}
        {/each}
        {#each bands as b (b.address)}
          {@const [lx, ly] = polar(cx, cy, r(b.band_au![1]) + 10, b.kind === "zone" ? -62 : 58)}
          <text class="band {b.kind}" x={lx} y={ly} aria-hidden="true">{label(b)}</text>
        {/each}
        {#each all as s, i (s.address)}
          {@const [x, y] = s.orbit ? pos(s) : [cx, cy]}
          {@const m = ctx.mark(s)}
          {@const sel = ctx.workspace.selection === s.address}
          <g
            bind:this={focusables[i]}
            class="body"
            role="button"
            tabindex="0"
            aria-label="{label(s)}{s.subtitle ? `, ${s.subtitle}` : ''}"
            aria-pressed={sel}
            aria-describedby={editable(s) ? "chart-drag-hint" : undefined}
            onpointerdown={(e) => (editable(s) ? startDrag(e, s) : ctx.workspace.select(s.address))}
            onkeydown={(e) => onkey(e, i, s)}
          >
            <circle class="hit" cx={x} cy={y} r={Math.max(22, m.size + 12)} />
            <Mark mark={m} cx={x} cy={y} {prefix} />
            {#if sel}<circle class="sel" cx={x} cy={y} r={m.size * (m.shape === "star" ? 3.8 : 2) + 6} />{/if}
            {#if drag?.address === s.address}
              <text class="dragv" x={x + m.size + 9} y={y + m.size + 16}>{drag.a_au} AU</text>
            {/if}
            {#if showLabel(s) && s.orbit}
              {@const left = x > w - 110}
              <text class="name" class:selname={sel} x={left ? x - m.size - 9 : x + m.size + 9} y={y - m.size - 4} text-anchor={left ? "end" : "start"}>{label(s)}</text>
            {/if}
          </g>
        {/each}
      </svg>
      <p id="chart-drag-hint" class="sr">Drag along the radius, or press Page Up or Page Down, to move the orbit.</p>
      {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
      <div class="legend">
        {#if scene.fixture}<Stamp kind="fixture" title="The sample system's inputs are fixture data; every derived value comes from the kernel's base-physics pack." />{/if}
        <span>Distances on a log scale</span>
        {#if params.unsurveyed_hatch}<span>Hatched: unsurveyed beyond {scene.surveyed_to_au} AU</span>{/if}
      </div>
      <div class="zoom" role="group" aria-label="Zoom">
        <button type="button" aria-label="Zoom in" onclick={() => zoomBy(1.25)}>+</button>
        <button type="button" aria-label="Zoom out" onclick={() => zoomBy(0.8)}>−</button>
        <button type="button" aria-label="Fit the system" onclick={() => (zoom = Number(params.zoom))}>Fit</button>
      </div>
    {/if}
  </ChartGround>
</div>

<style>
  .chart {
    width: 100%;
    height: 100%;
  }
  svg {
    display: block;
    font-family: var(--sse-font-body);
  }
  .body {
    cursor: pointer;
    outline: none;
  }
  .hit {
    fill: transparent;
  }
  .dragging,
  .dragging .body {
    cursor: grabbing;
  }
  .dragv {
    fill: var(--sse-accent-ink);
    font-family: var(--sse-font-mono);
    font-size: 12px;
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 4px;
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
  .problem {
    position: absolute;
    left: 24px;
    top: 16px;
    margin: 0;
    padding: 6px 10px;
    border: var(--sse-line) solid var(--sse-accent-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-accent-ink);
    font-size: 13px;
  }
  .body:focus-visible .hit {
    stroke: var(--sse-focus);
    stroke-width: 2;
    stroke-dasharray: 3 3;
  }
  .sel {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 1.2;
    stroke-dasharray: 2 3;
  }
  .name {
    fill: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 16px;
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 4px;
  }
  .selname {
    font-size: 18px;
  }
  .band {
    fill: var(--sse-ink-muted);
    font-size: 11px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 4px;
  }
  .band.zone {
    fill: var(--sse-zone-ink);
  }
  .dist {
    fill: var(--sse-ink-muted);
    font-family: var(--sse-font-mono);
    font-size: 10px;
  }
  .legend {
    position: absolute;
    left: 24px;
    bottom: 30px;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 14px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .zoom {
    position: absolute;
    right: 16px;
    bottom: 16px;
    display: flex;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
  }
  .zoom button {
    min-width: 44px;
    height: 40px;
    border: 0;
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 15px;
    cursor: pointer;
  }
  .zoom button + button {
    border-left: var(--sse-line) solid var(--sse-ink);
  }
  @media (max-width: 760px) {
    .legend {
      left: 16px;
      right: 160px;
      bottom: 12px;
      font-size: 11px;
    }
    .zoom {
      bottom: 12px;
      right: 12px;
    }
  }
</style>
