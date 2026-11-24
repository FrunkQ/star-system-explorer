<!--
  "Names here" for a whole place (NM6, naming design 4.12): which scheme names each layer of a system (or any place)
  and everything beneath it. Self-contained, for any card that shows a system as one thing (the sector and galaxy
  charts' system card, the place line): mount it with the session, the workspace, the place's address and label, and
  the depth. It reads the place's own names (read_names) to say where each layer's names come from now, and on the
  Build desk sends set_naming per layer: a scheme, or "As set above" to take the setting away; optionally for one
  kind of thing only, and optionally renaming names already kept. A typed name always wins and is never touched.
  Names never reach the physics.
-->
<script lang="ts">
  import { layerOrder, layerWords, namesOnStack, sourceWords, type Depth, type IntentResult, type NamesRead, type ViewSession, type Workspace } from "../../../ui-contract/src/index.ts";

  let { session, workspace, address, label, depth }: { session: ViewSession; workspace: Workspace; address: string; label: string; depth: Depth } = $props();
  const editable = $derived(workspace.mode === "edit" && session.seat.authority === "gm");
  const full = $derived(depth !== "curious");

  const stack = $derived.by(() => {
    void workspace.revision;
    const st = session.stack();
    return namesOnStack([...st.base, ...(st.campaign ? [st.campaign] : [])]);
  });
  const schemes = $derived(stack.schemes);
  const schemeLabel = (id: string) => schemes.find((s) => s.id === id)?.label ?? id;

  let read = $state.raw<NamesRead | null>(null);
  let failed = $state<string | null>(null);
  let reads = $state(0);
  $effect(() => {
    const a = address;
    void reads;
    failed = null;
    void session.readNames(a).then((r) => {
      if (a !== address) return;
      if (r.ok) read = r.value;
      else failed = r.message;
    });
  });
  // Every layer the campaign names by default, and every layer this place has a name in.
  const layers = $derived([...new Set([...stack.defaults.map((d) => d.layer), ...(read?.names ?? []).map((n) => n.layer), "designation", "common"])].map((layer) => ({ layer })).sort(layerOrder).map((l) => l.layer));
  const nameIn = (layer: string) => read?.names.find((n) => n.layer === layer) ?? null;

  const KINDS: { id: string; label: string }[] = [
    { id: "", label: "everything" },
    { id: "planet", label: "planets" },
    { id: "moon", label: "moons" },
    { id: "star", label: "stars" },
    { id: "small-body", label: "small bodies" },
  ];
  let ofKind = $state("");
  let renameKept = $state(false);
  let note = $state<string | null>(null);
  let busy = $state(false);
  async function nameHere(layer: string, scheme: string | null) {
    busy = true;
    const r: IntentResult = await session.send({ kind: "set_naming", address, layer, scheme, ...(ofKind ? { of_kind: ofKind } : {}), ...(renameKept ? { rename_fixed: true } : {}), at_yr: workspace.now_yr });
    busy = false;
    const what = `${ofKind ? (KINDS.find((k) => k.id === ofKind)?.label ?? ofKind) : "everything"} in ${label}`;
    note =
      r.status === "applied"
        ? scheme === null
          ? `${layerWords(layer)} names for ${what} follow the setting above again.`
          : `${layerWords(layer)} names for ${what} now come from ${schemeLabel(scheme)}${renameKept ? ", kept names too" : ""}. Typed names stay as they are.`
        : r.status === "pending"
          ? "Sent to the GM to approve."
          : r.status === "refused"
            ? r.reason
            : `The kernel refused this: ${r.message}`;
    if (r.status === "applied") reads++;
  }
</script>

<div class="nameshere" role="region" aria-label="Names here in {label}" data-names-here={address}>
  <span class="k">Names here</span>
  {#if !schemes.length}
    <p class="muted">No pack in this campaign carries names; Reference, Names can add the base names pack.</p>
  {:else if failed}
    <p class="muted">The names could not be read: {failed}</p>
  {:else}
    <dl>
      {#each layers as layer (layer)}
        {@const n = nameIn(layer)}
        <dt>{layerWords(layer)}</dt>
        <dd data-layer={layer}>
          {#if n}<span class="now">{n.text}</span><span class="muted small">{sourceWords(n, schemeLabel)}</span>{:else}<span class="muted small">{read ? "none here" : "reading…"}</span>{/if}
          {#if editable}
            <select aria-label="Name {layerWords(layer).toLowerCase()} in {label} with" data-names-here-layer={layer} disabled={busy} value="" onchange={(e) => { const v = (e.currentTarget as HTMLSelectElement).value; (e.currentTarget as HTMLSelectElement).value = ""; if (v) void nameHere(layer, v === "-" ? null : v); }}>
              <option value="">Name with…</option>
              {#each schemes as sc (sc.id)}<option value={sc.id}>{sc.label}</option>{/each}
              <option value="-">As set above</option>
            </select>
          {/if}
        </dd>
      {/each}
    </dl>
    {#if editable && full}
      <div class="opts">
        <label>For <select bind:value={ofKind} aria-label="Which things" data-names-here-kind>{#each KINDS as k (k.id)}<option value={k.id}>{k.label}</option>{/each}</select></label>
        <label><input type="checkbox" bind:checked={renameKept} data-names-here-kept /> rename names already kept</label>
      </div>
    {/if}
    {#if !editable}<p class="muted small">Choose the Build desk to name {label} and everything in it from a scheme.</p>{:else if full}<p class="muted small">A choice here names {label} and everything beneath it; a typed name always wins.</p>{/if}
  {/if}
  {#if note}<p class="muted small" role="status" data-names-here-note>{note}</p>{/if}
</div>

<style>
  .nameshere {
    padding: 8px 10px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
    font-size: 14px;
    color: var(--sse-ink);
  }
  .k {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  dl {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 4px 10px;
    margin: 6px 0;
  }
  dt {
    color: var(--sse-ink-muted);
  }
  dd {
    margin: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 2px 8px;
    align-items: baseline;
  }
  .now {
    font-family: var(--sse-font-display);
  }
  select {
    font: inherit;
    font-size: 13px;
    max-width: 100%;
  }
  .opts {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 14px;
    font-size: 13px;
  }
  p {
    margin: 4px 0 0;
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .small {
    font-size: 12.5px;
  }
</style>
