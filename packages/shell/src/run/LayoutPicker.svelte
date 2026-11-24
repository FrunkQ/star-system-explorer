<!-- A role's layout on the Switchboard (PB13, design 7.4), owned by the "Run desk and players' side" build: every
     layout the campaign has (shipped beside the shell, and made), choosing one changes the role's screens at once;
     "Edit this layout…" and "+ New layout…" open the one layout editor in place (new is the editor on a blank copy; a
     shipped layout is edited as a copy of its own). Saving keeps the layout with the campaign and gives it to the role. -->
<script lang="ts">
  import { SHIPPED_LAYOUTS, audienceOf, blankLayout, copyLayout, keepLayout, layoutIdFor, type PlayerScene, type ScreenDoc, type SeatPort, type Shown, type ViewSession, type Workspace } from "../../../ui-contract/src/index.ts";
  import LayoutEditor from "./LayoutEditor.svelte";

  let { seats, role, ws, session, live, onclose }: { seats: SeatPort; role: string; ws?: Workspace; session?: ViewSession; live: Shown; onclose: () => void } = $props();

  const seat = $derived(seats.plan().seats.find((s) => s.id === role));
  const current = $derived(seat?.screens?.[0] ?? "");
  const made = $derived(ws?.layouts ?? []);
  let editing = $state<ScreenDoc | null>(null);
  let failed = $state<string | null>(null);
  const taken = $derived([...made.map((d) => d.id), ...SHIPPED_LAYOUTS.map((l) => l.file)]);

  async function shipped(file: string): Promise<ScreenDoc> {
    const r = await fetch(new URL(`documents/${file || "players.screen.json"}`, document.baseURI));
    if (!r.ok) throw new Error(`no layout ${file}`);
    return (await r.json()) as ScreenDoc;
  }
  function choose(id: string) {
    seats.update(role, { screen: id || null });
    onclose();
  }
  async function edit() {
    failed = null;
    try {
      const mine = made.find((d) => d.id === current);
      editing = mine ? (structuredClone($state.snapshot(mine)) as ScreenDoc) : copyLayout(await shipped(current), taken);
    } catch (e) {
      failed = String(e);
    }
  }
  async function fresh() {
    failed = null;
    try {
      editing = blankLayout(await shipped(""), `${seat?.label ?? "New"} layout`, taken);
    } catch (e) {
      failed = String(e);
    }
  }
  function save(edited: ScreenDoc) {
    // A layout saved for the first time takes its id from its name.
    const doc = made.some((d) => d.id === edited.id) ? edited : { ...edited, id: layoutIdFor(edited.label, taken) };
    if (ws) ws.layouts = keepLayout(made, doc);
    seats.update(role, { screen: doc.id });
    editing = null;
    onclose();
  }

  // The preview: what the role knows of the place in view, as its screens would draw it.
  let scene = $state.raw<PlayerScene | null>(null);
  $effect(() => {
    const s = session;
    const st = seat;
    const scope = "scope" in live ? live.scope : undefined;
    if (!editing || !s || !st || !scope) return;
    void s.scene(scope, { audience: audienceOf(st) }).then((r) => (scene = r.ok ? r.scene : null));
  });
</script>

<div class="picker" role="group" aria-label="Layout of {seat?.label ?? role}" data-layout-picker={role}>
  {#if editing}
    <LayoutEditor layout={editing} {scene} onsave={save} onclose={() => (editing = null)} />
  {:else}
    <ul class="list">
      {#each SHIPPED_LAYOUTS as l (l.file)}
        <li><button type="button" aria-pressed={current === l.file} data-layout-choose={l.file || "bridge"} onclick={() => choose(l.file)}>{l.label}</button></li>
      {/each}
      {#each made as d (d.id)}
        <li><button type="button" aria-pressed={current === d.id} data-layout-choose={d.id} onclick={() => choose(d.id)}>{d.label} <span class="sub">made</span></button></li>
      {/each}
    </ul>
    <div class="row">
      <button type="button" class="quiet" data-layout-edit onclick={edit}>Edit this layout…</button>
      <button type="button" class="quiet" data-layout-new onclick={fresh}>+ New layout…</button>
      <button type="button" class="quiet" data-layout-close onclick={onclose}>Close</button>
    </div>
    {#if failed}<p class="sub" data-layout-failed>{failed}</p>{/if}
  {/if}
</div>

<style>
  .picker {
    display: grid;
    gap: 6px;
    padding: 8px 10px;
    margin: 4px 0 8px;
    border: var(--sse-line) solid var(--sse-rule);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
  }
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .list button {
    font: inherit;
    padding: 2px 10px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground);
    color: var(--sse-ink);
    cursor: pointer;
  }
  .list button[aria-pressed="true"] {
    border-color: var(--sse-accent);
    color: var(--sse-accent-ink, var(--sse-accent));
    font-weight: 600;
  }
  .row {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }
  .sub {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
</style>
