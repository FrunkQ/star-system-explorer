// Public surface of the client API. Shells and views import from here only.
// Deliberately absent: the kernel client and any kernel request shape.
export type { Json } from "./json.ts";
export { YEAR_S, INTENT_KINDS, checkIntent, yearsToSeconds, secondsToYears } from "./intent.ts";
export type { Intent, IntentKind, IntentSchema } from "./intent.ts";
export { decide, gmSeat, tableSeat, within } from "./seat.ts";
export type { Seat, AutoApprove, Decision } from "./seat.ts";
export type { ViewSession, IntentResult, HistoryEvent, DeviceEdge, DeriveRef, InputValue, Derived, Explanation, StackInfo, TrialResult } from "./session.ts";
export { CAMPAIGN_PACK, flowParams, editFlow, diffFlow, providerOf, campaignPack, packFor, checkRequest, flowColumns, paramText, siText } from "./physics.ts";
export { draftId, adopted, draftParam, candidateFor, previewRequest, adoptRequest, withdrawRequest, declineRequest, benchRef } from "./physics-edit.ts";
export type { Preview } from "./physics-edit.ts";
export type { FlowDoc, PackDoc, ParamEdit, ChangeRequest, RequestStatus } from "./physics.ts";
export { deriveScene, applyEdits, sig } from "./derivation.ts";
export type { DerivationDoc, DerivationRule, InputBinding, OutputBinding } from "./derivation.ts";
export { resolveParams } from "./view.ts";
export type { ViewModule, ViewContext, ViewHandle, ParamSpec, Tier, TopicQuery, Workspace, AppliedEdit } from "./view.ts";
export { commitEdit, undoEdit, redoEdit } from "./edit.ts";
export { checkScreen, LAYOUT_AREAS } from "./screen.ts";
export type { ScreenDoc, ScreenItem, CheckedScreen } from "./screen.ts";
export { markFor } from "./mark.ts";
export type { Mark, Binding, Symbology, Channel } from "./mark.ts";
export type { SystemScene, SceneSubject, SceneFact, SceneEvent, FactSource } from "./scene.ts";
