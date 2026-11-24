<!--
  The Switchboard (track PB2, handover/design/SSE4-playbook-and-switchboard-design.md section 3): everything the GM
  can show is a source on the left, everything that can show it a destination on the right, and a line joins them.
  Press Send to send the live view to Everyone; drag a source's dot onto a role, a screen or Everyone (or press the
  dot, then the destination); click a line, or a destination's hold switch, to hold it with a card and again to let it
  go; Hold all and Release all; Lift takes the top Everyone layer off; Undo and Redo step through tonight's board.
  Screens are dragged onto a role to move them, onto another screen to swap. The board itself (ui-contract board.ts)
  decides what each screen shows; this file only draws it and turns presses into board changes.
-->
<script lang="ts">
  import { addBookmark, addPreset, makeBookmark, back, drive, setSteer, steerOf, emptyPlaybook, findPage, findPreset, fire, hold, holdAll, lift, moveScreen, presetsInView, rejoin, release, releaseAll, resolve, send, swapScreens, type Board, type Dest, type HoldCard, type Playbook as PlaybookDoc, type Preset, type Shown, type TablePort, type Workspace } from "../../../ui-contract/src/index.ts";
  import Playbook from "./Playbook.svelte";
  import PresetMonitor from "./PresetMonitor.svelte";
  import { justSent, noteFired, noteSent, type JustSent } from "./fired.svelte.ts";

  let { table, ws, onopenseats }: { table: TablePort; ws?: Workspace; onopenseats?: () => void } = $props();

  const st = $derived(table.state);
  const board = $derived(st.board);
  const live = $derived(table.live());
  const words = (s: Shown) => ("off" in s ? (s.card ? s.card : "screens dark") : (s.title ?? "the opening place"));
  const cardWords = (c: HoldCard) => (c === "blank" ? "dark" : c === "plain" ? "awaiting transmission" : c === "joke" ? "a joke" : c.text);
  // What a destination shows now, in words, and whether it is held.
  function showing(b: Board, role: string, screen?: string): { text: string; held: boolean } {
    const r = resolve(b, role, screen);
    return r.held ? { text: `held: ${cardWords(r.held)}`, held: true } : { text: words(r.shown), held: false };
  }
  const roles = $derived(st.roles.map((r) => ({ ...r, screens: Object.entries(board.screens).filter(([, s]) => s.role === r.id) })));
  const evText = $derived(showing(board, "").text);
  const heldCount = $derived(Object.values(board.lines).filter((l) => l?.held).length + (board.all ? 1 : 0));
  // The card a click holds with: the kind used last (6.2), plain to begin with.
  let lastCard = $state<HoldCard>("plain");

  // The Playbook (PB3): kept with the campaign in the workspace; the open page and folded sections are this device's.
  const pb = $derived(ws?.playbook ?? emptyPlaybook());
  const bookmarks = $derived(ws?.bookmarks ?? []);
  const loadPref = (k: string) => {
    try {
      return localStorage.getItem(k);
    } catch {
      return null;
    }
  };
  const savePref = (k: string, v: string) => {
    try {
      localStorage.setItem(k, v);
    } catch {
      /* private window: the choice lasts this visit */
    }
  };
  let pageId = $state<string | null>(loadPref("sse4.playbook.page"));
  let folded = $state<Set<string>>(new Set(JSON.parse(loadPref("sse4.playbook.folded") ?? "[]") as string[]));
  $effect(() => savePref("sse4.playbook.page", pageId ?? ""));
  $effect(() => savePref("sse4.playbook.folded", JSON.stringify([...folded])));
  let selected = $state<string | null>(null);
  let editing = $state(false);
  function setPlaybook(next: PlaybookDoc, select?: string) {
    if (ws) ws.playbook = next;
    if (select) ((selected = select), (editing = true));
    if (selected && !findPreset(next, selected)) ((selected = null), (editing = false));
  }
  function firePreset(p: Preset, at?: Dest) {
    noteFired(p.id);
    act((b) => fire(b, p, bookmarks, st.roles.map((r) => r.id), at));
  }
  // Keep on Live (4.4): bookmark where the GM is and put it on the open page as a preset for Everyone, named ready to change.
  function keepLive() {
    if (!ws?.markHere) return;
    const b = ws.markHere();
    ws.bookmarks = addBookmark(bookmarks, b);
    const page = findPage(pb, pageId)?.page;
    const section = page?.sections.find((x) => !folded.has(x.id)) ?? page?.sections[0];
    const r = addPreset(pb, section?.id ?? "", { title: b.name, to: "everyone", steps: [{ show: { bookmark: b.id } }] });
    setPlaybook(r.pb, r.id);
  }
  // Who steers (PB3b, 5.3): Theirs or Read only per role or screen; a read-only one can be driven from the chart.
  const destName = (d: Dest) => (d === "everyone" ? "Everyone" : d.startsWith("role:") ? (st.roles.find((r) => `role:${r.id}` === d)?.label ?? d.slice(5)) : (board.screens[d.slice(7)]?.name ?? "a screen"));
  function startDriving(d: Dest) {
    act((b) => drive(b, d));
    // The GM's stage opens; as they fly, the screen follows.
    if (ws) ws.screen = "system";
  }
  // Keep on the "just now" strip: the send becomes a bookmark (its place and thing) and a preset on the open page.
  function keepSent(j: JustSent) {
    if (!ws || "off" in j.shown) return;
    const list = ws.bookmarks ?? [];
    const b = makeBookmark(list, { scope: j.shown.scope, now_yr: ws.now_yr, instrument: "system", selected: j.shown.focus ?? null }, j.shown.title ?? "Sent", "");
    ws.bookmarks = addBookmark(list, b);
    const page = findPage(pb, pageId)?.page;
    const section = page?.sections.find((x) => !folded.has(x.id)) ?? page?.sections[0];
    const to = j.dest === "everyone" ? "everyone" : j.dest.startsWith("role:") ? j.dest.slice(5) : (board.screens[j.dest.slice(7)]?.role ?? "everyone");
    const r = addPreset(pb, section?.id ?? "", { title: b.name, to, steps: [{ show: { bookmark: b.id } }] });
    setPlaybook(r.pb, r.id);
  }
  function dragPreset(e: PointerEvent, id: string) {
    startDrag(e, { kind: "source", id });
  }

  const act = (f: (b: Board) => Board) => table.act(f);
  function sendLive(dest: Dest) {
    const live = table.live();
    noteSent(live, dest);
    act((b) => send(b, dest, live, "live"));
  }
  function toggleHold(dest: Dest, role: string, screen?: string) {
    const line = board.lines[dest];
    if (line?.held) act((b) => release(b, dest));
    else
      act((b) =>
        hold(
          b,
          dest,
          lastCard,
          st.roles.map((r) => r.id),
        ),
      );
    void role;
    void screen;
  }
  function chooseCard(dest: Dest, c: HoldCard) {
    lastCard = c;
    act((b) => hold(b, dest, c));
  }

  // Dragging: a source's dot onto a destination; a screen onto a role (move) or another screen (swap). Every drag has
  // a press-then-press twin: press the dot (or a screen's grip), then press where it goes.
  let armed = $state<{ kind: "source"; id: string } | { kind: "screen"; id: string } | null>(null);
  let dragging = $state(false);
  function destAt(x: number, y: number): HTMLElement | null {
    return (document.elementFromPoint(x, y) as HTMLElement | null)?.closest<HTMLElement>("[data-dest]") ?? null;
  }
  function drop(el: HTMLElement | null) {
    const a = armed;
    armed = null;
    if (!a || !el) return;
    const dest = el.dataset.dest as Dest;
    if (a.kind === "source") {
      if (a.id === "live") return sendLive(dest);
      const p = findPreset(pb, a.id)?.preset;
      return void (p && firePreset(p, dest));
    }
    if (dest.startsWith("role:")) act((b) => moveScreen(b, a.id, dest.slice(5)));
    else if (dest.startsWith("screen:") && dest.slice(7) !== a.id) act((b) => swapScreens(b, a.id, dest.slice(7)));
  }
  function pressDest(e: MouseEvent | KeyboardEvent) {
    if (!armed) return;
    e.stopPropagation();
    drop(e.currentTarget as HTMLElement);
  }
  function startDrag(e: PointerEvent, what: NonNullable<typeof armed>) {
    if (e.button !== 0) return;
    const x0 = e.clientX;
    const y0 = e.clientY;
    let moved = false;
    const move = (m: PointerEvent) => {
      if (!moved && Math.hypot(m.clientX - x0, m.clientY - y0) > 6) ((moved = true), (dragging = true), (armed = what));
      if (moved) pointer = { x: m.clientX, y: m.clientY };
    };
    const up = (u: PointerEvent) => {
      removeEventListener("pointermove", move);
      removeEventListener("pointerup", up);
      dragging = false;
      pointer = null;
      if (moved) drop(destAt(u.clientX, u.clientY));
      else armed = armed && armed.kind === what.kind && armed.id === what.id ? null : what;
    };
    addEventListener("pointermove", move);
    addEventListener("pointerup", up);
  }
  let pointer = $state<{ x: number; y: number } | null>(null);

  // The lines, drawn over the board from each source's dot to what it reaches. Measured after layout.
  let root: HTMLElement | undefined = $state();
  let paths = $state<{ driving?: boolean; key: string; d: string; held: boolean; faint: boolean; mid: { x: number; y: number }; dest: Dest; role: string; screen?: string }[]>([]);
  let drag = $state<string | null>(null);
  function measure() {
    if (!root) return;
    const box = root.getBoundingClientRect();
    const dotAt = (id: string) => {
      const r = (root!.querySelector<HTMLElement>(`[data-source-dot="${CSS.escape(id)}"]`) ?? root!.querySelector<HTMLElement>('[data-source-dot="live"]'))?.getBoundingClientRect();
      return r ? { x: r.left + r.width / 2 - box.left, y: r.top + r.height / 2 - box.top } : null;
    };
    const liveDot = dotAt("live");
    if (!liveDot) return;
    const out: typeof paths = [];
    const layered = board.over.length > 0;
    for (const el of root.querySelectorAll<HTMLElement>("[data-dest]")) {
      const dest = el.dataset.dest as Dest;
      const line = board.lines[dest];
      const isLayer = dest === "everyone" && layered;
      if (!line && !isLayer) continue;
      const r = el.getBoundingClientRect();
      const x1 = r.left - box.left - 4;
      const y1 = r.top + Math.min(r.height / 2, 20) - box.top;
      // From the preset that sent it, when its row is in view; else from Live.
      const from = isLayer ? (board.over_from ?? [])[board.over.length - 1] : line?.from;
      const { x: sx, y: sy } = (from && from !== "live" ? dotAt(from) : null) ?? liveDot;
      const cx = (sx + x1) / 2;
      const role = el.dataset.role ?? "";
      const screen = el.dataset.screen;
      out.push({ driving: board.driving === dest, key: dest, d: `M${sx},${sy} C${cx},${sy} ${cx},${y1} ${x1},${y1}`, held: !!board.all || (!!line?.held && !isLayer), faint: layered && !isLayer, mid: { x: cx, y: (sy + y1) / 2 }, dest, role, ...(screen ? { screen } : {}) });
    }
    paths = out;
    const ad = armed?.kind === "source" ? (dotAt(armed.id) ?? liveDot) : liveDot;
    drag = pointer && armed ? `M${ad.x},${ad.y} L${pointer.x - box.left},${pointer.y - box.top}` : null;
  }
  $effect(() => {
    void board;
    void st.roles;
    void pointer;
    void pb;
    void pageId;
    void folded;
    void selected;
    void editing;
    requestAnimationFrame(measure);
  });
  $effect(() => {
    if (!root) return;
    const ro = new ResizeObserver(() => measure());
    ro.observe(root);
    addEventListener("scroll", measure, true);
    return () => (ro.disconnect(), removeEventListener("scroll", measure, true));
  });

  function key(e: KeyboardEvent) {
    const t = e.target as HTMLElement;
    if (t.closest("input, textarea, select, [contenteditable]")) return;
    if (e.key === "Escape" && armed) return void (armed = null);
    if ((e.key === "h" || e.key === "H") && !e.ctrlKey && !e.metaKey && !e.altKey) return void (e.preventDefault(), act((b) => (b.all ? releaseAll(b) : holdAll(b, lastCard === "joke" ? "plain" : lastCard))));
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") return void (e.preventDefault(), e.stopPropagation(), e.shiftKey ? table.redo() : table.undo());
    // Keys 1 to 9 fire the presets in view on the open page, in order (4.5).
    if (/^[1-9]$/.test(e.key) && !e.ctrlKey && !e.metaKey && !e.altKey) {
      const page = findPage(pb, pageId)?.page;
      const p = page ? presetsInView(page, folded)[Number(e.key) - 1] : undefined;
      if (p) (e.preventDefault(), firePreset(p));
    }
  }
  let copied = $state(false);
</script>

{#snippet steerCtl(dest: Dest, mode: "theirs" | "read")}
  <button type="button" class="quiet steer" class:read={mode === "read"} data-steer={dest} data-steer-mode={mode} onclick={(e) => (e.stopPropagation(), act((b) => setSteer(b, dest, mode === "read" ? "theirs" : "read")))} title={mode === "read" ? "Read only: it follows you and takes no input. Press for Theirs." : "Theirs: the player has the screen's own controls, within what you sent. Press for Read only."}>{mode === "read" ? "Read only" : "Theirs"}</button>
  {#if mode === "read"}
    {#if board.driving === dest}<button type="button" class="pri" data-let-go={dest} onclick={(e) => (e.stopPropagation(), act((b) => drive(b, null)))}>Let go</button>
    {:else}<button type="button" class="quiet" data-drive={dest} onclick={(e) => (e.stopPropagation(), startDriving(dest))} title="Drive it: your chart becomes its picture, drawn as its role knows it">Drive</button>{/if}
  {/if}
{/snippet}

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<section class="sb" bind:this={root} aria-label="Switchboard" onkeydown={key} data-switchboard data-armed={armed ? `${armed.kind}:${armed.id}` : undefined}>
  <header class="top">
    <h2>Switchboard</h2>
    {#if st.link}
      <span class="onair" data-on-air>On air · {st.players} {st.players === 1 ? "screen" : "screens"}{heldCount ? ` · ${heldCount} held` : ""}</span>
    {:else}
      <button type="button" class="pri" data-go-on-air onclick={() => table.start()}>Go on air</button>
    {/if}
    {#if board.driving}<span class="driving" data-driving={board.driving}>You are driving {destName(board.driving)} <button type="button" class="quiet" data-let-go onclick={() => act((b) => drive(b, null))}>Let go</button></span>{/if}
    <span class="grow"></span>
    <button type="button" data-board-undo disabled={!st.can_undo} onclick={() => table.undo()} title="Take back the last change to what screens show (Ctrl Z)">Undo</button>
    <button type="button" data-board-redo disabled={!st.can_redo} onclick={() => table.redo()} title="Ctrl Shift Z">Redo</button>
    <button type="button" class="strong" data-hold-all aria-pressed={!!board.all} onclick={() => act((b) => (b.all ? releaseAll(b) : holdAll(b, "plain")))} title="Hold every screen (H). Release all puts back each line as it was.">{board.all ? "Release all" : "Hold all"}</button>
  </header>
  <p class="hint">{armed ? (armed.kind === "source" ? "Now press where it goes: Everyone, a role or a screen. Escape to stop." : "Now press a role to move it there, or another screen to swap.") : "Send puts your view on every screen. Drag a dot to a role or a screen, or press it and then the place. Click a line to hold it."}</p>

  <div class="cols">
    <div class="sources">
      {#if selected && findPreset(pb, selected)}
        <PresetMonitor
          {pb}
          id={selected}
          {board}
          {bookmarks}
          roles={st.roles}
          bind:editing
          markHere={ws?.markHere}
          onchange={(next) => setPlaybook(next)}
          onbookmarks={(next) => ws && (ws.bookmarks = next)}
          onfire={firePreset}
          onclose={() => ((selected = null), (editing = false))}
        />
        <!-- Live stays a source while a preset is in the monitor: its dot is still here to drag. -->
        <p class="sub livebar">Live · {words(live)} <button type="button" class="dot" class:on={armed?.kind === "source" && armed.id === "live"} data-source-dot="live" aria-label="Send Live somewhere: press, then press where it goes" onpointerdown={(e) => startDrag(e, { kind: "source", id: "live" })}></button></p>
      {:else}
      <article class="live" data-source="live">
        <header>
          <span><b>Live</b> · <span data-live-title>{words(live)}</span></span>
          <span class="acts">
            {#if ws?.markHere}<button type="button" data-keep-live onclick={keepLive} title="Bookmark where you are and keep it on this page as a preset">Keep</button>{/if}
            <button type="button" class="pri" data-send-live onclick={() => sendLive("everyone")} title="Send your view to every screen">Send</button>
          </span>
        </header>
        <p class="sub">Your view, as each screen's role knows it. Only you see it until you send.</p>
        <footer>
          <span class="sub">drag</span>
          <button type="button" class="dot" class:on={armed?.kind === "source"} data-source-dot="live" aria-label="Send Live somewhere: press, then press where it goes" aria-pressed={armed?.kind === "source"} onpointerdown={(e) => startDrag(e, { kind: "source", id: "live" })} onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), (armed = armed ? null : { kind: "source", id: "live" }))}></button>
        </footer>
      </article>
      {/if}
      <Playbook {pb} {board} {bookmarks} roles={st.roles} bind:pageId bind:folded {selected} armed={armed?.kind === "source" && armed.id !== "live" ? armed.id : null} onchange={setPlaybook} onselect={(id) => ((selected = id), (editing = false))} onfire={firePreset} ondrag={dragPreset} />
      {#if justSent().length}
        <section class="just" aria-label="Just now" data-just-now>
          <h4>Just now</h4>
          <ol>
            {#each justSent() as j (j.id)}
              <li data-just={j.id}>
                <span class="jt">{"off" in j.shown ? "" : (j.shown.title ?? "the opening place")} <span class="sub">to {destName(j.dest)} · {j.at}</span></span>
                {#if ws}<button type="button" class="quiet" data-just-keep={j.id} onclick={() => keepSent(j)} title="Keep it as a preset on this page">Keep</button>{/if}
                <button type="button" class="quiet" data-just-again={j.id} onclick={() => (noteSent(j.shown, j.dest), act((b) => send(b, j.dest, j.shown, "live")))} title="Send it again">Again</button>
              </li>
            {/each}
          </ol>
        </section>
      {/if}
    </div>

    <div class="dests">
      <header class="dhead">
        <h3>Screens</h3>
        {#if st.link}
          <button type="button" data-copy-link onclick={() => (navigator.clipboard?.writeText(st.link!).catch(() => {}), (copied = true), setTimeout(() => (copied = false), 1500))}>{copied ? "Link copied" : "Copy the players' link"}</button>
          <a href={st.link} target="_blank" rel="noopener" data-players-link={st.link}>Open a screen here</a>
        {/if}
        {#if onopenseats}<button type="button" data-open-seats onclick={onopenseats}>Roles…</button>{/if}
      </header>

      <div class="everyone" class:layer={board.over.length > 0} data-dest="everyone" data-role="" role="button" tabindex="0" onclick={pressDest} onkeydown={(e) => (e.key === "Enter" || e.key === " ") && pressDest(e)}>
        <span><b>Everyone</b> · <span data-dest-shows>{board.over.length ? `${words(board.over[board.over.length - 1]!)}, over the set-up` : evText}</span></span>
        <span class="acts">
          {#if board.over.length}<button type="button" data-lift onclick={(e) => (e.stopPropagation(), act(lift))} title="Take this off; every screen goes back to what it had">Lift</button>{/if}
          {@render steerCtl("everyone", steerOf(board, ""))}
          <button type="button" class="quiet" data-hold="everyone" aria-pressed={!!board.lines.everyone?.held} onclick={(e) => (e.stopPropagation(), toggleHold("everyone", ""))}>{board.lines.everyone?.held ? "Release" : "Hold"}</button>
        </span>
      </div>

      {#each roles as r (r.id)}
        {@const own = board.lines[`role:${r.id}`]}
        {@const sh = showing(board, r.id)}
        <section class="role" data-role-id={r.id}>
          <div class="rhead" class:held={sh.held} data-dest={`role:${r.id}`} data-role={r.id} role="button" tabindex="0" onclick={pressDest} onkeydown={(e) => (e.key === "Enter" || e.key === " ") && pressDest(e)}>
            <span><b>{r.label}</b>{#if r.sealed}<span class="sub"> · sealed</span>{/if}</span>
            <span class="sub" data-dest-shows>{own ? sh.text : sh.held ? sh.text : `with Everyone`}</span>
            <span class="acts">
              {#if own && !own.held && own.before !== undefined}<button type="button" class="quiet" data-back={`role:${r.id}`} onclick={(e) => (e.stopPropagation(), act((b) => back(b, `role:${r.id}`)))} title="What it had before this send">Back</button>{/if}
              {#if own && !own.for_hold}<button type="button" class="quiet" data-rejoin={`role:${r.id}`} onclick={(e) => (e.stopPropagation(), act((b) => rejoin(b, `role:${r.id}`)))}>With Everyone</button>{/if}
              {@render steerCtl(`role:${r.id}`, steerOf(board, r.id))}
              <button type="button" class="quiet" data-hold={`role:${r.id}`} aria-pressed={!!own?.held} onclick={(e) => (e.stopPropagation(), toggleHold(`role:${r.id}`, r.id))}>{own?.held ? "Release" : "Hold"}</button>
              {#if own?.held}
                <select aria-label="Holding card for {r.label}" data-card={`role:${r.id}`} onclick={(e) => e.stopPropagation()} onchange={(e) => chooseCard(`role:${r.id}`, (e.currentTarget as HTMLSelectElement).value as HoldCard)} value={typeof own.held === "string" ? own.held : "plain"}>
                  <option value="plain">Awaiting transmission</option>
                  <option value="joke">A joke</option>
                  <option value="blank">Dark</option>
                </select>
              {/if}
            </span>
          </div>
          {#each r.screens as [id, s] (id)}
            {@const mineLine = board.lines[`screen:${id}`]}
            {@const ss = showing(board, r.id, id)}
            <div class="screen" class:held={ss.held} class:armed={armed?.kind === "screen" && armed.id === id} data-dest={`screen:${id}`} data-role={r.id} data-screen={id} role="button" tabindex="0" onclick={pressDest} onkeydown={(e) => (e.key === "Enter" || e.key === " ") && pressDest(e)}>
              <button type="button" class="grip" aria-label="Move {s.name}: press, then press a role, or another screen to swap" aria-pressed={armed?.kind === "screen" && armed.id === id} onpointerdown={(e) => (e.stopPropagation(), startDrag(e, { kind: "screen", id }))} onclick={(e) => e.stopPropagation()} onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), e.stopPropagation(), (armed = armed ? null : { kind: "screen", id }))}>⠿</button>
              <span class="sname"><b>{s.name}</b>{#if s.local}<span class="sub"> · this computer</span>{/if}</span>
              <span class="sub" data-dest-shows>Showing {ss.text}</span>
              <span class="acts">
                {#if mineLine && !mineLine.for_hold}<button type="button" class="quiet" data-rejoin={`screen:${id}`} onclick={(e) => (e.stopPropagation(), act((b) => rejoin(b, `screen:${id}`)))}>Back with {r.label}</button>{/if}
                {@render steerCtl(`screen:${id}`, steerOf(board, r.id, id))}
                <button type="button" class="quiet" data-hold={`screen:${id}`} aria-pressed={!!mineLine?.held} onclick={(e) => (e.stopPropagation(), toggleHold(`screen:${id}`, r.id, id))}>{mineLine?.held ? "Release" : "Hold"}</button>
              </span>
            </div>
          {:else}
            <p class="sub none">No screen tonight.{#if st.link} {#each table.links(r.id).slice(0, 1) as l (l.href)}<a href={l.href} target="_blank" rel="noopener" data-role-link={r.id}>Open one</a>{/each}{/if}</p>
          {/each}
        </section>
      {/each}
    </div>
  </div>

  <svg class="lines" aria-hidden="true">
    {#each paths as p (p.key)}
      <path d={p.d} class:held={p.held} class:faint={p.faint} class:driving={p.driving} data-line={p.key} />
    {/each}
    {#if drag}<path d={drag} class="dragline" />{/if}
  </svg>
  {#each paths as p (p.key)}
    {#if !p.faint}
      <button type="button" class="lozenge" class:held={p.held} style:left="{p.mid.x}px" style:top="{p.mid.y}px" data-line-hold={p.key} aria-label="{p.held ? 'Release' : 'Hold'} the line to {p.dest}" onclick={() => (p.dest === "everyone" && board.over.length ? act(lift) : toggleHold(p.dest, p.role, p.screen))}></button>
    {/if}
  {/each}
</section>

<style>
  .sb {
    position: relative;
    padding: 16px 24px 32px;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  .top {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
    border-bottom: var(--sse-line) solid var(--sse-ink);
    padding-bottom: 10px;
  }
  h2 {
    font-family: var(--sse-font-display);
    font-size: 26px;
    margin: 0;
  }
  h3 {
    font-family: var(--sse-font-display);
    font-size: 19px;
    margin: 0;
  }
  .grow {
    flex: 1;
  }
  .onair {
    border: var(--sse-line) solid var(--sse-ink);
    padding: 2px 8px;
    font-size: 13px;
  }
  .onair::before {
    content: "●";
    color: var(--sse-accent);
    margin-right: 6px;
  }
  button,
  select {
    font: inherit;
    font-size: 13px;
    color: var(--sse-ink);
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    padding: 3px 10px;
    cursor: pointer;
    transition: background var(--sse-motion-quick) var(--sse-ease);
  }
  button:hover:not(:disabled) {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  button.pri {
    background: var(--sse-accent);
    border-color: var(--sse-accent);
    color: var(--sse-on-ink);
  }
  button.strong {
    font-weight: 600;
  }
  button.quiet {
    border-color: var(--sse-rule);
    color: var(--sse-ink-muted);
  }
  button[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .hint,
  .sub {
    color: var(--sse-ink-muted);
    font-size: 13px;
  }
  .cols {
    display: grid;
    grid-template-columns: minmax(0, 360px) minmax(80px, 1fr) minmax(0, 520px);
    gap: 0;
    align-items: start;
  }
  .sources {
    grid-column: 1;
  }
  .dests {
    grid-column: 3;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .live {
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    box-shadow: var(--sse-card-shadow);
  }
  .live header,
  .live footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
  }
  .live footer {
    border-top: var(--sse-line) solid var(--sse-rule);
    justify-content: flex-end;
  }
  .live .acts {
    display: inline-flex;
    gap: 4px;
  }
  .livebar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 12px;
    margin: 0;
  }
  .live > .sub {
    padding: 0 12px;
  }
  .dot {
    width: 18px;
    height: 18px;
    padding: 0;
    border-radius: 50%;
    border: 2px solid var(--sse-ink);
    background: var(--sse-ground-raised);
    touch-action: none;
    margin-right: -22px;
  }
  .dot.on {
    background: var(--sse-accent);
    border-color: var(--sse-accent);
  }
  .dhead {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }
  .dhead h3 {
    flex: 1;
  }
  .dhead a,
  .none a {
    color: var(--sse-accent);
    font-size: 13px;
  }
  .everyone,
  .rhead,
  .screen {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    padding: 8px 12px;
    cursor: default;
  }
  [data-switchboard][data-armed] [data-dest] {
    cursor: copy;
    outline: 1px dashed var(--sse-accent);
    outline-offset: 2px;
  }
  .everyone {
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    justify-content: space-between;
  }
  .everyone.layer {
    box-shadow: var(--sse-card-shadow);
  }
  .role {
    border-bottom: var(--sse-line) solid var(--sse-ink);
  }
  .rhead b {
    font-family: var(--sse-font-display);
    font-size: 18px;
  }
  .rhead > .sub {
    flex: 1;
    text-align: right;
  }
  .acts {
    display: inline-flex;
    gap: 6px;
    align-items: center;
  }
  .screen {
    margin: 0 0 8px 14px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground-raised);
  }
  .screen .sname {
    min-width: 8em;
  }
  .screen > .sub {
    flex: 1;
  }
  .screen.armed {
    border-color: var(--sse-accent);
  }
  .held [data-dest-shows] {
    font-style: italic;
  }
  .grip {
    border: 0;
    background: none;
    padding: 0 4px;
    cursor: grab;
    touch-action: none;
    color: var(--sse-ink-muted);
  }
  .none {
    margin: 0 0 8px 14px;
  }
  .lines {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    overflow: visible;
  }
  .lines path {
    fill: none;
    stroke: var(--sse-ink);
    stroke-width: 1.5;
    transition: opacity var(--sse-motion) var(--sse-ease);
  }
  .lines path.held {
    stroke-dasharray: 10 8;
  }
  .lines path.faint {
    opacity: 0.35;
    stroke-dasharray: 2 4;
  }
  .lines path.dragline {
    stroke: var(--sse-accent);
    stroke-dasharray: 4 4;
  }
  .lozenge {
    position: absolute;
    width: 14px;
    height: 14px;
    margin: -7px 0 0 -7px;
    padding: 0;
    border-radius: 50%;
    border: 1.5px solid var(--sse-ink);
    background: var(--sse-ground-raised);
  }
  .lozenge.held {
    border-radius: 2px;
    transform: rotate(45deg);
    background: var(--sse-accent);
    border-color: var(--sse-accent);
  }
  .just {
    margin-top: 14px;
    border-top: var(--sse-line) solid var(--sse-rule);
    padding-top: 6px;
  }
  .just h4 {
    margin: 0 0 4px;
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    font-weight: normal;
  }
  .just ol {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .just li {
    display: flex;
    gap: 4px;
    align-items: center;
    font-size: 13px;
  }
  .jt {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .lines path.driving {
    stroke: var(--sse-accent);
    stroke-width: 2.5;
  }
  .driving {
    font-size: 13px;
    color: var(--sse-accent-ink, var(--sse-accent));
  }
  .steer.read {
    border-color: var(--sse-ink);
    color: var(--sse-ink);
  }
  .lozenge::after {
    content: "";
    position: absolute;
    inset: -8px;
  }
  @media (max-width: 760px) {
    .sb {
      padding: 12px;
    }
    .cols {
      grid-template-columns: minmax(0, 1fr);
      gap: 16px;
    }
    .sources,
    .dests {
      grid-column: 1;
    }
    .lines,
    .lozenge {
      display: none;
    }
    .dot {
      margin-right: 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    button,
    .lines path {
      transition: none;
    }
  }
</style>
