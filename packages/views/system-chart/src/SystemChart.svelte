<!--
  System chart view: the star at the centre, orbits on a log-radial scale,
  zones and belts as bands, unsurveyed territory hatched, and every body a
  keyboard-reachable control. Marks come from the symbology; the view only
  places them. A planet with moons opens its own frame: the planet at the
  centre and its moons about it (handover/rendering-rules.md), in kilometres.
-->
<script lang="ts">
  import { KM_PER_AU, asPlayers, distanceWords, lightWords, separation, hasLayer, hasTool, childrenOf, commitEdit, frameFor, legendFor, primaryOf, sig, subjectOf, type Json, type SceneSubject, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { System3D, fromSystemScene } from "../../system-3d/src/index.ts";
  import { ChartGround, HatchDefs, Hatch, Ink, Legend, Mark, Stamp, markExtent, SurveyGrid, distanceAt, fitScale, polar, radius } from "../../../chart-grammar/src/index.ts";

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
  const star = $derived(primaryOf(scene));
  // Orbits are local to their parent (handover/rendering-rules.md). The chart shows one frame at a time:
  // the star's (frame null) with its planets, or a planet's with its moons. Choosing a moon anywhere opens
  // its planet's frame; choosing a planet returns to the star's unless that planet is the centre.
  let frame = $state<string | null>(null);
  const framed = $derived(frame ? scene.subjects.find((s) => s.address === frame) : undefined);
  const centre = $derived(framed ?? star);
  const orbiting = $derived(centre ? childrenOf(scene, centre.address) : []);
  const moonsOf = (s: SceneSubject) => childrenOf(scene, s.address).length;
  $effect(() => {
    const sel = ctx.workspace.selection;
    if (frame && !framed) frame = null;
    if (sel && sel !== frame) {
      const f = frameFor(scene, sel);
      if (f !== frame) frame = f;
    }
  });
  function openFrame(s: SceneSubject) {
    frame = s.address;
    zoom = Number(params.zoom);
    ctx.workspace.select(s.address);
  }
  function closeFrame() {
    const back = frame;
    frame = null;
    zoom = Number(params.zoom);
    if (back) ctx.workspace.select(back);
  }
  const bands = $derived(frame ? [] : scene.subjects.filter((s) => s.band_au));
  const scale = $derived(
    fitScale(
      [...orbiting.map((s) => s.orbit!.a_au), ...bands.flatMap((b) => b.band_au!)],
      narrow ? 22 : 34,
      Math.max(60, Math.min(w, h) * (narrow ? 0.47 : 0.44) * zoom),
    ),
  );
  const r = (au: number) => radius(au, drag ? drag.scale : scale);
  const surveyR = $derived(r(scene.surveyed_to_au));
  // How far out a drag may move an orbit: the survey edge in the star's frame, a little past the outermost moon in a planet's.
  const limitAu = $derived(frame ? scale.outer_au * 2 : scene.surveyed_to_au);
  const km = (au: number) => `${sig(au * KM_PER_AU, 3).replace(/\B(?=(\d{3})+(?!\d))/g, ",")} km`;
  const dist = (au: number) => (frame ? km(au) : `${au} AU`);
  const unsurveyed = $derived(
    `M0 0H${w}V${h}H0Z M${cx - surveyR} ${cy}a${surveyR} ${surveyR} 0 1 0 ${surveyR * 2} 0a${surveyR} ${surveyR} 0 1 0 ${-surveyR * 2} 0Z`,
  );
  const decades = $derived(
    frame ? [1e5, 1e6, 1e7].map((k) => k / KM_PER_AU).filter((a) => a > scale.inner_au && a < scale.outer_au) : [0.1, 1, 10, 100].filter((a) => a > scale.inner_au && a < scene.surveyed_to_au),
  );

  const label = (s: SceneSubject) => s.label;
  // The GM's lens (U9) decides which layers are drawn; it never changes the data.
  const L = $derived(ctx.workspace.lens);
  const on = (id: Parameters<typeof hasLayer>[1]) => hasLayer(L, id);
  // What the lens's audience knows: each body's data view, from the kernel's scene for them.
  const known = $derived(ctx.workspace.lensScene ? new Map(ctx.workspace.lensScene.subjects.map((x) => [x.address, x.view])) : null);
  const through = $derived(asPlayers(L) && known !== null);
  const knows = (s: SceneSubject) => !known || s === star || known.has(s.address);
  const knownCount = $derived(known ? scene.subjects.filter((x) => (x.orbit || x.band_au) && known.has(x.address)).length : 0);
  const bodyCount = $derived(scene.subjects.filter((x) => x.orbit || x.band_au).length);
  // The key is generated from the symbology for the bodies on this chart.
  const key = $derived.by(() => {
    const host = ctx.symbology;
    if (!host) return null;
    const m = host.marker();
    const doc = host.registry().find((d) => d.id === m.id);
    return { entries: legendFor(m, { present: scene.subjects.map(subjectOf), names: host.names(), order: Array.isArray(doc?.legend) ? doc.legend : [] }), note: m.collapse, name: doc?.name ?? m.id };
  });

  // Dragging a body along its radius moves its orbit; the edit is an intent
  // sent when the drag ends. The scale is frozen during the drag.
  let drag = $state<{ address: string; a_au: number; moved: boolean; scale: typeof scale } | null>(null);
  let problem = $state<string | null>(null);
  const editable = (s: SceneSubject) => ctx.workspace.mode === "edit" && hasTool(L, "drag") && !!s.orbit && !!s.facts.find((f) => f.field === "a_au" && f.editable);
  const aOf = (s: SceneSubject) => (drag && drag.address === s.address ? drag.a_au : s.orbit!.a_au);
  const rr = (au: number) => radius(au, drag ? drag.scale : scale);
  const pos = (s: SceneSubject): [number, number] => polar(cx, cy, rr(aOf(s)), s.orbit!.phase_deg);
  let svgEl: SVGSVGElement | undefined = $state();

  // Rounded to three figures in the frame's own unit (AU for planets, km for moons), held in AU.
  function round(a: number): number {
    return frame ? Number(sig(a * KM_PER_AU, 3)) / KM_PER_AU : Number(sig(a, 3));
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
    const a = round(Math.min(limitAu, Math.max(drag.scale.inner_au, distanceAt(d, drag.scale))));
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
      display: dist(a),
      before: { value: f.value, display: f.display },
    });
    problem = r.status === "applied" ? null : r.status === "pending" ? "Sent to the GM to approve." : r.status === "refused" ? r.reason : r.message;
    ctx.workspace.select(s.address);
  }
  const showLabel = (s: SceneSubject) =>
    on("labels") && (params.labels === "all" || (params.labels === "selected" && ctx.workspace.selection === s.address));

  const focusables: SVGGElement[] = $state([]);
  function onkey(e: KeyboardEvent, i: number, s: SceneSubject) {
    if ((e.key === "PageUp" || e.key === "PageDown") && editable(s)) {
      e.preventDefault();
      void moveOrbit(s, round(s.orbit!.a_au * (e.key === "PageUp" ? 1.05 : 1 / 1.05)));
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      if (measuring) measurePick(s);
      else ctx.workspace.select(s.address);
    } else if (e.key === "Escape" && frame) {
      e.preventDefault();
      closeFrame();
    } else if (["ArrowRight", "ArrowDown", "ArrowLeft", "ArrowUp"].includes(e.key)) {
      e.preventDefault();
      const n = focusables.length;
      const next = e.key === "ArrowRight" || e.key === "ArrowDown" ? (i + 1) % n : (i - 1 + n) % n;
      focusables[next]?.focus();
    }
  }
  const all = $derived(centre ? [centre, ...orbiting] : orbiting);
  const selected = $derived(scene.subjects.find((s) => s.address === ctx.workspace.selection));
  const canOpen = $derived(!frame && selected && selected !== star && moonsOf(selected) > 0 ? selected : undefined);
  // V6: the same frame in 3D (view only; edits stay on the 2D chart).
  let dims = $state<"2d" | "3d">("2d");
  const bodies3d = $derived(dims === "3d" ? fromSystemScene(scene, ctx.mark) : []);
  // Measure (V14): pick two bodies; the chart draws the line and says how far apart they are now.
  let measuring = $state(false);
  let picks = $state<string[]>([]);
  function measurePick(s: SceneSubject) {
    picks = picks.length >= 2 || picks.includes(s.address) ? [s.address] : [...picks, s.address];
  }
  function toggleMeasure() {
    measuring = !measuring;
    picks = [];
  }
  const sep = $derived(picks.length === 2 ? separation(scene, picks[0]!, picks[1]!) : null);
  const at = (address: string): [number, number] | null => {
    const s = all.find((x) => x.address === address);
    if (!s) return null;
    return s === centre ? [cx, cy] : pos(s);
  };
  const zoomBy = (f: number) => (zoom = Math.min(6, Math.max(0.5, zoom * f)));
  // Hover peek (dynamic minimalism): with a mouse, resting on a body shows its
  // name and kind beside it; the card is one click away. Not on touch, and not
  // for the body already selected, which the card already describes.
  let peek = $state<{ address: string; x: number; y: number; r: number } | null>(null);
  const peeked = $derived(peek && peek.address !== ctx.workspace.selection && !drag && !measuring ? all.find((s) => s.address === peek!.address) : undefined);
</script>

<div class="chart" bind:clientWidth={w} bind:clientHeight={h}>
  {#if dims === "3d"}
    <System3D bodies={bodies3d} selection={ctx.workspace.selection} onselect={(id) => ctx.workspace.select(id)} title={scene.title} />
  {:else}
  <ChartGround>
    {#if w > 0 && h > 0}
      <svg
        bind:this={svgEl}
        width={w}
        height={h}
        viewBox="0 0 {w} {h}"
        role="group"
        aria-label={framed ? `Chart of ${framed.label} and its moons` : `System chart of ${scene.title}`}
        onpointermove={moveDrag}
        onpointerup={endDrag}
        onpointercancel={() => (drag = null)}
        class:dragging={drag !== null}
      >
        <HatchDefs {prefix} />
        {#if on("grid")}<SurveyGrid width={w} height={h} step={Number(params.grid_step)} labels={params.grid_labels === true && !narrow} />{/if}
        {#if params.unsurveyed_hatch && on("unsurveyed") && !frame}
          <Hatch {prefix} d={unsurveyed} pattern="hatch" opacity={0.5} />
          <Ink circle={{ cx, cy, r: surveyR }} tone="muted" dash="dashed" />
        {/if}
        {#each on("zones") ? bands : [] as b (b.address)}
          {@const r0 = r(b.band_au![0])}
          {@const r1 = r(b.band_au![1])}
          {#if b.kind === "zone"}
            <Ink circle={{ cx, cy, r: (r0 + r1) / 2 }} tone="zone" width={r1 - r0} />
          {:else}
            <Ink circle={{ cx, cy, r: (r0 + r1) / 2 }} tone="muted" dash="belt" />
          {/if}
        {/each}
        {#each on("distances") ? decades : [] as a (a)}
          <text class="dist" x={cx + 4} y={cy - r(a) - 4} aria-hidden="true">{dist(a)}</text>
        {/each}
        {#each orbiting as s (s.address)}
          {@const sel = ctx.workspace.selection === s.address}
          {#if drag?.address === s.address}
            <Ink circle={{ cx, cy, r: rr(s.orbit!.a_au) }} tone="faint" dash="dashed" />
            <Ink circle={{ cx, cy, r: rr(drag.a_au) }} tone="accent" weight="heavy" dash="dashed" />
          {:else if on("orbits") || sel}
            <Ink circle={{ cx, cy, r: rr(s.orbit!.a_au) }} tone={sel ? "accent" : "faint"} weight={sel ? "heavy" : "line"} />
          {/if}
        {/each}
        {#each on("zones") && on("labels") ? bands : [] as b (b.address)}
          {@const [lx, ly] = polar(cx, cy, r(b.band_au![1]) + 10, b.kind === "zone" ? -62 : 58)}
          <text class="band {b.kind}" x={lx} y={ly} aria-hidden="true">{label(b)}</text>
        {/each}
        {#each all as s, i (s.address)}
          {@const atCentre = s === centre}
          {@const [x, y] = atCentre ? [cx, cy] : pos(s)}
          {@const m0 = ctx.mark(s)}
          {@const m = atCentre && frame ? { ...m0, size: m0.size * 2.2 } : m0}
          {@const moons = !frame && !atCentre ? moonsOf(s) : 0}
          {@const view = known?.get(s.address)}
          {@const sel = ctx.workspace.selection === s.address}
          <g
            bind:this={focusables[i]}
            class="body"
            data-address={s.address}
            class:unknown={through && !knows(s)}
            data-known={known ? (knows(s) ? (view ?? "yes") : "no") : undefined}
            role="button"
            tabindex="0"
            aria-label="{label(s)}{s.subtitle ? `, ${s.subtitle}` : ''}{through && !knows(s) ? ', not known to them' : ''}"
            aria-pressed={sel}
            aria-describedby={editable(s) ? "chart-drag-hint" : undefined}
            onpointerdown={(e) => (measuring ? measurePick(s) : editable(s) && !atCentre ? startDrag(e, s) : ctx.workspace.select(s.address))}
            ondblclick={() => moons > 0 && openFrame(s)}
            onpointerenter={(e) => e.pointerType === "mouse" && (peek = { address: s.address, x, y, r: markExtent(m) })}
            onpointerleave={() => peek?.address === s.address && (peek = null)}
            onkeydown={(e) => onkey(e, i, s)}
            data-moons={moons || undefined}
          >
            <circle class="hit" cx={x} cy={y} r={Math.max(22, m.size + 12)} />
            <Mark mark={m} cx={x} cy={y} {prefix} />
            {#each { length: on("moons") ? Math.min(moons, 4) : 0 } as _, k (k)}
              {@const [mx, my] = polar(x, y, markExtent(m) + 5, 200 + k * 28)}
              <circle class="moontick" cx={mx} cy={my} r="1.6" aria-hidden="true" />
            {/each}
            {#if sel}<circle class="sel" cx={x} cy={y} r={Math.max(markExtent(m) + 4, m.size * 2 + 6)} />{/if}
            {#if drag?.address === s.address}
              <text class="dragv" x={x + m.size + 9} y={y + m.size + 16}>{dist(drag.a_au)}</text>
            {/if}
            {#if showLabel(s) && (s.orbit || atCentre) && !(atCentre && !frame)}
              {@const left = x > w - 110}
              <text class="name" class:selname={sel} x={left ? x - m.size - 9 : x + m.size + 9} y={y - m.size - 4} text-anchor={left ? "end" : "start"}>{label(s)}</text>
              {#if on("known") && known && !atCentre}
                <text class="knownv" x={left ? x - m.size - 9 : x + m.size + 9} y={y - m.size + 9} text-anchor={left ? "end" : "start"} aria-hidden="true">{view ? view.replace(/-/g, " ") : "not known to the table"}</text>
              {/if}
            {/if}
          </g>
        {/each}
        {#if measuring && picks.length}
          {@const p0 = at(picks[0]!)}
          {@const p1 = picks[1] ? at(picks[1]) : null}
          {#if p0}<circle class="mpick" cx={p0[0]} cy={p0[1]} r="14" />{/if}
          {#if p1}<circle class="mpick" cx={p1[0]} cy={p1[1]} r="14" />{/if}
          {#if p0 && p1}<line class="mline" x1={p0[0]} y1={p0[1]} x2={p1[0]} y2={p1[1]} />{/if}
        {/if}
      </svg>
      {#if peeked && peek}
        {#key peek.address}
          <div class="peek" class:up={peek.y > h - 90} style="left: {peek.x}px; top: {peek.y}px; --off: {peek.r + 14}px" aria-hidden="true" data-peek={peek.address}>
            <b>{label(peeked)}</b>{#if peeked.subtitle}<span>{peeked.subtitle}</span>{/if}
          </div>
        {/key}
      {/if}
      {#if measuring}
        <div class="measure" role="status" aria-label="Measurement" data-measure={sep ? sep.now_au : picks.length}>
          {#if sep}
            <b>{sep.from} to {sep.to}</b>
            <span>Now {distanceWords(sep.now_au)}, {lightWords(sep.light_s)}.</span>
            {#if sep.range_au && sep.range_au[0] !== sep.range_au[1]}<span>Closest {distanceWords(sep.range_au[0])}, farthest {distanceWords(sep.range_au[1])}.</span>{/if}
            <span class="fine">From the orbits as drawn: circular, in one plane, at the ruler's time. Distances on the chart are on a log scale; these are true.</span>
          {:else if picks.length === 1}
            <span>Now pick a second body.</span>
          {:else}
            <span>Measure: pick two bodies on the chart.</span>
          {/if}
        </div>
      {/if}
      {#if framed}
        <div class="frame" role="group" aria-label="Chart frame">
          <button type="button" onclick={closeFrame}>‹ {scene.title}</button>
          <span>Moons of <b>{framed.label}</b>, distances from its centre</span>
        </div>
      {:else if canOpen}
        <div class="frame">
          <button type="button" onclick={() => openFrame(canOpen)}>Moons of {canOpen.label} ›</button>
        </div>
      {/if}
      <p id="chart-drag-hint" class="sr">Drag along the radius, or press Page Up or Page Down, to move the orbit.</p>
      {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
      <div class="legend">
        {#if scene.fixture}<Stamp kind="fixture" title="The sample system's inputs are fixture data; every derived value comes from the kernel's base-physics pack." />{/if}
        <span>Distances on a log scale{frame ? ", in kilometres" : ""}</span>
        {#if through}<span class="through" data-known-count={knownCount}>{L?.label}: {knownCount} of {bodyCount} known</span>{/if}
        {#if key?.entries.length && on("key")}
          <details class="key">
            <summary>Key · {key.name}</summary>
            <div class="key-body"><Legend entries={key.entries} prefix="sck" note={key.note} label="Key to the chart's marks" /></div>
          </details>
        {/if}
        {#if params.unsurveyed_hatch && on("unsurveyed") && !frame}<span>Hatched: unsurveyed beyond {scene.surveyed_to_au} AU</span>{/if}
      </div>
      <div class="zoom" role="group" aria-label="Zoom">
        <button type="button" aria-label="Zoom in" onclick={() => zoomBy(1.25)}>+</button>
        <button type="button" aria-label="Zoom out" onclick={() => zoomBy(0.8)}>−</button>
        <button type="button" aria-label={frame ? "Fit the moons" : "Fit the system"} onclick={() => (zoom = Number(params.zoom))}>Fit</button>
        <button type="button" aria-pressed={measuring} onclick={toggleMeasure} title="Measure the distance between two bodies">Measure</button>
      </div>
    {/if}
  </ChartGround>
  {/if}
  <div class="dims" role="group" aria-label="Chart in 2D or 3D">
    <button type="button" aria-pressed={dims === "2d"} onclick={() => (dims = "2d")}>2D</button>
    <button type="button" aria-pressed={dims === "3d"} onclick={() => (dims = "3d")}>3D</button>
  </div>
</div>

<style>
  .chart {
    position: relative;
    width: 100%;
    height: 100%;
  }
  .dims {
    position: absolute;
    right: 236px;
    bottom: 16px;
    display: flex;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
  }
  .dims button {
    min-width: 44px;
    height: 40px;
    border: 0;
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
    cursor: pointer;
  }
  .dims button + button {
    border-left: var(--sse-line) solid var(--sse-ink);
  }
  .dims button[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
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
  .moontick {
    fill: var(--sse-ink);
  }
  .frame {
    position: absolute;
    left: 92px;
    right: 16px;
    top: 16px;
    flex-wrap: wrap;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .frame b {
    color: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    font-weight: normal;
  }
  .frame button {
    min-height: 36px;
    padding: 0 12px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
    cursor: pointer;
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
    top: 60px;
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
    transform-box: fill-box;
    transform-origin: center;
    animation: sel-in var(--sse-motion) var(--sse-ease);
  }
  /* The selection ring closes in on the body chosen. */
  @keyframes sel-in {
    from {
      transform: scale(1.7);
      opacity: 0;
    }
  }
  .body:hover .hit {
    fill: var(--sse-rule);
    fill-opacity: 0.45;
  }
  .peek {
    position: absolute;
    transform: translate(-50%, var(--off));
    display: flex;
    align-items: center;
    flex-direction: column;
    gap: 1px;
    padding: 5px 9px;
    background: var(--sse-panel-ground);
    color: var(--sse-panel-ink);
    border: var(--sse-frame-width) solid var(--sse-frame-line);
    box-shadow: 3px 3px 0 var(--sse-frame-line);
    font-size: 12px;
    pointer-events: none;
    white-space: nowrap;
    z-index: 2;
    animation: peek-in var(--sse-motion-quick) var(--sse-ease);
  }
  @keyframes peek-in {
    from {
      opacity: 0;
    }
  }
  .peek.up {
    transform: translate(-50%, calc(-100% - var(--off)));
  }
  .peek b {
    font-family: var(--sse-title-font);
    font-weight: 600;
    font-size: 13px;
  }
  .peek span {
    color: var(--sse-panel-muted);
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
  .body.unknown {
    opacity: 0.25;
  }
  .knownv {
    font-family: var(--sse-font-body);
    font-size: 10px;
    font-style: italic;
    fill: var(--sse-ink-muted);
  }
  .through {
    color: var(--sse-accent-ink);
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
  .key summary {
    cursor: pointer;
    color: var(--sse-ink);
  }
  .key[open] .key-body {
    position: absolute;
    left: 0;
    bottom: calc(100% + 6px);
    width: min(360px, calc(100vw - 48px));
    padding: 10px 12px;
    background: var(--sse-panel-ground);
    border: var(--sse-frame-width) solid var(--sse-frame-line);
    box-shadow: var(--sse-frame-shadow);
  }
  .measure {
    position: absolute;
    left: 24px;
    bottom: 64px;
    max-width: min(420px, calc(100% - 32px));
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 12px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    font-family: var(--sse-font-body);
    font-size: 13px;
    color: var(--sse-ink);
  }
  .measure .fine {
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .mline {
    stroke: var(--sse-accent);
    stroke-width: 1.5;
    stroke-dasharray: 5 4;
    pointer-events: none;
  }
  .mpick {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 1.5;
    pointer-events: none;
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
  .zoom button[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .zoom button + button {
    border-left: var(--sse-line) solid var(--sse-ink);
  }
  @media (max-width: 760px) {
    .dims {
      right: 12px;
      bottom: auto;
      top: 12px;
    }
    .frame {
      left: 16px;
      top: 12px;
    }
    .frame span {
      display: none;
    }
    .legend {
      left: 16px;
      right: 250px;
      bottom: 12px;
      font-size: 11px;
    }
    .zoom {
      bottom: 12px;
      right: 12px;
    }
  }
</style>
