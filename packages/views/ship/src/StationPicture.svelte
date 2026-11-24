<script lang="ts">
  // A station's picture on its card: the record read from the construct's facts and its template's words, the
  // builder's rules from the base-ships and base-tech packs, drawn by StationPlans. Kept on the construct as
  // `construct.design` once the GM picks a shape or remixes on the Build desk; play desks see it locked.
  import StationPlans from "./StationPlans.svelte";
  import { stationPackData, type StationPlace, type StationRecipe, type StationRecord, type StationSkin } from "../../../renderer/src/index.ts";
  import type { PlayerSubject } from "../../../ui-contract/src/index.ts";

  let {
    construct,
    name,
    words,
    tags = [],
    place = "space",
    gravity_g,
    locked = true,
    onchange,
  }: {
    construct: PlayerSubject;
    name: string;
    /** The template's explain and any notes: what picks the shape. */
    words?: string;
    tags?: string[];
    place?: StationPlace;
    /** On the ground, the world's surface gravity in standard gravities. */
    gravity_g?: number;
    locked?: boolean;
    onchange?: (r: StationRecipe) => void;
  } = $props();

  const SHIPS = "packs/base-ships.json";
  const TECH = "packs/base-tech.json";
  type Loaded = { data: NonNullable<ReturnType<typeof stationPackData>>; skin: StationSkin };
  let loaded = $state<Loaded | null | "failed">(null);
  $effect(() => {
    let live = true;
    void Promise.all([fetch(SHIPS).then((r) => r.json()), fetch(TECH).then((r) => r.json())])
      .then(([ships, tech]) => {
        const s = ships?.provides?.other ?? {};
        const data = stationPackData(s, tech?.provides?.other ?? {});
        if (live) loaded = data && s.station_skin ? { data, skin: s.station_skin as StationSkin } : "failed";
      })
      .catch(() => live && (loaded = "failed"));
    return () => (live = false);
  });

  const fact = (f: string) => construct.facts.find((x) => x.field === f)?.value;
  const num = (f: string) => (typeof fact(f) === "number" ? (fact(f) as number) : undefined);
  const record = $derived.by<StationRecord>(() => {
    const modules = fact("construct.modules");
    const r: StationRecord = { name, population: num("construct.crew_max") ?? num("construct.crew") ?? 1, place, tags, words: words ?? "" };
    const mass = num("construct.mass_kg");
    const mw = num("construct.power_mw");
    const cargo = num("construct.cargo_t");
    if (mass !== undefined) r.mass_kg = mass;
    if (mw !== undefined) r.power_w = mw * 1e6;
    if (cargo !== undefined) r.cargo_t = cargo;
    if (Array.isArray(modules)) r.modules = modules.map(String);
    if (gravity_g !== undefined) r.gravity_g = gravity_g;
    return r;
  });
  const kept = $derived.by<StationRecipe | undefined>(() => {
    const d = fact("construct.design");
    const o = typeof d === "string" ? (() => { try { return JSON.parse(d); } catch { return null; } })() : d;
    return o && typeof o === "object" && typeof Reflect.get(o, "shape") === "string" ? (o as StationRecipe) : undefined;
  });
</script>

{#if loaded === "failed"}
  <p class="muted">The station builder's rules (base-ships and base-tech packs) did not load, so the station is not drawn.</p>
{:else if loaded}
  {#key construct.address}
    <StationPlans station={record} rules={loaded.data.rules} shapes={loaded.data.shapes} limits={loaded.data.limits} skin={loaded.skin} {...kept ? { recipe: kept } : {}} {locked} {...onchange ? { onchange } : {}} />
  {/key}
{/if}

<style>
  .muted { color: var(--sse-ink-muted); font-size: 0.85em; }
</style>
