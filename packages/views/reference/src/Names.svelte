<!--
  The Names page of the Reference screen (NM6, naming design 4.12), under "Names and classes": how this campaign
  names things. Three parts: the campaign's defaults (which scheme names each layer, and the pack that set it); the
  schemes on the stack, each with its pack; and a live sample of the chosen scheme, twelve names for a chosen kind
  with "say it" beneath (Curious), IPA and meanings (Science), and the form and digest (Engine). Samples are the
  kernel's keyed draws (name_sample), the same on every machine. When no pack on the stack carries names, the page
  shows the base names pack's schemes from a trial on the scratch kernel and offers to add it. Nothing here touches
  physics.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import NamesHere from "./NamesHere.svelte";
  import { layerWords, meaningWords, namesOnStack, sampleKinds, saysMore, sortWords, stackPacks, type Depth, type NameSample, type PackDoc, type ViewSession, type Workspace } from "../../../ui-contract/src/index.ts";

  let { session, workspace, depth, offer = "", here = null }: { session: ViewSession; workspace: Workspace; depth: Depth; offer?: string; here?: { address: string; label: string } | null } = $props();
  const full = $derived(depth !== "curious");

  const info = $derived.by(() => {
    void workspace.revision;
    return session.stack();
  });
  const packs = $derived([...info.base, ...(info.campaign ? [info.campaign] : [])]);
  const onStack = $derived(namesOnStack(packs));

  // No names on the stack: offer the base names pack, shown from a trial until it is added.
  let offered = $state.raw<PackDoc | null>(null);
  let offerFailed = $state<string | null>(null);
  $effect(() => {
    if (onStack.schemes.length || offered) return;
    if (!offer) return void (offerFailed = "this screen offers no names pack");
    untrack(() => {
      void fetch(new URL(offer, document.baseURI))
        .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`HTTP ${r.status}`))))
        .then((p: PackDoc) => (offered = p))
        .catch((e: Error) => (offerFailed = e.message));
    });
  });
  const trying = $derived(!onStack.schemes.length && !!offered);
  const shown = $derived(trying ? namesOnStack([offered!]) : onStack);

  let chosen = $state<string | null>(null);
  const scheme = $derived(shown.schemes.find((s) => s.id === chosen) ?? shown.schemes.find((s) => s.id === shown.defaults.find((d) => d.layer === "common")?.scheme) ?? shown.schemes.find((s) => s.sort !== "catalogue") ?? shown.schemes[0] ?? null);
  let kind = $state("planet");
  const kinds = $derived(scheme ? sampleKinds(scheme) : []);
  const kindShown = $derived(kinds.includes(kind) ? kind : (kinds[0] ?? "planet"));
  let start = $state(0);

  let sample = $state.raw<NameSample | null>(null);
  let sampleError = $state<string | null>(null);
  let asked = 0;
  $effect(() => {
    const ask = scheme ? { scheme: scheme.id, kind: kindShown, count: 12, start } : null;
    const extra = trying ? offered : null;
    const added = info.added;
    if (!ask) return;
    untrack(() => {
      const n = ++asked;
      sampleError = null;
      const got = extra
        ? session.trial({ added: [...added, extra] }, (p) => p.nameSample(ask)).then((t) => (t.ok ? t.result : { ok: false as const, code: "stack", message: t.report }))
        : session.nameSample(ask);
      void got.then((r) => {
        if (n !== asked) return;
        if (r.ok) sample = r.value;
        else sampleError = r.message;
      });
    });
  });

  const editable = $derived(workspace.mode === "edit" && session.seat.authority === "gm");
  let note = $state<string | null>(null);
  let adding = $state(false);
  async function addOffered() {
    if (!offered) return;
    adding = true;
    const r = await stackPacks(session, workspace, [...info.added, offered]);
    adding = false;
    note = r.status === "applied" ? "The base names pack is in this campaign. Worlds now carry its names; remove it on the Packs screen." : r.status === "refused" ? r.reason : "Not added.";
  }
  const labelOf = (id: string) => shown.schemes.find((s) => s.id === id)?.label ?? id;
</script>

<div class="names" data-names={trying ? "offered" : shown.schemes.length ? "stack" : offerFailed ? "none" : "loading"}>
  <section class="part" aria-labelledby="names-defaults">
    <h2 id="names-defaults">This campaign</h2>
    {#if trying}
      <p class="note" data-names-offer>No pack in this campaign carries names yet, so worlds go by their catalogue labels. The base names pack has the schemes below: catalogue designations, English-sounding names, ten real languages and two invented ones.</p>
      {#if editable}
        <button type="button" class="btn ink" data-names-add onclick={addOffered} disabled={adding}>Add the base names pack</button>
      {:else}
        <p class="muted">Choose the Build desk to add it.</p>
      {/if}
    {:else if !shown.schemes.length}
      <p class="note">{offerFailed ? `No pack in this campaign carries names, and the base names pack could not be read (${offerFailed}).` : "Reading the name schemes…"}</p>
    {/if}
    {#if note}<p class="note" role="status">{note}</p>{/if}
    {#if shown.defaults.length}
      <dl class="defaults" data-names-defaults>
        {#each shown.defaults as d (d.layer)}
          <dt>{layerWords(d.layer)}</dt>
          <dd data-layer={d.layer}>{labelOf(d.scheme)}{#if full}<span class="muted">{` · ${d.pack}`}</span>{/if}</dd>
        {/each}
        {#if !shown.defaults.some((d) => d.layer === "common")}<dt>Common</dt><dd class="muted">none: worlds keep the names they were given</dd>{/if}
      </dl>
    {/if}
    {#if full}<p class="muted small">A setting on a system or world names everything beneath it, and a name you type always wins.</p>{/if}
    {#if here && onStack.schemes.length}<NamesHere {session} {workspace} address={here.address} label={here.label} {depth} />{/if}
  </section>

  {#if shown.schemes.length}
    <section class="part" aria-labelledby="names-schemes">
      <h2 id="names-schemes">Schemes</h2>
      <ul class="schemes" aria-label="Name schemes">
        {#each shown.schemes as s (s.id)}
          <li>
            <button type="button" class="scheme" aria-pressed={scheme?.id === s.id} data-scheme={s.id} onclick={() => ((chosen = s.id), (start = 0))}>
              <span class="label">{s.label}</span>
              <span class="k">{sortWords(s.sort)}</span>
            </button>
          </li>
        {/each}
      </ul>
    </section>

    {#if scheme}
      <section class="part sample" aria-labelledby="names-sample" data-sample={scheme.id}>
        <h2 id="names-sample">{scheme.label}</h2>
        <p class="about">{scheme.about}</p>
        {#if full}<p class="muted small">From {scheme.pack}{scheme.mix ? `; draws from ${scheme.mix.map(([id, w]) => `${labelOf(id)} (${w})`).join(", ")}` : ""}.</p>{/if}
        <div class="controls">
          <label>Names for <select bind:value={kind} onchange={() => (start = 0)} aria-label="Kind of thing">{#each kinds as k (k)}<option value={k}>{k === "small-body" ? "small body" : k}s</option>{/each}</select></label>
          <button type="button" class="btn" data-names-more onclick={() => (start += 12)}>Another twelve</button>
        </div>
        {#if sampleError}
          <p class="note" role="alert">The sample could not be drawn: {sampleError}</p>
        {:else if !sample}
          <p class="muted" aria-busy="true">Drawing names…</p>
        {:else}
          <ol class="drawn" aria-label="Sample names" data-digest={sample.digest} aria-busy={sample.scheme !== scheme.id}>
            {#each sample.names as n, i (i)}
              {@const means = meaningWords(n)}
              <li data-name={n.text}>
                <span class="name">{n.text}</span>
                {#if n.native && n.native !== n.text}<span class="native" lang="und">{n.native}</span>{/if}
                {#if saysMore(n)}<span class="say">say {n.say}{#if full && n.ipa} <span class="ipa">{n.ipa}</span>{/if}</span>{/if}
                {#if full && means}<span class="means">means {means}</span>{/if}
                {#if depth === "engine"}<code class="form">{n.form}</code>{/if}
              </li>
            {/each}
          </ol>
          {@const hints = Object.entries(Object.assign({}, ...sample.names.map((n) => n.hints ?? {})) as Record<string, string>)}
          {#if full && hints.length}<p class="muted small hints" data-hints>{#each hints as [h, how], i (h)}{i ? "; " : ""}<b>{h}</b> {how}{/each}.</p>{/if}
          {#if depth === "engine"}<p class="muted small">Names {start + 1} to {start + 12} of the sample stream; digest <code>{sample.digest.slice(0, 12)}</code>.</p>{/if}
        {/if}
      </section>
    {/if}
  {/if}
</div>

<style>
  .names {
    display: grid;
    grid-template-columns: minmax(0, 260px) minmax(0, 260px) minmax(0, 1fr);
    gap: 20px;
    align-items: start;
  }
  .part {
    min-width: 0;
  }
  h2 {
    margin: 0 0 8px;
    font-family: var(--sse-font-display);
    font-size: 17px;
    font-weight: 600;
  }
  p {
    margin: 0 0 10px;
    line-height: 1.5;
  }
  .note {
    color: var(--sse-ink);
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .small {
    font-size: 13px;
  }
  .k {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .defaults {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 4px 12px;
    margin: 0 0 10px;
  }
  .defaults dt {
    color: var(--sse-ink-muted);
  }
  .defaults dd {
    margin: 0;
  }
  .schemes {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
  }
  .scheme {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    width: 100%;
    height: auto;
    padding: 6px 10px;
    border: var(--sse-line) solid transparent;
    background: none;
    color: var(--sse-ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .scheme:hover {
    border-color: var(--sse-rule);
  }
  .scheme[aria-pressed="true"] {
    border-color: var(--sse-ink);
    background: var(--sse-ground-raised);
  }
  .about {
    max-width: 60ch;
  }
  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 16px;
    align-items: center;
    margin: 4px 0 12px;
  }
  .drawn {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 10px 18px;
  }
  .drawn li {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .name {
    font-family: var(--sse-font-display);
    font-size: 18px;
    overflow-wrap: anywhere;
  }
  .native,
  .say,
  .means,
  .form {
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .hints {
    margin-top: 12px;
  }
  .ipa {
    font-family: var(--sse-font-mono);
  }
  .btn {
    height: auto;
    padding: 6px 12px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    cursor: pointer;
  }
  .btn.ink {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  /* A phone: one column, the schemes as a sideways strip. */
  @media (max-width: 720px) {
    .names {
      grid-template-columns: minmax(0, 1fr);
    }
    .schemes {
      grid-auto-flow: column;
      grid-auto-columns: max-content;
      overflow-x: auto;
    }
  }
</style>
