<script lang="ts">
  // The party's ship on a players' screen (track MC, MC8): its picture, who and what is aboard, and its tanks, as far
  // as the table knows them. Read from this seat's own scene, so the kernel has already left out what the table does
  // not know (Known to, and each fact's tier); nothing here can change the ship (Alex, 24 Nov: play never changes
  // the ship itself, and a players' screen changes nothing at all). A sealed screen has no kernel: it says so.
  import { FUEL_FIELDS, STATE_WORDS, fuelGauges, seenAboard, shipRecordOf, tonnesWords, type FuelShip, type FuelTank, type PartyShip, type PlayerScene, type ViewContext, type ViewSession } from "../../../ui-contract/src/index.ts";
  import Picture from "./Picture.svelte";

  let { session, party, tick = "" }: { session: ViewSession | null; party: PartyShip; /** Changes when the table's view changes (a scene hash): the card reads again. */ tick?: string } = $props();

  let scene = $state.raw<PlayerScene | null>(null);
  let fuelShip = $state.raw<FuelShip | null>(null);
  let note = $state<string | null>(null);
  /** Only the ship's system could be read: who and what is aboard is not known here. */
  let partial = $state(false);
  let gen = 0;
  $effect(() => {
    void tick;
    const s = session;
    const p = party;
    const g = ++gen;
    if (!s) return void ((scene = null), (note = "This screen shows only what the game master sends; the ship's card needs a screen that runs the campaign."));
    // Everyone and everything aboard lives at the universe's root, so the card reads the root scene; if that cannot be
    // read here, the ship's own system still gives its picture and tanks.
    const sys = p.ship.split("/").slice(0, 2).join("/");
    void s.scene(s.universe.root).then(async (r) => {
      if (g !== gen) return;
      if (r.ok) return void ((scene = r.scene), (note = null), (partial = false));
      const own = await s.scene(sys);
      if (g !== gen) return;
      scene = own.ok ? own.scene : null;
      partial = own.ok;
      note = own.ok ? null : own.message;
    });
    void shipRecordOf(s, p.nav).then((r) => g === gen && (fuelShip = r.ok ? (r.ship as unknown as FuelShip) : null));
  });

  const ship = $derived(scene?.subjects.find((x) => x.address === party.ship) ?? null);
  const seen = $derived(scene ? seenAboard(scene.subjects, party.ship) : { people: [], cargo: [], other: [] });
  // A tank the table has not yet learned the level of reads as unknown, never as the default full.
  const fact = (k: string) => ship?.facts.find((f) => f.field === k);
  const gauges = $derived(fuelShip && ship ? fuelGauges(fuelShip, (k) => fact(k)?.value) : []);
  const unknownTank = (t: FuelTank) => fact(FUEL_FIELDS[t])?.placeholder !== undefined;
  const TANK_WORDS: Record<FuelTank, string> = { propellant: "Reaction mass", jump: "Jump fuel" };
  // The shared Pictures panel reads only its session here (nothing is kept from a players' screen).
  const ctx = $derived(session ? ({ session, workspace: {}, params: {}, mark: () => ({}) } as unknown as ViewContext) : null);
</script>

<section class="party-card" aria-label={`${party.name}, the party's ship`} data-party-card>
  <h3>{party.name}</h3>
  {#if note}<p class="muted small" data-party-card-note>{note}</p>{/if}
  {#if scene && !ship}
    <p class="muted small" data-party-card-unknown>The table knows nothing more about this ship yet.</p>
  {:else if ship && ctx}
    <Picture {ctx} construct={ship} nav={party.nav} name={party.name} />

    {#if partial}<p class="muted small" data-party-card-partial>Who and what is aboard cannot be read on this screen.</p>{:else}
    <h5>Crew</h5>
    {#if seen.people.length}
      <ul class="list" data-party-crew>
        {#each seen.people as e (e.address)}
          <li data-person={e.name}><strong>{e.name}</strong>{#if e.role}<span class="muted"> · {e.role}</span>{/if}{#if e.being}<span class="muted"> · {e.being}</span>{/if}{#if e.state && e.state !== "awake"}<span class="muted"> · {STATE_WORDS[e.state]}</span>{/if}</li>
        {/each}
      </ul>
    {:else}<p class="muted small">No one the table knows of.</p>{/if}

    <h5>Manifest</h5>
    {#if seen.cargo.length || seen.other.length}
      <ul class="list" data-party-cargo>
        {#each [...seen.cargo, ...seen.other] as e (e.address)}
          <li data-lot={e.name}><strong>{e.name}</strong>{#if e.amount !== undefined}<span class="muted"> · {e.amount}{e.unit ? ` ${e.unit}` : ""}</span>{/if}{#if e.what && e.what !== e.name}<span class="muted"> · {e.what}</span>{/if}</li>
        {/each}
      </ul>
    {:else}<p class="muted small">Nothing the table knows of.</p>{/if}
    {/if}

    {#if gauges.length}
      <h5>Fuel</h5>
      <div class="gauges">
        {#each gauges as g (g.tank)}
          <div class="gauge" data-fuel={g.tank}>
            <span>{TANK_WORDS[g.tank]}</span>
            {#if unknownTank(g.tank)}
              <span class="muted small" data-fuel-unknown>not known</span><span></span>
            {:else}
              <span class="bar-track" class:low={g.share < 0.25} role="meter" aria-valuemin="0" aria-valuemax={g.full_t} aria-valuenow={g.now_t} aria-label={`${TANK_WORDS[g.tank]}: ${Math.round(g.share * 100)}% full`} data-now={g.now_t}><span class="bar-fill" style:width="{g.share * 100}%"></span></span>
              <span class="mono">{tonnesWords(g.now_t)} of {tonnesWords(g.full_t)}</span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</section>

<style>
  .party-card { display: flex; flex-direction: column; gap: 4px; min-width: min(420px, calc(100vw - 32px)); max-width: min(560px, calc(100vw - 32px)); }
  h3 { margin: 0; font-size: 16px; }
  h5 { margin: 8px 0 0; font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--sse-ink-muted); }
  .list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; }
  .gauges { display: grid; grid-template-columns: max-content minmax(80px, 1fr) max-content; gap: 6px 10px; align-items: center; margin: 4px 0; }
  .gauge { display: contents; }
  .bar-track { display: block; height: 10px; border: 1px solid var(--sse-rule); background: var(--sse-ground); }
  .bar-fill { display: block; height: 100%; background: var(--sse-ink); }
  .bar-track.low .bar-fill { background: var(--sse-alert, var(--sse-accent)); }
  .small { font-size: 12px; margin: 0; }
  .muted { color: var(--sse-ink-muted); }
  .mono { font-family: var(--sse-font-mono, monospace); font-size: 12px; }
</style>
