<!--
  Map colours (territory overlays OV4): the theme's map palette, eight colours tuned to its ground, each paired with a
  pattern for themes and eyes that need more than colour, and a sample sector drawn in the Atlas look, so a GM sees how
  map layers (powers, claims, influence) will read under the theme in force before any campaign has one. The sample is
  drawn by the same code as every map: holdings turned into areas by the kernel's reach rule (areasFromHoldings),
  slots given so neighbours never share one (assignSlots), drawn by Overlays with its key. On the Build and Run desks a
  holding the table does not know is shown in pencil. Presentation only: the sample is not campaign data.
-->
<script lang="ts">
  import { AreaKey, Overlays } from "../../../chart-grammar/src/index.ts";
  import { MAP_PATTERNS, MAP_SLOTS, areasFromHoldings, assignSlots, insideRings, mapSlotToken, type AreaItem, type Holding, type MapSlot, type ViewOverlay } from "../../../ui-contract/src/index.ts";

  let { oneColour, gm }: { oneColour: boolean; gm: boolean } = $props();

  // A made-up sector, 40 by 24 parsecs, its layer reaching 6 pc: three powers, a claim across a border, a guild's
  // influence, and a secret holding.
  const STARS: { place: string; name: string; at: [number, number] }[] = [
    { place: "s1", name: "Tarn", at: [6, 6] },
    { place: "s2", name: "Halloran", at: [10, 13] },
    { place: "s3", name: "Mire", at: [4, 19] },
    { place: "s4", name: "Cassel", at: [15, 5] },
    { place: "s5", name: "Ostra", at: [19, 11] },
    { place: "s6", name: "Vey", at: [17, 19] },
    { place: "s7", name: "Lumen", at: [25, 6] },
    { place: "s8", name: "Arden", at: [27, 14] },
    { place: "s9", name: "Quill", at: [24, 21] },
    { place: "s10", name: "Sabre", at: [33, 4] },
    { place: "s11", name: "Ness", at: [35, 12] },
    { place: "s12", name: "Corran", at: [32, 20] },
    { place: "s13", name: "Hollow", at: [38, 21] },
  ];
  const POWERS: Record<string, string> = { crown: "Meridian Crown", league: "Free League", vesh: "Vesh Compact", lamps: "Guild of Lamps", watch: "The Quiet Watch" };
  const h = (place: string, key: string, more: Partial<Holding> = {}): Holding => ({ place, layer: "powers", key, status: "holds", via: "held", why: "", ...more });
  const HOLDINGS: Holding[] = [
    h("s1", "crown"),
    h("s2", "crown"),
    h("s3", "crown"),
    h("s4", "crown"),
    h("s5", "league"),
    h("s6", "league"),
    h("s7", "league"),
    h("s8", "league"),
    h("s10", "vesh"),
    h("s11", "vesh"),
    h("s12", "vesh"),
    h("s5", "crown", { status: "claims" }),
    h("s9", "lamps", { status: "influences", strength: 0.7, extent: { reach: { value: 7, unit: "pc" } } }),
    h("s13", "watch"),
  ];
  const SECRET = new Set(["watch"]);
  const W = 40;
  const H = 24;

  const slots = $derived.by(() => {
    // Answers touch when any of their places lie within two reaches of each other (the guild's 7 pc is the longest).
    const keys = [...new Set(HOLDINGS.map((x) => x.key))];
    const where = (k: string) => HOLDINGS.filter((x) => x.key === k).map((x) => STARS.find((s) => s.place === x.place)!.at);
    const touching: [string, string][] = [];
    for (const a of keys) for (const b of keys) if (a < b && where(a).some((p) => where(b).some((q) => Math.hypot(p[0] - q[0], p[1] - q[1]) <= 2 * 7))) touching.push([a, b]);
    return assignSlots(keys.map((key) => ({ key })), touching);
  });
  const inkOf = (key: string) => {
    const slot = slots.get(key) ?? 1;
    return oneColour ? { slot, pattern: MAP_PATTERNS[slot - 1]! } : { slot };
  };
  const items = $derived<AreaItem[]>(
    areasFromHoldings(
      HOLDINGS.filter((x) => gm || !SECRET.has(x.key)),
      { frame: "galaxy-plane-pc", bounds: [0, 0, W, H], positionOf: (p) => STARS.find((s) => s.place === p)?.at ?? null, inkOf, nameOf: (k) => POWERS[k], pencil: (x) => SECRET.has(x.key), reach: 6, cells: 120 },
    ),
  );

  let w = $state(640);
  const px = $derived(Math.max(260, Math.min(720, w)));
  const k = $derived(px / W);
  const at = (p: readonly number[]) => ({ x: p[0]! * k, y: p[1]! * k });
  const overlays = $derived<ViewOverlay[]>([{ id: "sample-powers", label: "Powers", pack: "sample", frame: "galaxy-plane-pc", layers: [{ kind: "areas", as: "fill", items }] }]);
  let lift = $state<string | null>(null);
  let pointed = $state<AreaItem | null>(null);
  let picked = $state<string | null>(null);
  const holderAt = (p: [number, number]) => items.find((i) => i.status === "holds" && insideRings(i.rings, p))?.key;
  const pointWords = (it: AreaItem) => {
    const name = POWERS[it.key] ?? it.key;
    const how = it.status === "holds" ? "holds this" : it.status === "claims" ? "claims this" : `has influence here${it.strength !== undefined ? ` (${Math.round(it.strength * 100)}%)` : ""}`;
    return `${name} ${how}${it.pencil ? ". Not known to the table" : ""}.`;
  };
</script>

<section aria-labelledby="sym-map" data-map-colours>
  <h2 id="sym-map">Map colours</h2>
  <p class="muted">Map layers (powers, claims, influence) use the theme's own eight map colours, tuned to its ground and checked for every colour vision; each has a pattern for when colour cannot carry it. The sample below is drawn the way every map is: a band of colour along the inside of each border, a bare middle, names lettered along the land.</p>
  <ul class="slots" aria-label="The map palette">
    {#each MAP_SLOTS as s (s)}
      <li data-map-slot={s}>
        <svg width="34" height="22" viewBox="0 0 34 22" aria-hidden="true">
          <rect x="1" y="1" width="32" height="20" rx="2" style:fill="var({mapSlotToken(s as MapSlot)})" fill-opacity="0.3" style:stroke="var({mapSlotToken(s as MapSlot)})" stroke-width="1.4" />
        </svg>
        <span class="small">{s}, {MAP_PATTERNS[s - 1]}</span>
      </li>
    {/each}
  </ul>

  <div class="plate" bind:clientWidth={w}>
    <svg class="sample" width={px} height={H * k} viewBox="0 0 {px} {H * k}" role="img" aria-label="A sample sector in the Atlas look" data-map-sample={items.length}>
      <rect width={px} height={H * k} class="ground" />
      <Overlays {overlays} {at} scale={k} {lift} onpoint={(it) => (pointed = it)} onpick={(it) => (picked = it.key)} />
      {#each STARS as s (s.place)}
        {@const p = at(s.at)}
        <circle cx={p.x} cy={p.y} r="2.6" class="star" />
        <text x={p.x + 5} y={p.y - 4} class="sname" class:held={!!holderAt(s.at)}>{s.name}</text>
      {/each}
    </svg>
    <p class="muted small point" role="status" data-map-point>{pointed ? pointWords(pointed) : picked ? `${POWERS[picked] ?? picked}: its card opens here on a real map.` : "Point at a territory for who holds it; press it for its card."}</p>
  </div>
  <AreaKey {items} nameOf={(k) => POWERS[k] ?? k} bind:lift onpick={(k) => (picked = k)} title="Key for the sample" />
  {#if gm}<p class="muted small">The Quiet Watch is in pencil: the table does not know it, so only the GM's map shows it.</p>{/if}
</section>

<style>
  section {
    display: grid;
    gap: 8px;
  }
  h2 {
    margin: 0;
  }
  p {
    margin: 0;
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .small {
    font-size: 12px;
  }
  .slots {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 6px 12px;
  }
  .slots li {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .plate {
    max-width: 720px;
  }
  .sample {
    display: block;
    border: var(--sse-line) solid var(--sse-rule);
  }
  .ground {
    fill: var(--sse-chart-ground, var(--sse-ground));
  }
  .star {
    fill: var(--sse-ink);
  }
  .sname {
    font-family: var(--sse-font-body);
    font-size: 10.5px;
    fill: var(--sse-ink-muted);
    paint-order: stroke;
    stroke: var(--sse-ground);
    stroke-width: 3px;
  }
  .sname.held {
    fill: var(--sse-ink);
  }
  .point {
    min-height: 1.5em;
    margin-top: 4px;
  }
</style>
