<!--
  Galaxies nearby in 3D (grand zoom GZ5 part 1, design section 7.2 and 7.3): a sphere of space centred on our own
  galaxy, every galaxy the kernel's sky knows within the reach at its real place (its direction from home times its
  distance), with stalks to our disc's plane. Two reaches: the group (1.5 Mpc) and the neighbourhood (12 Mpc). Seen
  from above, tilted or edge-on; Real draws true sizes (most are points of light), Readable one stated factor larger.
  Each galaxy is drawn by the kernel's own facts: its type, half-light size, axis ratio, tilt on the sky and colour.
  HUD: names, dark-halo rings and the scale bar. Pressing one opens its card. Read only, GM only.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { NEARBY_REACHES, cellHalo, galaxyDistanceWords, galaxyColour, galaxyTypeWords, haloR200, lightTravelWords, nearbyAt, nearbyRadius, nearbyScaleBar, projectNearby, readNearby, readableFactor, sci, withinReach, type NearGalaxy, type NearbyRead, type NearbyReach, type NearbySeen, type ViewSession } from "../../../ui-contract/src/index.ts";

  let { session, seen, scale, hud, onchart }: { session: ViewSession; seen: NearbySeen; scale: "real" | "readable"; hud: boolean; onchart: (address: string) => void } = $props();

  let read = $state<NearbyRead | null>(null);
  let failed = $state<string | null>(null);
  let reach = $state<NearbyReach>("group");
  let picked = $state<string | null>(null);
  let hover = $state<string | null>(null);
  // One read of the kernel's sky for galaxies only (no stars): a few seconds, then everything here is drawing.
  $effect(() => {
    const s = session;
    untrack(() => {
      read = null;
      failed = null;
    });
    void s.skyFromHere({ stars: false, galaxies: true, limit_mag: 30 }).then((r) => {
      if (!r.ok) return void (failed = r.message);
      const got = readNearby(r.sky);
      if (!got) return void (failed = "the sky came back with no galaxies");
      read = got;
    });
  });

  const S = 560;
  const R = $derived(NEARBY_REACHES.find((x) => x.id === reach)!.mpc);
  const factor = $derived(readableFactor(R));
  const inReach = $derived(read ? withinReach(read.galaxies, R) : []);
  const home = $derived(read && read.home_r200_mpc && read.home_halo_msun ? { r200_mpc: read.home_r200_mpc, mass_msun: read.home_halo_msun } : null);
  // Drawn far side first, so nearer galaxies sit over farther ones.
  const marks = $derived(
    inReach
      .map((g) => ({ g, ...projectNearby(nearbyAt(g), seen, S, R), r: nearbyRadius(g, S, R, scale, factor), colour: galaxyColour(g.b_v) }))
      .sort((a, b) => a.depth - b.depth || (a.g.address < b.g.address ? -1 : 1)),
  );
  // Names under HUD: the biggest first, skipping any that would sit on a name already placed.
  const labels = $derived.by(() => {
    const at = new Map(marks.map((m) => [m.g.address, m]));
    const kept: { address: string; x: number; y: number }[] = [];
    for (const g of [...inReach.filter((g) => g.address === picked), ...inReach]) {
      const m = at.get(g.address);
      if (!m || kept.some((k) => k.address === g.address)) continue;
      const x = m.x + m.r + 4;
      const y = m.y + 3;
      if (g.address !== picked && kept.some((k) => Math.abs(k.y - y) < 12 && Math.abs(k.x - x) < 90)) continue;
      kept.push({ address: g.address, x, y });
      if (kept.length >= 12) break;
    }
    return kept;
  });
  // Stalks for the galaxies that matter at this reach: every one in the group; in the neighbourhood the big ones,
  // so a thousand stalks do not hide the galaxies.
  const stalked = (g: NearGalaxy) => R <= 2 || g.stellar_mass_msun >= 1e10 || g.address === picked;
  const centre = $derived(projectNearby([0, 0, 0], seen, S, R));
  const pxPerMpc = $derived((S / 2 - 24) / R);
  const bar = $derived(nearbyScaleBar(S, R));
  const one = $derived(read?.galaxies.find((g) => g.address === picked) ?? null);
  const said = $derived(read?.galaxies.find((g) => g.address === hover) ?? null);
  const nameOf = (a: string) => {
    const c = cellHalo(a);
    if (c) return `Halo ${c.key}, cell ${c.cell.join(".")}`;
    const sat = /\/prog:([^/]+)$/.exec(a);
    return sat ? `Satellite ${sat[1]}` : (a.split("/").at(-1) ?? a);
  };
  const choose = (g: NearGalaxy) => (picked = picked === g.address ? null : g.address);
</script>

<div class="near" data-nearby={read ? inReach.length : ""} data-nearby-reach={reach}>
  <div class="bar">
    <div class="seg" role="group" aria-label="How far out">
      {#each NEARBY_REACHES as x (x.id)}
        <button type="button" class:on={reach === x.id} aria-pressed={reach === x.id} onclick={() => (reach = x.id)} data-reach={x.id}>{x.label} · {x.mpc} Mpc</button>
      {/each}
    </div>
    <span class="chip" data-nearby-scale={scale}>{scale === "readable" ? `galaxies drawn ${factor} times size` : "true sizes: most are points of light"}</span>
    <span class="chip">as it is now, shapes as seen from home</span>
  </div>

  {#if failed}
    <p class="note" role="alert">The galaxies around home could not be read: {failed}</p>
  {:else if !read}
    <p class="hint" data-nearby-reading>Reading the galaxies around home from the kernel's sky…</p>
  {:else}
    <div class="split">
      <figure class="map">
        <svg viewBox="0 0 {S} {S}" role="img" aria-label="Galaxies within {R} megaparsecs of home, {seen === 'above' ? 'seen from above our disc' : seen === 'edge' ? 'seen edge-on along our disc' : 'seen tilted'}" onpointerleave={() => (hover = null)}>
          <rect class="ground" x="0" y="0" width={S} height={S} />
          {#if seen === "edge"}
            <line class="plane" x1="24" y1={S / 2} x2={S - 24} y2={S / 2} />
          {:else}
            <ellipse class="plane" cx={S / 2} cy={centre.fy} rx={S / 2 - 24} ry={seen === "above" ? S / 2 - 24 : (S / 2 - 24) * 0.5} />
          {/if}
          {#if hud && home}
            {#each marks as m (m.g.address)}
              {@const rr = haloR200(m.g.halo_mass_msun, home) * pxPerMpc}
              {#if m.g.halo_mass_msun >= 1e11 && !m.g.satellite && rr > 3}<circle class="halo" cx={m.x} cy={m.y} r={rr} />{/if}
            {/each}
            <circle class="halo home" cx={centre.x} cy={centre.y} r={home.r200_mpc * pxPerMpc} data-home-halo />
          {/if}
          {#each marks as m (m.g.address)}
            {#if seen !== "above" && stalked(m.g) && Math.abs(m.y - m.fy) > 1}<line class="stalk" x1={m.fx} y1={m.fy} x2={m.x} y2={m.y} />{/if}
          {/each}
          <!-- Our own galaxy at the middle: a disc in our own plane. -->
          <ellipse class="us" cx={centre.x} cy={centre.y} rx={Math.max(2.5, 0.015 * pxPerMpc * (scale === "readable" ? factor : 1))} ry={Math.max(1.2, 0.015 * pxPerMpc * (scale === "readable" ? factor : 1) * (seen === "above" ? 1 : seen === "edge" ? 0.12 : 0.5))} data-home-galaxy />
          {#each marks as m (m.g.address)}
            <g
              class="gal"
              class:on={m.g.address === picked}
              role="button"
              tabindex="0"
              aria-label="{nameOf(m.g.address)}: {galaxyTypeWords(m.g.morphology)}, {galaxyDistanceWords(m.g.distance_mpc)} away"
              onclick={() => choose(m.g)}
              onkeydown={(e) => (e.key === "Enter" || e.key === " ") && choose(m.g)}
              onpointerenter={() => (hover = m.g.address)}
              data-galaxy={m.g.address}
              data-morphology={m.g.morphology}
            >
              {#if m.r > 2.5}
                <ellipse cx={m.x} cy={m.y} rx={m.r} ry={Math.max(1, m.r * m.g.axis_ratio)} transform="rotate({-m.g.position_angle_deg} {m.x} {m.y})" fill={m.colour} />
              {:else}
                <circle cx={m.x} cy={m.y} r={m.r + 0.6} fill={m.colour} />
              {/if}
              {#if m.g.address === picked}<circle class="ring" cx={m.x} cy={m.y} r={m.r + 5} />{/if}
            </g>
          {/each}
          {#if hud}
            {#each labels as l (l.address)}<text class="lbl" x={l.x} y={l.y} data-nearby-name>{nameOf(l.address)}</text>{/each}
            <text class="lbl us-lbl" x={centre.x + 6} y={centre.y - 6}>Our galaxy</text>
            <line class="scale" x1="24" y1={S - 18} x2={24 + bar.px} y2={S - 18} data-nearby-bar />
            <text class="lbl" x="24" y={S - 24}>{bar.words}</text>
          {/if}
        </svg>
        <figcaption class="hint" data-nearby-said>
          {#if said}<b>{nameOf(said.address)}</b>: {galaxyTypeWords(said.morphology)}, {sci(said.stellar_mass_msun)} suns of stars, {galaxyDistanceWords(said.distance_mpc)} away.
          {:else}{inReach.length} galaxies within {R} Mpc of ours, each where the sky sees it. {read.remains ? `${read.remains} satellite${read.remains === 1 ? "" : "s"} torn into streams and ${read.merged} merged into ours are counted, not drawn. ` : ""}Galaxies whose halo is under the catalogue's floor are not placed.{/if}
        </figcaption>
      </figure>

      <aside class="card" data-galaxy-card={one?.address ?? ""}>
        {#if one}
          <h3>{nameOf(one.address)}</h3>
          <p class="type">{galaxyTypeWords(one.morphology)[0]!.toUpperCase() + galaxyTypeWords(one.morphology).slice(1)}{one.satellite ? ", one of our own satellites" : ""}</p>
          <dl>
            <dt>Stars</dt><dd>{sci(one.stellar_mass_msun)} suns, forming {sci(one.sfr_msun_yr)} a year</dd>
            <dt>Dark halo</dt><dd>{sci(one.halo_mass_msun)} suns</dd>
            <dt>Distance</dt><dd>{galaxyDistanceWords(one.distance_mpc)}; its light takes {lightTravelWords(one.distance_mpc)} to reach us</dd>
            <dt>Half its light within</dt><dd>{Math.round(one.half_light_kpc * 10) / 10} kpc</dd>
            {#if one.orbit_words.length}<dt>Its orbit</dt><dd>{one.orbit_words.join(", ")}</dd>{/if}
          </dl>
          {#if cellHalo(one.address)}
            <button type="button" class="go" onclick={() => onchart(one!.address)} data-chart-nearby>Chart this galaxy</button>
          {/if}
        {:else}
          <p class="hint">Press a galaxy for its card.</p>
        {/if}
      </aside>
    </div>
  {/if}
</div>

<style>
  .near {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 12px;
    align-items: center;
  }
  .seg {
    display: inline-flex;
    flex-wrap: wrap;
  }
  .seg button,
  .go {
    font: inherit;
    min-height: 34px;
    padding: 0 10px;
    border: 1px solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    margin-right: -1px;
    cursor: pointer;
  }
  .seg button.on,
  .go {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .go {
    margin-top: 10px;
  }
  .chip {
    font-size: 12px;
    color: var(--sse-ink-muted);
    border: 1px solid var(--sse-rule);
    padding: 2px 8px;
    border-radius: 999px;
  }
  .split {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(220px, 300px);
    gap: 16px;
  }
  @media (max-width: 760px) {
    .split {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .map {
    margin: 0;
  }
  svg {
    width: 100%;
    max-width: 560px;
    aspect-ratio: 1;
    display: block;
  }
  /* Space is drawn as an inverted panel (ink ground), so the galaxies read as lights in the chart room. */
  .ground {
    fill: var(--sse-ink);
  }
  .plane {
    fill: none;
    stroke: color-mix(in srgb, var(--sse-ground) 25%, transparent);
    stroke-dasharray: 3 5;
  }
  .stalk {
    stroke: color-mix(in srgb, var(--sse-ground) 30%, transparent);
  }
  .halo {
    fill: none;
    stroke: color-mix(in srgb, var(--sse-ground) 22%, transparent);
    stroke-dasharray: 2 4;
  }
  .halo.home {
    stroke: color-mix(in srgb, var(--sse-accent) 60%, transparent);
  }
  .us {
    fill: var(--sse-accent);
  }
  .gal {
    cursor: pointer;
  }
  .gal:hover ellipse,
  .gal:hover circle:not(.ring) {
    stroke: var(--sse-ground);
    stroke-width: 0.8;
  }
  .ring {
    fill: none;
    stroke: var(--sse-ground);
  }
  .lbl {
    fill: color-mix(in srgb, var(--sse-ground) 85%, transparent);
    font-size: 10px;
    pointer-events: none;
  }
  .us-lbl {
    fill: var(--sse-accent);
  }
  .scale {
    stroke: color-mix(in srgb, var(--sse-ground) 85%, transparent);
    stroke-width: 2;
  }
  .hint,
  .type {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .note {
    color: var(--sse-accent);
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 4px 10px;
    margin: 8px 0;
  }
  dt {
    color: var(--sse-ink-muted);
  }
  dd {
    margin: 0;
  }
</style>
