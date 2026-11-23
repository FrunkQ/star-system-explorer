<!--
  World sheet on the chart room (X3a): the stacked pack's codes for the world
  in view (every digit with its why), or for every body of the system when no
  world is picked. Reads the kernel's `codes` at the campaign's now.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import type { Json, SystemScene, ViewContext } from "../../../ui-contract/src/index.ts";
  import WorldCodes from "../../world-codes/src/WorldCodes.svelte";
  import { inputFromPacks, sheetOf, type CodesResponse, type WorldCodesInput } from "../../world-codes/src/sheet.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const session = untrack(() => ctx.session);
  const ws = $derived(ctx.workspace);
  const packs = $derived.by(() => {
    void ws.revision;
    const s = session.stack();
    return [...s.base, ...(s.campaign ? [s.campaign] : [])] as unknown[];
  });
  const sheet = $derived(sheetOf(packs));
  const names = $derived(Object.fromEntries(scene.subjects.map((s) => [s.address, s.label])));
  const picked = $derived(ws.selection && scene.subjects.some((s) => s.address === ws.selection) ? ws.selection : null);

  let input = $state<WorldCodesInput | null>(null);
  let note = $state<string | null>(null);
  $effect(() => {
    const scope = picked ?? scene.system;
    void ws.revision;
    if (!sheet || !scope) return;
    untrack(async () => {
      const now = await session.campaignNow();
      const at_s = now.ok && now.at_s !== undefined ? now.at_s : 0n;
      const r = await session.codes?.({ scope, at_s });
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
    <WorldCodes scene={input} params={params as Record<string, unknown>} onpick={(a) => (ws.selection = a)} />
  {:else if note}
    <p class="lede">{note}</p>
  {:else}
    <p class="lede muted">Reading the codes…</p>
  {/if}
</div>

<style>
  .sheet { color: var(--sse-ink); font-size: 13px; }
  .lede { font-size: 14px; max-width: 46em; }
  .muted { color: var(--sse-ink-muted); }
</style>
