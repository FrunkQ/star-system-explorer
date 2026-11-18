<!--
  Text: prose about one subject from the vocabulary layer's sentences. What
  the audience has not been given is said plainly ("Its orbit is not yet
  scanned"), never guessed. Minerva (A3) will write richer prose from the
  same facts; this is the plain layer.
-->
<script lang="ts">
  import { describeFact, sentence, subjectName, type PlayerContext, type PlayerScene } from "../../../ui-contract/src/index.ts";
  import { focusOf } from "./pick.ts";

  let { ctx, scene }: { ctx: PlayerContext; scene: PlayerScene } = $props();
  const subject = $derived(focusOf(scene, ctx.selection()));
  const max = $derived(Number(ctx.params.max_sentences ?? 0));
  const lines = $derived.by(() => {
    const all = subject ? subject.facts.map((f) => sentence(ctx.vocab, subject, f)).filter((x): x is string => !!x) : [];
    return max > 0 ? all.slice(0, max) : all;
  });
  const story = $derived(subject && ctx.params.show_story !== false ? subject.facts.filter((f) => f.field.startsWith("story.") && typeof f.value === "string") : []);
  const kind = $derived(subject ? subject.facts.find((f) => f.field === "kind") : undefined);
</script>

<article class="ptext size-{String(ctx.params.size ?? "normal")}" aria-label="About {subject ? subjectName(subject) : 'nothing'}">
  {#if subject}
    {#if ctx.params.show_heading !== false}<h2>{subjectName(subject)}{#if kind}<span class="k">&nbsp;· {describeFact(ctx.vocab, kind).text}</span>{/if}</h2>{/if}
    <p>{lines.join(" ") || "Nothing more is known."}</p>
    {#each story as s (s.field)}<blockquote>{String(s.value)}</blockquote>{/each}
  {/if}
</article>

<style>
  .ptext {
    padding: 14px 18px;
    color: var(--sse-ink);
    text-shadow: var(--sse-text-glow);
    font-family: var(--sse-font-display);
    font-size: 17px;
    line-height: 1.55;
  }
  .size-small {
    font-size: 15px;
  }
  .size-large {
    font-size: 21px;
  }
  h2 {
    margin: 0 0 6px;
    font-family: var(--sse-title-font);
    font-style: var(--sse-title-style);
    text-transform: var(--sse-title-case);
    font-size: 22px;
    font-weight: 600;
  }
  .k {
    font-weight: 400;
    font-style: italic;
    color: var(--sse-ink-muted);
  }
  p {
    margin: 0;
    max-width: 62ch;
  }
  blockquote {
    margin: 10px 0 0;
    padding-left: 12px;
    border-left: var(--sse-line-heavy) solid var(--sse-accent);
    font-style: italic;
  }
</style>
