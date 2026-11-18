// Seats (H8.7) and the approval rule. The GM is the seat whose audience is
// all knowledge and whose intents apply at once (H8.8).
import { INTENT_KINDS, type Intent, type IntentKind } from "./intent.ts";

export interface AutoApprove {
  intent: IntentKind;
  /** Addresses (and everything inside them) where the intent applies without asking. */
  scope: string[];
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
}

export type Decision = { action: "apply" } | { action: "ask_gm" } | { action: "refuse"; reason: string };

export function gmSeat(): Seat {
  return { id: "gm", label: "Game master", audience: "all", authority: "gm", may_send: [...INTENT_KINDS], sealed: false };
}

export function tableSeat(): Seat {
  return { id: "table", label: "Table", audience: "everyone", authority: "player", may_send: [], sealed: false };
}

/** `inner` is `outer` or lies inside it (address segments, not string prefixes). */
export function within(inner: string, outer: string): boolean {
  return inner === outer || inner.startsWith(outer + "/");
}

export function decide(seat: Seat, intent: Intent): Decision {
  if (!seat.may_send.includes(intent.kind)) return { action: "refuse", reason: `seat ${seat.label} may not send ${intent.kind}` };
  if (seat.authority === "gm") return { action: "apply" };
  // Physics changes are never auto-approved: they go to the GM (H8.8).
  if (intent.kind === "adopt_physics") return { action: "ask_gm" };
  const ok = (seat.auto_approve ?? []).some((a) => a.intent === intent.kind && a.scope.some((s) => within(intent.address, s)));
  return ok ? { action: "apply" } : { action: "ask_gm" };
}
