// Public surface of the client API. Shells and views import from here only.
// Deliberately absent: the kernel client and any kernel request shape.
export type { Json } from "./json.ts";
export { YEAR_S, INTENT_KINDS, checkIntent, yearsToSeconds, secondsToYears } from "./intent.ts";
export type { Intent, IntentKind, IntentSchema } from "./intent.ts";
export { decide, gmSeat, tableSeat, within } from "./seat.ts";
export type { Seat, AutoApprove, Decision } from "./seat.ts";
export type { ViewSession, IntentResult, HistoryEvent, DeviceEdge } from "./session.ts";
export { resolveParams } from "./view.ts";
export type { ViewModule, ViewContext, ViewHandle, ParamSpec, Tier, TopicQuery, Workspace } from "./view.ts";
export { checkScreen, LAYOUT_AREAS } from "./screen.ts";
export type { ScreenDoc, ScreenItem, CheckedScreen } from "./screen.ts";
export { markFor } from "./mark.ts";
export type { Mark, Binding, Symbology, Channel } from "./mark.ts";
export type { SystemScene, SceneSubject, SceneFact, SceneEvent, FactSource } from "./scene.ts";
