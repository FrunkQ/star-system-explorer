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
export type { StylesHost, ViewModule, ViewContext, PlayerContext, ViewHandle, ParamSpec, Tier, TopicQuery, Workspace, AppliedEdit } from "./view.ts";
export { commitEdit, undoEdit, redoEdit } from "./edit.ts";
export { checkScreen, boundScene, LAYOUT_AREAS } from "./screen.ts";
export { resolveStyle, checkStyle, settingCost, settingsCost, promotedOf, settingsDiff, COSTS, TIER_RUNS_UP_TO } from "./style.ts";
export type { StyleDoc, StyleCheck } from "./style.ts";
export type { ScreenDoc, ScreenItem, CheckedScreen } from "./screen.ts";
export { subjectOf, playerSubjectOf } from "./mark.ts";
export { resolveSymbology, markerFor, legendFor, sameLook, checkSymbology, CHANNELS, PATTERNS, BODY_SHAPES } from "./symbology.ts";
export type { Mark, Binding, Channel, Shape, Pattern, Scale, SymbologyDoc, Marker, LegendEntry, MarkSubject, ThemeCapability, SymbologyHost, TagColour } from "./symbology.ts";
export { parseV3Palette, symbologyFromV3Palette, coloursFromV3Save, withV3TagColours, objectColourSummary, campaignCopy, v3SaveText } from "./v3-colours.ts";
export type { V3TokenMap, V3ImportLine, V3PaletteImport, V3ObjectColour, V3SaveColours } from "./v3-colours.ts";
export type { SystemScene, SceneSubject, SceneFact, SceneEvent, FactSource } from "./scene.ts";
export { describeFact, wordsFor, fact, subjectName, parentOf, sentence, checkPlayerScene } from "./player-scene.ts";
export type { PlayerScene, PlayerSubject, PlayerFact, Vocab, FieldWords, FactText } from "./player-scene.ts";
export { packSummary, previewStack, stackPacks, checkPackDoc } from "./packs-edit.ts";
export type { PackSummary, ValueChange, StackPreview } from "./packs-edit.ts";
export { observeOf, checkSetup, playerDataPack, setupChanges, applySetup, enterFacts, ladderOf, PLAYER_DATA_PACK, DEFAULT_LADDER } from "./data-views.ts";
export type { DataView, ObserveSetup, Override, TagMode, UnknownStyle, AuthoredFacts } from "./data-views.ts";
export { showReference } from "./reference.ts";
export type { ReferenceIn, ShownReference } from "./reference.ts";
export { GmHost, PlayerReplica, memoryRoom, broadcastHub, relayHub, multiHub, P2P_PROTOCOL } from "./p2p.ts";
export type { Hub, Wire, Manifest, LogEntry, HostState, ReplicaStatus } from "./p2p.ts";
// The session class as a type only (shells hold the one host-browser.ts opened; they cannot make one).
export type { Session } from "./session.ts";
