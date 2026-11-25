<script lang="ts">
  // Send a postcard (design section 7 and 12A point 8): write on the back (it opens holding "Wish you were here!"),
  // see what goes with it, then copy it, copy a link, or save a file. The message belongs to this sending only.
  import { onMount } from "svelte";
  import { DEFAULT_MESSAGE, FROM_MAX, MESSAGE_MAX, goesWith, makeMessage, postcardFile, postcardLink, postcardText, withMessage, type Postcard } from "../../../ui-contract/src/index.ts";

  let { card, stamp = null, onpack, onclose }: { card: Postcard; stamp?: string | null; onpack?: () => Promise<Postcard | string>; onclose: () => void } = $props();

  let text = $state(DEFAULT_MESSAGE);
  let from = $state("");
  let packed = $state<Postcard | null>(null);
  let refused = $state<string | null>(null);
  let note = $state<string | null>(null);
  onMount(() => {
    if (card.world.how !== "here" || !onpack) return void (packed = card);
    void onpack().then((r) => (typeof r === "string" ? (refused = r) : (packed = r)));
  });
  const ready = $derived(packed ? withMessage(stamp && !packed.stamp ? { ...packed, stamp } : packed, text.trim() || from.trim() ? makeMessage(text, from) : null) : null);
  const goes = $derived(packed ? goesWith(packed) : null);
  const link = $derived(ready ? postcardLink(ready) : null);

  async function copy(what: "text" | "link") {
    if (!ready) return;
    const t = what === "link" ? (link?.ok ? link.url : "") : postcardText(ready);
    if (!t) return;
    try {
      await navigator.clipboard.writeText(t);
      note = what === "link" ? "Link copied. Anyone can open it, with or without SSE4." : "Copied. Paste it into SSE4 anywhere and it opens as a visit.";
    } catch {
      note = "The browser would not let SSE4 write to the clipboard.";
    }
  }
  function save() {
    if (!ready) return;
    const f = postcardFile(ready);
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([f.text], { type: "application/json" }));
    a.download = f.name;
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
    note = `Saved ${f.name}.`;
  }
</script>

<div class="send" role="group" aria-label="Send {card.name}" data-postcard-sendbox>
  <label>Write on the back <textarea bind:value={text} maxlength={MESSAGE_MAX} rows="3" data-postcard-message-input></textarea></label>
  <label>From <input bind:value={from} maxlength={FROM_MAX} placeholder="Your name" /></label>
  <p class="quiet">Plain words only. The message goes with this sending, never to the shop.</p>
  {#if refused}<p data-postcard-refused>{refused}</p>{/if}
  {#if goes}<p class="goes" data-postcard-whatgoes><b>What goes with it:</b> {goes.words}</p>{/if}
  <div class="row">
    <button type="button" disabled={!ready} onclick={() => copy("text")} data-postcard-copy>Copy</button>
    <button type="button" disabled={!link?.ok} title={link && !link.ok ? link.why : "A link that opens it anywhere"} onclick={() => copy("link")} data-postcard-link>Copy link</button>
    <button type="button" disabled={!ready} onclick={save} data-postcard-file>Save file</button>
    <button type="button" onclick={onclose}>Close</button>
  </div>
  {#if link && !link.ok}<p class="quiet">{link.why}</p>{/if}
  {#if note}<p role="status" data-postcard-sent>{note}</p>{/if}
</div>

<style>
  .send {
    margin-top: 8px;
    padding: 12px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
    display: grid;
    gap: 6px;
    font-size: 13px;
  }
  label {
    display: grid;
    gap: 2px;
  }
  textarea,
  input {
    font: inherit;
    padding: 4px 6px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground);
    color: var(--sse-ink);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  button {
    font: inherit;
    padding: 4px 10px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .quiet {
    opacity: 0.7;
    font-size: 12px;
    margin: 0;
  }
  .goes {
    margin: 0;
  }
</style>
