<!--
  A star on the Body screen (R5a): its photosphere in its own colour, darkened
  to the limb, with spots where its activity cycle puts them, or its corona in
  X-rays. Facts from the kernel: temperature, radius and mass from the system,
  activity (spin, spots, cycle, X-rays, flares, surface convection) from read_activity.
-->
<script lang="ts">
  import { constant, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { cycleActivity, drawCorona, narrateCorona, churningGranules, coronaScaleHeight, CORONA_K, cycleOf, drawStar, flaresAt, flareWhiteLight, limbLumps, narrateLumps, patchContrast, patchRadiusDeg, readLiveStar, GRANULE_DT, granulesShow, narrateGranules, narrateStar, SPOT_DELTA_K, spotsFor, STAR_REFERENCES, type Convection, type LiveStar, type StarDraw, type StarReader } from "../../../renderer/src/index.ts";

  let { ctx, scene, address, label, size }: { ctx: ViewContext; scene: SystemScene; address: string; label: string; size: number } = $props();

  let got = $state<LiveStar | null>(null);
  let failed = $state<string | null>(null);
  let band = $state<StarDraw["light"]>("visible");
  let turn = $state(0);
  /** The disc hidden, as a world in front or a coronagraph hides it, so the visible corona shows (VP-S3). */
  let hidden = $state(false);
  let canvas = $state<HTMLCanvasElement | undefined>();

  // The star as the kernel has it (live-star.ts, shared with the system view's True star).
  $effect(() => {
    const a = address;
    const sys = scene.system;
    got = null;
    failed = null;
    let live = true;
    void readLiveStar(ctx.session as unknown as StarReader, sys, a).then((r) => {
      if (!live) return;
      if (typeof r === "string") failed = r;
      else got = r;
    });
    return () => {
      live = false;
    };
  });

  const cyc = $derived(got ? cycleOf(got) : null);
  const coverage = $derived(Number(got?.moment.spot_coverage ?? 0));
  const spots = $derived(got ? spotsFor(address, coverage, cyc) : []);
  const H = $derived(got ? coronaScaleHeight(got.mass, got.radius) : 0.1);
  const now = $derived(got?.age_s ?? 0);
  const active = $derived(got ? flaresAt(got, spots, now) : []);
  const next = $derived((got?.flares ?? []).filter((f) => f.at_s > now).sort((x, y) => x.at_s - y.at_s).slice(0, 5));
  // Granules: the kernel's cells, drawn when each is at least a few pixels wide on this disc. A generation of cells
  // lives granule_lifetime_s, so the picture shows the generation alive at the star's present moment.
  const conv = $derived((got?.activity.convection ?? null) as unknown as Convection | null);
  const discPx = $derived(Math.round(size * Math.min(2, typeof devicePixelRatio === "number" ? devicePixelRatio : 1) * 0.92));
  // Cells as the True view draws them (star-churn.ts), held at the star's present: each at its own age in its life.
  const granules = $derived(conv && granulesShow(conv.granules_across, discPx) ? churningGranules(address, conv.granules_across, now, conv.granule_lifetime_s) : null);
  // The limb lifted by the cells by the kernel's scale height: lumpy on a red supergiant, round on the Sun.
  const lumps = $derived(got ? limbLumps(conv, got.radius) : 0);
  const spin_days = $derived(Number(got?.activity.rotation_period_s ?? 0) / constant("day"));

  $effect(() => {
    const c = canvas;
    const g = got;
    if (!c || !g) return;
    const n = Math.round(size * Math.min(2, typeof devicePixelRatio === "number" ? devicePixelRatio : 1));
    c.width = c.height = n;
    const x = c.getContext("2d")!;
    const img = x.createImageData(n, n);
    if (hidden && band === "visible") {
      drawCorona(img, g, 0.3);
      x.putImageData(img, 0, 0);
      return;
    }
    drawStar(img, { teff_k: g.teff, spots, lon_deg: turn, lat_deg: 7, light: band, corona_h: H, flares: active.map(({ lat_deg, lon_deg, strength, flare }) => {
        // The kernel's white-light patch (star-events.ts): its own area and temperature, at its strength now over its peak.
        const wl = flareWhiteLight(flare);
        const k = (strength * Math.max(1, Number(g.moment.xray_w))) / Math.max(1, flare.peak_w);
        return { lat_deg, lon_deg, strength, ...(wl ? { patch: { radius_deg: patchRadiusDeg(wl.area_m2, g.radius), contrast: patchContrast(wl.t_k, g.teff), k } } : {}) };
      }), granules, lumps }, band === "visible" ? 0.92 : 0.6);
    x.putImageData(img, 0, 0);
  });

  const pow = (w: number) => `${(w / 1e20).toPrecision(2)} × 10²⁰ W`;
  const when = (s: number) => {
    const d = (s - now) / constant("day");
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
    <div class="disc" tabindex="0" role="img" aria-label={narrateStar(label, got.teff, spots.length, coverage, String(got.activity.regime), band)} onkeydown={key} data-band={band} data-spots={spots.length} data-granules={granules ? granules.n : 0}>
      <canvas bind:this={canvas} style={`width:${size}px;height:${size}px`}></canvas>
    </div>
    <dl class="facts">
      <dt>View</dt>
      <dd>
        <span class="seg" role="group" aria-label="Band">
          <button aria-pressed={band === "visible"} onclick={() => (band = "visible")}>Visible</button>
          <button aria-pressed={band === "xray"} onclick={() => (band = "xray")}>X-ray</button>
        </span>
        {#if band === "visible"}<button class="hide" aria-pressed={hidden} onclick={() => (hidden = !hidden)} data-hide-disc>Hide the disc</button>{/if}
        <small data-corona-words={hidden && band === "visible" ? "yes" : "no"}>{band === "visible" ? (hidden ? narrateCorona(cycleActivity(got), !!cyc) : "Actual colour") : "Mapped colour: X-rays have no colour; brightness is the corona's emission measure on a log scale"}</small>
      </dd>
      <dt>Surface</dt>
      <dd>{Math.round(got.teff).toLocaleString("en-GB")} K, darker towards the limb (grey atmosphere)</dd>
      <dt>Granules</dt>
      <dd data-granule-words>{narrateGranules(conv, discPx)}{granules && narrateLumps(lumps, discPx) ? ` ${narrateLumps(lumps, discPx)}` : ""}</dd>
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
      <dd class="why">Spot places are drawn from the star and the cycle (the kernel gives coverage, not maps); spots are {SPOT_DELTA_K} K cooler; the corona is taken at {(CORONA_K / 1e6).toFixed(1)} MK; granule cells are placed from the star (the kernel gives their size and life) with a temperature spread of {(GRANULE_DT * 100).toFixed(1)}%, set from the Sun's measured contrast. Turn with the arrow keys. Sources: {STAR_REFERENCES.map((r) => r.text.split(".")[0]).join("; ")}.</dd>
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
  .seg button,
  .hide {
    font: inherit;
    font-size: 12px;
    background: none;
    color: var(--sse-ink);
    border: 1px solid var(--sse-line);
    padding: 1px 8px;
    cursor: pointer;
  }
  .hide {
    margin-left: 6px;
  }
  .seg button[aria-pressed="true"],
  .hide[aria-pressed="true"] {
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
