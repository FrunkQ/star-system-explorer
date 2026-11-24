<!-- The table on the status line (Run desk and players' side, owned by the "Run desk and players' side" build): the
     players' link, what the table is shown (Send this view, Screens off), seats, and players' requests; with the
     Seats dialog it opens. -->
<script lang="ts">
  import SeatsPanel from "./SeatsPanel.svelte";
  import ArrivalSlip from "./ArrivalSlip.svelte";
  import OnAir from "./OnAir.svelte";
  import type { Share } from "../share.svelte.ts";
  import type { PartyShip, SceneSubject, Shown, ViewSession } from "../../../ui-contract/src/index.ts";

  let {
    share,
    session,
    system,
    title,
    subjects,
    seatBodies,
    scope,
    at_yr,
    landingUnsent,
    here,
    onswitchboard,
    chip = true,
    party,
    now_yr,
    onchange,
  }: {
    share: Share;
    session: ViewSession;
    /** The system in view. */
    system: string;
    title: string;
    subjects: readonly SceneSubject[];
    /** The bodies a seat's knowledge is listed for (the star and what orbits it). */
    seatBodies: SceneSubject[];
    scope: string;
    at_yr: number;
    /** A landing is in view that the table has not been sent. */
    landingUnsent: boolean;
    /** What "Send this view" sends: the place in view. */
    here: () => Shown;
    /** Open the Switchboard. */
    onswitchboard: () => void;
    /** Show the on-air chip here (false where the top bar has it). */
    chip?: boolean;
    /** The party's ship, for the offer to tell the table about a world it arrives at. */
    party: PartyShip | null;
    /** The campaign's now, in years. */
    now_yr: number;
    /** A change the table made (an approved request, a seat's grant): the physics moved. */
    onchange: () => void;
  } = $props();

  let seatsDialog: HTMLDialogElement | undefined = $state();
  let seatsOpen = $state(false);
</script>

<span class="share" aria-label="Players">
  {#if share.state.link}
    <!-- On air (PB2): in the top bar on the Run desk, here on the others. The one Send of the live view stays here. -->
    {@const shown = share.state.shown}
    {#if chip}<OnAir {share} {onswitchboard} />{/if}
    {#if "off" in shown || shown.scope !== system || (landingUnsent && !shown.landing)}<button type="button" class="linkbtn" data-send-view onclick={() => share.show(here())} title="Show every screen {title}, as far as each knows it">Send this view</button>{/if}
    <a class="quietlink" href={share.state.link} target="_blank" rel="noopener" data-players-link={share.state.link}>players' link</a>
    <button type="button" class="linkbtn quiet" onclick={() => ((seatsOpen = true), seatsDialog?.showModal())}>{share.state.plan.seats.length > 1 ? `${share.state.plan.seats.length} seats` : "More seats"}</button>
    <ArrivalSlip {session} {party} {subjects} {now_yr} {onchange} />
    {#if share.state.waiting.length}
      <span class="requests" role="group" aria-label="Players' requests">
        {#each share.state.waiting as r (r.from + r.id)}
          <span class="req" data-request={r.id}>
            {r.intent.kind === "set_value" ? `${r.intent.field} of ${subjects.find((x) => x.address === (r.intent as { address: string }).address)?.label ?? (r.intent as { address: string }).address} → ${JSON.stringify(r.intent.value)}` : r.intent.kind}
            <button type="button" class="linkbtn" onclick={async () => (await share.host.approve(r.id, r.from), onchange())}>Approve</button>
            <button type="button" class="linkbtn" onclick={() => share.host.decline(r.id, r.from)}>Decline</button>
          </span>
        {/each}
      </span>
    {/if}
  {:else}
    <button type="button" class="linkbtn" onclick={() => window.open(share.start(here()), "_blank", "noopener")}>Share with players</button>
  {/if}
</span>

<dialog bind:this={seatsDialog} class="help seats-dialog" aria-label="Seats" onclose={() => (seatsOpen = !!seatsDialog?.open)}>
  {#if seatsOpen}
    <SeatsPanel {share} {session} bodies={seatBodies} {scope} {at_yr} {onchange} {here} />
  {/if}
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

<style>
  .share {
    display: inline-flex;
    gap: 10px;
    align-items: center;
  }
  .requests {
    display: inline-flex;
    gap: 12px;
    color: var(--sse-ink);
  }
  .req {
    display: inline-flex;
    gap: 6px;
    padding: 2px 6px;
    border: var(--sse-line) solid var(--sse-accent);
    border-radius: var(--sse-radius);
  }
  .share a {
    font: inherit;
    color: var(--sse-ink);
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    padding: 0 8px;
    cursor: pointer;
  }
  .quietlink {
    color: var(--sse-ink-muted);
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
  .help {
    max-width: 600px;
    width: calc(100vw - 32px);
    box-sizing: border-box;
    border: var(--sse-line) solid var(--sse-ink);
    box-shadow: var(--sse-card-shadow);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    padding: 20px 24px;
  }
  .seats-dialog {
    max-width: 640px;
    width: calc(100vw - 32px);
  }
  .seats-dialog form[method="dialog"] {
    margin-top: 16px;
  }
  .quiet {
    color: var(--sse-ink-muted);
  }
  .help::backdrop {
    background: rgba(0, 0, 0, 0.25);
  }
  .help button {
    height: 40px;
    padding: 0 16px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ink);
    color: var(--sse-on-ink);
    font-family: var(--sse-font-body);
  }
</style>
