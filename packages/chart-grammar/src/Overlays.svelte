<!-- Pack overlays over a map (overlay.ts): lines, hexes and labels already placed in the map's frame, turned into
     pixels by the view's own `at`. Drawn in theme ink, thin and quiet, under the map's marks; the view decides when
     (under its HUD switch) and which (its Layers tool).
     Map layers of areas (territory, OV4) are drawn in the Atlas look: a band of the answer's map colour along the
     inside of its border, fading to a bare middle so stars and orbits stay readable; the line says the status (solid
     holds, dashed claims, dotted influences, tint by strength); a claim is hatched in the claimant's colour over the
     holder's tint; a divided place is striped; the name is lettered along the area's long axis in spaced italic
     capitals (a holder's, not a claim's or an influence's: those are named on pointing and in the key) and drops out
     when the area is too small to carry it. An area the table does not know is drawn in pencil
     (SV3): no tint, a dotted edge, its name quiet. With `onpick`, pointing at an area inks its edge (and calls
     `onpoint`), and pressing it calls `onpick`; `lift` quietens every answer but one (the key's pointing). -->
<script lang="ts">
  import { drawn, hexCorners, letterArea, mapSlotToken, MAP_PATTERNS, type AreaItem, type OverlayInk, type ViewOverlay } from "../../ui-contract/src/index.ts";

  let {
    overlays,
    at,
    scale,
    lift = null,
    onpoint,
    onpick,
  }: {
    overlays: readonly ViewOverlay[];
    at: (p: readonly number[]) => { x: number; y: number };
    scale: number;
    /** An answer's key to lift (the rest go quiet), as the key's rows point at them. */
    lift?: string | null;
    onpoint?: (it: AreaItem | null) => void;
    onpick?: (it: AreaItem) => void;
  } = $props();
  const uid = $props.id();
  let pointed = $state<AreaItem | null>(null);
  function point(it: AreaItem | null) {
    pointed = it;
    onpoint?.(it);
  }
  // The Atlas look's band along the inside of a border, in screen pixels: three strokes clipped to the area, each
  // wider and fainter, so the colour fades towards the middle.
  const BAND = [
    { w: 34, o: 0.07 },
    { w: 20, o: 0.1 },
    { w: 8, o: 0.16 },
  ];
  const STATUS_WORDS = { holds: "holds", claims: "claims", influences: "influences" } as const;
  // Every ink an area uses, for its hatch (claims) and pattern fills (one-colour themes).
  const inkKey = (k: OverlayInk) => (typeof k === "string" ? k : `s${k.slot}`);
  const areaInks = $derived.by(() => {
    const m = new Map<string, OverlayInk>();
    for (const o of overlays) for (const l of o.layers) if (l.kind === "areas") for (const it of l.items) m.set(inkKey(it.ink), it.ink);
    return [...m];
  });
  const colourOf = (k: OverlayInk) => (typeof k === "string" ? (k === "accent" ? "var(--sse-accent)" : k === "faint" ? "var(--sse-ink-faint, var(--sse-ink-muted))" : "var(--sse-ink)") : `var(${mapSlotToken(k.slot)}, var(--sse-ink))`);
  const inkOfKey = (key: string): OverlayInk | null => {
    for (const o of overlays) for (const l of o.layers) if (l.kind === "areas") for (const it of l.items) if (it.key === key) return it.ink;
    return null;
  };
  // Lettering in screen space: along the long axis, sized to the area, dropped when the name will not fit.
  function lettering(it: AreaItem) {
    if (!it.label) return null;
    const l = letterArea(it.rings);
    if (!l) return null;
    const c = at(l.at);
    const t = ((it.label_turn ?? l.turn) * Math.PI) / 180;
    const e = at([l.at[0] + (Math.cos(t) * l.length) / 2, l.at[1] + (Math.sin(t) * l.length) / 2]);
    const w = at([l.at[0] - (Math.sin(t) * l.width) / 2, l.at[1] + (Math.cos(t) * l.width) / 2]);
    const long = 2 * Math.hypot(e.x - c.x, e.y - c.y);
    const wide = 2 * Math.hypot(w.x - c.x, w.y - c.y);
    let deg = (Math.atan2(e.y - c.y, e.x - c.x) * 180) / Math.PI;
    if (deg > 90) deg -= 180;
    if (deg < -90) deg += 180;
    const text = it.label.toUpperCase();
    const size = Math.max(0, Math.min(18, wide * 0.3, (long * 0.8) / (text.length * 0.95)));
    return size < 9 ? null : { x: c.x, y: c.y, deg, size, text };
  }
  // A plain ink is a class; a map palette slot (OV2) is the theme's slot colour, plain ink where a theme has none yet.
  const inkClass = (k: OverlayInk) => (typeof k === "string" ? k : "slot");
  const inkColour = (k: OverlayInk) => (typeof k === "string" ? undefined : `var(${mapSlotToken(k.slot)}, var(--sse-ink))`);
  const ring = (ps: readonly [number, number][]) => ps.map((p) => at(p)).map((q, i) => `${i ? "L" : "M"}${drawn(q.x)},${drawn(q.y)}`).join("") + "Z";
  const pts = (ps: readonly [number, number][]) => ps.map((p) => at(p)).map((q) => `${drawn(q.x)},${drawn(q.y)}`).join(" ");
</script>

<g class="overlays" class:pickable={!!onpick} data-overlays={overlays.length} aria-hidden="true">
  {#if areaInks.length}
    <defs>
      {#each areaInks as [k, ink] (k)}
        <!-- A claim's hatch and a divided place's stripes, in the answer's colour. -->
        <pattern id="{uid}-h-{k}" width="7" height="7" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
          <line x1="0" y1="0" x2="0" y2="7" style:stroke={colourOf(ink)} stroke-width="1.6" stroke-opacity="0.55" />
        </pattern>
        {#each MAP_PATTERNS.slice(1) as pat (pat)}
          <pattern id="{uid}-p-{k}-{pat}" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform={pat === "back-stripes" ? "rotate(-45)" : pat === "stripes" ? "rotate(45)" : undefined}>
            {#if pat === "stripes" || pat === "back-stripes"}<line x1="0" y1="0" x2="0" y2="8" style:stroke={colourOf(ink)} stroke-width="1.2" />
            {:else if pat === "dots"}<circle cx="4" cy="4" r="1.2" style:fill={colourOf(ink)} />
            {:else if pat === "cross"}<path d="M0 4H8M4 0V8" style:stroke={colourOf(ink)} stroke-width="0.9" fill="none" />
            {:else if pat === "waves"}<path d="M0 4Q2 2 4 4T8 4" style:stroke={colourOf(ink)} stroke-width="0.9" fill="none" />
            {:else if pat === "grid"}<path d="M0 0H8M0 0V8" style:stroke={colourOf(ink)} stroke-width="0.8" fill="none" />
            {:else}<path d="M0 6L4 2L8 6" style:stroke={colourOf(ink)} stroke-width="0.9" fill="none" />{/if}
          </pattern>
        {/each}
      {/each}
    </defs>
  {/if}
  {#each overlays as o (o.id)}
    <g data-overlay={o.id}>
      {#each o.layers as l, li (li)}
        {#if l.kind === "lines"}
          {#each l.items as it, i (i)}
            {#if it.closed}<polygon class={inkClass(l.ink)} style:stroke={inkColour(l.ink)} points={pts(it.points)} />{:else}<polyline class={inkClass(l.ink)} style:stroke={inkColour(l.ink)} points={pts(it.points)} />{/if}
          {/each}
        {:else if l.kind === "hexes"}
          {#each l.items as it, i (i)}
            {@const c = at(it.centre)}
            <polygon class="hex {inkClass(l.ink)}" style:stroke={inkColour(l.ink)} points={pts(hexCorners(it.centre, it.size, it.orient, l.rotation_deg ?? 0))} />
            {#if it.label}<text class="hexlabel {inkClass(l.ink)}" x={c.x} y={c.y - it.size * scale * 0.55} font-size={Math.min(11, it.size * scale * 0.42)}>{it.label}</text>{/if}
          {/each}
        {:else if l.kind === "areas"}
          {#each l.items as it, i (i)}
            {@const d = it.rings.map(ring).join("")}
            {@const cid = `${uid}-a-${o.id}-${li}-${i}`}
            {@const colour = colourOf(it.ink)}
            {@const pattern = typeof it.ink === "string" ? undefined : it.ink.pattern}
            {@const fills = !it.pencil && l.as !== "edges" && l.as !== "marks"}
            <g
              class="areaset"
              class:quiet={lift !== null && lift !== it.key}
              class:lifted={lift === it.key || pointed === it}
              class:pencil={it.pencil}
              data-area={it.key}
              data-status={it.status}
              data-pencil={it.pencil || undefined}
            >
              {#if fills}
                <clipPath id={cid}><path {d} clip-rule="evenodd" /></clipPath>
                <g clip-path="url(#{cid})">
                  {#if pattern && pattern !== "plain"}
                    <path {d} fill="url(#{uid}-p-{inkKey(it.ink)}-{pattern})" fill-rule="evenodd" opacity="0.5" />
                  {/if}
                  {#if it.status === "claims"}
                    <path {d} fill="url(#{uid}-h-{inkKey(it.ink)})" fill-rule="evenodd" data-area-hatch />
                  {:else}
                    {#each BAND as b (b.w)}
                      <path class="band" {d} style:stroke={colour} style:stroke-width="{b.w}px" style:stroke-opacity={b.o * (it.status === "influences" ? 0.5 + 0.5 * (it.strength ?? 1) : 1)} data-area-band />
                    {/each}
                  {/if}
                  {#if it.divided?.length}
                    {#each it.divided.slice(1) as part (part.key)}
                      {@const other = inkOfKey(part.key)}
                      {#if other}<path {d} fill="url(#{uid}-h-{inkKey(other)})" fill-rule="evenodd" opacity={Math.min(1, part.share * 2)} data-area-divided={part.key} />{/if}
                    {/each}
                  {/if}
                </g>
              {/if}
              <path class="area {it.status} {l.as}" class:penline={it.pencil} {d} style:stroke={it.pencil ? undefined : colour} />
              {#if onpick}
                <path
                  class="hit"
                  {d}
                  fill-rule="evenodd"
                  role="presentation"
                  onpointerenter={() => point(it)}
                  onpointerleave={() => point(null)}
                  onclick={() => onpick?.(it)}
                ><title>{it.label ?? it.key}: {STATUS_WORDS[it.status]}{it.pencil ? " (not known to the table)" : ""}</title></path>
              {/if}
              {#if l.as !== "edges" && it.status === "holds"}
                {@const lt = lettering(it)}
                {#if lt}
                  <text class="letter" class:penname={it.pencil} x={lt.x} y={lt.y} font-size={lt.size} transform="rotate({lt.deg} {lt.x} {lt.y})" style:fill={it.pencil ? undefined : colour} data-area-label={it.key}>{lt.text}</text>
                {/if}
              {/if}
            </g>
          {/each}
        {:else}
          {#each l.items as it, i (i)}
            {@const c = at(it.at)}
            <text class="label {inkClass(l.ink)}" x={c.x} y={c.y}>{it.text}</text>
          {/each}
        {/if}
      {/each}
    </g>
  {/each}
</g>

<style>
  polygon,
  polyline {
    fill: none;
    stroke-width: var(--sse-line, 1px);
    vector-effect: non-scaling-stroke;
  }
  .ink {
    stroke: var(--sse-ink);
  }
  .faint {
    stroke: var(--sse-ink-faint, var(--sse-ink-muted));
  }
  .accent {
    stroke: var(--sse-accent);
  }
  .area {
    fill: none;
    fill-rule: evenodd;
    stroke: var(--sse-ink);
    stroke-width: var(--sse-line, 1px);
    vector-effect: non-scaling-stroke;
  }
  .area {
    stroke-width: 1.4px;
  }
  .area.claims {
    stroke-dasharray: 6 4;
  }
  .area.influences {
    stroke-dasharray: 1 3;
  }
  .band {
    fill: none;
    vector-effect: non-scaling-stroke;
    stroke-linejoin: round;
  }
  .areaset {
    transition: opacity 160ms ease;
  }
  .areaset.quiet {
    opacity: 0.3;
  }
  .areaset.lifted .area {
    stroke-width: 2.6px;
  }
  /* Pencil (SV3): the table does not know it, so it is drawn as the GM's own note, quiet and dotted. */
  .area.penline {
    stroke: var(--sse-ink-muted);
    stroke-dasharray: 1.5 2.5;
  }
  .hit {
    fill: transparent;
    stroke: none;
    pointer-events: none;
  }
  .pickable {
    pointer-events: none;
  }
  .pickable .hit {
    pointer-events: visiblePainted;
    cursor: pointer;
  }
  text.letter {
    font-family: var(--sse-font-display, serif);
    font-style: italic;
    letter-spacing: 0.28em;
    text-anchor: middle;
    dominant-baseline: middle;
    opacity: 0.85;
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 3px;
    stroke-opacity: 0.6;
  }
  text.letter.penname {
    fill: var(--sse-ink-muted);
    opacity: 0.8;
  }
  @media (prefers-reduced-motion: reduce) {
    .areaset {
      transition: none;
    }
  }
  .hex {
    stroke-width: 0.5px;
  }
  text {
    stroke: none;
    text-anchor: middle;
    dominant-baseline: middle;
    font-family: var(--sse-font-mono, monospace);
    font-size: 11px;
  }
  text.ink {
    fill: var(--sse-ink);
  }
  text.faint {
    fill: var(--sse-ink-muted);
  }
  text.accent {
    fill: var(--sse-accent);
  }
</style>
