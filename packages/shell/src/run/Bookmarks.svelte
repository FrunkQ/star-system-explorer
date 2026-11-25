<!-- Bookmarks (PB2b, design section 4A): the one thing made out in the app. A button beside Undo and Redo on every desk
     opens a short list: "+ Bookmark where I am" at the top, then the campaign's bookmarks. Choosing one goes back there
     exactly as it was left. Rename, update to where you are now, remove, and drag to reorder. Nothing about players. -->
<script lang="ts">
  import { ToolButton, opensFrom } from "../../../chart-grammar/src/index.ts";
  import { bookmarkReachable, isPostcard, moveBookmark, removeBookmark, renameBookmark, updateBookmark, type Bookmark, type Whereabouts } from "../../../ui-contract/src/index.ts";

  let {
    list,
    scope,
    mark,
    where,
    go,
    onchange,
    postcard,
  }: {
    list: readonly Bookmark[];
    /** The place in view: another campaign's bookmarks cannot be opened from here. */
    scope: string;
    /** A new bookmark of where the GM is now. */
    mark: () => Bookmark;
    /** Where the GM is now (for "Update to here"). */
    where: () => Whereabouts;
    go: (b: Bookmark) => void;
    onchange: (next: Bookmark[]) => void;
    /** "Make a postcard" (PC5, postcards design 6): opens it on the wall; returns why it cannot be one yet, or null. */
    postcard?: (b: Bookmark) => string | null;
  } = $props();

  let open = $state(false);
  let anchor = $state<HTMLElement | null>(null);
  let pop = $state<HTMLElement | null>(null);
  let renaming = $state<string | null>(null);
  let note = $state<string | null>(null);
  let dragFrom = $state<number | null>(null);
  let dragOver = $state<number | null>(null);

  function toggle() {
    open = !open;
    renaming = null;
    note = null;
  }
  function add() {
    const b = mark();
    onchange([...list, b]);
    renaming = b.id;
    note = null;
  }
  function choose(b: Bookmark) {
    if (!bookmarkReachable(b, scope)) return void (note = `${b.name} is in another campaign.`);
    go(b);
    open = false;
  }
  function commit(id: string, name: string) {
    onchange(renameBookmark(list, id, name));
    renaming = null;
  }
  function drop(to: number) {
    if (dragFrom !== null) onchange(moveBookmark(list, dragFrom, to));
    dragFrom = dragOver = null;
  }
  function focusInput(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
  $effect(() => {
    if (!open) return;
    const away = (e: PointerEvent) => {
      const t = e.target as Node;
      if (!pop?.contains(t) && !anchor?.contains(t)) open = false;
    };
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !renaming) (e.stopPropagation(), (open = false), anchor?.querySelector("button")?.focus());
    };
    document.addEventListener("pointerdown", away, true);
    document.addEventListener("keydown", key, true);
    return () => (document.removeEventListener("pointerdown", away, true), document.removeEventListener("keydown", key, true));
  });
</script>

<span class="bm" bind:this={anchor} data-bookmarks={list.length}>
  <ToolButton label="Bookmarks" icon="M7 3h10v18l-5-4-5 4z" pressed={open} keyhint="B" onclick={toggle} />
  {#if open}
    <div class="pop" role="dialog" aria-label="Bookmarks" bind:this={pop} use:opensFrom={() => anchor} data-bookmarks-list>
      <button type="button" class="add" onclick={add} data-bookmark-add>+ Bookmark where I am</button>
      {#if list.length}
        <ol>
          {#each list as b, i (b.id)}
            <li
              data-bookmark={b.id}
              class:over={dragOver === i && dragFrom !== i}
              class:away={!bookmarkReachable(b, scope)}
              draggable={renaming !== b.id}
              ondragstart={(e) => ((dragFrom = i), e.dataTransfer?.setData("text/plain", b.id))}
              ondragover={(e) => (e.preventDefault(), (dragOver = i))}
              ondragleave={() => dragOver === i && (dragOver = null)}
              ondrop={(e) => (e.preventDefault(), drop(i))}
              ondragend={() => (dragFrom = dragOver = null)}
            >
              <span class="grip" aria-hidden="true">⋮⋮</span>
              {#if renaming === b.id}
                <input
                  value={b.name}
                  aria-label="Name"
                  data-bookmark-name
                  use:focusInput
                  onkeydown={(e) => {
                    if (e.key === "Enter") commit(b.id, e.currentTarget.value);
                    else if (e.key === "Escape") (e.stopPropagation(), (renaming = null));
                  }}
                  onblur={(e) => renaming === b.id && commit(b.id, e.currentTarget.value)}
                />
              {:else}
                <button type="button" class="go" onclick={() => choose(b)} ondblclick={() => (renaming = b.id)} data-bookmark-go={b.id} title={bookmarkReachable(b, scope) ? "Go back there" : "In another campaign"}>{b.name}</button>
              {/if}
              {#if isPostcard(b)}<span class="stamp" data-bookmark-stamp={b.id} title="A postcard" aria-label="A postcard"></span>{/if}
              <span class="acts">
                {#if postcard && bookmarkReachable(b, scope)}<button type="button" class="act" aria-label="Make a postcard of {b.name}" title={isPostcard(b) ? "Open its postcard" : "Make a postcard"} data-bookmark-postcard={b.id} onclick={() => { const why = postcard(b); if (why) note = why; else open = false; }}>✉</button>{/if}
                <button type="button" class="act" aria-label="Rename {b.name}" title="Rename" onclick={() => (renaming = b.id)}>✎</button>
                <button type="button" class="act" aria-label="Update {b.name} to where I am" title="Update to where I am" data-bookmark-update={b.id} onclick={() => ((note = `${b.name} now keeps where you are.`), onchange(updateBookmark(list, b.id, where())))}>↻</button>
                <button type="button" class="act" aria-label="Remove {b.name}" title="Remove" data-bookmark-remove={b.id} onclick={() => onchange(removeBookmark(list, b.id))}>×</button>
              </span>
            </li>
          {/each}
        </ol>
      {:else}
        <p class="hint">A bookmark keeps where you are and how you are looking. Choose it to come back here; on the Switchboard it becomes something to send.</p>
      {/if}
      {#if note}<p class="hint" role="status">{note}</p>{/if}
    </div>
  {/if}
</span>

<style>
  .bm {
    position: relative;
    display: block;
  }
  .pop {
    position: absolute;
    left: 52px;
    top: 0;
    z-index: 20;
    width: 280px;
    max-height: 60vh;
    overflow: auto;
    padding: 8px;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    box-shadow: 0 6px 18px rgb(0 0 0 / 0.14);
    font-family: var(--sse-font-body);
    font-size: 13px;
  }
  .add {
    width: 100%;
    text-align: left;
    font: inherit;
    padding: 6px 8px;
    border: var(--sse-line) dashed var(--sse-ink);
    border-radius: var(--sse-radius);
    background: transparent;
    color: var(--sse-accent-ink, var(--sse-accent));
    cursor: pointer;
  }
  .add:hover {
    border-style: solid;
  }
  ol {
    list-style: none;
    margin: 6px 0 0;
    padding: 0;
  }
  li {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 0;
    border-top: var(--sse-line) solid transparent;
  }
  li.over {
    border-top-color: var(--sse-accent);
  }
  li.away .go {
    color: var(--sse-ink-muted);
  }
  .grip {
    cursor: grab;
    color: var(--sse-ink-muted);
    font-size: 10px;
    letter-spacing: -2px;
    padding: 0 2px;
  }
  .go {
    flex: 1;
    min-width: 0;
    text-align: left;
    font: inherit;
    padding: 4px 6px;
    background: none;
    border: 0;
    border-radius: var(--sse-radius);
    color: var(--sse-ink);
    cursor: pointer;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .go:hover,
  .go:focus-visible {
    background: var(--sse-ground);
  }
  input {
    flex: 1;
    min-width: 0;
    font: inherit;
    padding: 3px 6px;
  }
  /* A postcard's mark: a small stamp with a perforated edge. */
  .stamp {
    flex: none;
    width: 10px;
    height: 12px;
    margin-right: 4px;
    border: 1px dashed var(--sse-ink-muted);
    border-radius: 1px;
    background: var(--sse-accent-wash, transparent);
  }
  .acts {
    display: inline-flex;
    opacity: 0;
    transition: opacity var(--sse-motion-quick, 120ms) var(--sse-ease, ease-out);
  }
  li:hover .acts,
  li:focus-within .acts {
    opacity: 1;
  }
  .act {
    font: inherit;
    width: 24px;
    height: 24px;
    padding: 0;
    background: none;
    border: 0;
    color: var(--sse-ink-muted);
    cursor: pointer;
  }
  .act:hover {
    color: var(--sse-ink);
  }
  .hint {
    margin: 8px 2px 2px;
    color: var(--sse-ink-muted);
    line-height: 1.4;
  }
  @media (prefers-reduced-motion: reduce) {
    .acts {
      transition: none;
    }
  }
</style>
