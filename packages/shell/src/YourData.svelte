<!--
  Your data (V3 gap 27), in the Campaign dialog: how much this browser keeps for SSE4, asking it to keep it, a
  backup, a diagnostic file, a set-aside campaign if one failed to open, and Clear all. Rarely touched, so it lives
  with the campaign it is about (Alex, 23 Nov 2026: "rarely touched things in menus that have relevant context").
-->
<script lang="ts">
  import { askToKeep, bytesWords, clearAll, diagnostic, downloadText, restoreSetAside, setAsideText, storageUse, type DiagnosticState, type StorageUse } from "./your-data.ts";

  let { onbackup, report }: { onbackup: () => void; report: () => Promise<DiagnosticState> } = $props();

  let use = $state<StorageUse | null>(null);
  let asked = $state<string | null>(null);
  let confirmClear = $state(false);
  let aside = $state(setAsideText());
  const refresh = async () => (use = await storageUse());
  void refresh();

  async function keep() {
    const ok = await askToKeep();
    asked = ok ? "The browser will keep it." : ok === false ? "The browser said no for now. Browsers often agree once a site is bookmarked or used often." : "This browser cannot be asked.";
    await refresh();
  }
  async function diag() {
    downloadText(await diagnostic(await report()), `sse4-diagnostic-${new Date().toISOString().slice(0, 10)}.json`);
  }
  function downloadAside() {
    if (aside) downloadText(aside, "sse4-set-aside-campaign.json");
  }
  function tryAgain() {
    if (restoreSetAside()) location.reload();
  }
  async function clear() {
    await clearAll();
    location.reload();
  }
  const kept = $derived(use ? use.campaign + use.setAside + use.other : 0);
</script>

<section class="your-data" data-your-data aria-labelledby="your-data-title">
  <h3 id="your-data-title">Your data</h3>
  {#if use}
    <p data-used>
      This browser keeps <b>{bytesWords(kept)}</b> for SSE4: the campaign {bytesWords(use.campaign)}{#if use.setAside}, a set-aside campaign {bytesWords(use.setAside)}{/if}, settings and keys {bytesWords(use.other)}.
      {#if use.usage && use.quota}The site uses {bytesWords(use.usage)} of the {bytesWords(use.quota)} the browser allows.{/if}
    </p>
    <p data-persisted={String(use.persisted)}>
      {#if use.persisted}The browser has agreed to keep it, even when space runs low.
      {:else}The browser may clear it when space runs low. <button type="button" class="linkbtn" data-keep onclick={keep}>Ask the browser to keep it</button>{/if}
      {#if asked}<span class="said">{asked}</span>{/if}
    </p>
  {/if}
  {#if aside}
    <p role="status" data-set-aside>A campaign that did not open is set aside here, not lost. <button type="button" class="linkbtn" onclick={tryAgain}>Try opening it again</button> · <button type="button" class="linkbtn" onclick={downloadAside}>Download it</button></p>
  {/if}
  <p class="row">
    <button type="button" onclick={onbackup}>Download a backup</button>
    <button type="button" data-diagnostic onclick={diag} title="What ran and what went wrong, for Tars or a problem report. No keys, no campaign.">Diagnostic file</button>
  </p>
  {#if confirmClear}
    <p role="alert">This forgets everything SSE4 keeps in this browser: the campaign, any set-aside copy, your settings and your agent keys. Download a backup first to keep the campaign.</p>
    <p class="row"><button type="button" data-clear-all onclick={clear}>Clear it all</button> <button type="button" onclick={() => (confirmClear = false)}>Keep it</button></p>
  {:else}
    <p class="row"><button type="button" class="quiet-btn" onclick={() => (confirmClear = true)}>Clear all data…</button></p>
  {/if}
</section>

<style>
  .your-data p {
    margin: 6px 0;
    font-size: 14px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  button:not(.linkbtn) {
    height: 40px;
    padding: 0 16px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ink);
    color: var(--sse-on-ink);
    font-family: var(--sse-font-body);
  }
  button.quiet-btn {
    background: none;
    color: var(--sse-ink);
  }
  .said {
    display: block;
    color: var(--sse-ink-muted);
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
</style>
