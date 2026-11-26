<!-- The instrument rail (top-level design 2.2; navigation section): every way of looking at the place in view, in
     words, built from the screens' own rail entries, with the doors in and out (a world, the galaxy, the Workshop). -->
<script lang="ts">
  import type { RailScale, railOf } from "../../../ui-contract/src/index.ts";

  let {
    railScale,
    rail,
    screen,
    title,
    selName,
    backWords,
    back,
    go,
    toWorld,
    toSystem,
    toGalaxy,
    toWorkshop,
    workshopDoor = true,
  }: {
    railScale: RailScale;
    rail: ReturnType<typeof railOf>;
    /** The screen in view. */
    screen: string;
    /** The system in view's title. */
    title: string;
    selName: string | null;
    /** Above a system: the name "‹ Back to …" says (the world or system the GM came from). */
    backWords: string;
    /** Above a system: back down to where the GM came from. */
    back: () => void;
    go: (screen: string) => void;
    toWorld: () => void;
    toSystem: () => void;
    toGalaxy: () => void;
    toWorkshop: () => void;
    /** The Workshop door at the foot of the rail; off on Run, where it is in More (three desks design section 5). */
    workshopDoor?: boolean;
  } = $props();
</script>

<nav class="rail" aria-label="Instruments" data-scale={railScale}>
  {#if railScale === "workshop"}
    <button type="button" class="door back" data-go="back" onclick={toSystem}>‹ Back to {title}</button>
  {:else if railScale !== "system" && railScale !== "world"}
    <button type="button" class="door back" data-go="back" onclick={back}>‹ Back to {backWords}</button>
  {:else if railScale === "system"}
    <button type="button" class="door open" data-go="world" onclick={() => toWorld()} title={selName ? `Look at ${selName} itself: globe, cutaway, life` : "Pick a world to look at closely"}>{selName ? `Open ${selName}` : "Open a world"} ›</button>
  {/if}
  {#each rail as g (g.group)}
    <div class="group" role="group" aria-label={g.words}>
      <h2>{g.words}</h2>
      {#each g.lines as l (l.screen)}
        <button type="button" class="inst" aria-pressed={screen === l.screen} data-go={l.screen} title={l.hint} onclick={() => go(l.screen)}>{l.label}</button>
      {/each}
    </div>
  {/each}
  {#if railScale === "system"}<button type="button" class="door" data-go="out" onclick={toGalaxy} title="The galaxy around this system, the galaxies nearby and the cosmic web: they are on the place line above the system">Out to the galaxy ›</button>{/if}
  {#if railScale !== "workshop" && workshopDoor}<button type="button" class="door" data-go="workshop" onclick={toWorkshop} title="Physics flows, packs, player screens and data, symbols, styles: everything that changes the tool">Workshop ›</button>{/if}
</nav>

<style>
  .rail {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px 12px 18px 18px;
    border-right: var(--sse-line) solid var(--sse-rule);
    overflow: auto;
  }
  .rail .group {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .rail h2 {
    margin: 0 0 4px;
    font-family: var(--sse-font-mono);
    font-size: 10px;
    font-weight: normal;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .rail .inst {
    position: relative;
    text-align: left;
    font: inherit;
    font-size: 14px;
    padding: 5px 8px;
    margin-left: -8px;
    border: 0;
    background: none;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .rail .inst:hover {
    background: var(--sse-ground-raised);
  }
  .rail .inst[aria-pressed="true"] {
    font-weight: 600;
  }
  .rail .inst[aria-pressed="true"]::before {
    content: "";
    position: absolute;
    left: -6px;
    top: 6px;
    bottom: 6px;
    width: 3px;
    background: var(--sse-accent);
    animation: rail-in var(--sse-motion) var(--sse-ease);
  }
  .rail .door {
    text-align: left;
    font: inherit;
    font-size: 13px;
    padding: 6px 10px;
    border: var(--sse-line) solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .rail .door.open {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .rail .door[data-go="workshop"] {
    margin-top: auto;
    border-color: var(--sse-rule);
    color: var(--sse-ink-muted);
  }
  .rail .door.back {
    border: 0;
    padding: 0;
    color: var(--sse-accent);
  }
  @media (max-width: 760px) {
    .rail {
      flex-direction: row;
      align-items: center;
      gap: 6px;
      padding: 8px 12px;
      border-right: 0;
      border-bottom: var(--sse-line) solid var(--sse-rule);
      overflow-x: auto;
      overflow-y: hidden;
      white-space: nowrap;
      scrollbar-width: none;
    }
    .rail .group {
      flex-direction: row;
      gap: 6px;
    }
    .rail > *,
    .rail .group > * {
      flex: none;
    }
    .rail h2 {
      display: none;
    }
    .rail .inst {
      margin: 0;
      min-height: 36px;
      padding: 0 12px;
      border: 1px solid var(--sse-rule);
    }
    .rail .inst[aria-pressed="true"] {
      border-color: var(--sse-ink);
    }
    .rail .inst[aria-pressed="true"]::before {
      display: none;
    }
    .rail .door {
      min-height: 36px;
      white-space: nowrap;
    }
    .rail .door[data-go="workshop"] {
      margin-top: 0;
    }
  }
  @keyframes rail-in {
    from {
      transform: scaleY(0);
    }
  }
</style>
