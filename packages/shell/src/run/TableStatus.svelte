<!-- The table on the status line (Run desk and players' side, owned by the "Run desk and players' side" build): the
     on-air chip off the Run desk, Send this view, the players' link and the arrival offer. Roles, their links and
     players' requests live on the Switchboard's role cards (PB7); the Seats dialog is gone. -->
<script lang="ts">
  import ArrivalSlip from "./ArrivalSlip.svelte";
  import OnAir from "./OnAir.svelte";
  import { noteSent } from "./fired.svelte.ts";
  import type { Share } from "../share.svelte.ts";
  import type { PartyShip, SceneSubject, Shown, ViewSession } from "../../../ui-contract/src/index.ts";

  let {
    share,
    session,
    system,
    title,
    subjects,
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

</script>

<span class="share" aria-label="Players">
  {#if share.state.link}
    <!-- On air (PB2): in the top bar on the Run desk, here on the others. The one Send of the live view stays here. -->
    {@const shown = share.state.shown}
    {#if chip}<OnAir {share} {onswitchboard} />{/if}
    {#if "off" in shown || shown.scope !== system || (landingUnsent && !shown.landing)}<button type="button" class="linkbtn" data-send-view onclick={() => { const h = here(); noteSent(h, "everyone"); share.show(h); }} title="Show every screen {title}, as far as each knows it">Send this view</button>{/if}
    <a class="quietlink" href={share.state.link} target="_blank" rel="noopener" data-players-link={share.state.link}>players' link</a>
    <button type="button" class="linkbtn quiet" data-roles onclick={onswitchboard} title="Roles, their links and requests are on the Switchboard">{share.state.plan.seats.length > 1 ? `${share.state.plan.seats.length} roles` : "Roles"}</button>
    <ArrivalSlip {session} {party} {subjects} {now_yr} {onchange} />
  {:else}
    <button type="button" class="linkbtn" onclick={() => window.open(share.start(here()), "_blank", "noopener")}>Share with players</button>
  {/if}
</span>


<style>
  .share {
    display: inline-flex;
    gap: 10px;
    align-items: center;
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
  .quiet {
    color: var(--sse-ink-muted);
  }
</style>
