<!--
  The name card (NM6, naming design 4.12), opened from "Names" on a body card: every name the thing has, by layer,
  with where it came from (typed, drawn from a scheme and kept, or drawn when read), how to say it, what its parts
  mean, and under a typed name the one it would have had. Read through the kernel's read_names, so a player seat sees
  only the layers its view discloses. On the Build desk the GM can type a name for a layer (rename), put a typed one
  back to its drawn name, and choose the scheme that names a layer here and beneath ("Names here", set_naming). The
  common name is typed in the card's Name field, as before.
-->
<script lang="ts">
  import { layerOrder, layerWords, meaningWords, namesOnStack, saysMore, sourceWords, type Depth, type IntentResult, type NamesRead, type ViewSession, type Workspace } from "../../../ui-contract/src/index.ts";

  let { session, workspace, address, label, depth, onclose }: { session: ViewSession; workspace: Workspace; address: string; label: string; depth: Depth; onclose: () => void } = $props();
  const editable = $derived(workspace.mode === "edit" && session.seat.authority === "gm");
  const full = $derived(depth !== "curious");

  let read = $state.raw<NamesRead | null>(null);
  let failed = $state<string | null>(null);
  let reads = $state(0);
  $effect(() => {
    const a = address;
    void reads;
    failed = null;
    failed = null;
    void session.readNames(a).then((r) => {
      if (a !== address) return;
      if (r.ok) read = r.value;
      else failed = r.message;
    });
  });
  const schemes = $derived.by(() => {
    const st = session.stack();
    return namesOnStack([...st.base, ...(st.campaign ? [st.campaign] : [])]).schemes;
  });
  const schemeLabel = (id: string) => schemes.find((s) => s.id === id)?.label ?? id;
  const names = $derived([...(read?.names ?? [])].sort(layerOrder));

  // The GM's changes: one intent each, then the card reads its names again.
  let typing = $state<string | null>(null);
  let typed = $state("");
  let note = $state<string | null>(null);
  async function send(intent: Parameters<ViewSession["send"]>[0], done: string) {
    const r: IntentResult = await session.send(intent);
    note = r.status === "applied" ? done : r.status === "pending" ? "Sent to the GM to approve." : r.status === "refused" ? r.reason : `The kernel refused this: ${r.message}`;
    if (r.status === "applied") reads++;
  }
  function rename(layer: string, text: string | null) {
    typing = null;
    void send({ kind: "rename", address, layer, text, at_yr: workspace.now_yr }, text === null ? `${layerWords(layer)} name put back.` : `${layerWords(layer)} name typed.`);
  }
  function nameHere(layer: string, scheme: string | null) {
    void send({ kind: "set_naming", address, layer, scheme, at_yr: workspace.now_yr }, scheme === null ? `${layerWords(layer)} names follow the setting above again.` : `${label} and everything beneath take ${layerWords(layer).toLowerCase()} names from ${schemeLabel(scheme)}.`);
  }
</script>

<div class="namecard" role="region" aria-label="Names of {label}" data-name-card={address}>
  <div class="head">
    <span class="k">Names</span>
    <button type="button" class="close" onclick={onclose} aria-label="Close names">×</button>
  </div>
  {#if failed}
    <p class="muted">The names could not be read: {failed}</p>
  {:else if !read}
    <p class="muted" aria-busy="true">Reading names…</p>
  {:else if !names.length}
    <p class="muted">No names here beyond what the card shows.</p>
  {:else}
    <dl>
      {#each names as n (n.layer)}
        {@const means = n.named ? meaningWords({ ...n.named, text: n.text }) : ""}
        <dt>{layerWords(n.layer)}</dt>
        <dd data-layer={n.layer} data-source={n.source}>
          <span class="name">{n.text}</span>
          {#if n.named?.native && n.named.native !== n.text}<span class="native" lang="und">{n.named.native}</span>{/if}
          <span class="from">{sourceWords(n, schemeLabel)}</span>
          {#if n.named && saysMore({ ...n.named, text: n.text })}<span class="say">say {n.named.say}{#if full && n.named.ipa} <span class="ipa">{n.named.ipa}</span>{/if}</span>{/if}
          {#if full && means}<span class="say">means {means}</span>{/if}
          {#if n.original && n.original !== n.text}<span class="say" data-original>would otherwise be {n.original}</span>{/if}
          {#if depth === "engine" && n.named}<code class="form">{n.named.form}</code>{/if}
          {#if editable}
            <span class="acts">
              {#if n.layer !== "common"}
                {#if typing === n.layer}
                  <input
                    aria-label="{layerWords(n.layer)} name"
                    data-name-input={n.layer}
                    bind:value={typed}
                    onkeydown={(e) => {
                      if (e.key === "Enter" && typed.trim()) (e.preventDefault(), rename(n.layer, typed));
                      if (e.key === "Escape") (e.stopPropagation(), (typing = null));
                    }} />
                {:else}
                  <button type="button" class="link" data-name-type={n.layer} onclick={() => ((typing = n.layer), (typed = n.text))}>Type a name</button>
                {/if}
                {#if n.source === "typed"}<button type="button" class="link" data-name-restore={n.layer} onclick={() => rename(n.layer, null)}>Put back {n.original ?? "the drawn name"}</button>{/if}
              {/if}
              {#if schemes.length}
                <select aria-label="Name {layerWords(n.layer).toLowerCase()} here with" data-name-here={n.layer} value="" onchange={(e) => { const v = (e.currentTarget as HTMLSelectElement).value; (e.currentTarget as HTMLSelectElement).value = ""; if (v) nameHere(n.layer, v === "-" ? null : v); }}>
                  <option value="">Names here…</option>
                  {#each schemes as sc (sc.id)}<option value={sc.id}>{sc.label}</option>{/each}
                  <option value="-">As set above</option>
                </select>
              {/if}
            </span>
          {/if}
        </dd>
      {/each}
    </dl>
  {/if}
  {#if note}<p class="muted" role="status" data-name-note>{note}</p>{/if}
</div>

<style>
  .namecard {
    margin: 8px 18px 10px;
    padding: 10px 12px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
    font-size: 14px;
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 6px;
  }
  .k {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .close {
    height: auto;
    border: none;
    background: none;
    color: var(--sse-ink-muted);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 12px;
    margin: 0;
  }
  dt {
    color: var(--sse-ink-muted);
  }
  dd {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .name {
    font-family: var(--sse-font-display);
    font-size: 16px;
    overflow-wrap: anywhere;
  }
  .from,
  .say,
  .native,
  .form {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .acts {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
    align-items: center;
    margin-top: 2px;
    font-size: 12px;
  }
  .acts input,
  .acts select {
    font: inherit;
    max-width: 100%;
  }
  .link {
    height: auto;
    padding: 0;
    border: none;
    background: none;
    color: var(--sse-accent-ink);
    font: inherit;
    text-decoration: underline;
    cursor: pointer;
  }
  .ipa {
    font-family: var(--sse-font-mono);
  }
  .muted {
    margin: 0;
    color: var(--sse-ink-muted);
  }
</style>
