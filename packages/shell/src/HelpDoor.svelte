<!--
  The Help door (V3 gap 31): one door in More, and "?" anywhere. Three pages: What's new (from whats-new.json, one
  entry per stage end), Getting started (the stage 1 walk, handover/test-drive.md, as a few lines a newcomer can follow
  with the doors that start it) and Keyboard. V3's separate guides become tours in stage 6, and its physics reference
  is the flows themselves and Look up, so Help stays small. The dot on More's Help marks an entry this device has not
  seen; opening What's new marks it seen (the shell's openHelp, or the page switch here).
-->
<script lang="ts" module>
  import doc from "./documents/whats-new.json";
  import { loadPref, savePref } from "./prefs.ts";

  export type HelpPage = "new" | "start" | "keys";
  export interface NewsEntry {
    id: string;
    date: string;
    title: string;
    items: string[];
  }
  export const NEWS: NewsEntry[] = doc.entries;
  const SEEN = "help.seen";
  /** True when the newest entry is one this device has not opened yet. */
  export function unseenNews(): boolean {
    return NEWS.length > 0 && loadPref(SEEN) !== NEWS[0]!.id;
  }
  export function markNewsSeen(): void {
    if (NEWS.length) savePref(SEEN, NEWS[0]!.id);
  }
</script>

<script lang="ts">
  import { Segmented } from "../../chart-grammar/src/index.ts";

  let {
    page = $bindable("new"),
    onstart,
    onask,
    onsol,
  }: { page?: HelpPage; onstart: () => void; onask: () => void; onsol: () => void } = $props();

  const PAGES: { value: HelpPage; label: string }[] = [
    { value: "new", label: "What's new" },
    { value: "start", label: "Getting started" },
    { value: "keys", label: "Keyboard" },
  ];
  const day = (d: string) => new Date(`${d}T12:00:00Z`).toLocaleDateString(undefined, { day: "numeric", month: "long", year: "numeric" });
  function choose(v: HelpPage) {
    page = v;
    if (v === "new") markNewsSeen();
  }
</script>

<h2 id="help-title">Help</h2>
<Segmented label="Help page" value={page} options={PAGES} onchange={choose} slide />

<div class="page" data-help-page={page}>
  {#if page === "new"}
    {#each NEWS as e (e.id)}
      <section class="entry" data-news={e.id}>
        <h3>{e.title}</h3>
        <p class="when">{day(e.date)}</p>
        <ul>{#each e.items as it (it)}<li>{it}</li>{/each}</ul>
      </section>
    {/each}
  {:else if page === "start"}
    <p>A first walk round, about five minutes. Nothing needs a key or an account.</p>
    <ol class="walk">
      <li><b>Open the Solar System</b> from the start sheet, or <button type="button" class="linkbtn" data-help-sol onclick={onsol}>open it now</button>.</li>
      <li><b>Pick a world</b> on the chart. Its card shows what it is made of and how it moves.</li>
      <li><b>Ask why.</b> Every value has a "why?" beside it: where it came from, at the depth you choose (Curious, Science or Engine).</li>
      <li><b>Change something.</b> Choose the Build desk and type a new value. Whatever follows from it changes too, and Ctrl+Z takes it back.</li>
      <li><b>Open the world</b> with "Open … ›" on its card: its globe, Mag for its magnetic field, and Cutaway on the rail.</li>
      <li><b>Run time</b> with Play on the ruler at the bottom.</li>
      <li><b>Go out and back.</b> The place line at the top goes out to the galaxy and the cosmic web; "‹ Back" on the rail returns you.</li>
      <li><b>Look things up.</b> "Look up" on the rail has elements, substances, constants and units.</li>
    </ol>
    <p class="doors">
      <button type="button" class="linkbtn" onclick={onstart}>Start sheet</button>
      <button type="button" class="linkbtn" onclick={onask}>Ask Tars what you can do here</button>
      <a class="linkbtn" href="https://github.com/FrunkQ/star-system-explorer/issues" target="_blank" rel="noopener">Report a problem</a>
    </p>
  {:else}
    <dl>
      <dt>?</dt><dd>This help, on the Keyboard page</dd>
      <dt>Tab</dt><dd>Move between the chart's bodies, the card and the ruler</dd>
      <dt>Arrows</dt><dd>On the chart, the next or previous body; on the ruler, 1 % of the span</dd>
      <dt>Enter</dt><dd>Select the focused body; on the card, ink the typed value</dd>
      <dt>Esc</dt><dd>Put a typed value back, or clear the selection</dd>
      <dt>[ and ]</dt><dd>Previous or next body</dd>
      <dt>C, T</dt><dd>Go to the chart, go to the time ruler</dd>
      <dt>Page Up, Page Down, Home, End</dt><dd>On the ruler, larger steps</dd>
      <dt>Drag a body, or Page Up, Page Down on it</dt><dd>Move its orbit out or in (Build desk)</dd>
      <dt>Ctrl+C, Ctrl+X, Ctrl+V</dt><dd>Copy the selected body with everything that orbits it; cut a pasted body; paste a copied body (Build desk)</dd>
      <dt>Ctrl+Z, Ctrl+Shift+Z</dt><dd>Undo, redo (each is a new correction on the chart's history)</dd>
      <dt>why?</dt><dd>On any value: where it came from, at three depths (Curious, Science, Engine); "Open the flow" puts its physics on the bench</dd>
      <dt>P</dt><dd>Chart or Physics: the flow editor and change requests</dd>
      <dt>V, E, R</dt><dd>Desk: Explore (look), Build (change), Run (the table)</dd>
      <dt>D</dt><dd>Depth: Curious, Science, Engine (how much every card and number says)</dd>
      <dt>N</dt><dd>Chart room or Night chart</dd>
      <dt>L</dt><dd>See the desk through the next lens (Basic, Advanced, Building, Running, as players see it)</dd>
    </dl>
  {/if}
</div>

<style>
  h2 {
    margin: 0 0 12px;
    font-family: var(--sse-font-display);
  }
  .page {
    margin: 14px 0 16px;
    max-height: min(60vh, 520px);
    overflow-y: auto;
    font-size: 14px;
  }
  .entry + .entry {
    margin-top: 16px;
    padding-top: 12px;
    border-top: var(--sse-line) solid var(--sse-rule);
  }
  h3 {
    margin: 0;
    font-size: 15px;
  }
  .when {
    margin: 2px 0 6px;
    color: var(--sse-ink-muted);
    font-size: 13px;
  }
  ul,
  ol {
    margin: 0;
    padding-left: 20px;
  }
  li + li {
    margin-top: 4px;
  }
  .doors {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 16px;
    margin-top: 12px;
  }
  .linkbtn {
    border: none;
    background: none;
    padding: 0;
    height: auto;
    color: var(--sse-accent-ink);
    font: inherit;
    text-decoration: underline;
    cursor: pointer;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 16px;
    margin: 0;
  }
  dt {
    font-family: var(--sse-font-mono);
    font-size: 13px;
  }
  dd {
    margin: 0;
  }
</style>
