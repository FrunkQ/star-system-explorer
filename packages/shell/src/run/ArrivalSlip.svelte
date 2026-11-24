<!-- Reveal on arrival (landing run-through, gap 6): when the party's ship comes to a world the table does not fully
     know, one slip on the table's line offers to tell them. Nothing is revealed by itself; the press is one set_known
     at the campaign's now, with Undo; Not now drops the offer. -->
<script lang="ts">
  import { arrivalOffer, arrivalReveal, tableOf, yearsToSeconds, type ArrivalOffer, type PartyShip, type SceneSubject, type ViewSession } from "../../../ui-contract/src/index.ts";

  let {
    session,
    party,
    subjects,
    now_yr,
    onchange,
  }: {
    session: ViewSession;
    party: PartyShip | null;
    subjects: readonly SceneSubject[];
    /** The campaign's now, in years: reads and the reveal are made there. */
    now_yr: number;
    /** The reveal was made or undone: what the table knows moved. */
    onchange: () => void;
  } = $props();

  let offer = $state<ArrivalOffer | null>(null);
  let undo = $state<string | null>(null);
  let told = $state("");
  let why = $state("");
  // The party as last seen, so only a move makes an offer (opening a campaign with the ship already there does not).
  let before: PartyShip | null | undefined = undefined;
  const at = () => ({ at_yr: now_yr, at_s: String(yearsToSeconds(now_yr)) });

  $effect(() => {
    const p = party ? ($state.snapshot(party) as PartyShip) : null;
    const was = before;
    before = p;
    if (was === undefined || !p) return;
    const name = subjects.find((s) => s.address === p.at)?.label ?? "";
    let live = true;
    void session.knownTo(p.at, undefined, BigInt(at().at_s)).then((r) => {
      if (!live) return;
      const o = arrivalOffer(was, p, name, r.ok ? tableOf(r.value) : null);
      if (o) ((offer = o), (undo = null), (told = ""), (why = ""));
    });
    return () => (live = false);
  });

  async function tell() {
    if (!offer) return;
    const r = await session.send(arrivalReveal(offer, "everyone", at()));
    if (r.status !== "applied") return void (why = r.status === "refused" ? r.reason : r.status === "error" ? r.message : "sent to the GM");
    told = `The table knows ${subjects.find((s) => s.address === offer!.world)?.label ?? "it"} now.`;
    undo = r.event_id ?? null;
    offer = null;
    onchange();
  }
  async function takeBack() {
    if (!undo) return;
    const r = await session.send({ kind: "undo_known", undo, ...at() });
    if (r.status === "applied") ((undo = null), (told = ""), onchange());
  }
</script>

{#if offer}
  <span class="slip" role="status" data-arrival={offer.world}>
    {offer.line}
    <button type="button" class="linkbtn" data-arrival-tell onclick={tell}>{offer.action}</button>
    <button type="button" class="linkbtn quiet" onclick={() => (offer = null)}>Not now</button>
    {#if why}<span class="why">{why}</span>{/if}
  </span>
{:else if told}
  <span class="slip" role="status" data-arrival-told>
    {told}
    {#if undo}<button type="button" class="linkbtn" data-arrival-undo onclick={takeBack}>Undo</button>{/if}
    <button type="button" class="linkbtn quiet" onclick={() => ((told = ""), (undo = null))} aria-label="Dismiss">×</button>
  </span>
{/if}

<style>
  .slip {
    display: inline-flex;
    gap: 8px;
    align-items: baseline;
    padding: 0 8px;
    border-left: 2px solid var(--sse-accent);
    animation: slip-in var(--sse-motion-quick) var(--sse-ease);
  }
  .linkbtn {
    color: var(--sse-accent);
    font: inherit;
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    text-decoration: underline;
  }
  .linkbtn.quiet {
    color: var(--sse-ink-muted);
  }
  .why {
    color: var(--sse-ink-muted);
  }
  @keyframes slip-in {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .slip {
      animation: none;
    }
  }
</style>
