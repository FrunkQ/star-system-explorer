<!--
  Readout: one value as a number, a gauge or a status light. The subject is
  the selection (or a fixed address); the field and the style come from the
  screen document. An unknown value shows its placeholder, never zero.
-->
<script lang="ts">
  import { describeFact, fact, subjectName, wordsFor, type PlayerContext, type PlayerScene } from "../../../ui-contract/src/index.ts";
  import { focusOf } from "./pick.ts";

  let { ctx, scene }: { ctx: PlayerContext; scene: PlayerScene } = $props();
  const p = $derived(ctx.params);
  const subject = $derived(typeof p.subject === "string" && p.subject !== "selected" ? scene.subjects.find((s) => s.address === p.subject) : focusOf(scene, ctx.selection()));
  const style = $derived(String(p.style ?? "value"));
  const f = $derived(subject && p.field ? fact(subject, String(p.field)) : undefined);
  const d = $derived(f ? describeFact(ctx.vocab, f) : null);
  const label = $derived(p.label ? String(p.label) : style === "light" ? String(ctx.vocab.tags?.[String(p.tag)] ?? p.tag) : wordsFor(ctx.vocab, String(p.field ?? "")).label);
  const range = $derived.by((): [number, number] => {
    const r = p.range as number[] | undefined;
    return r && r.length === 2 ? [r[0]!, r[1]!] : [0, 1];
  });
  const t = $derived(d && "number" in d && d.number !== undefined ? Math.max(0, Math.min(1, (d.number - range[0]) / (range[1] - range[0]))) : null);
  const lit = $derived(style === "light" && !!subject?.tags.includes(String(p.tag)));
</script>

<div class="readout" data-style={style} role="group" aria-label="{label}{subject ? `, ${subjectName(subject)}` : ''}">
  <div class="cap"><span>{label}</span>{#if subject}<span class="who">{subjectName(subject)}</span>{/if}</div>
  {#if style === "light"}
    <div class="lamp"><span class="bulb" class:lit aria-hidden="true"></span><span class="state">{lit ? "on" : "off"}</span></div>
  {:else if !d}
    <div class="big unknown">no data</div>
  {:else if d.state === "unknown"}
    <div class="big unknown">{d.text}</div>
  {:else}
    <div class="big">{d.text}</div>
    {#if style === "gauge" && t !== null}
      <div class="gauge" role="meter" aria-valuemin={range[0]} aria-valuemax={range[1]} aria-valuenow={"number" in d ? d.number : undefined} aria-label={label}>
        <span class="fill" style="width: {t * 100}%"></span>
      </div>
      <div class="ends"><span>{range[0]}</span><span>{range[1]}</span></div>
    {/if}
  {/if}
</div>

<style>
  .readout {
    padding: 12px 16px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  .cap {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .who {
    font-family: var(--sse-font-display);
    font-style: italic;
    text-transform: none;
    letter-spacing: 0;
    font-size: 13px;
  }
  .big {
    margin-top: 4px;
    font-family: var(--sse-font-mono);
    font-size: 28px;
    line-height: 1.2;
  }
  .big.unknown {
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 18px;
    color: var(--sse-ink-muted);
  }
  .gauge {
    position: relative;
    height: 10px;
    margin-top: 8px;
    border: var(--sse-line) solid var(--sse-ink);
  }
  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    background: var(--sse-accent);
  }
  .ends {
    display: flex;
    justify-content: space-between;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .lamp {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 6px;
  }
  .bulb {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
  }
  .bulb.lit {
    background: var(--sse-accent);
    box-shadow: 0 0 0 3px var(--sse-ground-raised), 0 0 0 4px var(--sse-accent);
  }
  .state {
    font-family: var(--sse-font-mono);
    font-size: 14px;
    text-transform: uppercase;
  }
</style>
