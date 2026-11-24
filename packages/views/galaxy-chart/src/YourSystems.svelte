<!--
  Your systems (V3 gaps 17 and 39, U6 part 2b, part 1): every system in the campaign on one map, seen from above,
  where its own data puts it (a V3 starmap's positions), with a stalk for its height above or below the map. Resting
  the mouse on a system says what it holds (GM only: stars, planets, moons, minor bodies, ships and stations, life,
  anything remarkable) without opening it; a press opens it. A system with no place yet is listed beside the map,
  never put somewhere made up. Presentation only: the kernel's facts, read back once per system.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { layoutSystems, placeHash, readCampaignSystems, mapScaleBar, summaryWords, systemSummary, type CampaignSystem, type Json, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";

  let { ctx, scene }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  let listed = $state<CampaignSystem[] | null>(null);
  let failed = $state<string | null>(null);
  let gen = 0;
  // Read again when the campaign changes (a system adopted, a body added): the workspace's revision moves.
  $effect(() => {
    void ctx.workspace.revision;
    const g = ++gen;
    readCampaignSystems(ctx.session).then((r) => {
      if (g !== gen) return;
      if (!r.ok) return void ((failed = r.message), (listed = []));
      failed = null;
      listed = r.systems;
    });
  });
  // Bodies the shell has shown this session, by system: a system whose read-back holds no bodies (the campaign's
  // own, which lives in the shipped fixture, not the universe) still says what it holds once the chart has drawn it.
  let seen = $state<Record<string, SystemScene["subjects"]>>({});
  let ownSystem = $state<string | null>(null);
  $effect(() => {
    if (!scene.subjects.length) return;
    if (untrack(() => seen[scene.system]) !== scene.subjects) seen = { ...untrack(() => seen), [scene.system]: scene.subjects };
    if (scene.fixture) ownSystem = scene.system;
  });
  const filled = (x: CampaignSystem): CampaignSystem => {
    const s = !x.subjects.length ? seen[x.system] : undefined;
    return s ? { ...x, subjects: s, summary: systemSummary(s) } : x;
  };
  // The campaign's own system is on the chart but not always in the universe's list: it joins from the scene.
  const all = $derived.by<CampaignSystem[]>(() => {
    if (!listed) return [];
    const own = ownSystem ?? (scene.fixture ? scene.system : null);
    const rows = listed.map(filled);
    const mine = own && !rows.some((x) => x.system === own) && seen[own] ? [{ system: own, title: scene.fixture ? scene.title : "This campaign's own system", source: "campaign", position: null, unit: null, subjects: seen[own]!, summary: systemSummary(seen[own]!) }] : [];
    return [...mine, ...rows];
  });
  /** The campaign's own system opens as itself (an empty system in the link); the others by address. */
  const link = (x: CampaignSystem) => placeHash({ scale: "system", system: x.system === (ownSystem ?? "") ? "" : x.system });
  const S = 560;
  let zoom = $state(1);
  let pan = $state<[number, number]>([0, 0]);
  const laid = $derived(layoutSystems(all, S, 56));
  const at = $derived(new Map(laid.marks.map((m) => [m.system, m])));
  const placed = $derived(all.filter((x) => at.has(x.system)));
  const unplaced = $derived(all.filter((x) => !at.has(x.system)));
  const unit = $derived(placed.find((x) => x.unit)?.unit ?? null);
  const unitWords = $derived(unit === "ly" ? "light years" : unit === "pc" ? "parsecs" : unit ? unit : "map units");
  const bar = $derived(mapScaleBar(laid.unitsPerPx / zoom, 110));

  // Zoom and slide (interface rule 11: a flat chart zooms and slides): the wheel or a pinch zooms about the pointer,
  // a drag slides. Double-click puts it back.
  const view = $derived(`${S / 2 - S / 2 / zoom - pan[0]} ${S / 2 - S / 2 / zoom - pan[1]} ${S / zoom} ${S / zoom}`);
  let svg = $state<SVGSVGElement | null>(null);
  function toMap(e: { clientX: number; clientY: number }): [number, number] {
    const r = svg?.getBoundingClientRect();
    if (!r || !r.width) return [S / 2, S / 2];
    const [vx, vy, vw] = view.split(" ").map(Number) as [number, number, number];
    return [vx + ((e.clientX - r.left) / r.width) * vw, vy + ((e.clientY - r.top) / r.height) * vw];
  }
  function onwheel(e: WheelEvent) {
    e.preventDefault();
    const [mx, my] = toMap(e);
    const z = Math.min(40, Math.max(1, zoom * (e.deltaY < 0 ? 1.25 : 0.8)));
    // Keep the point under the pointer where it is.
    const cx = S / 2 - pan[0];
    const cy = S / 2 - pan[1];
    const nx = mx - ((mx - cx) * zoom) / z;
    const ny = my - ((my - cy) * zoom) / z;
    zoom = z;
    pan = z === 1 ? [0, 0] : [S / 2 - nx, S / 2 - ny];
  }
  let drag: { x: number; y: number; pan: [number, number]; moved: boolean } | null = null;
  function ondown(e: PointerEvent) {
    if (e.button !== 0) return;
    drag = { x: e.clientX, y: e.clientY, pan, moved: false };
  }
  function onmove(e: PointerEvent) {
    if (!drag || !svg) return;
    const r = svg.getBoundingClientRect();
    const k = S / zoom / (r.width || 1);
    const dx = (e.clientX - drag.x) * k;
    const dy = (e.clientY - drag.y) * k;
    if (Math.abs(dx) + Math.abs(dy) > 2) drag.moved = true;
    if (drag.moved) pan = [drag.pan[0] + dx, drag.pan[1] + dy];
  }
  function onup() {
    drag = null;
  }

  // The hover peek (dynamic minimalism): with a mouse, resting on a system says what it holds.
  let peek = $state<{ system: string; x: number; y: number } | null>(null);
  const peeked = $derived(peek ? all.find((x) => x.system === peek!.system) : undefined);
  function rest(e: PointerEvent, x: CampaignSystem) {
    if (e.pointerType !== "mouse") return;
    const host = (e.currentTarget as Element).closest(".ys")?.getBoundingClientRect();
    const r = (e.currentTarget as Element).getBoundingClientRect();
    if (!host) return;
    peek = { system: x.system, x: r.left + r.width / 2 - host.left, y: r.bottom - host.top };
  }
  const leave = (x: CampaignSystem) => peek?.system === x.system && (peek = null);
  const size = (x: CampaignSystem) => 5 + Math.min(4, x.summary.stars) * 1.5;
</script>

<div class="ys" data-your-systems={listed ? all.length : "reading"}>
  <header class="head">
    <h2>Your systems</h2>
    <p class="lede" data-lede>
      {#if !listed}Reading the campaign's systems…{:else if !all.length}This campaign holds no systems yet. Make one, or open a V3 save.{:else}{all.length} {all.length === 1 ? "system" : "systems"} in this campaign{placed.length ? `, ${placed.length} placed by ${placed.length === 1 ? "its" : "their"} own map` : ""}. Rest on one to see what it holds; press it to open it.{/if}
    </p>
  </header>
  {#if failed}<p class="note" role="alert">Could not list the campaign's systems: {failed}</p>{/if}

  <div class="split">
    {#if placed.length}
      <figure class="map">
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <svg
          bind:this={svg}
          viewBox={view}
          role="img"
          aria-label="The campaign's systems seen from above"
          {onwheel}
          onpointerdown={ondown}
          onpointermove={onmove}
          onpointerup={onup}
          onpointerleave={onup}
          ondblclick={() => ((zoom = 1), (pan = [0, 0]))}
          data-zoom={zoom}
        >
          <rect class="box" x="8" y="8" width={S - 16} height={S - 16} />
          {#each placed as x (x.system)}
            {@const m = at.get(x.system)!}
            <a href={link(x)} class="sys" data-system={x.system} aria-label="{x.title}: {summaryWords(x.summary)}" onpointerenter={(e) => rest(e, x)} onpointerleave={() => leave(x)} onclick={(e) => drag?.moved && e.preventDefault()}>
              {#if Math.abs(m.pz) > 1}<line class="stalk" x1={m.px} y1={m.py} x2={m.px} y2={m.py + m.pz} />{/if}
              <circle class="star" class:life={x.summary.life.length > 0} cx={m.px} cy={m.py - m.pz} r={size(x) / Math.sqrt(zoom)} />
              {#if x.summary.stars > 1}<circle class="ring" cx={m.px} cy={m.py - m.pz} r={(size(x) + 4) / Math.sqrt(zoom)} />{/if}
              <text class="name" x={m.px + (size(x) + 6) / Math.sqrt(zoom)} y={m.py - m.pz + 4 / Math.sqrt(zoom)} style="font-size: {18 / Math.sqrt(zoom)}px">{x.title}</text>
            </a>
          {/each}
        </svg>
        <figcaption class="hint">
          {#if placed.length > 1}<span class="bar" style="width: {(bar.px / S) * 100}%" data-scale-bar></span>
          {bar.units.toLocaleString("en-GB")} {unitWords} · {/if}seen from above, x across, y up; a stalk drops to the map from a system above or below it. A ringed dot holds more than one star; a green one holds life. Wheel to zoom, drag to slide, double-click to put it back.
        </figcaption>
      </figure>
    {/if}

    {#if unplaced.length}
      <aside class="unplaced" data-unplaced={unplaced.length}>
        <h3>{placed.length ? "Not on the map yet" : "In this campaign"}</h3>
        <ul>
          {#each unplaced as x (x.system)}
            <li>
              <a href={link(x)} data-system={x.system} onpointerenter={(e) => rest(e, x)} onpointerleave={() => leave(x)}>
                <b>{x.title}</b>
                <span class="sum">{summaryWords(x.summary)}</span>
              </a>
            </li>
          {/each}
        </ul>
        <p class="hint">{placed.length ? "Their data gives no place among the others." : "None of them has a place of its own yet."} Placing systems in the galaxy's own stars comes with the campaign's galaxy (G7); until then a system sits where its V3 map put it, or here.</p>
      </aside>
    {/if}
  </div>

  {#if peeked && peek}
    {#key peek.system}
      <div class="peek" style="left: {peek.x}px; top: {peek.y}px" aria-hidden="true" data-system-peek={peek.system}>
        <b>{peeked.title}</b>
        <span data-peek-summary>{summaryWords(peeked.summary)}</span>
        {#each peeked.summary.remarkable as r (r)}<span class="mark">{r}</span>{/each}
        {#if peeked.position}<span class="where">at {[peeked.position.x, peeked.position.y, peeked.position.z].map((v) => +v.toFixed(2)).join(", ")} {unitWords}</span>{/if}
      </div>
    {/key}
  {/if}
</div>

<style>
  .ys {
    position: relative;
    color: var(--sse-ink);
    font-size: 13px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 1180px;
    padding-top: 10px;
  }
  .head h2 {
    margin: 0;
    font-family: var(--sse-title-font);
  }
  .lede {
    margin: 4px 0 0;
    color: var(--sse-ink-muted);
  }
  .note {
    color: var(--sse-accent);
  }
  .split {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    align-items: flex-start;
  }
  .map {
    margin: 0;
    flex: 1 1 420px;
    max-width: min(640px, 72vh);
  }
  svg {
    width: 100%;
    aspect-ratio: 1;
    display: block;
    touch-action: none;
    cursor: grab;
  }
  .box {
    fill: none;
    stroke: var(--sse-rule);
    vector-effect: non-scaling-stroke;
  }
  .star {
    fill: var(--sse-ink);
    stroke: var(--sse-ground);
    stroke-width: 1.5;
    vector-effect: non-scaling-stroke;
  }
  .star.life {
    fill: var(--sse-accent);
  }
  .ring {
    fill: none;
    stroke: var(--sse-ink);
    vector-effect: non-scaling-stroke;
  }
  .stalk {
    stroke: var(--sse-ink-muted);
    stroke-dasharray: 2 3;
    vector-effect: non-scaling-stroke;
  }
  .sys:hover .star,
  .sys:focus-visible .star {
    stroke: var(--sse-accent);
    stroke-width: 3;
  }
  .name {
    fill: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 4px;
  }
  .hint {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .bar {
    display: block;
    height: 0;
    border-top: 2px solid var(--sse-ink);
    margin-bottom: 4px;
  }
  .unplaced {
    flex: 1 1 260px;
    max-width: 420px;
  }
  .unplaced h3 {
    margin: 0 0 6px;
    font-size: 14px;
  }
  .unplaced ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .unplaced a {
    display: flex;
    flex-direction: column;
    padding: 6px 8px;
    color: inherit;
    text-decoration: none;
    border-left: 2px solid transparent;
  }
  .unplaced a:hover,
  .unplaced a:focus-visible {
    border-left-color: var(--sse-accent);
    background: color-mix(in srgb, var(--sse-accent) 8%, transparent);
  }
  .sum {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .peek {
    position: absolute;
    transform: translate(-50%, 10px);
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
  .peek b {
    font-family: var(--sse-title-font);
    font-weight: 600;
    font-size: 13px;
  }
  .peek span {
    color: var(--sse-panel-muted);
  }
  .peek .mark {
    font-style: italic;
  }
  @media (max-width: 600px) {
    .map,
    .unplaced {
      flex-basis: 100%;
      max-width: none;
    }
  }
</style>
