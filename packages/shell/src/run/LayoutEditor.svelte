<!-- The layout editor (PB13, playbook and switchboard design 7.4), owned by the "Run desk and players' side" build:
     one editor for a new layout and an existing one, opened from a role's layout on the Switchboard. Four lines: the
     arrangement, the boxes on it (each a part showing whatever is sent to the role, with that part's own controls on or
     off, AT13), and the name; Save keeps it with the campaign and gives it to the role. Boxes are reordered by dragging
     (one onto another swaps them) or with the arrows. The preview beside it is the role's screen as it would be, drawn
     from what the role knows. Boxes holding a bookmark, a file or a link, and the look line, come in later parts. -->
<script lang="ts">
  import { untrack } from "svelte";
  import { ARRANGEMENTS, addBox, arrangementOf, checkScreen, moveBox, rearrange, removeBox, sorted, swapBoxes, type Arrangement, type PlayerScene, type ScreenDoc, type Vocab } from "../../../ui-contract/src/index.ts";
  import { PLAYER_STACK, ScreenRuntime } from "../../../views/player-stack/src/index.ts";
  import vocabDoc from "../documents/vocab.plain.json";

  let { layout, scene, onsave, onclose }: { layout: ScreenDoc; scene: PlayerScene | null; onsave: (doc: ScreenDoc) => void; onclose: () => void } = $props();

  let doc = $state<ScreenDoc>(untrack(() => structuredClone($state.snapshot(layout)) as ScreenDoc));
  const vocab = vocabDoc as unknown as Vocab;
  const PART_WORDS: Record<string, string> = {
    "sse.view.chart": "Chart",
    "sse.view.system-3d": "System in 3D",
    "sse.view.card": "Card",
    "sse.view.table": "Table",
    "sse.view.text": "Text",
    "sse.view.readout": "Readout",
    "sse.view.surface": "Landing view",
  };
  const parts = PLAYER_STACK.map((v) => ({ id: v.id, words: PART_WORDS[v.id] ?? v.id.replace("sse.view.", ""), switchable: v.switchable ?? [] }));
  const partOf = (id: string) => parts.find((p) => p.id === id);
  const boxes = $derived(sorted(doc));
  // The arrangement chosen (an empty layout reads as Blank, so the choice is kept, not read back).
  let arrangement = $state<Arrangement>(untrack(() => arrangementOf(layout)));
  const room = $derived(ARRANGEMENTS[arrangement === "blank" ? "one" : arrangement].areas.length);
  const checked = $derived(checkScreen($state.snapshot(doc) as ScreenDoc, PLAYER_STACK));
  let adding = $state("sse.view.card");
  let dragged = $state<number | null>(null);

  const controlsOf = (i: number) => {
    const b = boxes[i]!;
    const c = b.controls;
    return Object.fromEntries((partOf(b.view)?.switchable ?? []).map((s) => [s.id, c === "all" ? true : c === "none" ? false : typeof c === "object" && typeof c[s.id] === "boolean" ? c[s.id]! : s.default]));
  };
  function setBox(i: number, f: (b: ScreenDoc["items"][number]) => ScreenDoc["items"][number]) {
    const list = sorted(doc);
    list[i] = f(list[i]!);
    doc = { ...doc, items: list };
  }
  function setControl(i: number, id: string, on: boolean) {
    setBox(i, (b) => {
      const now = { ...controlsOf(i), [id]: on };
      const defaults = Object.fromEntries((partOf(b.view)?.switchable ?? []).map((s) => [s.id, s.default]));
      const diff = Object.fromEntries(Object.entries(now).filter(([k, v]) => defaults[k] !== v));
      const all = Object.values(now);
      const next = { ...b };
      if (all.length && all.every((v) => !v)) next.controls = "none";
      else if (Object.keys(diff).length) next.controls = diff;
      else delete next.controls;
      return next;
    });
  }
  function setView(i: number, view: string) {
    setBox(i, (b) => {
      const n = { ...b, view };
      delete n.params;
      delete n.controls;
      delete n.style;
      return n;
    });
  }
  const valid = $derived(doc.label.trim().length > 0 && checked.ok);
</script>

<div class="wrap"><div class="editor" role="group" aria-label="Layout editor" data-layout-editor={doc.id}>
  <div class="form">
    <label>Name <input type="text" bind:value={doc.label} maxlength="60" data-layout-name /></label>
    <label
      >Arrangement
      <select value={arrangement} data-layout-arrangement onchange={(e) => ((arrangement = e.currentTarget.value as Arrangement), (doc = rearrange(doc, arrangement)))}>
        {#each Object.entries(ARRANGEMENTS) as [k, a] (k)}<option value={k}>{a.words}</option>{/each}
      </select>
    </label>
    <p class="hint">Each box shows whatever is sent to this role, drawn by its part from what the role knows. Drag a box onto another to swap them.</p>
    <ol class="boxes" aria-label="Boxes">
      {#each boxes as b, i (b.area)}
        {@const p = partOf(b.view)}
        <li
          data-layout-box={i}
          data-box-part={b.view}
          draggable="true"
          class:dragging={dragged === i}
          ondragstart={(e) => ((dragged = i), e.dataTransfer?.setData("text/plain", String(i)))}
          ondragend={() => (dragged = null)}
          ondragover={(e) => e.preventDefault()}
          ondrop={(e) => {
            e.preventDefault();
            if (dragged !== null) doc = swapBoxes(doc, dragged, i);
            dragged = null;
          }}
        >
          <span class="grip" aria-hidden="true">⠿</span>
          <select aria-label="Part in box {i + 1}" value={b.view} data-box-view onchange={(e) => setView(i, e.currentTarget.value)}>
            {#each parts as q (q.id)}<option value={q.id}>{q.words}</option>{/each}
          </select>
          {#if p?.switchable.length}
            <span class="ctls">
              {#each p.switchable as c (c.id)}
                <label title={c.words}><input type="checkbox" checked={controlsOf(i)[c.id]} data-box-control={c.id} onchange={(e) => setControl(i, c.id, e.currentTarget.checked)} /> {c.id}</label>
              {/each}
            </span>
          {:else}<span class="sub">a picture</span>{/if}
          <span class="moves">
            <button type="button" aria-label="Move box {i + 1} up" disabled={i === 0} data-box-up onclick={() => (doc = moveBox(doc, i, i - 1))}>↑</button>
            <button type="button" aria-label="Move box {i + 1} down" disabled={i === boxes.length - 1} data-box-down onclick={() => (doc = moveBox(doc, i, i + 1))}>↓</button>
            <button type="button" aria-label="Take box {i + 1} off" data-box-remove onclick={() => (doc = removeBox(doc, i))}>×</button>
          </span>
        </li>
      {/each}
    </ol>
    {#if boxes.length < room || arrangement === "blank"}
      <div class="add">
        <select aria-label="Part for a new box" bind:value={adding} data-box-new-part>
          {#each parts as q (q.id)}<option value={q.id}>{q.words}</option>{/each}
        </select>
        <button type="button" data-box-add onclick={() => (arrangement === "blank" && (arrangement = "one"), (doc = addBox(doc, adding, arrangement)))}>+ Box</button>
      </div>
    {/if}
    {#if !checked.ok}<ul class="bad" data-layout-errors>{#each checked.errors as e (e)}<li>{e}</li>{/each}</ul>{/if}
    <div class="row">
      <button type="button" class="go" disabled={!valid} data-layout-save onclick={() => onsave($state.snapshot(doc) as ScreenDoc)}>Save and use</button>
      <button type="button" class="quiet" data-layout-cancel onclick={onclose}>Cancel</button>
    </div>
  </div>
  <div class="preview" aria-label="Preview" data-layout-preview>
    {#if checked.ok && scene && boxes.length}
      {#key JSON.stringify(doc.items) + doc.layout}<ScreenRuntime screen={checked.screen} {scene} {vocab} compact still />{/key}
    {:else if !boxes.length}<p class="sub">Blank: a screen with nothing on it until something is sent.</p>
    {:else if !scene}<p class="sub">Reading what this role knows…</p>{/if}
  </div>
</div></div>

<style>
  /* Side by side where the column is wide enough, else the preview under the form. */
  .wrap {
    container-type: inline-size;
  }
  .editor {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 14px;
    padding: 12px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
  }
  .form {
    display: grid;
    gap: 8px;
    align-content: start;
  }
  label {
    display: grid;
    gap: 2px;
    font-size: 13px;
  }
  .ctls label {
    display: inline-flex;
    gap: 3px;
    align-items: center;
  }
  .boxes {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
  }
  .boxes li {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    padding: 4px 6px;
    border: var(--sse-line) solid var(--sse-rule);
    border-radius: var(--sse-radius);
    background: var(--sse-ground);
  }
  .boxes li.dragging {
    opacity: 0.5;
  }
  .grip {
    cursor: grab;
    color: var(--sse-ink-muted);
  }
  .moves {
    margin-left: auto;
    display: inline-flex;
    gap: 2px;
  }
  .ctls {
    display: inline-flex;
    gap: 8px;
  }
  .sub,
  .hint {
    color: var(--sse-ink-muted);
    font-size: 12px;
    margin: 0;
  }
  .add,
  .row {
    display: flex;
    gap: 6px;
  }
  .bad {
    color: var(--sse-warn, var(--sse-accent));
    font-size: 12px;
    margin: 0;
    padding-left: 16px;
  }
  .preview {
    min-width: 0;
    max-height: 520px;
    overflow: auto;
  }
  button.go {
    font-weight: 600;
  }
  @container (min-width: 760px) {
    .editor {
      grid-template-columns: minmax(0, 340px) minmax(0, 1fr);
    }
  }
</style>
