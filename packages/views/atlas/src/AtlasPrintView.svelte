<!--
  The print view (AT4; atlas design 14A note 7, Alex 25 Nov 01:43): every page of the book as a small sheet, in order,
  each with a tick box, its number and name, and how many sheets of paper it takes. Ticked sheets are bright,
  unticked fade back; all are ticked on opening. Quick picks (All, None, Only the page I was on, This chapter), a
  running count, Ink (As shown, or Black and white: a true toner-friendly form), Paper (A4 or Letter), then Print.
  The thumbnails are the page's own blocks drawn small in the chosen ink, so what you see is what the printer gets.
-->
<script lang="ts">
  import { tick, untrack } from "svelte";
  import type { AtlasPage as Page, ContentsLine } from "../../../ui-contract/src/index.ts";
  import AtlasPage from "./AtlasPage.svelte";
  import Segmented from "../../../chart-grammar/src/Segmented.svelte";

  let {
    lines,
    pageOf,
    current,
    chapter,
    title,
    subtitle,
    look,
    bwLook,
    dark,
    footer,
    paper = $bindable(),
    ink = $bindable(),
    onprint,
    oncopy,
    onsave,
    onclose,
  }: {
    lines: ContentsLine[];
    pageOf: (address: string) => Page | null;
    current: string | null;
    chapter: string[];
    title: string;
    subtitle: string;
    look: string;
    bwLook: string;
    dark: boolean;
    footer: string;
    paper: "A4" | "Letter";
    ink: "shown" | "bw";
    onprint: (addresses: string[]) => void;
    oncopy: (addresses: string[], markdown: boolean) => void;
    onsave: (addresses: string[]) => void;
    onclose: () => void;
  } = $props();

  // The printable box of one sheet (the @page margins of AtlasPrint: 16 mm each side, 16 top, 18 bottom).
  const MM = 96 / 25.4;
  const box = $derived(paper === "Letter" ? { w: 215.9 - 32, h: 279.4 - 34 } : { w: 210 - 32, h: 297 - 34 });
  const THUMB = 168;
  const scale = $derived(THUMB / (box.w * MM));

  // All ticked on opening: the view is made fresh each time it opens.
  let ticked = $state<string[]>(untrack(() => lines.map((l) => l.address)));
  const pages = $derived(lines.map((l) => ({ line: l, page: pageOf(l.address) })).filter((x) => x.page !== null) as { line: ContentsLine; page: Page }[]);
  // Sheets per page, measured from the page drawn at the sheet's own width.
  let heights = $state<Record<string, number>>({});
  const sheetsOf = (a: string) => Math.max(1, Math.ceil((heights[a] ?? 0) / (box.h * MM)));
  const chosen = $derived(lines.map((l) => l.address).filter((a) => ticked.includes(a)));
  const sheets = $derived(chosen.reduce((n, a) => n + sheetsOf(a), 0));
  const countWords = $derived(`${chosen.length} of ${pages.length} ${pages.length === 1 ? "page" : "pages"}, ${sheets} ${sheets === 1 ? "sheet" : "sheets"}`);

  function measure(node: HTMLElement, address: string) {
    const ro = new ResizeObserver(() => (heights = { ...heights, [address]: node.scrollHeight }));
    ro.observe(node);
    return { destroy: () => ro.disconnect() };
  }
  const toggle = (a: string) => (ticked = ticked.includes(a) ? ticked.filter((x) => x !== a) : [...ticked, a]);

  let dialog: HTMLElement | undefined = $state();
  $effect(() => {
    tick().then(() => dialog?.querySelector<HTMLElement>("[data-print-go]")?.focus({ preventScroll: true }));
  });
  function onkey(e: KeyboardEvent) {
    if (e.key === "Escape") (e.preventDefault(), onclose());
  }
</script>

<div class="veil" role="presentation" onkeydown={onkey}>
  <div class="view" bind:this={dialog} role="dialog" aria-modal="true" aria-label="Print" data-print-view>
    <header class="bar">
      <h2>Print: {title}<small>{subtitle}</small></h2>
      <Segmented label="Ink" options={[{ value: "shown", label: "As shown" }, { value: "bw", label: "Black and white" }]} value={ink} onchange={(v) => (ink = v as "shown" | "bw")} />
      <Segmented label="Paper" options={[{ value: "A4", label: "A4" }, { value: "Letter", label: "Letter" }]} value={paper} onchange={(v) => (paper = v as "A4" | "Letter")} />
      <span class="count" aria-live="polite" data-print-count>{countWords}</span>
      <button type="button" class="go" data-print-go disabled={!chosen.length} onclick={() => onprint(chosen)}>Print</button>
      <button type="button" onclick={onclose}>Close</button>
    </header>
    {#if dark && ink === "shown"}
      <p class="warn" role="note">This edition is dark: printed as shown it covers every sheet in ink. Black and white saves it.</p>
    {/if}
    <div class="quick" role="group" aria-label="Choose pages">
      <button type="button" onclick={() => (ticked = lines.map((l) => l.address))}>All</button>
      <button type="button" onclick={() => (ticked = [])}>None</button>
      <button type="button" disabled={!current} onclick={() => (ticked = current ? [current] : [])}>Only the page I was on</button>
      <button type="button" disabled={!chapter.length} onclick={() => (ticked = [...chapter])}>This chapter</button>
      <span class="gapfill"></span>
      <button type="button" disabled={!chosen.length} onclick={() => oncopy(chosen, false)}>Copy as text</button>
      <button type="button" disabled={!chosen.length} onclick={() => oncopy(chosen, true)}>Copy as Markdown</button>
      <button type="button" disabled={!chosen.length} onclick={() => onsave(chosen)}>Save as a web page</button>
    </div>
    <ol class="grid">
      {#each pages as { line, page } (line.address)}
        {@const on = ticked.includes(line.address)}
        <li class="sheet" class:off={!on}>
          <label>
            <input type="checkbox" checked={on} onchange={() => toggle(line.address)} aria-label="Print {line.number}. {line.title}" />
            <span class="thumb" style="width: {THUMB}px; height: {THUMB * (box.h / box.w)}px" aria-hidden="true">
              <span class="paper {ink === 'bw' ? 'ink-bw' : ''}" style="{ink === 'bw' ? bwLook : look}; width: {box.w * MM}px; transform: scale({scale})" use:measure={line.address} inert>
                <AtlasPage {page} onlink={() => {}} {footer} />
              </span>
            </span>
          </label>
          <p class="cap"><span>{line.number}. {line.title}</span><span>{sheetsOf(line.address)} {sheetsOf(line.address) === 1 ? "sheet" : "sheets"}</span></p>
        </li>
      {/each}
    </ol>
  </div>
</div>

<style>
  .veil {
    position: fixed;
    inset: 0;
    z-index: 40;
    background: color-mix(in srgb, var(--sse-ink) 35%, transparent);
    display: grid;
    padding: 2vh 2vw;
  }
  .view {
    background: var(--sse-ground);
    color: var(--sse-ink);
    border: 1px solid var(--sse-rule);
    border-radius: 6px;
    overflow: auto;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 16px;
    padding: 12px 16px;
    background: var(--sse-ground-raised, var(--sse-ground));
    border-bottom: 1px solid var(--sse-rule);
    position: sticky;
    top: 0;
    z-index: 1;
  }
  h2 {
    margin: 0 auto 0 0;
    font: italic 400 1.25rem/1.2 var(--sse-font-display, var(--sse-font-body));
  }
  h2 small {
    margin-left: 10px;
    font: 400 0.85rem var(--sse-font-body);
    font-style: normal;
    color: var(--sse-ink-muted);
  }
  .count {
    color: var(--sse-ink-muted);
    font-size: 0.9rem;
  }
  button {
    font: inherit;
    padding: 6px 12px;
    border: 1px solid var(--sse-ink);
    border-radius: 4px;
    background: var(--sse-ground-raised, var(--sse-ground));
    color: var(--sse-ink);
    cursor: pointer;
  }
  button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  button.go {
    background: var(--sse-accent);
    border-color: var(--sse-accent);
    color: var(--sse-on-ink);
    font-weight: 600;
  }
  .warn {
    margin: 10px 16px 0;
    color: var(--sse-ink-muted);
  }
  .quick {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding: 12px 16px 0;
  }
  .gapfill {
    flex: 1;
  }
  .grid {
    list-style: none;
    margin: 0;
    padding: 16px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(184px, 1fr));
    gap: 20px 18px;
  }
  .sheet label {
    display: block;
    position: relative;
    cursor: pointer;
  }
  .sheet input {
    position: absolute;
    top: 8px;
    left: 8px;
    z-index: 1;
    width: 20px;
    height: 20px;
    accent-color: var(--sse-accent);
  }
  .thumb {
    display: block;
    overflow: hidden;
    border: 1px solid var(--sse-rule);
    background: var(--sse-ground-raised, var(--sse-ground));
    box-shadow: 0 1px 3px color-mix(in srgb, var(--sse-ink) 18%, transparent);
    transition: opacity 160ms ease, filter 160ms ease;
  }
  .paper {
    display: block;
    transform-origin: 0 0;
    min-height: 100%;
    background: var(--sse-ground-raised, var(--sse-ground));
    pointer-events: none;
    container: atlas / inline-size;
  }
  .paper :global(.atlas-page) {
    max-width: none;
    box-shadow: none;
    border: 0;
  }
  .off .thumb {
    opacity: 0.45;
    filter: saturate(0.4);
  }
  .cap {
    display: flex;
    justify-content: space-between;
    margin: 6px 0 0;
    font-size: 0.85rem;
    color: var(--sse-ink-muted);
  }
  .cap span:first-child {
    color: var(--sse-ink);
  }
  @media (prefers-reduced-motion: reduce) {
    .thumb {
      transition: none;
    }
  }
</style>
