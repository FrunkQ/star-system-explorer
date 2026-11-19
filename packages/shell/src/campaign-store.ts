// Where the chart room keeps the open campaign between visits (U10): one
// save in this browser's local storage, written after every change, plus the
// same save as a file the GM downloads or opens. Storage can be absent or
// full (private windows, quotas): every call says what happened instead of
// throwing, and the campaign always opens, fresh if it has to.
import { checkCampaignSave, type CampaignSave } from "../../ui-contract/src/index.ts";

export const CAMPAIGN_KEY = "sse4.campaign";
/** A save that could not be reopened is set aside here, never thrown away. */
export const SET_ASIDE_KEY = "sse4.campaign.set-aside";

export function readStored(root: string): { save: CampaignSave | null; problem: string | null } {
  let raw: string | null = null;
  try {
    raw = localStorage.getItem(CAMPAIGN_KEY);
  } catch {
    return { save: null, problem: null };
  }
  if (!raw) return { save: null, problem: null };
  try {
    const doc = JSON.parse(raw) as unknown;
    const bad = checkCampaignSave(doc, root);
    return bad.length ? { save: null, problem: bad.join("; ") } : { save: doc as CampaignSave, problem: null };
  } catch {
    return { save: null, problem: "the kept save is not readable" };
  }
}

/** Keep a save; null when kept, else a sentence on why not. */
export function writeStored(save: CampaignSave): string | null {
  try {
    localStorage.setItem(CAMPAIGN_KEY, JSON.stringify(save));
    return null;
  } catch (e) {
    return /quota/i.test(String(e)) ? "this browser's storage is full" : "this browser does not allow storage here";
  }
}

/** Move the kept save aside (it failed to reopen), so the next visit starts fresh but nothing is lost. */
export function setAside(): void {
  try {
    const raw = localStorage.getItem(CAMPAIGN_KEY);
    if (raw) localStorage.setItem(SET_ASIDE_KEY, raw);
    localStorage.removeItem(CAMPAIGN_KEY);
  } catch {
    // Nothing kept, nothing to move.
  }
}

export function clearStored(): void {
  try {
    localStorage.removeItem(CAMPAIGN_KEY);
  } catch {
    // Nothing kept.
  }
}

/** Offer a save as a file download. */
export function download(save: CampaignSave, name: string): void {
  const url = URL.createObjectURL(new Blob([JSON.stringify(save)], { type: "application/json" }));
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
