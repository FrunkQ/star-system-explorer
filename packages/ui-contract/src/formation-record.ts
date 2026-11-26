/**
 * How a system on the chart was made (kernel `formation_record`, the Formation screen F10). A grown system's run is
 * re-run from the request its adoption kept; an imported or authored system has none, and the reason is given in
 * words. When the pipeline has changed since the system was adopted, the re-run is labelled, because the history it
 * shows may then differ from the one the campaign's facts came from.
 */
import type { FormationRan, FormationRequest } from "./make.ts";

/** The kernel's answer. */
export interface FormationRecordRead {
  type: "formation_record";
  system: string;
  source: "formation" | "v3" | "facts" | string;
  request?: FormationRequest;
  adopted_manifest?: string;
  record?: FormationRan;
  why?: string;
}

/** What the Formation screen has to show for a system. */
export type FormationSource =
  | { kind: "run"; run: FormationRan; seed: string; system: string; drift: boolean }
  | { kind: "none"; system: string; source: string; why: string }
  | { kind: "failed"; system: string; why: string };

interface RecordReader {
  formationRecord?(system: string): Promise<{ ok: true; result: FormationRecordRead } | { ok: false; code: string; message: string }>;
}

/** Read how `system` was made. Never throws: a failure is said in words. */
export async function formationOf(session: RecordReader, system: string): Promise<FormationSource> {
  if (!session.formationRecord) return { kind: "failed", system, why: "This screen cannot read the universe here (players' screens have no formation history)." };
  const r = await session.formationRecord(system);
  if (!r.ok) return { kind: "failed", system, why: r.message };
  const x = r.result;
  if (!x.record) {
    const why = x.why ?? "This system has no formation history.";
    return { kind: "none", system, source: x.source, why: why.charAt(0).toUpperCase() + why.slice(1) + "." };
  }
  const now = (x.record.run.history as { manifest_hash?: string }).manifest_hash;
  return { kind: "run", run: x.record, seed: x.request?.seed ?? "", system, drift: !!x.adopted_manifest && !!now && x.adopted_manifest !== now };
}
