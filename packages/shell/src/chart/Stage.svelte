<!-- The stage (system chart and body cards section, owned by the chart room build): the view in front with its view
     controls, the tools beside it, the undo and clipboard notes, and the card. App says which views are mounted and
     mounts them (`view`); this lays them out. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { ToolButton, ViewControls } from "../../../chart-grammar/src/index.ts";
  import { actionById, hasTool, partShows, type Desk, type Json, type ViewChoice, type ViewControlsSpec, type ViewModule, type Workspace } from "../../../ui-contract/src/index.ts";

  type Mount = { module: ViewModule; params: Record<string, Json> };
  let {
    ws,
    desk,
    lensId,
    ahead,
    viewSpec,
    setView,
    chartMount,
    cardMount,
    openings,
    view,
    clipArmed,
    cuttable,
    undoNote,
    clipNote,
    undo,
    copyBody,
    openAdd,
    typeAdd,
    cutBody,
    openKeys,
    extraTools,
  }: {
    ws: Workspace;
    desk: Desk;
    lensId: string;
    /** A look ahead is open (the views show its snapshot). */
    ahead: boolean;
    viewSpec: ViewControlsSpec | undefined;
    setView: (c: ViewChoice) => void;
    chartMount: Mount | undefined;
    cardMount: Mount | undefined;
    /** Bumped to mount the chart's view afresh (a place opened again). */
    openings: number;
    /** Mounts a view module on a node (App's, with the session, workspace, marks and styles). */
    view: (node: HTMLElement, m: Mount) => { update?: (m: Mount) => void; destroy?: () => void } | void;
    clipArmed: boolean;
    cuttable: boolean;
    undoNote: string | null;
    clipNote: string | null;
    undo: (back: boolean) => void;
    copyBody: () => void;
    openAdd: () => void;
    /** While the Add ghost follows the pointer (ED4): open Add here to type a distance instead. */
    typeAdd?: () => void;
    cutBody: () => void;
    openKeys: () => void;
    /** Tools another section adds after Redo (the Run desk's Bookmarks, PB2b), so the stage imports nothing of theirs. */
    extraTools?: Snippet;
  } = $props();

  // The view controls' height: what sits below them starts there, however many rows they wrap to.
  let viewbarH = $state(0);
  // Build's Copy and Add here say what the object menu and More say: the words and keys come from the one action list
  // (ui-contract actions.ts). A tool's name has no "…": the button is the action itself.
  const tool = (id: string) => {
    const a = actionById(id)!;
    return { label: a.label({ address: "", kind: "", label: "" }).replace(/…$/, ""), key: a.key };
  };
</script>

<main class="stage" style:--viewbar-h={viewbarH ? `${viewbarH}px` : undefined} data-viewbar={viewSpec ? "" : undefined} data-desk={desk} data-screen={ws.screen} data-page={cardMount ? undefined : ""} data-lens={lensId} data-branch={ws.branch ? "open" : ahead ? "ahead" : undefined}>
  {#if ws.branch}<div class="branch-stamp" aria-hidden="true">What if · not kept</div>{/if}
  {#if viewSpec && ws.view}<div class="viewbar" bind:clientHeight={viewbarH} data-chart-overlay><ViewControls spec={viewSpec} choice={ws.view} onchange={setView} /></div>{/if}
  <div class="area-chart">{#if chartMount}{#key openings}<div class="fill" use:view={chartMount}></div>{/key}{/if}</div>
  <nav class="tools" aria-label="Tools" data-chart-overlay>
    {#if partShows("tools", desk, "select")}<ToolButton label="Select" icon="M5 3l14 8-6 2-2 6z" pressed={true} />{/if}
    {#if hasTool(ws.lens, "undo") && (partShows("tools", desk, "undo") || ws.branch)}<ToolButton label="Undo" icon="M9 14L4 9l5-5M4 9h11a5 5 0 0 1 0 10h-3" keyhint="Control+Z" disabled={ws.undo.length === 0 || ws.mode !== "edit" || !!ws.branch} onclick={() => undo(true)} />
    <ToolButton label="Redo" icon="M15 14l5-5-5-5M20 9H9a5 5 0 0 0 0 10h3" keyhint="Control+Shift+Z" disabled={ws.redo.length === 0 || ws.mode !== "edit" || !!ws.branch} onclick={() => undo(false)} />{/if}
    {@render extraTools?.()}
    {#if clipArmed}<ToolButton label={tool("copy").label} icon="M8 8h11v11H8zM5 16V5h11" keyhint={tool("copy").key} disabled={!ws.selection} onclick={copyBody} />{/if}
    {#if clipArmed}<ToolButton label={tool("add").label} icon="M12 5v14M5 12h14" keyhint={tool("add").key} onclick={openAdd} />{/if}
    {#if clipArmed && cuttable}<ToolButton label="Cut" icon="M6 4l12 16M18 4L6 20M4 18a2 2 0 1 0 4 0M16 18a2 2 0 1 0 4 0" keyhint="Control+X" onclick={cutBody} />{/if}
    {#if partShows("tools", desk, "keys")}<ToolButton label="Keyboard shortcuts" icon="M3 7h18v10H3zM7 11h1M11 11h1M15 11h2M7 14h10" keyhint="?" onclick={openKeys} />{/if}
  </nav>
  {#if undoNote}<p class="undo-note" role="status">{undoNote}</p>{/if}
  {#if ws.adding && ws.adding.a_au === null}
    <p class="undo-note clip-note" role="status" data-add-note>Press on the chart where it should go round {ws.adding.label}; the ghost says how far out. Esc puts it away.{#if typeAdd} <button type="button" class="type-add" data-add-type onclick={typeAdd}>Type a distance</button>{/if}</p>
  {:else if clipNote}<p class="undo-note clip-note" role="status" data-clip-note>{clipNote}</p>{/if}
  <aside class="area-card" aria-label={ws.screen === "physics" ? "Change requests" : "Selected body"}>{#if cardMount}<div use:view={cardMount} data-chart-overlay></div>{/if}</aside>
</main>

<style>
  .branch-stamp {
    position: absolute;
    top: 14px;
    left: 50%;
    transform: translateX(-50%) rotate(-1deg);
    z-index: 3;
    padding: 2px 10px;
    border: var(--sse-line) solid var(--sse-accent);
    color: var(--sse-accent-ink);
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    background: var(--sse-ground);
    pointer-events: none;
  }
  .stage[data-branch] {
    outline: 2px dashed var(--sse-accent);
    outline-offset: -4px;
  }
  .stage {
    position: relative;
    min-height: 0;
  }
  .area-chart,
  .fill {
    position: absolute;
    inset: 0;
  }
  .area-chart .fill {
    position: relative;
    height: 100%;
  }
  .stage[data-screen="body"] .area-chart,
  .stage[data-screen="cutaway"] .area-chart,
  .stage[data-screen="compare"] .area-chart,
  .stage[data-screen="life"] .area-chart {
    inset: 24px 428px 24px 96px;
    overflow: auto;
  }
  .stage[data-page] .area-chart {
    inset: 24px 24px 24px 96px;
    overflow: auto;
  }
  .stage[data-page] .area-chart .fill {
    height: auto;
    min-height: 100%;
    padding: 0 0 24px;
  }
  .stage[data-page] .area-card {
    display: none;
  }
  .viewbar {
    position: absolute;
    left: 96px;
    top: 16px;
    right: 440px;
    z-index: 2;
    pointer-events: none;
  }
  .viewbar > :global(*) {
    pointer-events: auto;
    width: fit-content;
  }
  .stage[data-viewbar] .undo-note {
    top: calc(var(--viewbar-h, 68px) + 16px);
  }
  .stage[data-viewbar][data-screen="body"] .area-chart,
  .stage[data-viewbar][data-screen="cutaway"] .area-chart,
  .stage[data-viewbar][data-screen="compare"] .area-chart,
  .stage[data-viewbar][data-page] .area-chart {
    top: calc(var(--viewbar-h, 68px) + 16px);
  }
  .tools {
    position: absolute;
    left: 24px;
    top: 24px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .undo-note {
    position: absolute;
    left: 88px;
    top: 24px;
    margin: 0;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .type-add {
    margin-left: 6px;
    padding: 0 4px;
    border: 0;
    border-bottom: var(--sse-line) dotted var(--sse-accent-ink);
    background: none;
    color: var(--sse-accent-ink);
    font: inherit;
    cursor: pointer;
  }
  .clip-note {
    top: 48px;
    padding: 4px 8px;
    background: var(--sse-ground);
    max-width: min(560px, calc(100% - 520px));
  }
  .area-card {
    position: absolute;
    right: 24px;
    top: 24px;
    width: 380px;
    max-height: calc(100% - 110px);
    overflow: auto;
    padding: 0 6px 6px 0;
  }
  @media (max-width: 760px) {
    .tools {
      display: none;
    }
    .stage {
      display: contents;
    }
    .viewbar {
      position: static;
      padding: 8px 12px 6px;
      pointer-events: auto;
    }
    .area-chart {
      position: relative;
      height: 44vh;
      border-bottom: var(--sse-line) solid var(--sse-ink);
    }
    .area-chart .fill {
      position: absolute;
    }
    .stage:not([data-screen="system"]) .area-chart {
      height: 72dvh;
    }
    .stage:not([data-screen="system"]) .area-chart .fill {
      overflow: auto;
    }
    .stage:is([data-screen="physics"], [data-screen="body"], [data-screen="cutaway"], [data-screen="compare"], [data-screen="life"], [data-page]) .area-chart {
      height: auto;
    }
    .stage:is([data-screen="physics"], [data-screen="body"], [data-screen="cutaway"], [data-screen="compare"], [data-screen="life"], [data-page]) .area-chart .fill {
      position: relative;
    }
    .stage[data-screen="body"] .area-chart,
    .stage[data-screen="cutaway"] .area-chart,
    .stage[data-screen="compare"] .area-chart,
    .stage[data-screen="life"] .area-chart,
    .stage[data-page] .area-chart {
      inset: auto;
      padding: 12px 16px;
    }
    .undo-note {
      position: absolute;
      left: 88px;
      top: 24px;
      margin: 0;
      font-size: 13px;
      color: var(--sse-ink-muted);
    }
    .area-card {
      position: static;
      width: auto;
      max-height: none;
      overflow: visible;
      /* A card opening from its body (U12) slides a little sideways; it must never make the page scroll. */
      overflow-x: clip;
      margin: 16px 16px 24px;
      padding: 0 6px 6px 0;
    }
  }
  @media (max-width: 760px) {
    .clip-note {
      position: static;
      max-width: none;
      margin: 8px 16px;
    }
  }
</style>
