<!--
  The monitor with a preset in it (PB3, design 4.6): selecting a preset in the Playbook shows it here, at the top,
  before anyone else sees it; Fire sends it. Edit turns the monitor into the editor for that one thing: its name, where
  it goes, what its kind allows (the bookmark and "Update from live", the words, the card) and a note to self. Editing
  never sends: what is on screens stays as it was sent until it is fired again. Done or Escape goes back to Live.
-->
<script lang="ts">
  import { addBookmark, findPreset, isUp, presetWords, removePreset, updatePreset, type Board, type Bookmark, type HoldCard, type Playbook, type Preset, type Step } from "../../../ui-contract/src/index.ts";
  import { firedAt } from "./fired.svelte.ts";

  let {
    pb,
    id,
    board,
    bookmarks,
    roles,
    editing = $bindable(),
    markHere,
    onchange,
    onbookmarks,
    onfire,
    onclose,
  }: {
    pb: Playbook;
    id: string;
    board: Board;
    bookmarks: readonly Bookmark[];
    roles: readonly { id: string; label: string }[];
    editing: boolean;
    markHere?: () => Bookmark;
    onchange: (next: Playbook) => void;
    onbookmarks: (next: Bookmark[]) => void;
    onfire: (p: Preset) => void;
    onclose: () => void;
  } = $props();

  const found = $derived(findPreset(pb, id));
  const k = $derived(found?.preset ?? null);
  const roleWords = (to: string) => (to === "everyone" ? "Everyone" : (roles.find((r) => r.id === to)?.label ?? `${to} (no such role tonight)`));
  let note = $state<string | null>(null);

  function setStep(i: number, s: Step) {
    if (!k) return;
    const steps = k.steps.map((x, j) => (j === i ? s : x));
    onchange(updatePreset(pb, k.id, { steps }));
  }
  function updateFromLive(i: number) {
    if (!k || !markHere) return;
    const b = markHere();
    onbookmarks(addBookmark(bookmarks, { ...b, name: `${k.title} (from live)` }));
    setStep(i, { show: { bookmark: b.id } });
    note = "Now shows where you are. Fire again to send it.";
  }
  const cardValue = (c: HoldCard) => (typeof c === "string" ? c : "text");
  function key(e: KeyboardEvent) {
    if (e.key === "Escape" && !(e.target as HTMLElement).closest("select")) (e.stopPropagation(), editing ? (editing = false) : onclose());
  }
</script>

{#if k}
  {@const up = isUp(board, k)}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <article class="monitor" data-monitor={k.id} data-editing={editing ? "" : undefined} onkeydown={key} aria-label="Preset {k.title}">
    <header>
      <span class="mt"><b data-monitor-title>{k.title}</b>{#if !k.steps.every((x) => "say" in x)} <span class="sub to">▸ {roleWords(k.to)}</span>{/if}</span>
      <span class="acts">
        {#if editing}
          <button type="button" class="pri" data-monitor-done onclick={() => (editing = false)}>Done</button>
        {:else}
          <button type="button" data-monitor-edit onclick={() => (editing = true)}>Edit</button>
          <button type="button" class="pri" class:on={up} aria-pressed={up} data-monitor-fire onclick={() => onfire(k)}>{up ? "Take back" : k.steps.every((x) => "say" in x) ? "Mark read" : "Fire"}</button>
        {/if}
        <button type="button" class="quiet" aria-label="Back to Live" title="Back to Live (Escape)" data-monitor-close onclick={onclose}>×</button>
      </span>
    </header>
    {#if !editing}
      {#each k.steps as s, i (i)}
        {#if "say" in s}
          <p class="say" data-monitor-say>{s.say.text || "No words yet: press Edit to write them."}</p>
        {/if}
      {/each}
      <p class="sub what">{presetWords(k, bookmarks)}{#if firedAt(k.id)} · fired at {firedAt(k.id)}{/if}{#if up} · on screens now; editing changes nothing there until it is fired again{/if}</p>
      {#if k.note}<p class="sub">Note: {k.note}</p>{/if}
    {:else}
      <div class="form">
        <label>Name <input value={k.title} data-edit-title onchange={(e) => onchange(updatePreset(pb, k.id, { title: e.currentTarget.value.trim() || k.title }))} /></label>
        <label
          >Goes to
          <select value={k.to} data-edit-to onchange={(e) => onchange(updatePreset(pb, k.id, { to: e.currentTarget.value }))}>
            <option value="everyone">Everyone</option>
            {#each roles as r (r.id)}<option value={r.id}>{r.label}</option>{/each}
            {#if k.to !== "everyone" && !roles.some((r) => r.id === k.to)}<option value={k.to}>{k.to} (no such role tonight)</option>{/if}
          </select>
        </label>
        {#each k.steps as s, i (i)}
          {#if "show" in s}
            <label
              >Shows
              <select value={s.show.bookmark} data-edit-bookmark onchange={(e) => setStep(i, { show: { bookmark: e.currentTarget.value } })}>
                {#each bookmarks as b (b.id)}<option value={b.id}>{b.name}</option>{/each}
                {#if !bookmarks.some((b) => b.id === s.show.bookmark)}<option value={s.show.bookmark}>a bookmark that has gone</option>{/if}
              </select>
            </label>
            {#if markHere}<button type="button" class="ghost" data-update-from-live onclick={() => updateFromLive(i)}>Update from live</button>{/if}
          {:else if "reveal" in s}
            <label
              >Tells them
              <select value={s.reveal.landmark} data-edit-landmark onchange={(e) => setStep(i, { reveal: { ...s.reveal, landmark: e.currentTarget.value as "known" | "heard_of" } })}>
                <option value="known">it is there (known)</option>
                <option value="heard_of">of it (heard of)</option>
              </select>
            </label>
          {:else if "say" in s}
            <label class="wide">Words to read <textarea rows="4" value={s.say.text} data-edit-say oninput={(e) => setStep(i, { say: { text: e.currentTarget.value } })}></textarea></label>
          {:else if "hold" in s}
            <label
              >Card
              <select value={cardValue(s.hold.card)} data-edit-card onchange={(e) => setStep(i, { hold: { card: e.currentTarget.value === "text" ? { text: "Stand by" } : (e.currentTarget.value as HoldCard) } })}>
                <option value="plain">Awaiting transmission</option>
                <option value="joke">A joke</option>
                <option value="blank">Dark</option>
                <option value="text">My own words</option>
              </select>
            </label>
            {#if typeof s.hold.card === "object"}<label>Words <input value={s.hold.card.text} onchange={(e) => setStep(i, { hold: { card: { text: e.currentTarget.value } } })} /></label>{/if}
          {/if}
        {/each}
        <label class="wide">Note to self <input value={k.note ?? ""} placeholder="Wait for them to ask about the moon" data-edit-note onchange={(e) => onchange(updatePreset(pb, k.id, { note: e.currentTarget.value }))} /></label>
        <button type="button" class="ghost danger" data-edit-remove onclick={() => (onchange(removePreset(pb, k.id)), onclose())}>Remove this preset</button>
      </div>
    {/if}
    {#if note}<p class="sub" role="status">{note}</p>{/if}
  </article>
{/if}

<style>
  .monitor {
    border: var(--sse-line) solid var(--sse-accent);
    background: var(--sse-ground-raised);
    box-shadow: var(--sse-card-shadow);
    padding: 0 0 8px;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
  }
  .acts {
    display: inline-flex;
    gap: 4px;
    flex: none;
  }
  .acts button {
    white-space: nowrap;
  }
  .mt {
    min-width: 0;
  }
  .to {
    margin: 0;
    white-space: nowrap;
  }
  button,
  select,
  input,
  textarea {
    font: inherit;
    font-size: 13px;
    color: var(--sse-ink);
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    padding: 3px 8px;
  }
  button {
    cursor: pointer;
  }
  button.pri {
    background: var(--sse-accent);
    border-color: var(--sse-accent);
    color: var(--sse-on-ink);
  }
  button.pri.on {
    background: var(--sse-ink);
    border-color: var(--sse-ink);
  }
  button.quiet,
  .ghost {
    border-color: var(--sse-rule);
    color: var(--sse-ink-muted);
    background: transparent;
  }
  .danger:hover {
    color: var(--sse-accent);
  }
  .say {
    margin: 4px 12px 8px;
    font-family: var(--sse-font-display);
    font-size: 22px;
    line-height: 1.35;
    white-space: pre-wrap;
  }
  .sub {
    color: var(--sse-ink-muted);
    font-size: 13px;
    margin: 0 12px;
  }
  .what {
    margin-bottom: 4px;
  }
  .form {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 12px;
    padding: 0 12px;
    align-items: end;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  label.wide {
    width: 100%;
  }
  textarea {
    font-family: var(--sse-font-display);
    font-size: 16px;
  }
</style>
