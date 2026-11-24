<script lang="ts">
  // A description and GM notes for a place: the whole campaign or one system (V3 gap 42). It opens from the place's
  // name on the place line, as a card, and edits in place: a change is kept when the box loses focus (or on
  // Ctrl+Enter), as a new note in the campaign record superseding the last (ui-contract place-notes.ts). Clearing a box
  // withdraws its note. The description is for everyone; GM notes are for the game master only. Notes never change
  // the universe's state, so this is safe on any desk.
  import { opensFrom } from "../../../chart-grammar/src/motion.ts";
  import { PLACE_NOTE, placeNoteEdit, placeNoteOf, type PlaceNoteKind, type PlaceText, type ViewSession } from "../../../ui-contract/src/index.ts";

  let {
    session,
    address,
    label,
    kicker = "",
    from = null,
    onclose,
  }: {
    session: Pick<ViewSession, "record" | "addNote">;
    /** The place's address: the universe's root for the campaign, the system's address for a system. */
    address: string;
    label: string;
    /** What the place is ("System", "The whole campaign"). */
    kicker?: string;
    from?: Element | null;
    onclose: () => void;
  } = $props();

  const KINDS: PlaceNoteKind[] = ["description", "gm-notes"];
  let current = $state<Record<PlaceNoteKind, PlaceText | null>>({ description: null, "gm-notes": null });
  let text = $state<Record<PlaceNoteKind, string>>({ description: "", "gm-notes": "" });
  let phase = $state<"reading" | "ready" | "saving" | "kept" | "error">("reading");
  let error = $state("");
  let root = $state<HTMLElement | null>(null);
  // Fixed under what opened it, so a bar that clips its own overflow (the place line) never clips the card.
  const at = (() => {
    const r = from?.getBoundingClientRect();
    return r ? { top: r.bottom + 8, left: Math.max(8, Math.min(r.left, innerWidth - 388)) } : { top: 72, left: 16 };
  })();

  async function read() {
    if (!session.record) return void ((phase = "error"), (error = "The record is read on the game master's machine."));
    const r = await session.record(address);
    if (!r.ok) return void ((phase = "error"), (error = r.message));
    for (const k of KINDS) {
      current[k] = placeNoteOf(r.record.notes, k);
      text[k] = current[k]?.text ?? "";
    }
    phase = "ready";
  }
  $effect(() => {
    void address;
    void read();
  });
  $effect(() => {
    if (phase === "ready") queueMicrotask(() => root?.querySelector<HTMLTextAreaElement>("textarea")?.focus());
  });

  async function keep(k: PlaceNoteKind) {
    const e = placeNoteEdit(address, k, current[k], text[k]);
    if (!e || !session.addNote) return;
    phase = "saving";
    const r = await session.addNote(e);
    if (!r.ok) return void ((phase = "error"), (error = r.message));
    current[k] = e.text ? { id: r.note_id, text: e.text, at_s: current[k]?.at_s ?? 0 } : null;
    phase = "kept";
  }
  async function close() {
    for (const k of KINDS) await keep(k);
    onclose();
  }
</script>

<svelte:window
  onpointerdowncapture={(e) => root && !root.contains(e.target as Node) && !(from && from.contains(e.target as Node)) && void close()}
  onkeydowncapture={(e) => e.key === "Escape" && root?.contains(document.activeElement) && (e.stopPropagation(), void close())} />

<section class="notes" bind:this={root} aria-label="{label}: description and notes" data-place-notes={address} style:top="{at.top}px" style:left="{at.left}px" use:opensFrom={() => from}>
  <header>
    <span class="name">{label}</span>
    {#if kicker}<span class="kicker">{kicker}</span>{/if}
    <button type="button" class="x" aria-label="Close" onclick={() => void close()}>×</button>
  </header>
  {#each KINDS as k (k)}
    <label class="field" data-note-kind={k}>
      <span class="k">{PLACE_NOTE[k].words}{#if k === "gm-notes"}<span class="only">&nbsp;· you alone</span>{/if}</span>
      <textarea
        rows={k === "description" ? 4 : 3}
        placeholder={PLACE_NOTE[k].hint}
        disabled={phase === "reading" || phase === "error"}
        bind:value={text[k]}
        onblur={() => void keep(k)}
        onkeydown={(e) => e.key === "Enter" && (e.ctrlKey || e.metaKey) && (e.preventDefault(), void keep(k))}></textarea>
    </label>
  {/each}
  <p class="status" role="status" data-notes-state={phase}>
    {phase === "reading" ? "Reading the record…" : phase === "saving" ? "Keeping…" : phase === "kept" ? "Kept in the record. Earlier words stay in its history." : phase === "error" ? error : "Kept when you leave a box, or Ctrl+Enter."}
  </p>
</section>

<style>
  .notes {
    position: fixed;
    z-index: 40;
    width: min(380px, calc(100vw - 32px));
    padding: 10px 14px 8px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    box-shadow: var(--sse-card-shadow);
    font: 13px/1.35 var(--sse-font-body);
    text-align: left;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-bottom: 6px;
  }
  .name {
    font: 600 15px var(--sse-font-display);
  }
  .kicker {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .x {
    margin-left: auto;
    min-width: 28px;
    min-height: 28px;
    border: none;
    background: none;
    color: var(--sse-ink-muted);
    font-size: 18px;
    cursor: pointer;
  }
  .field {
    display: grid;
    gap: 3px;
    margin: 6px 0;
  }
  .k {
    font: 11px var(--sse-font-mono);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .only {
    text-transform: none;
    letter-spacing: 0;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 6px 8px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: 13px/1.4 var(--sse-font-body);
    resize: vertical;
  }
  textarea:focus-visible {
    outline: 2px solid var(--sse-accent);
    outline-offset: 0;
  }
  [data-note-kind="gm-notes"] textarea {
    border-style: dashed;
  }
  .status {
    margin: 4px 0 0;
    color: var(--sse-ink-muted);
    font-size: 11.5px;
  }
  @media (max-width: 600px) {
    .notes {
      position: fixed;
      left: 8px !important;
      right: 8px;
      top: auto !important;
      bottom: 8px;
      width: auto;
    }
    textarea {
      font-size: 16px;
    }
  }
</style>
