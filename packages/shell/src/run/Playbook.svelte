<!--
  The open Playbook page on the Switchboard (PB3, design section 4): a chapter and page picker in one line, then the
  page's sections (press a heading to fold it; folding is this device's own), each with its presets. A preset row has
  two parts: its body selects it (the monitor above shows it; nothing is sent) and its fire button sends it, one press,
  no confirmation; pressed again while its picture is up, it takes it back. Its dot drags it to any role or screen.
  "+ Add" at a section's foot makes a preset from a bookmark, words to read aloud, or a holding card. Rows drag to
  reorder, onto another section, or onto a page in the picker.
-->
<script lang="ts">
  import { addChapter, addPage, addPreset, addSection, findPage, isUp, reveals, wordsOnly, movePreset, movePresetToPage, pages, presetWords, removePart, rename, type Board, type Bookmark, type Playbook, type Preset } from "../../../ui-contract/src/index.ts";
  import { KnownEye } from "../../../chart-grammar/src/index.ts";
  import { firedAt } from "./fired.svelte.ts";

  let {
    pb,
    board,
    bookmarks,
    roles,
    pageId = $bindable(),
    folded = $bindable(),
    selected,
    armed,
    onchange,
    onselect,
    onfire,
    ondrag,
  }: {
    pb: Playbook;
    board: Board;
    bookmarks: readonly Bookmark[];
    roles: readonly { id: string; label: string }[];
    pageId: string | null;
    folded: Set<string>;
    selected: string | null;
    /** The preset whose dot is armed (press-then-press). */
    armed: string | null;
    onchange: (next: Playbook, select?: string) => void;
    onselect: (id: string | null) => void;
    onfire: (p: Preset) => void;
    ondrag: (e: PointerEvent, id: string) => void;
  } = $props();

  const open = $derived(findPage(pb, pageId));
  const roleWords = (to: string) => (to === "everyone" ? "Everyone" : (roles.find((r) => r.id === to)?.label ?? to));
  // Numbers for keys 1 to 9: only presets in unfolded sections count.
  const numbered = $derived.by(() => {
    const m = new Map<string, number>();
    let n = 0;
    for (const s of open?.page.sections ?? []) if (!folded.has(s.id)) for (const k of s.presets) if (++n <= 9) m.set(k.id, n);
    return m;
  });

  let renaming = $state<string | null>(null);
  let adding = $state<string | null>(null);
  let choosing = $state<{ section: string; kind: "bookmark" | "reveal" } | null>(null);
  let pickerOpen = $state(false);
  let rowDrag = $state<string | null>(null);
  let dropAt = $state<string | null>(null);

  function fold(id: string) {
    const next = new Set(folded);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    folded = next;
  }
  function add(section: string, kind: "bookmark" | "reveal" | "say" | "hold", bookmark?: Bookmark) {
    const to = "everyone";
    const r =
      kind === "reveal" && bookmark?.selected
        ? addPreset(pb, section, { title: `Reveal ${bookmark.name}`, to, steps: [{ reveal: { address: bookmark.selected, landmark: "known", name: bookmark.name } }] })
        : kind === "bookmark" && bookmark
        ? addPreset(pb, section, { title: bookmark.name, to, steps: [{ show: { bookmark: bookmark.id } }] })
        : kind === "say"
          ? addPreset(pb, section, { title: "Read aloud", to, steps: [{ say: { text: "" } }] })
          : addPreset(pb, section, { title: "Hold", to, steps: [{ hold: { card: "plain" } }] });
    adding = choosing = null;
    onchange(r.pb, r.id);
  }
  function commitName(id: string, v: string) {
    renaming = null;
    onchange(rename(pb, id, v));
  }
  function focusInput(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
  function dropRow(section: string, index: number) {
    if (rowDrag) onchange(movePreset(pb, rowDrag, section, index));
    rowDrag = dropAt = null;
  }
  function dropOnPage(page: string) {
    if (rowDrag) onchange(movePresetToPage(pb, rowDrag, page));
    rowDrag = dropAt = null;
  }
</script>

{#snippet nameEdit(id: string, value: string, label: string)}
  <input
    class="name"
    {value}
    aria-label={label}
    data-pb-name={id}
    use:focusInput
    onkeydown={(e) => {
      if (e.key === "Enter") commitName(id, e.currentTarget.value);
      else if (e.key === "Escape") (e.stopPropagation(), (renaming = null));
    }}
    onblur={(e) => renaming === id && commitName(id, e.currentTarget.value)}
  />
{/snippet}

<section class="pb" aria-label="Playbook" data-playbook data-page={open?.page.id}>
  <div class="picker">
    <button type="button" class="pick" aria-expanded={pickerOpen} onclick={() => (pickerOpen = !pickerOpen)} data-page-picker title="Choose a page: changing page changes nothing on any screen">
      {#if open}<span class="sub">{open.chapter.title} ·</span> <b>{open.page.title}</b>{/if} ▾
    </button>
    {#if open}<button type="button" class="ghost" aria-label="Rename this page" title="Rename this page" onclick={() => (renaming = open.page.id)}>✎</button>{/if}
  </div>
  {#if renaming && open && renaming === open.page.id}{@render nameEdit(open.page.id, open.page.title, "Page name")}{/if}
  {#if pickerOpen}
    <div class="pages" role="group" aria-label="Pages" data-pages>
      {#each pb.chapters as c (c.id)}
        <div class="chapter">
          {#if renaming === c.id}{@render nameEdit(c.id, c.title, "Chapter name")}{:else}<span class="ctitle" role="button" tabindex="0" ondblclick={() => (renaming = c.id)} onkeydown={(e) => e.key === "F2" && (renaming = c.id)} title="Double-click to rename">{c.title}</span>{/if}
          {#each c.pages as p (p.id)}
            <button type="button" class="pg" class:on={open?.page.id === p.id} class:drop={rowDrag && dropAt === p.id} data-page-choose={p.id} onclick={() => ((pageId = p.id), (pickerOpen = false))} ondragover={(e) => (e.preventDefault(), (dropAt = p.id))} ondrop={(e) => (e.preventDefault(), dropOnPage(p.id))}>{p.title}</button>
          {/each}
          <button type="button" class="ghost add" data-add-page={c.id} onclick={() => {
            const r = addPage(pb, c.id, "New page");
            onchange(r.pb);
            pageId = r.page;
            renaming = r.page;
            pickerOpen = false;
          }}>+ Page</button>
        </div>
      {/each}
      <button type="button" class="ghost add" data-add-chapter onclick={() => {
        const r = addChapter(pb, `Chapter ${pb.chapters.length + 1}`);
        onchange(r.pb);
        pageId = r.page;
        pickerOpen = false;
      }}>+ Chapter</button>
      {#if open && pages(pb).length > 1}<button type="button" class="ghost add" data-remove-page onclick={() => (onchange(removePart(pb, open.page.id)), (pageId = null), (pickerOpen = false))}>Remove this page</button>{/if}
    </div>
  {/if}

  {#if open}
    {#each open.page.sections as s (s.id)}
      {@const shut = folded.has(s.id)}
      <div class="section" data-section={s.id} data-folded={shut ? "" : undefined}>
        <div class="shead">
          {#if renaming === s.id}
            {@render nameEdit(s.id, s.title, "Section name")}
          {:else}
            <button type="button" class="fold" aria-expanded={!shut} onclick={() => fold(s.id)} ondblclick={() => (renaming = s.id)} title="Fold or open (double-click to rename)" ondragover={(e) => (e.preventDefault(), (dropAt = s.id))} ondrop={(e) => (e.preventDefault(), dropRow(s.id, s.presets.length))} class:drop={rowDrag && dropAt === s.id}>
              <span class="caret" aria-hidden="true">{shut ? "▸" : "▾"}</span>{s.title}{#if shut}<span class="sub"> · {s.presets.length} {s.presets.length === 1 ? "preset" : "presets"}</span>{/if}
            </button>
          {/if}
        </div>
        {#if !shut}
          <ol>
            {#each s.presets as k, i (k.id)}
              {@const up = isUp(board, k)}
              {@const at = firedAt(k.id)}
              {@const n = numbered.get(k.id)}
              <li
                class="row"
                class:sel={selected === k.id}
                class:up
                class:drop={rowDrag && dropAt === k.id && rowDrag !== k.id}
                data-preset={k.id}
                data-up={up ? "" : undefined}
                draggable="true"
                ondragstart={(e) => ((rowDrag = k.id), e.dataTransfer?.setData("text/plain", k.id))}
                ondragover={(e) => (e.preventDefault(), (dropAt = k.id))}
                ondrop={(e) => (e.preventDefault(), dropRow(s.id, i))}
                ondragend={() => (rowDrag = dropAt = null)}
              >
                <button type="button" class="body" onclick={() => onselect(selected === k.id ? null : k.id)} data-preset-select={k.id} title={presetWords(k, bookmarks)}>
                  {#if n}<span class="num" aria-hidden="true">{n}</span>{/if}
                  {#if reveals(k)}<span class="eye" title="Tells {roleWords(k.to)} something" data-reveals><KnownEye landmark="known" size={12} /></span>{/if}
                  <span class="ktitle">{k.title}</span>
                  {#if at}<span class="tick" data-fired={k.id} title="Fired tonight at {at}">✓ {at}</span>{/if}
                </button>
                <button type="button" class="fire" class:on={up} aria-pressed={up} data-fire={k.id} onclick={() => onfire(k)} title={up ? "Take it back" : wordsOnly(k) ? "Mark read" : `Send to ${roleWords(k.to)}${n ? ` (${n})` : ""}`}>{wordsOnly(k) ? "Read" : `▸ ${roleWords(k.to)}`}</button>
                {#if !wordsOnly(k)}
                  <button type="button" class="dot" class:on={armed === k.id} data-source-dot={k.id} aria-label="Send {k.title} somewhere else: press, then press where it goes" aria-pressed={armed === k.id} onpointerdown={(e) => ondrag(e, k.id)}></button>
                {/if}
              </li>
            {/each}
          </ol>
          {#if adding === s.id}
            <div class="kinds" role="group" aria-label="Add to {s.title}">
              {#if choosing?.section === s.id}
                {@const usable = choosing.kind === "reveal" ? bookmarks.filter((b) => b.selected) : bookmarks}
                {#if usable.length}
                  {#each usable as b (b.id)}<button type="button" class="ghost" data-add-bookmark={b.id} onclick={() => add(s.id, choosing!.kind, b)}>{b.name}</button>{/each}
                {:else}
                  <span class="sub">{choosing.kind === "reveal" ? "No bookmark of a thing yet. Select it on the chart and press Bookmarks (B)." : "No bookmarks yet. Go there on the chart and press Bookmarks (B)."}</span>
                {/if}
                <button type="button" class="ghost" onclick={() => (choosing = null)}>Back</button>
              {:else}
                <button type="button" class="ghost" data-add-kind="bookmark" onclick={() => (choosing = { section: s.id, kind: "bookmark" })}>A bookmark</button>
                <button type="button" class="ghost" data-add-kind="reveal" onclick={() => (choosing = { section: s.id, kind: "reveal" })}>A reveal</button>
                <button type="button" class="ghost" data-add-kind="say" onclick={() => add(s.id, "say")}>Read aloud</button>
                <button type="button" class="ghost" data-add-kind="hold" onclick={() => add(s.id, "hold")}>A hold</button>
                <button type="button" class="ghost" onclick={() => (adding = null)}>Cancel</button>
              {/if}
            </div>
          {:else}
            <button type="button" class="ghost add" data-add-to={s.id} onclick={() => ((adding = s.id), (choosing = null))}>+ Add to {s.title}</button>
          {/if}
        {/if}
      </div>
    {/each}
    <div class="foot">
      <button type="button" class="ghost add" data-add-section onclick={() => {
        const r = addSection(pb, open.page.id, "New section");
        onchange(r.pb);
        renaming = r.section;
      }}>+ Section</button>
    </div>
  {/if}
</section>

<style>
  button {
    font: inherit;
    font-size: 13px;
    color: var(--sse-ink);
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    padding: 3px 10px;
    cursor: pointer;
    transition: background var(--sse-motion-quick) var(--sse-ease);
  }
  button:hover {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .pb {
    margin-top: 14px;
    font-size: 14px;
  }
  .picker {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .pick {
    flex: 1;
    text-align: left;
    background: transparent;
    border: 0;
    border-bottom: var(--sse-line) solid var(--sse-ink);
    border-radius: 0;
    padding: 4px 2px;
  }
  .pick:hover {
    background: var(--sse-ground-raised) !important;
    color: var(--sse-ink) !important;
  }
  .pages {
    margin: 6px 0;
    padding: 8px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .chapter {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    align-items: center;
  }
  .ctitle {
    width: 100%;
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .pg.on {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .drop {
    outline: 2px solid var(--sse-accent);
  }
  .section {
    margin-top: 10px;
  }
  .fold {
    background: none;
    border: 0;
    padding: 2px 0;
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .fold:hover {
    background: none !important;
    color: var(--sse-ink) !important;
  }
  .fold .sub {
    text-transform: none;
    letter-spacing: 0;
  }
  .caret {
    display: inline-block;
    width: 1.2em;
  }
  ol {
    list-style: none;
    margin: 4px 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .row {
    display: flex;
    align-items: stretch;
    gap: 4px;
    position: relative;
  }
  .row.drop {
    outline: none;
    box-shadow: 0 -2px 0 var(--sse-accent);
  }
  .body {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 6px;
    text-align: left;
    background: var(--sse-ground-raised);
    cursor: grab;
  }
  .row.sel .body {
    box-shadow: inset 3px 0 0 var(--sse-accent);
  }
  .eye {
    display: inline-flex;
    color: var(--sse-ink-muted);
  }
  .num {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .ktitle {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tick {
    font-size: 11px;
    color: var(--sse-ink-muted);
    white-space: nowrap;
  }
  .fire {
    white-space: nowrap;
  }
  .fire.on {
    background: var(--sse-accent);
    border-color: var(--sse-accent);
    color: var(--sse-on-ink);
  }
  .dot {
    align-self: center;
    width: 14px;
    height: 14px;
    padding: 0;
    border-radius: 50%;
    border: 2px solid var(--sse-ink);
    background: var(--sse-ground-raised);
    touch-action: none;
    margin-right: -20px;
  }
  .dot.on {
    background: var(--sse-accent);
    border-color: var(--sse-accent);
  }
  .ghost {
    border-color: var(--sse-rule);
    color: var(--sse-ink-muted);
    background: transparent;
  }
  .add {
    align-self: flex-start;
    border-style: dashed;
    margin-top: 4px;
  }
  .kinds {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 4px;
  }
  .name {
    font: inherit;
    width: 100%;
    padding: 3px 6px;
  }
  .sub {
    color: var(--sse-ink-muted);
    font-size: 13px;
  }
  .foot {
    margin-top: 10px;
  }
</style>
