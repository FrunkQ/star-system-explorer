<!--
  Flow editor: a real flow from the pack stack drawn as ink on the chart
  ground, inputs on the left, atoms by depth, published quantities on the
  right, with the values of one real subject. Changing a parameter writes the
  GM's draft change request; nothing changes in the physics until the draft
  is adopted in the change-request panel. The selected node explains itself
  in the three lenses.
-->
<script lang="ts">
  import {
    benchRef,
    diffFlow,
    draftId,
    draftParam,
    flowColumns,
    paramText,
    previewRequest,
    siText,
    type Derived,
    type Explanation,
    type FlowDoc,
    type Json,
    type Preview,
    type SystemScene,
    type ViewContext,
  } from "../../../ui-contract/src/index.ts";
  import { Lenses, References, Stamp } from "../../../chart-grammar/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const ws = $derived(ctx.workspace);
  const session = $derived(ctx.session);

  const flows = $derived.by(() => {
    void ws.revision;
    return session.flows().filter((f) => !f.startsWith("toy."));
  });
  const flow = $derived(ws.bench?.flow ?? String(params.flow));
  // The flow on screen is the one on the bench, so the other views of the screen see the same one.
  $effect(() => {
    if (!ws.bench) ws.bench = { flow: String(params.flow) };
  });
  const inForce = $derived.by(() => {
    void ws.revision;
    return session.flowDoc(flow);
  });
  const doc = $derived(inForce?.doc);
  const shipped = $derived.by(() => {
    void ws.revision;
    const base = session.stack().base;
    for (let i = base.length - 1; i >= 0; i--) {
      const f = base[i]!.provides?.flows?.find((x) => x.id === flow);
      if (f) return f;
    }
    return undefined;
  });
  const changedFromShipped = $derived(doc && shipped ? diffFlow(shipped, doc).params : []);
  const isBase = $derived(!!inForce && session.stack().base.some((p) => p.id === inForce.pack));
  const draft = $derived(ws.requests.find((r) => r.id === draftId(flow) && r.status === "draft"));
  const bench = $derived(benchRef(scene, ws, flow));
  const editable = $derived(ws.mode === "edit" && session.seat.authority === "gm");

  let selected = $state<string | null>(null);
  $effect(() => {
    selected = ws.bench?.node || null;
  });
  const node = $derived(doc?.nodes.find((n) => n.id === selected) ?? doc?.nodes[0]);

  // Values now, and with the draft, for the bench's subject.
  let now = $state<Record<string, Derived> | null>(null);
  let preview = $state<Preview | null>(null);
  let gen = 0;
  $effect(() => {
    void ws.revision;
    const b = bench;
    const d = draft;
    const g = ++gen;
    if (!b) {
      now = null;
      preview = null;
      return;
    }
    session.derive(b.ref).then((r) => g === gen && (now = r.ok ? r.values : null));
    if (d && d.edits.length) previewRequest(session, ws, d, b.ref).then((p) => g === gen && (preview = p));
    else preview = null;
  });
  const after = (kind: string) => preview?.values.find((v) => v.kind === kind)?.after ?? null;

  // Explanation of the selected node, through the first quantity it publishes.
  let explained = $state<Explanation | null>(null);
  let lens = $state<"curious" | "science" | "engine">("curious");
  $effect(() => {
    void ws.revision;
    const kind = doc?.publishes.find((p) => p.from.split(".")[0] === node?.id)?.kind;
    const b = bench;
    explained = null;
    if (kind && b) session.explain(b.ref, kind).then((r) => (explained = r.ok ? r.explanation : null));
  });

  // Layout, in CSS pixels. Columns: flow inputs, atoms by depth, published quantities.
  const PAD = 24,
    GAP = 64,
    IN_W = 160,
    NODE_W = 216,
    PUB_W = 200,
    HEAD = 50,
    ROW = 26,
    CHIP = 44;
  type Box = { id: string; x: number; y: number; h: number; ins: string[]; outs: string[]; params: [string, number][]; atom: string };
  const layout = $derived.by(() => {
    if (!doc) return null;
    const cols = flowColumns(doc);
    const port = (p: string) => p.split(".");
    const boxes = new Map<string, Box>();
    let tallest = 0;
    cols.forEach((col, c) => {
      let y = PAD;
      for (const id of col) {
        const n = doc.nodes.find((x) => x.id === id)!;
        const ins = [...new Set(doc.wires.filter((w) => port(w.to)[0] === id).map((w) => port(w.to)[1]!))];
        const outs = [...new Set([...doc.wires.filter((w) => port(w.from)[0] === id).map((w) => port(w.from)[1]!), ...doc.publishes.filter((p) => port(p.from)[0] === id).map((p) => port(p.from)[1]!)])];
        const ps = Object.entries(n.params ?? {});
        const h = HEAD + Math.max(ins.length, outs.length, 1) * ROW + (ps.length ? ps.length * (ROW + 14) + 8 : 0) + 10;
        boxes.set(id, { id, x: PAD + IN_W + GAP + c * (NODE_W + GAP), y, h, ins, outs, params: ps, atom: n.atom });
        y += h + 28;
      }
      tallest = Math.max(tallest, y);
    });
    const pubX = PAD + IN_W + GAP + cols.length * (NODE_W + GAP);
    const inputs = doc.inputs.map((i, k) => ({ ...i, x: PAD, y: PAD + k * (CHIP + 18) }));
    const pubs = doc.publishes.map((p, k) => ({ ...p, x: pubX, y: PAD + k * (CHIP + 18) }));
    const anchor = (p: string, side: "out" | "in"): [number, number] | null => {
      const [n, name] = port(p);
      if (n === "in") {
        const i = inputs.find((x) => x.name === name);
        return i ? [i.x + IN_W, i.y + CHIP / 2] : null;
      }
      const b = boxes.get(n!);
      if (!b) return null;
      const list = side === "out" ? b.outs : b.ins;
      const k = list.indexOf(name!);
      return [side === "out" ? b.x + NODE_W : b.x, b.y + HEAD + k * ROW + ROW / 2];
    };
    const curve = (a: [number, number], b: [number, number]) => {
      const dx = Math.max(30, (b[0] - a[0]) / 2);
      return `M${a[0]},${a[1]} C${a[0] + dx},${a[1]} ${b[0] - dx},${b[1]} ${b[0]},${b[1]}`;
    };
    const wires = [
      ...doc.wires.map((w) => ({ key: `${w.from}>${w.to}`, a: anchor(w.from, "out"), b: anchor(w.to, "in"), pub: false })),
      ...doc.publishes.map((p, k) => ({ key: `${p.from}>pub:${p.kind}`, a: anchor(p.from, "out"), b: [pubs[k]!.x, pubs[k]!.y + CHIP / 2] as [number, number], pub: true })),
    ]
      .filter((w) => w.a && w.b)
      .map((w) => ({ key: w.key, d: curve(w.a!, w.b!), pub: w.pub }));
    const height = Math.max(tallest, PAD + Math.max(inputs.length, pubs.length) * (CHIP + 18)) + PAD;
    return { boxes: [...boxes.values()], inputs, pubs, wires, width: pubX + PUB_W + PAD, height };
  });

  const num = (v: number) => (Math.abs(v) >= 1e5 || (Math.abs(v) < 1e-3 && v !== 0) ? v.toExponential(3) : String(Number(v.toPrecision(4))));
  const pretty = (k: string) => k.replace(/^[a-z]+\./, "").replace(/_/g, " ");
  const short = (atom: string) => atom.split(".").slice(-1)[0]!.replace(/_/g, " ");
  const UNIT: Record<string, string> = { l_sun: "L☉", m_sun: "M☉", au: "AU", gyr: "Gyr", one: "" };
  const inputText = (name: string) => {
    const v = bench?.ref.inputs[name];
    const u = v?.unit ? (UNIT[v.unit] ?? v.unit) : "";
    return v ? `${num(v.value)}${u ? ` ${u}` : ""}` : "no value";
  };

  // Fit the drawing to the room the screen leaves it (never enlarged); scroll below 60 %.
  let room = $state(0);
  const fit = $derived(layout && room ? Math.max(0.6, Math.min(1, room / layout.width)) : 1);
  const draftValue = (n: string, p: string) => draft?.edits.find((e) => e.node === n && e.param === p)?.after;
  const shippedValue = (n: string, p: string) => changedFromShipped.find((e) => e.node === n && e.param === p)?.before;

  let errors = $state<Record<string, string>>({});
  function commit(n: string, p: string, text: string, input: HTMLInputElement) {
    const v = Number(text.trim());
    const key = `${n}.${p}`;
    if (!text.trim() || !Number.isFinite(v)) {
      errors = { ...errors, [key]: "Type a number." };
      input.value = paramText(draftValue(n, p) ?? doc!.nodes.find((x) => x.id === n)!.params![p]!);
      return;
    }
    const { [key]: _, ...rest } = errors;
    errors = rest;
    draftParam(session, ws, flow, n, p, v);
  }
</script>

<section class="editor" aria-label="Flow editor">
  <header class="head">
    <div class="title">
      <label class="k" for="flow-pick">Flow</label>
      <select id="flow-pick" value={flow} onchange={(e) => ws.openFlow((e.currentTarget as HTMLSelectElement).value)}>
        {#each flows as f (f)}<option value={f}>{f}</option>{/each}
      </select>
      {#if doc}<span class="meta">v{doc.version} · from {inForce?.pack}</span>{/if}
      {#if !isBase}<Stamp kind="changed-physics" title="Published by the campaign's own physics pack" />{/if}
    </div>
    <div class="for">
      {#if bench}<span class="k">Values for</span> <span class="who">{bench.label}</span>{:else}<span class="muted">No body uses this flow yet; values show once one does.</span>{/if}
    </div>
  </header>

  {#if !doc || !layout}
    <p class="muted pad">The stack has no flow {flow}.</p>
  {:else}
    <div class="canvas" role="group" aria-label="Flow {flow}">
      <div class="room" bind:clientWidth={room}></div>
      <div class="sizer" style="width: {layout.width * fit}px; height: {layout.height * fit}px">
      <div class="plane" style="width: {layout.width}px; height: {layout.height}px; transform: scale({fit})">
        <svg class="wires" width={layout.width} height={layout.height} aria-hidden="true">
          {#each layout.wires as w (w.key)}<path d={w.d} class:pub={w.pub} />{/each}
        </svg>
        {#each layout.inputs as i (i.name)}
          <div class="chip input" style="left: {i.x}px; top: {i.y}px; width: {IN_W}px; height: {CHIP}px">
            <span class="cap">{i.name}</span>
            <span class="val">{inputText(i.name)}</span>
          </div>
        {/each}
        {#each layout.boxes as b (b.id)}
          <div class="node" class:sel={node?.id === b.id} style="left: {b.x}px; top: {b.y}px; width: {NODE_W}px; height: {b.h}px">
            <button type="button" class="nhead" aria-pressed={node?.id === b.id} onclick={() => (selected = b.id)} aria-label="Node {b.id}, {short(b.atom)}">
              <span class="nid">{b.id}</span>
              <span class="atom">{short(b.atom)}</span>
            </button>
            <div class="ports">
              <ul class="ins">{#each b.ins as p (p)}<li>{p}</li>{/each}</ul>
              <ul class="outs">{#each b.outs as p (p)}<li>{p}</li>{/each}</ul>
            </div>
            {#each b.params as [p, v] (p)}
              {@const dv = draftValue(b.id, p)}
              {@const was = shippedValue(b.id, p)}
              <div class="param" class:drafted={dv !== undefined}>
                <label for="p-{b.id}-{p}">{p.replace(/_/g, " ")}</label>
                {#if editable}
                  <input
                    id="p-{b.id}-{p}"
                    inputmode="decimal"
                    value={paramText(dv ?? v)}
                    onchange={(e) => commit(b.id, p, e.currentTarget.value, e.currentTarget)}
                    onkeydown={(e) => {
                      if (e.key === "Enter") e.currentTarget.blur();
                      if (e.key === "Escape") e.currentTarget.value = paramText(dv ?? v);
                    }}
                    aria-describedby="p-{b.id}-{p}-note"
                    autocomplete="off"
                    spellcheck="false"
                  />
                {:else}
                  <output id="p-{b.id}-{p}">{paramText(v)}</output>
                {/if}
                <span class="pnote" id="p-{b.id}-{p}-note">
                  {#if errors[`${b.id}.${p}`]}{errors[`${b.id}.${p}`]}
                  {:else if dv !== undefined}draft · in force {paramText(v)}
                  {:else if was !== undefined}changed · shipped {paramText(was)}{/if}
                </span>
              </div>
            {/each}
          </div>
        {/each}
        {#each layout.pubs as p (p.kind)}
          {@const v = now?.[p.kind]}
          {@const a = after(p.kind)}
          <div class="chip pub" class:moved={a && v && a.value !== v.value} style="left: {p.x}px; top: {p.y}px; width: {PUB_W}px; height: {CHIP}px" data-kind={p.kind}>
            <span class="cap">{pretty(p.kind)}</span>
            <span class="val">
              {#if v}{num(v.value)} {siText(v.dim)}{:else}…{/if}
              {#if a && v && a.value !== v.value}<span class="to">→ {num(a.value)}</span>{/if}
            </span>
          </div>
        {/each}
      </div>
      </div>
    </div>

    {#if node}
      <aside class="explain" aria-label="Explain node {node.id}">
        <div class="ehead">
          <span class="k">Explain</span>
          <span class="nid">{node.id}</span>
          <span class="atom">{node.atom}</span>
        </div>
        {#if explained}
          <p class="formula">{explained.formula_display}</p>
          <Lenses text={explained.lenses} bind:lens />
          {#if explained.references.length}<References refs={explained.references} inline />{/if}
        {:else if !doc.publishes.some((p) => p.from.split(".")[0] === node.id)}
          <p class="muted">This node publishes nothing directly; its values feed the nodes after it.</p>
        {:else}
          <p class="muted">{bench ? "Asking the kernel…" : "Explanations need real values: pick a body this flow serves."}</p>
        {/if}
      </aside>
    {/if}
  {/if}
</section>

<style>
  .editor {
    position: absolute;
    inset: 0;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) auto;
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 20px;
    padding: 14px 24px 10px 88px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .title {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  select {
    min-height: 36px;
    max-width: 100%;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font-family: var(--sse-font-mono);
    font-size: 13px;
    padding: 0 8px;
  }
  .k {
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .meta,
  .muted {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .who {
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 16px;
  }
  .pad {
    padding: 24px;
  }
  .canvas {
    overflow: auto;
    min-height: 0;
    padding-left: 64px;
    padding-right: 404px;
  }
  .room {
    height: 0;
  }
  .sizer {
    position: relative;
  }
  .plane {
    position: relative;
    transform-origin: 0 0;
  }
  .wires {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  .wires path {
    fill: none;
    stroke: var(--sse-ink);
    stroke-width: 1.25;
  }
  .wires path.pub {
    stroke-dasharray: 4 3;
    stroke: var(--sse-ink-muted);
  }
  .chip {
    position: absolute;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 0 10px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
  }
  .chip.pub {
    border-style: dashed;
  }
  .chip.pub.moved {
    border: var(--sse-line-heavy) solid var(--sse-accent);
  }
  .cap {
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .val {
    font-family: var(--sse-font-mono);
    font-size: 13px;
    white-space: nowrap;
  }
  .to {
    color: var(--sse-accent-ink);
    font-weight: 600;
  }
  .node {
    position: absolute;
    box-sizing: border-box;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    box-shadow: var(--sse-card-shadow);
  }
  .node.sel {
    outline: var(--sse-line-heavy) solid var(--sse-accent);
    outline-offset: 2px;
  }
  .nhead {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: center;
    width: 100%;
    height: 50px;
    padding: 0 10px;
    border: 0;
    border-bottom: var(--sse-line) solid var(--sse-rule);
    background: transparent;
    color: var(--sse-ink);
    text-align: left;
    cursor: pointer;
  }
  .nid {
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .atom {
    font-family: var(--sse-font-display);
    font-size: 16px;
    line-height: 1.1;
  }
  .ports {
    display: flex;
    justify-content: space-between;
  }
  .ports ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .ports li {
    height: 26px;
    line-height: 26px;
    padding: 0 10px;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .outs li {
    text-align: right;
  }
  .param {
    display: grid;
    grid-template-columns: 1fr 88px;
    align-items: center;
    column-gap: 8px;
    margin: 0 10px 8px;
    padding-top: 6px;
    border-top: var(--sse-line) dotted var(--sse-rule);
  }
  .param label {
    font-size: 12px;
  }
  .param input,
  .param output {
    width: 100%;
    box-sizing: border-box;
    border: 0;
    border-bottom: var(--sse-line) solid var(--sse-ink-faint);
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-mono);
    font-size: 14px;
    text-align: right;
    padding: 2px 0;
  }
  .param input:focus {
    outline: none;
    border-bottom: var(--sse-line-heavy) solid var(--sse-accent);
  }
  .param.drafted input {
    color: var(--sse-accent-ink);
    font-weight: 600;
  }
  .pnote {
    grid-column: 1 / -1;
    min-height: 14px;
    font-size: 11px;
    color: var(--sse-accent-ink);
  }
  .explain {
    border-top: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    padding: 10px 404px 12px 88px;
    max-height: 34vh;
    overflow: auto;
  }
  .ehead {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }
  .ehead .k {
    color: var(--sse-accent-ink);
  }
  .formula {
    margin: 6px 0 0;
    font-family: var(--sse-font-display);
    font-size: 18px;
  }
  @media (max-width: 760px) {
    .editor {
      position: relative;
    }
    .head {
      padding: 10px 12px;
    }
    .canvas {
      padding: 0;
    }
    .explain {
      padding: 10px 12px;
      max-height: none;
    }
  }
</style>
