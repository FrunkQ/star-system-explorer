// Seats (H8.7) and the approval rule. The GM is the seat whose audience is
// all knowledge and whose intents apply at once (H8.8).
import type { SeatBrand } from "./brand.ts";
import { INTENT_KINDS, type Intent, type IntentKind } from "./intent.ts";

export interface AutoApprove {
  intent: IntentKind;
  /** Addresses (and everything inside them) where the intent applies without asking. */
  scope: string[];
  /** For set_value: only these fields apply without asking (absent: any field). */
  fields?: string[];
}

export interface Seat {
  id: string;
  label: string;
  /** Whose knowledge the seat shows; "all" for the GM, "everyone" for the default table. */
  audience: string;
  authority: "gm" | "player";
  may_send: IntentKind[];
  auto_approve?: AutoApprove[];
  sealed: boolean;
  /** Screen documents this seat opens (V10): one or several, each its own window or device. Absent: the default player screen. */
  screens?: string[];
  /** Data view the seat looks through instead of its audience's own (H8.7 `lens`). */
  lens?: string;
  /** Tabletops this seat's view may be shown inside (V3 gap 26, `?embed=1`): exact origins, no wildcards. */
  embed_origins?: string[];
  /** A faction's name and logo shown on this seat's screens (V3 gap 25). Presentation only. */
  brand?: SeatBrand;
  /**
   * Quick overrides (V3 gap 25): the GM's live switches for this seat's screens, set from the Seats panel and sent
   * at once ("hide labels" on a wall screen). Presentation only, never saved with the campaign: player view
   * parameters by name, laid over every screen this seat shows.
   */
  live?: Record<string, boolean>;
}

export type Decision = { action: "apply" } | { action: "ask_gm" } | { action: "refuse"; reason: string };

export function gmSeat(): Seat {
  return { id: "gm", label: "Game master", audience: "all", authority: "gm", may_send: [...INTENT_KINDS], sealed: false };
}

export function tableSeat(): Seat {
  return { id: "table", label: "Table", audience: "everyone", authority: "player", may_send: [], sealed: false };
}

/** The table in a shared session (V4/V5): sees "everyone", and may ask the GM to set a value (never applied without approval). */
export function sharedTableSeat(): Seat {
  return { ...tableSeat(), may_send: ["set_value"] };
}

/** `inner` is `outer` or lies inside it (address segments, not string prefixes). */
export function within(inner: string, outer: string): boolean {
  return inner === outer || inner.startsWith(outer + "/");
}

export function decide(seat: Seat, intent: Intent): Decision {
  if (!seat.may_send.includes(intent.kind)) return { action: "refuse", reason: `seat ${seat.label} may not send ${intent.kind}` };
  if (seat.authority === "gm") return { action: "apply" };
  // What an audience knows is the GM's to grant (V10): a player seat never sets disclosure, even with auto-approve.
  if (intent.kind === "set_value" && (intent.field === "observe" || intent.field.startsWith("observe."))) return { action: "refuse", reason: "what players know is set by the game master" };
  // Physics, pack and disclosure changes are never auto-approved: they go to the GM (H8.8).
  if (intent.kind === "adopt_physics" || intent.kind === "stack_packs" || intent.kind === "scan_toggle" || intent.kind === "grant") return { action: "ask_gm" };
  // Placing and withdrawing bodies is the GM's (paste and cut on the Build desk): never auto-approved.
  if (intent.kind === "set_values" || intent.kind === "withdraw" || intent.kind === "restore") return { action: "ask_gm" };
  // What things are called is the GM's (NM2): a player's rename goes to the GM.
  if (intent.kind === "rename" || intent.kind === "set_naming") return { action: "ask_gm" };
  const ok = (seat.auto_approve ?? []).some(
    (a) => a.intent === intent.kind && a.scope.some((s) => within(intent.address, s)) && (!a.fields || (intent.kind === "set_value" && a.fields.includes(intent.field))),
  );
  return ok ? { action: "apply" } : { action: "ask_gm" };
}

/** The kernel audience a seat sees as. */
export function audienceOf(seat: Pick<Seat, "audience" | "authority">): string {
  return seat.authority === "gm" && seat.audience === "all" ? "gm" : seat.audience;
}
