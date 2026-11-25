<!--
  On this computer (PB9, playbook and switchboard design 3.2), owned by the "Run desk and players' side" build: the
  displays attached to the GM's computer (the second monitor, the projector, the wall TV), each drawn as a small box in
  its real place. Press one and choose a role: SSE opens a window on it that fills it at a press (F does the same).
  Where the browser cannot list displays, one button opens a window to drag there. Each window opened from here is a
  screen on the board, marked "this computer"; Split (Whole, Two, Three, Four) is kept to the list of open windows, and
  each part is then a screen of its own. Windows are this visit's: they are not kept with the campaign.
-->
<script lang="ts">
  import { SPLITS, displayLink, displayMap, displaysFrom, type Display, type Split } from "../../../ui-contract/src/index.ts";
  import { closeDisplay, openDisplay, opened, resplit } from "./displays.svelte.ts";

  let { roles, linkOf, onclose }: { roles: readonly { id: string; label: string }[]; linkOf: (role: string) => string | undefined; onclose: () => void } = $props();

  type ScreenDetails = { screens: { label?: string; left?: number; top?: number; availLeft?: number; availTop?: number; width: number; height: number; isPrimary?: boolean }[] };
  const canList = typeof window !== "undefined" && "getScreenDetails" in window;
  let displays = $state<Display[]>([]);
  let asked = $state(false);
  let failed = $state<string | null>(null);
  let chosen = $state<string | null>(null);
  let role = $state(untrackFirst());
  function untrackFirst() {
    return roles[0]?.id ?? "table";
  }
  const map = $derived(displayMap(displays));

  async function find() {
    failed = null;
    asked = true;
    try {
      const d = (await (window as unknown as { getScreenDetails: () => Promise<ScreenDetails> }).getScreenDetails()).screens;
      displays = displaysFrom(d.map((s) => ({ ...s, left: s.left ?? s.availLeft, top: s.top ?? s.availTop })));
      chosen = displays.find((x) => !x.primary)?.id ?? displays[0]?.id ?? null;
    } catch {
      failed = "This browser did not say which displays are attached. Open a window and drag it there instead.";
    }
  }
  function open(d: Display | null) {
    const join = linkOf(role);
    if (!join) return;
    const name = d?.label ?? "This computer";
    openDisplay({ display: d, role, name, href: displayLink(join, name) });
  }
  const roleName = (id: string) => roles.find((r) => r.id === id)?.label ?? id;
</script>

<div class="otc" role="group" aria-label="Screens on this computer" data-on-this-computer>
  <p class="sub">A display on this computer becomes a screen: choose one and the role it shows, then press Open. The window fills the display at a press, or with F.</p>
  <label class="role">For <select bind:value={role} data-otc-role>{#each roles as r (r.id)}<option value={r.id}>{r.label}</option>{/each}</select></label>
  {#if canList && !failed}
    {#if !asked}
      <button type="button" data-otc-find onclick={find}>Find the displays</button>
    {:else if displays.length}
      <div class="map" aria-label="The displays, where they sit">
        {#each displays as d, i (d.id)}
          {@const b = map[i]!}
          <button type="button" class="disp" aria-pressed={chosen === d.id} data-otc-display={d.id} style:left="{b.x}%" style:top="{b.y}%" style:width="{b.w}%" style:height="{b.h}%" onclick={() => (chosen = d.id)} title="{d.label}, {d.width} by {d.height}">{d.label}</button>
        {/each}
      </div>
      <button type="button" class="go" data-otc-open disabled={!chosen} onclick={() => open(displays.find((d) => d.id === chosen) ?? null)}>Open on {displays.find((d) => d.id === chosen)?.label ?? "a display"}</button>
    {/if}
  {:else}
    {#if failed}<p class="sub" data-otc-failed>{failed}</p>{/if}
    <button type="button" class="go" data-otc-open onclick={() => open(null)}>Open a window to drag there</button>
  {/if}

  {#if opened.length}
    <ul class="open" aria-label="Windows open on this computer">
      {#each opened as w (w.key)}
        <li data-otc-window={w.key}>
          <span><b>{w.name}</b> <span class="sub">{roleName(w.role)}</span></span>
          <label class="sub">Split <select value={w.split} data-otc-split onchange={(e) => resplit(w.key, e.currentTarget.value as Split)}>{#each Object.entries(SPLITS) as [k, s] (k)}<option value={k}>{s.words}</option>{/each}</select></label>
          <button type="button" class="quiet" data-otc-close onclick={() => closeDisplay(w.key)}>Close</button>
        </li>
      {/each}
    </ul>
  {/if}
  <button type="button" class="quiet" data-otc-done onclick={onclose}>Done</button>
</div>

<style>
  .otc {
    display: grid;
    gap: 8px;
    padding: 10px 12px;
    margin: 6px 0 10px;
    border: var(--sse-line) solid var(--sse-rule);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
    font-size: 13px;
  }
  .sub {
    color: var(--sse-ink-muted);
    font-size: 12px;
    margin: 0;
  }
  .role {
    display: inline-flex;
    gap: 6px;
    align-items: center;
  }
  .map {
    position: relative;
    width: 100%;
    max-width: 320px;
    aspect-ratio: 100 / 60;
  }
  .disp {
    position: absolute;
    box-sizing: border-box;
    padding: 2px;
    font-size: 11px;
    overflow: hidden;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: 2px;
    background: var(--sse-ground);
    color: var(--sse-ink);
    cursor: pointer;
  }
  .disp[aria-pressed="true"] {
    border-color: var(--sse-accent);
    background: color-mix(in srgb, var(--sse-accent) 16%, var(--sse-ground));
    font-weight: 600;
  }
  .open {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
  }
  .open li {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }
  .open li > span {
    flex: 1;
  }
  button.go {
    justify-self: start;
    font-weight: 600;
  }
  button.quiet {
    justify-self: start;
  }
</style>
