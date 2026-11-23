<!--
  The designed ship at a world (V3 gap item 14, stages 2 to 4, on the Voyages "At world" card): the ship as it is,
  drawn from its parts, and on request the same ship fitted for what this world asks of it (a shield to aerobrake, lift,
  legs, a parachute), with the kernel's new verdicts, and the aerobraking pass watched in 3D at its hottest. The
  kernel judges every pass; the renderer sizes and draws. Rules and styles come from the base-ships pack.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { designShip, fitAbilities, passGlow, type AbilityRules, type DesignRules, type DesignShip, type FitResult, type Propellants, type ShipStyle } from "../../../renderer/src/index.ts";
  import type { ShipAtWorld, ShipAtWorldAsk, ViewContext } from "../../../ui-contract/src/index.ts";
  import ShipPlans from "./ShipPlans.svelte";

  let { ctx, ask, ship, world, fuels = {} }: { ctx: ViewContext; ask: ShipAtWorldAsk; ship: DesignShip; world: ShipAtWorld; fuels?: Propellants } = $props();

  type Pack = { rules: DesignRules & { abilities: AbilityRules }; styles: Record<string, ShipStyle> };
  let pack = $state<Pack | null>(null);
  let missing = $state(false);
  // The shell serves the pack beside the page (scripts/build-shell.sh).
  const SHIPS_PACK = "packs/base-ships.json";
  $effect(() => {
    void fetch(new URL(SHIPS_PACK, document.baseURI))
      .then((r) => r.json())
      .then((p: { provides: { other: { ship_design: Pack["rules"]; ship_styles: Pack["styles"] } } }) => (pack = { rules: p.provides.other.ship_design, styles: p.provides.other.ship_styles }))
      .catch(() => (missing = true));
  });

  // What this world asks of a ship: air to brake in, ground to land on.
  const can = (id: string) => world.checks.some((c) => c.id === id && c.verdict !== "not-here");
  const asks = $derived({ aerobrake: can("aerocapture") ? [world.name] : [], land: can("entry") || can("powered-landing") ? [world.name] : [] });
  const askable = $derived(asks.aerobrake.length + asks.land.length > 0);

  let fitted = $state<FitResult | null>(null);
  let busy = $state(false);
  let failed = $state<string | null>(null);
  let watching = $state(false);
  // A new world or ship starts from the ship as it is.
  $effect(() => {
    void [world.body, ship.name];
    untrack(() => ((fitted = null), (watching = false), (failed = null)));
  });
  async function fit() {
    if (!pack) return;
    busy = true;
    failed = null;
    try {
      const style = Object.keys(pack.styles)[0]!;
      const layout = designShip(ship, fuels, pack.rules, style, pack.styles[style]!, 1);
      const where = { ...(ask.fixture ? { fixture: ask.fixture } : {}), ...(ask.nav ? { nav: ask.nav } : {}), ...(ask.system ? { system: ask.system } : {}), ...(ask.at_s !== undefined ? { at_s: ask.at_s } : {}), ...(ask.v_inf_m_s !== undefined ? { v_inf_m_s: ask.v_inf_m_s } : {}) };
      fitted = await fitAbilities((a) => ctx.session.shipAtWorld(a), where, ship, layout, pack.styles[style]!.hull, asks, pack.rules.abilities);
    } catch (e) {
      failed = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
  // The pass at its hottest, from the fitted ship's own aerocapture check.
  const pass = $derived.by(() => {
    const w = fitted?.report?.worlds.find((x) => x.body === world.body || x.name === world.name);
    const q = w?.checks.find((c) => c.id === "aerocapture")?.figures?.peak_heat_w_m2;
    return fitted?.fit.shield && q ? passGlow(q, pack?.rules.abilities.shield_emissivity ?? 0.85) : null;
  });
</script>

{#if pack}
  <div class="ship-at" data-ship-at-world>
    {#key fitted}
      <ShipPlans {ship} {fuels} rules={pack.rules} styles={pack.styles} segments={24} {...fitted ? { abilities: fitted } : {}} />
    {/key}
    <div class="bar">
      {#if askable && !fitted}
        <button type="button" onclick={fit} disabled={busy} data-fit-for-world>{busy ? "Fitting…" : `Fit it for ${world.name}`}</button>
        <span class="muted">Sizes a heat shield, lift, legs or a parachute for what {world.name} asks, and asks the kernel again.</span>
      {/if}
      {#if pass}
        <button type="button" aria-pressed={watching} onclick={() => (watching = !watching)} data-watch-pass>Watch the pass</button>
      {/if}
      {#if failed}<span class="warn">{failed}</span>{/if}
    </div>
    {#if watching && pass && fitted}
      <figure data-pass-view>
        <ShipPlans {ship} {fuels} rules={pack.rules} styles={pack.styles} segments={32} abilities={fitted} {pass} />
        <figcaption class="muted">
          The aerobraking pass at its hottest: the shield's face at about {Math.round(pass.shield_k).toLocaleString("en-GB")} K (radiative balance with the kernel's peak heating), the bow shock standing off its nose, radiators folded behind it. The shock layer's glow is drawn for the eye.
        </figcaption>
      </figure>
    {/if}
  </div>
{:else if missing}
  <p class="muted">The ship designer's rules (base-ships pack) did not load, so the ship is not drawn.</p>
{/if}

<style>
  .ship-at { display: grid; gap: 6px; margin: 6px 0; }
  .bar { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .muted { color: var(--sse-ink-muted); font-size: 0.85em; }
  .warn { color: var(--sse-alert); }
  figure { margin: 0; }
</style>
