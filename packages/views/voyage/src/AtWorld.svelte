<!--
  What the ship can do where the voyage ends (V8, kernel ship_at_world): the
  ship designed from its own numbers (and, on request, fitted for this world and watched through an aerobraking
  pass), then each act at the world (aerocapture,
  entry, runway landing, powered landing, take-off) with how close it runs to
  its limits, what could go wrong, a rough chance and what would make it safe.
  Nothing is refused and nothing is rolled: the GM makes the call. The kernel
  does every sum; this only words the answer.
-->
<script lang="ts">
  import { chanceWords, difficultyLadderOf, gameDifficulty, verdictWords, worstDangers, type ShipAtWorld, type ShipAtWorldAsk, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { readPropellants, type Propellants, type ShipRecord } from "../../../renderer/src/index.ts";
  import { ShipAtWorldView } from "../../ship/src/index.ts";

  // `arriving`: how the voyage gets there, in words, when the ask carries the leg's own speed.
  // `name`: the world's name as the chart shows it, used when the kernel's report carries only its address.
  let { ctx, ask, arriving, name }: { ctx: ViewContext; ask: ShipAtWorldAsk | null; arriving?: string; name?: string } = $props();
  // A game pack's own task ladder (X3h): its numbers beside SSE4's words, on the GM's screens only; nothing rolled.
  const ladder = $derived.by(() => {
    void ctx.workspace.revision;
    if (ctx.session.seat.authority !== "gm") return null;
    const st = ctx.session.stack();
    return difficultyLadderOf([...st.base, ...(st.campaign ? [st.campaign] : [])] as unknown as Parameters<typeof difficultyLadderOf>[0]);
  });
  let world = $state<ShipAtWorld | null>(null);
  let ship = $state<ShipRecord | null>(null);
  let fuels = $state<Propellants>({});
  let failed = $state<string | null>(null);

  const cap = (t: string) => `${t[0]!.toUpperCase()}${t.slice(1)}`;
  let gen = 0;
  $effect(() => {
    const a = ask;
    if (!a || !a.worlds.length) return void (world = null);
    const g = ++gen;
    ctx.session.shipAtWorld(a).then(async (r) => {
      if (g !== gen) return;
      if (!r.ok) return void ((failed = r.message), (world = null));
      failed = null;
      const w = r.report.worlds[0] ?? null;
      world = w && name && (!w.name || w.name === w.body || w.name.includes("/body:")) ? { ...w, name } : w;
      const s = r.report.ship as unknown as ShipRecord;
      ship = s;
      const f = await readPropellants(ctx.session, s.fuels ?? []).catch(() => ({}));
      if (g === gen) fuels = f;
    });
  });

  const acts = $derived((world?.checks ?? []).filter((c) => c.verdict !== "not-here"));
  const notHere = $derived((world?.checks ?? []).filter((c) => c.verdict === "not-here"));
</script>

{#if failed}<p class="warn" data-at-world-failed>{failed}</p>{/if}
{#if world}
  <section class="at" data-at-world={world.body}>
    <h4>At {world.name}</h4>
    {#if ship && ask}<ShipAtWorldView {ctx} {ask} {ship} {world} {fuels} />{/if}
    <p class="muted">{world.v_inf_m_s < 1 ? "Arrives with its drive braked to the world's own speed" : `${cap(arriving ?? world.arrival)}, ${(world.v_inf_m_s / 1000).toFixed(1)} km/s over escape speed`}. Nothing here is refused or rolled: the dangers are for you to weigh.</p>
    <ul class="acts">
      {#each acts as c (c.id)}
        {@const ds = worstDangers(c)}
        <li data-act={c.id} data-verdict={c.verdict}>
          <span class="act">{c.words}</span>
          <span class="verdict" data-level={c.verdict}>{verdictWords(c.verdict)}</span>
          {#each ds as d (d.id)}
            <p class="danger" data-danger={d.id} data-level={d.level}>
              {d.why}. If it goes wrong ({chanceWords(d.chance.p)}): {d.if_wrong}.
              {#if gameDifficulty(ladder, { p_wrong: d.chance.p, role: d.role })}{@const g = gameDifficulty(ladder, { p_wrong: d.chance.p, role: d.role })!}<span class="game" data-game-difficulty title={`${g.system}: a typical operator's task for this chance, from the game pack; nothing is rolled`}>{g.words}</span>{/if}
            </p>
          {/each}
          {#if c.would_pass?.length}<p class="muted">Safer with {c.would_pass.join("; or ")}.</p>{/if}
          <details>
            <summary>Why?</summary>
            <ul>{#each c.why as w, i (i)}<li>{w}</li>{/each}</ul>
            {#if ds.length}<p class="muted">Chances: {ds[0]!.chance.basis}.</p>{/if}
          </details>
        </li>
      {/each}
    </ul>
    {#each notHere as c (c.id)}<p class="muted" data-not-here={c.id}>{c.words}: {c.why[0]}.</p>{/each}
  </section>
{/if}

<style>
  .at {
    margin-top: 12px;
    border-top: 1px solid var(--sse-rule);
    padding-top: 8px;
  }
  h4 {
    margin: 0 0 6px;
    font-size: 14px;
  }
  .acts {
    list-style: none;
    padding: 0;
    margin: 8px 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .act {
    font-weight: 600;
    margin-right: 8px;
  }
  .verdict {
    font-family: var(--sse-font-mono, monospace);
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .verdict[data-level="dangerous"],
  .danger[data-level="danger"] {
    color: var(--sse-alert, var(--sse-accent));
  }
  .game {
    margin-left: 6px;
    padding: 0 5px;
    border: 1px solid var(--sse-rule);
    font-family: var(--sse-font-mono, monospace);
    font-size: 0.9em;
    white-space: nowrap;
  }
  .danger {
    margin: 4px 0 0;
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .warn {
    color: var(--sse-alert, var(--sse-accent));
  }
  details {
    margin-top: 4px;
    color: var(--sse-ink-muted);
  }
  summary {
    cursor: pointer;
  }
</style>
