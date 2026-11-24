// Keeping a campaign in the browser (U10). A save is the kernel's own export
// of the open universe (`export_chronicle`, the authority) plus the GM's
// workspace overlay that is not in the kernel yet: values applied to the
// fixture scene (until the kernel builds that scene), the undo and redo
// stacks, and the ruler's time. Reopening imports the export, so the state
// hash after reopening is the hash that was saved.
import type { Json } from "./json.ts";
import type { AppliedEdit } from "./view.ts";

export const CAMPAIGN_SCHEMA = "sse.campaign-save/1";

export interface CampaignSave {
  schema: typeof CAMPAIGN_SCHEMA;
  /** When it was kept, ISO 8601 (the device's clock; presentation only). */
  saved_at: string;
  /** The campaign's title as the GM saw it. */
  title: string;
  /** The universe it belongs to (`u:<id>`), checked against the export's root. */
  root: string;
  /** State hash when kept: reopening must give the same. */
  state_hash: string;
  /** `sse.chronicle-export/1`, kept exactly as the kernel gave it. */
  export: Json;
  workspace: {
    applied: Record<string, { value: Json; display: string; event_id: string }>;
    undo: AppliedEdit[];
    redo: AppliedEdit[];
    now_yr: number;
    /** U11: the campaign's now and where its calendar starts (older saves have neither; they open at now_yr). */
    campaign_now_yr?: number;
    campaign_epoch_yr?: number;
    /** Where the GM put each pack overlay (overlay.ts), by overlay id (older saves have none). */
    overlay_anchors?: Record<string, import("./overlay.ts").OverlayAnchor>;
    /** The party's ship (older saves have none). */
    party?: import("./party.ts").PartyShip | null;
  };
  /**
   * The packs over the opening stack (Packs screen) and the campaign's own
   * physics pack: the kernel keeps the stack apart from the chronicle, so the
   * save carries it and reopening loads it before the import.
   */
  stack?: { added: Json[]; campaign: Json | null };
}

/** Problems with a document offered as a campaign save, as sentences; empty when it can be reopened. */
export function checkCampaignSave(doc: unknown, root?: string): string[] {
  const d = doc as Partial<CampaignSave> | null;
  if (!d || typeof d !== "object") return ["not a campaign save"];
  if (d.schema !== CAMPAIGN_SCHEMA) return [`not a campaign save (expected ${CAMPAIGN_SCHEMA})`];
  const out: string[] = [];
  const ex = d.export as { format?: string; root?: string; events?: unknown[] } | undefined;
  if (!ex || typeof ex !== "object" || !Array.isArray(ex.events)) out.push("it holds no chronicle export");
  else if (ex.root !== d.root) out.push("its chronicle belongs to another universe than it says");
  if (root && d.root !== root) out.push(`it belongs to ${d.root}, not this campaign (${root})`);
  if (!d.workspace || typeof d.workspace !== "object" || !Array.isArray(d.workspace.undo) || !Array.isArray(d.workspace.redo)) out.push("its workspace is missing");
  if (typeof d.state_hash !== "string") out.push("its state hash is missing");
  if (d.stack !== undefined && (!d.stack || !Array.isArray(d.stack.added))) out.push("its pack stack is not readable");
  return out;
}

export function makeCampaignSave(x: { title: string; root: string; state_hash: string; export: Json; workspace: CampaignSave["workspace"]; saved_at: string; stack?: CampaignSave["stack"] }): CampaignSave {
  return { schema: CAMPAIGN_SCHEMA, saved_at: x.saved_at, title: x.title, root: x.root, state_hash: x.state_hash, export: x.export, workspace: x.workspace, ...(x.stack ? { stack: x.stack } : {}) };
}

/** How many events a save holds, for the words beside it. */
export function savedEvents(s: CampaignSave): number {
  const ex = s.export as { events?: unknown[] };
  return Array.isArray(ex.events) ? ex.events.length : 0;
}

/** A file name for a downloaded save: "sse-campaign-kestrel-reach-2026-11-19.json". */
export function saveFileName(title: string, saved_at: string): string {
  const slug = title.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "campaign";
  return `sse-campaign-${slug}-${saved_at.slice(0, 10)}.json`;
}

/**
 * The part of an export that may travel to players' screens: the universe's
 * events only. The campaign record (GM notes, kept values, actors: J1, design
 * 8A) rides in the same export as `record` and `record_head` and must never
 * go on the shared manifest; seats get their share through disclosures on
 * their own channel. The universe's state hash does not depend on the record.
 */
export function publicExport(ex: Json): Json {
  if (!ex || typeof ex !== "object" || Array.isArray(ex)) return ex;
  const out: Record<string, Json> = {};
  for (const [k, v] of Object.entries(ex as Record<string, Json>)) if (k !== "record" && k !== "record_head") out[k] = v;
  return out;
}
