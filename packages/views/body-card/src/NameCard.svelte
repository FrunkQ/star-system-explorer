<!--
  The name card (NM6, naming design 4.12), opened from "Names" on a body card: every name the thing has, by layer,
  with where it came from (typed, drawn from a scheme and kept, or drawn when read), how to say it, what its parts
  mean, and under a typed name the one it would have had. Read through the kernel's read_names, so a player seat sees
  only the layers its view discloses. Read only: typing a name stays on the card's Name field.
-->
<script lang="ts">
  import { layerOrder, layerWords, meaningWords, namesOnStack, saysMore, sourceWords, type Depth, type NamesRead, type ViewSession } from "../../../ui-contract/src/index.ts";

  let { session, address, label, depth, onclose }: { session: ViewSession; address: string; label: string; depth: Depth; onclose: () => void } = $props();
  const full = $derived(depth !== "curious");

  let read = $state.raw<NamesRead | null>(null);
  let failed = $state<string | null>(null);
  $effect(() => {
    const a = address;
    read = null;
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
        </dd>
      {/each}
    </dl>
  {/if}
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
  .ipa {
    font-family: var(--sse-font-mono);
  }
  .muted {
    margin: 0;
    color: var(--sse-ink-muted);
  }
</style>
