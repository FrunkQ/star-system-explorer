<!--
  Make your own lens (U9, H8.6): start from the lens in use, switch layers,
  tools and card fields on or off, save it under a name. The GM's lenses live
  in this browser's profile; they leave it as a presentation pack (a file the
  Packs screen can stack, or another GM can load here), and come back
  identical. Nothing here changes data.
-->
<script lang="ts">
  import { LAYERS, TOOLS, chartNameLayers, checkLens, copyLens, lensPack, lensesFromPack, saved, type Density, type LayerId, type Lens, type LensCard, type ToolId } from "../../ui-contract/src/index.ts";

  let { from, lenses, fieldsByKind, kinds, onsave, ondelete }: { from: Lens; lenses: Lens[]; fieldsByKind: Record<string, { id: string; label: string }[]>; kinds: string[]; onsave: (l: Lens) => void; ondelete: (id: string) => void } = $props();

  const own = $derived(from.origin === "mine");
  // A working copy: the GM's own lens is edited in place, any other is copied first.
  const start = (): Lens => structuredClone(saved(from));
  let draft = $state<Lens>(start());
  let label = $state("");
  let kind = $state("*");
  let problem = $state("");
  let note = $state("");
  $effect(() => {
    void from.id;
    draft = start();
    label = from.origin === "mine" ? from.label : `${from.label} (mine)`;
    kind = "*";
  });

  const fields = $derived(fieldsByKind[kind] ?? fieldsByKind["*"] ?? []);
  const rule = $derived(draft.cards.find((c) => c.subject_kind === kind));
  const base = $derived(draft.cards.find((c) => c.subject_kind === "*") ?? { subject_kind: "*", density: "full" as Density });
  function ruleFor(k: string): LensCard {
    let r = draft.cards.find((c) => c.subject_kind === k);
    if (!r) {
      r = { subject_kind: k, density: base.density, ...(base.fields ? { fields: [...base.fields] } : {}) };
      draft.cards.push(r);
    }
    return draft.cards.find((c) => c.subject_kind === k)!;
  }
  const showing = (f: string) => {
    const r = rule ?? base;
    return !r.fields || r.fields.includes(f);
  };
  function toggleField(f: string, on: boolean) {
    const r = ruleFor(kind);
    const list = r.fields ?? fields.map((x) => x.id);
    r.fields = on ? fields.map((x) => x.id).filter((x) => x === f || list.includes(x)) : list.filter((x) => x !== f);
  }
  function allFields() {
    const r = ruleFor(kind);
    delete r.fields;
  }
  function toggle<T extends string>(list: T[], id: T, on: boolean): T[] {
    return on ? [...list.filter((x) => x !== id), id] : list.filter((x) => x !== id);
  }

  function save(e: SubmitEvent) {
    e.preventDefault();
    if (!label.trim()) return void (problem = "Give the lens a name, for example Political.");
    const snap = $state.snapshot(draft) as Lens;
    const taken = lenses.filter((l) => l.id !== (own ? from.id : "")).map((l) => l.id);
    const copied: Lens = own ? { ...snap, label: label.trim(), origin: "mine" } : { ...copyLens(snap, label.trim(), taken), cards: snap.cards, layers: snap.layers, tools: snap.tools };
    // Names on the chart (NM6): absent means the common name, else the designation.
    const { names: _n, ...rest } = copied;
    const next: Lens = snap.names ? { ...rest, names: snap.names } : rest;
    const bad = checkLens(next);
    problem = bad.join("; ");
    if (bad.length) return;
    onsave(next);
    note = own ? "Saved." : `Saved as ${next.label}. It is on the lens switcher now.`;
  }

  // Leaving the browser: the GM's lenses as a presentation pack.
  const mine = $derived(lenses.filter((l) => l.origin === "mine"));
  const packHref = $derived(mine.length ? URL.createObjectURL(new Blob([JSON.stringify(lensPack(mine, { id: "mine.lenses" }), null, 1)], { type: "application/json" })) : "");
  $effect(() => {
    const h = packHref;
    return () => h && URL.revokeObjectURL(h);
  });
  async function load(e: Event) {
    const file = (e.currentTarget as HTMLInputElement).files?.[0];
    if (!file) return;
    let doc: unknown;
    try {
      doc = JSON.parse(await file.text());
    } catch {
      problem = `${file.name} is not a JSON file.`;
      return;
    }
    const r = lensesFromPack(doc);
    problem = r.problems.join("; ");
    for (const l of r.lenses) onsave({ ...l, origin: "mine" });
    note = r.lenses.length ? `Loaded ${r.lenses.map((l) => l.label).join(", ")}.` : r.problems.length ? "" : "That pack has no lenses.";
  }
  const kindLabel = (k: string) => (k === "*" ? "Any body" : k[0]!.toUpperCase() + k.slice(1));
</script>

<section class="lp" aria-labelledby="lens-title">
  <h2 id="lens-title">{own ? `Lens: ${from.label}` : "Make your own lens"}</h2>
  <p class="hint">{own ? "Your own lens. Changes apply when you save." : `Starting from ${from.label}.`} A lens never changes data: a field it hides is untouched. Honesty stamps always show on your view.</p>
  <form onsubmit={save}>
    <label class="name">Lens name<input bind:value={label} /></label>
    <fieldset>
      <legend>On the chart</legend>
      {#each LAYERS as l (l.id)}
        <label class="check"><input type="checkbox" checked={draft.layers.includes(l.id)} onchange={(e) => (draft.layers = toggle<LayerId>(draft.layers, l.id, e.currentTarget.checked))} /> {l.label}</label>
      {/each}
      <label class="check" data-lens-names>Names <select aria-label="Names on the chart" value={draft.names ?? ""} onchange={(e) => (draft.names = e.currentTarget.value || undefined)}>{#each chartNameLayers() as n (n.id)}<option value={n.id}>{n.label}</option>{/each}</select></label>
    </fieldset>
    <fieldset>
      <legend>Tools on the bar</legend>
      {#each TOOLS as t (t.id)}
        <label class="check"><input type="checkbox" checked={draft.tools.includes(t.id)} onchange={(e) => (draft.tools = toggle<ToolId>(draft.tools, t.id, e.currentTarget.checked))} /> {t.label}</label>
      {/each}
    </fieldset>
    <fieldset class="card">
      <legend>The card</legend>
      <label>For <select bind:value={kind} aria-label="Card for">{#each ["*", ...kinds] as k (k)}<option value={k}>{kindLabel(k)}{k !== "*" && !draft.cards.some((c) => c.subject_kind === k) ? " (as any body)" : ""}</option>{/each}</select></label>
      <label>Density <select aria-label="Card density" value={(rule ?? base).density} onchange={(e) => (ruleFor(kind).density = e.currentTarget.value as Density)}><option value="line">One line</option><option value="compact">Compact</option><option value="full">Full, with notes</option></select></label>
      <div class="fields">
        <label class="check"><input type="checkbox" checked={!(rule ?? base).fields} onchange={(e) => (e.currentTarget.checked ? allFields() : (ruleFor(kind).fields = []))} /> Every field</label>
        {#each fields as f (f.id)}
          <label class="check"><input type="checkbox" checked={showing(f.id)} onchange={(e) => toggleField(f.id, e.currentTarget.checked)} /> {f.label}</label>
        {/each}
      </div>
    </fieldset>
    <div class="row">
      <button type="submit" class="primary">{own ? "Save changes" : "Save lens"}</button>
      {#if own}<button type="button" class="linkbtn" onclick={() => ondelete(from.id)}>Delete this lens</button>{/if}
    </div>
  </form>
  <div class="row io">
    {#if packHref}<a href={packHref} download="my-lenses.pack.json" data-lens-pack>Export my lenses as a pack</a>{/if}
    <label class="file">Load lenses from a pack <input type="file" accept="application/json,.json" onchange={load} /></label>
  </div>
  {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
  {#if note}<p class="note" role="status">{note}</p>{/if}
</section>

<style>
  .lp {
    font-family: var(--sse-font-body);
    font-size: 14px;
    color: var(--sse-ink);
  }
  h2 {
    margin: 0 0 6px;
    font-family: var(--sse-font-display);
    font-weight: normal;
  }
  .hint {
    color: var(--sse-ink-muted);
    font-size: 12px;
    margin: 0 0 12px;
  }
  form {
    display: grid;
    gap: 10px;
  }
  .name input {
    display: block;
    width: 100%;
    margin-top: 4px;
    min-height: 36px;
    padding: 0 8px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: inherit;
  }
  fieldset {
    border: var(--sse-line) solid var(--sse-rule);
    padding: 6px 10px 8px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 4px 12px;
  }
  .card {
    grid-template-columns: 1fr 1fr;
  }
  .fields {
    grid-column: 1 / -1;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 4px 12px;
  }
  legend {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .check {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  select {
    margin-left: 6px;
    min-height: 32px;
    font: inherit;
    background: var(--sse-ground);
    color: var(--sse-ink);
    border: var(--sse-line) solid var(--sse-ink);
  }
  .row {
    display: flex;
    gap: 16px;
    align-items: center;
    flex-wrap: wrap;
  }
  .io {
    margin-top: 14px;
    font-size: 13px;
  }
  .io a,
  .linkbtn {
    color: var(--sse-accent);
    font: inherit;
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    text-decoration: underline;
  }
  .file input {
    display: block;
    margin-top: 4px;
    font-size: 12px;
  }
  .primary {
    height: 40px;
    padding: 0 16px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ink);
    color: var(--sse-on-ink);
    font-family: var(--sse-font-body);
    cursor: pointer;
  }
  .problem {
    color: var(--sse-accent-ink);
  }
  .note {
    color: var(--sse-ink-muted);
  }
  @media (max-width: 760px) {
    .card {
      grid-template-columns: 1fr;
    }
  }
</style>
