// Public surface of the client API. Shells and views import from here only.
// Deliberately absent: the kernel client and any kernel request shape.
export type { Json } from "./json.ts";
export { YEAR_S, INTENT_KINDS, checkIntent, yearsToSeconds, secondsToYears } from "./intent.ts";
export type { Intent, IntentKind, IntentSchema } from "./intent.ts";
export { audienceOf, decide, gmSeat, tableSeat, within } from "./seat.ts";
export type { Seat, AutoApprove, Decision } from "./seat.ts";
export type { ViewSession, IntentResult, HistoryEvent, DeviceEdge, DeriveRef, InputValue, Derived, Explanation, StackInfo, TrialResult, StackCandidate, Probe, SceneOptions } from "./session.ts";
export { CAMPAIGN_PACK, flowParams, editFlow, diffFlow, providerOf, campaignPack, packFor, checkRequest, flowColumns, paramText, siText } from "./physics.ts";
export { draftId, adopted, draftParam, candidateFor, previewRequest, adoptRequest, withdrawRequest, declineRequest, benchRef } from "./physics-edit.ts";
export type { Preview } from "./physics-edit.ts";
export type { FlowDoc, PackDoc, ParamEdit, ChangeRequest, RequestStatus } from "./physics.ts";
export { deriveScene, applyEdits, sig } from "./derivation.ts";
export type { DerivationDoc, DerivationRule, InputBinding, OutputBinding } from "./derivation.ts";
export { resolveParams } from "./view.ts";
export type { ViewModule, ViewContext, PlayerContext, ViewHandle, ParamSpec, Tier, TopicQuery, Workspace, AppliedEdit } from "./view.ts";
export { commitEdit, undoEdit, redoEdit } from "./edit.ts";
export { checkScreen, boundScene, LAYOUT_AREAS } from "./screen.ts";
export type { ScreenDoc, ScreenItem, CheckedScreen } from "./screen.ts";
export { subjectOf, playerSubjectOf } from "./mark.ts";
export { resolveSymbology, markerFor, legendFor, sameLook, checkSymbology, CHANNELS, PATTERNS, BODY_SHAPES } from "./symbology.ts";
export type { Mark, Binding, Channel, Shape, Pattern, Scale, SymbologyDoc, Marker, LegendEntry, MarkSubject, ThemeCapability, SymbologyHost } from "./symbology.ts";
export type { SystemScene, SceneSubject, SceneFact, SceneEvent, FactSource } from "./scene.ts";
export { describeFact, wordsFor, fact, subjectName, parentOf, sentence, checkPlayerScene } from "./player-scene.ts";
export type { PlayerScene, PlayerSubject, PlayerFact, Vocab, FieldWords, FactText } from "./player-scene.ts";
export { packSummary, previewStack, stackPacks, checkPackDoc } from "./packs-edit.ts";
export type { PackSummary, ValueChange, StackPreview } from "./packs-edit.ts";
export { observeOf, checkSetup, playerDataPack, setupChanges, applySetup, enterFacts, ladderOf, PLAYER_DATA_PACK, DEFAULT_LADDER } from "./data-views.ts";
export type { DataView, ObserveSetup, Override, TagMode, UnknownStyle, AuthoredFacts } from "./data-views.ts";
