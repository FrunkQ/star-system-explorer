<!--
  A star on the Body screen (R5a): its photosphere in its own colour, darkened
  to the limb, with spots where its activity cycle puts them, or its corona in
  X-rays. Facts from the kernel: temperature, radius and mass from the system,
  activity (spin, spots, cycle, X-rays, flares) from read_activity.
-->
<script lang="ts">
  import type { Json, SystemScene, ViewContext } from "../../../ui-contract/src/index.ts";
  import { coronaScaleHeight, CORONA_K, drawStar, narrateStar, SPOT_DELTA_K, spotsFor, STAR_REFERENCES, type StarDraw } from "../../../renderer/src/index.ts";

  let { ctx, scene, address, label, size }: { ctx: ViewContext; scene: SystemScene; address: string; label: string; size: number } = $props();

  type Flare = { at_s: number; rise_s: number; decay_s: number; peak_w: number; energy_j: number };
  type Got = { teff: number; mass: number; radius: number; lum: number; age_s: number; activity: Record<string, Json>; moment: Record<string, Json>; flares: Flare[] };
  let got = $state<Got | null>(null);
  let failed = $state<string | null>(null);
  let band = $state<StarDraw["light"]>("visible");
  let turn = $state(0);
  let canvas = $state<HTMLCanvasElement | undefined>();

  const SIGMA = 5.670374419e-8;
  const YEAR = 365.25 * 86400;

  $effect(() => {
    const a = address;
    const sys = scene.system;
    got = null;
    failed = null;
    let live = true;
    (async () => {
      const rs = await ctx.session.resolvedSystem(sys);
      if (!live) return;
      if (!rs.ok) return void (failed = rs.message);
      const f = rs.result.facts[a] ?? {};
      const teff = Number(f["star.effective_temperature"]);
      const mass = Number(f["body.mass"]);
      const radius = Number(f["body.radius"]);
      const age_s = Number(rs.result.facts[sys]?.["system.age"] ?? 0);
      if (!(teff > 0 && mass > 0 && radius > 0)) return void (failed = "The kernel has no temperature, mass and radius for this star yet, so there is nothing true to draw.");
      const lum = 4 * Math.PI * radius * radius * SIGMA * teff ** 4;
      const star = { address: a, mass_kg: mass, radius_m: radius, luminosity_w: lum, teff_k: teff, age_s: age_s > 0 ? age_s : 4.6e9 * YEAR };
      const r = await ctx.session.readActivity({ stars: [star], at_s: star.age_s, window: { from_s: star.age_s - 2 * 86400, to_s: star.age_s + YEAR, min_energy_j: 1e24 } });
      if (!live) return;
      if (!r.ok) return void (failed = r.message);
      const one = r.stars[0] as { activity: Record<string, Json>; moment: Record<string, Json>; flares?: { flares?: Flare[] } };
      got = { teff, mass, radius, lum, age_s: star.age_s, activity: one.activity, moment: one.moment, flares: one.flares?.flares ?? [] };
    })();
    return () => {
      live = false;
    };
  });

  const cyc = $derived.by(() => {
    const ph = got?.moment.cycle_phase;
    const c = got?.activity.cycle as { period_s?: number } | undefined;
    if (typeof ph !== "number" || !c?.period_s) return null;
    // Cycle progress from minimum: the activity factor is 1 + A sin(2 pi phase), lowest at phase 0.75.
    const p = (((ph - 0.75) % 1) + 1) % 1;
    return { phase: p, number: Math.floor((got!.age_s / c.period_s) + 0.25) };
  });
  const coverage = $derived(Number(got?.moment.spot_coverage ?? 0));
  const spots = $derived(got ? spotsFor(address, coverage, cyc) : []);
  const H = $derived(got ? coronaScaleHeight(got.mass, got.radius) : 0.1);
  const now = $derived(got?.age_s ?? 0);
  const active = $derived((got?.flares ?? []).filter((f) => now >= f.at_s - f.rise_s && now <= f.at_s + f.decay_s));
  const next = $derived((got?.flares ?? []).filter((f) => f.at_s > now).sort((x, y) => x.at_s - y.at_s).slice(0, 5));
  const spin_days = $derived(Number(got?.activity.rotation_period_s ?? 0) / 86400);

  $effect(() => {
    const c = canvas;
    const g = got;
    if (!c || !g) return;
    const n = Math.round(size * Math.min(2, typeof devicePixelRatio === "number" ? devicePixelRatio : 1));
    c.width = c.height = n;
    const x = c.getContext("2d")!;
    const img = x.createImageData(n, n);
    drawStar(img, { teff_k: g.teff, spots, lon_deg: turn, lat_deg: 7, light: band, corona_h: H, flares: active.map((f, i) => ({ lat_deg: spots[i % Math.max(1, spots.length)]?.lat_deg ?? 15, lon_deg: spots[i % Math.max(1, spots.length)]?.lon_deg ?? 0, strength: f.peak_w / Math.max(1, Number(g.moment.xray_w)) })) }, band === "visible" ? 0.92 : 0.6);
    x.putImageData(img, 0, 0);
  });

  const pow = (w: number) => `${(w / 1e20).toPrecision(2)} × 10²⁰ W`;
  const when = (s: number) => {
    const d = (s - now) / 86400;
    return d < 1 ? `in ${Math.round(d * 24)} h` : `in ${Math.round(d)} days`;
  };
  function key(e: KeyboardEvent) {
    if (e.key === "ArrowLeft") turn = (turn + 350) % 360;
    else if (e.key === "ArrowRight") turn = (turn + 10) % 360;
    else return;
    e.preventDefault();
  }
</script>

<div class="star" data-star>
  {#if failed}
    <p class="note">{failed}</p>
  {:else if !got}
    <p class="note">Reading {label}'s activity from the kernel…</p>
  {:else}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (a turnable star: arrow keys) -->
    <div class="disc" tabindex="0" role="img" aria-label={narrateStar(label, got.teff, spots.length, coverage, String(got.activity.regime), band)} onkeydown={key} data-band={band} data-spots={spots.length}>
      <canvas bind:this={canvas} style={`width:${size}px;height:${size}px`}></canvas>
    </div>
    <dl class="facts">
      <dt>View</dt>
      <dd>
        <span class="seg" role="group" aria-label="Band">
          <button aria-pressed={band === "visible"} onclick={() => (band = "visible")}>Visible</button>
          <button aria-pressed={band === "xray"} onclick={() => (band = "xray")}>X-ray</button>
        </span>
        <small>{band === "visible" ? "Actual colour" : "Mapped colour: X-rays have no colour; brightness is the corona's emission measure on a log scale"}</small>
      </dd>
      <dt>Surface</dt>
      <dd>{Math.round(got.teff).toLocaleString("en-GB")} K, darker towards the limb (grey atmosphere)</dd>
      <dt>Activity</dt>
      <dd data-regime>{got.activity.regime}{spin_days ? `, turning every ${spin_days.toFixed(1)} days (${got.activity.rotation_from})` : ""}</dd>
      <dt>Spots</dt>
      <dd data-spot-words>{spots.length ? `${spots.length} covering ${(coverage * 100).toFixed(coverage < 0.01 ? 2 : 1)}%${cyc ? `, at about ${Math.round(28 - 20 * cyc.phase)}° from the equator, ${Math.round(cyc.phase * 100)}% through the cycle` : ", spread to high latitudes (no clean cycle)"}` : "None"}</dd>
      <dt>X-rays</dt>
      <dd>{pow(Number(got.moment.xray_w))}; corona scale height {(H * got.radius / 1e6).toFixed(0)} thousand km at {(CORONA_K / 1e6).toFixed(1)} MK</dd>
      <dt>Flares</dt>
      <dd data-flares>
        {#if active.length}<strong>Flaring now.</strong>{/if}
        {#if next.length}
          Next above 10²⁴ J: {next.map((f) => `${when(f.at_s)} (${(f.energy_j / 1e24).toPrecision(2)} × 10²⁴ J)`).join(", ")}.
        {:else}
          None above 10²⁴ J in the coming year.
        {/if}
      </dd>
      <dt>Why</dt>
      <dd class="why">Spot places are drawn from the star and the cycle (the kernel gives coverage, not maps); spots are {SPOT_DELTA_K} K cooler; the corona is taken at {(CORONA_K / 1e6).toFixed(1)} MK. Turn with the arrow keys. Sources: {STAR_REFERENCES.map((r) => r.text.split(".")[0]).join("; ")}.</dd>
    </dl>
  {/if}
</div>

<style>
  .star {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    align-items: flex-start;
  }
  canvas {
    max-width: 100%;
    display: block;
  }
  .disc {
    max-width: 100%;
    border-radius: 50%;
  }
  .disc:focus-visible {
    outline: 2px solid var(--sse-accent);
    outline-offset: 4px;
  }
  .facts {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 4px 12px;
    font-size: 13px;
    margin: 0;
    max-width: 380px;
  }
  dt {
    color: var(--sse-ink-muted);
  }
  dd {
    margin: 0;
  }
  .seg button {
    font: inherit;
    font-size: 12px;
    background: none;
    color: var(--sse-ink);
    border: 1px solid var(--sse-line);
    padding: 1px 8px;
    cursor: pointer;
  }
  .seg button[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  small,
  .why {
    color: var(--sse-ink-muted);
    font-size: 12px;
    display: block;
  }
  .note {
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
</style>
