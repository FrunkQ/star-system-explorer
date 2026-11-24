<!--
  World sheet on the chart room (X3a): the stacked pack's codes for the world
  in view (every digit with its why), or for every body of the system when no
  world is picked. Reads the kernel's `codes` at the campaign's now.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import type { Json, ProfileRolled, SystemScene, ViewContext } from "../../../ui-contract/src/index.ts";
  import WorldCodes from "../../world-codes/src/WorldCodes.svelte";
  import { inputFromPacks, sheetOf, sheetsOf, type CodesResponse, type WorldCodesInput } from "../../world-codes/src/sheet.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const session = untrack(() => ctx.session);
  const ws = $derived(ctx.workspace);
  const packs = $derived.by(() => {
    void ws.revision;
    const s = session.stack();
    return [...s.base, ...(s.campaign ? [s.campaign] : [])] as unknown[];
  });
  const sheets = $derived(sheetsOf(packs));
  /** Which sheet to read through (a pack may give more than one: the profile line, an extended profile). */
  let chosen = $state<string | undefined>(undefined);
  const sheet = $derived(sheetOf(packs, chosen && sheets.some((s) => s.id === chosen) ? chosen : undefined));
  const names = $derived(Object.fromEntries(scene.subjects.map((s) => [s.address, s.label])));
  const picked = $derived(ws.selection && scene.subjects.some((s) => s.address === ws.selection) ? ws.selection : null);

  // WBH part 2: a pack's rolled profiles for the picked world (GM only, seeded, a preview before anything is written).
  type RollDef = { id: string; label?: string; about?: string; sheet: string; source?: string };
  const programs = $derived(packs.flatMap((p) => ((p as { provides?: { other?: { rolls?: RollDef[] } } })?.provides?.other?.rolls ?? [])));
  let rolled = $state<ProfileRolled | null>(null);
  let rollNote = $state<string | null>(null);
  async function rollIt(program: string, apply: boolean) {
    if (!picked) return;
    const now = await session.campaignNow();
    const at_s = now.ok && now.at_s !== undefined ? now.at_s : 0n;
    const r = await session.rollProfile?.({ program, body: picked, apply, at_s });
    if (!r) return void (rollNote = "This seat cannot roll profiles.");
    if (!r.ok) return void ((rollNote = r.message), (rolled = null));
    if (apply) {
      const n = r.result.steps.filter((x) => x.how === "rolled" || x.how === "zero").length;
      rollNote = `${n} code${n === 1 ? "" : "s"} written as hand-set codes, each with how it was rolled. Change any of them by hand.`;
      rolled = null;
      ws.revision++;
    } else ((rolled = r.result), (rollNote = null));
  }

  let input = $state<WorldCodesInput | null>(null);
  let note = $state<string | null>(null);
  $effect(() => {
    const scope = picked ?? scene.system;
    void ws.revision;
    const id = sheet?.id;
    if (!sheet || !scope) return;
    untrack(async () => {
      const now = await session.campaignNow();
      const at_s = now.ok && now.at_s !== undefined ? now.at_s : 0n;
      const r = await session.codes?.({ scope, ...(id ? { sheet: id } : {}), at_s });
      if (!r) return void (note = "This seat cannot read codes.");
      if (!r.ok) return void ((note = r.message), (input = null));
      const resp = r.result as unknown as CodesResponse;
      note = resp.readings.length ? null : "The kernel holds no facts for this system yet: adopt it (or import it from a sector) and its codes read here.";
      input = resp.readings.length ? inputFromPacks(packs, resp, { names, ...(picked ? { selected: picked } : {}), title: picked ? names[picked] : scene.title }) : null;
    });
  });
</script>

<div class="sheet" role="region" aria-label="World codes">
  {#if !sheet}
    <p class="lede">No pack in the stack gives a code sheet. Add a game pack under Workshop, Packs, and each world's codes read here.</p>
  {:else if input}
    {#if sheets.length > 1}
      <label class="pick">Read as
        <select bind:value={chosen} data-code-sheet>
          {#each sheets as s, i}<option value={i === 0 ? undefined : s.id}>{s.label ?? s.id}</option>{/each}
        </select>
      </label>
    {/if}
    <WorldCodes scene={input} params={params as Record<string, unknown>} onpick={(a) => (ws.selection = a)} />
    {#if picked && session.seat.authority === "gm"}
      {#each programs.filter((p) => p.sheet === (sheet?.id ?? "")) as p}
        <section class="roll" aria-label={p.label ?? p.id} data-roll={p.id}>
          <button type="button" class="quiet-btn" title={p.about} onclick={() => rollIt(p.id, false)} data-roll-preview>{p.label ?? p.id}…</button>
          {#if rolled && rolled.program === p.id && rolled.body === picked}
            <ul class="steps">
              {#each rolled.steps as st}
                <li class:muted={st.how === "kept" || st.how === "skipped"}><b>{st.label}</b> {st.symbol} <span class="muted">· {st.why}</span></li>
              {/each}
            </ul>
            <button type="button" class="go" onclick={() => rollIt(p.id, true)} data-roll-apply>Write these codes</button>
            {#if p.source}<p class="small muted">{p.source}</p>{/if}
          {/if}
        </section>
      {/each}
      {#if rollNote}<p class="small" aria-live="polite" data-roll-note>{rollNote}</p>{/if}
    {/if}
  {:else if note}
    <p class="lede">{note}</p>
  {:else}
    <p class="lede muted">Reading the codes…</p>
  {/if}
</div>

<style>
  .sheet { color: var(--sse-ink); font-size: 13px; }
  .lede { font-size: 14px; max-width: 46em; }
  .quiet-btn, .go { font: inherit; min-height: 34px; padding: 0 10px; border: 1px solid var(--sse-rule); background: transparent; color: var(--sse-ink); cursor: pointer; display: inline-flex; align-items: center; transition: transform var(--sse-motion-quick, 120ms) var(--sse-ease, ease-out); }
  .quiet-btn:hover, .go:hover { transform: translateY(-1px); border-color: var(--sse-ink); }
  .go { background: var(--sse-ink); color: var(--sse-ground); border-color: var(--sse-ink); }
  @media (prefers-reduced-motion: reduce) { .quiet-btn, .go { transition: none; } }
  .roll { margin-top: 10px; display: flex; flex-direction: column; gap: 6px; align-items: flex-start; }
  .steps { margin: 0; padding-left: 18px; }
  .small { font-size: 12px; }
  .pick { display: inline-flex; gap: 6px; align-items: center; margin-bottom: 8px; color: var(--sse-ink-muted); }
  .muted { color: var(--sse-ink-muted); }
</style>
