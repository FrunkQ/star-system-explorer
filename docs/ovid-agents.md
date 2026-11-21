# Ovid Next: what SSE4's agents reuse

Status: K0 draft for Alex's review. Source read: Ovid Next `README.md`, `LICENSING.md`,
`docs/guide/how-ovid-works.md`, and the design notes `2026-10-23-naso.md`, `2026-10-25-models-in-the-page.md`,
`2026-10-26-best-model-for-each-job.md`, `2026-10-29-tech-support.md`. The browser source under `browser/src/models/`,
`agent/builder.ts` and `play/support-*.ts` was read in A1 part 2 (19 Nov); what was lifted is listed in
`handover/packages/A1.md`, part 2, and in `NOTICE`.

## 1. Licence first

Ovid is source-available under BSL 1.1. SSE4 is Apache-2.0. Alex owns all three codebases and has said credit is all
that is needed, so lifting is allowed, but each lifted file must be relicensed by Alex as Apache-2.0 at the point it
enters this repo. A1's commit says so in its message and adds a line to `NOTICE`. No Ovid file is copied before then.

## 2. Reuse, by SSE4 agent package

| Ovid piece | What it does in Ovid | SSE4 use | Package |
|---|---|---|---|
| Link LLMs (Settings): LM Studio and Ollama found on the PC, or one OpenRouter key; "Another provider" by `/v1` address | Bring-your-own-key provider set-up | Same set-up screen and the same three provider kinds. Agents are optional; the app is fully usable with no key. | A1 |
| Jobs (`models/links.ts`): each block asks for a job (writing, deciding, building), each job has one linked model | Decouples what is asked from which model answers | SSE4 jobs: Building (Orac), Describing (Minerva), Support (Tars), Deciding (small, cheap checks). | A1 |
| Keys stay on the main PC; other devices borrow the model, never the key | Key handling | Keys never leave the GM's browser. Player devices never call a provider; Minerva's text reaches them as data. | A1 |
| `models/picks.json`: measured recommendations per job, shipped in the page, works offline | Model choice with evidence | A shipped picks file per SSE4 job. Rows only for models we measured; "not measured yet" otherwise. | A1 |
| `models/ledger.ts` and `models/estimate.ts`: usage ledger and cost estimate before a run | Cost honesty | Every agent action shows its estimate first and its cost after. Daily cap, as Ovid's tech support has. | A1 |
| `models/loaded-context.ts`: never send more than the model was loaded with | Avoids a class of local-model failures | Same guard. | A1 |
| Naso's loop: plan first, draft, check, fix up to three times, open as an unsaved draft, never save or overwrite a built-in, never sees a key | Flow-building agent | Orac's loop, unchanged in shape. Orac's drafts are change requests on flows, boxes, packs, screens, data views, sifters, programs and class tables, validated by the kernel (K4 validation reports every problem at once, which suits the fix loop). A draft runs on a chronicle branch (K3) and is committed only by the GM. | A2 |
| "Says which block is missing, in the exact form the design thread asks for" | Gap reporting | Orac writes a missing-atom issue in the issue-spec-test-code form (P5). | A2 |
| Naso's Bibliotheca: keyed entries of about 800 characters, retrieved by their keys (`how-ovid-works.md`) | Agent memory of the parts, sized for small models | SSE4 needs the same for atoms, packs and views. The explanation bundles' three lenses (K4) are the source; each atom gets keyed short entries generated from them, so the agent reads what the kernel says, not a copy. | A2, A4 |
| Large library for strong models, a trimmed one for small local models | Graceful degradation | Same: full catalogue for Opus-class, keyed entries only for small models. | A2 |
| Tech support: rules first, model last; reads the shape of a failed call, never its words; a fixed list of fixes; nothing healed in silence; a real bug becomes a GitHub issue the person reads whole and sends | Support agent | Tars follows the same contract. Tars reads run manifests, validation reports and harness output (all shape, no story text) and answers from the docs and explanation bundles. Bug reports go out only by the GM's own Submit. Add the humour setting Alex asked for. | A4 |
| Check this PC (`models-in-the-page.md` section 4): measures where each job should run | Device capability | Already done differently in SSE4: the kernel's capability tiers (K6) measure work units per second. Tars can explain a device's tier using that measurement. | A4 |

## 3. Not reused

| Ovid piece | Why not |
|---|---|
| Ovid Code's Python service, receipts, store and six operations | A coding scaffold for building Ovid; SSE4 uses `handover/` and package logs instead. |
| Metamorphosis window and screen blocks | SSE4's UI is the chart-room GM workspace (U-packages); Orac's output appears there. |
| Story memory and Intimate or Private guards | SSE4 has no story memory; Minerva works from sifter hits and explanation bundles (A3), and the data view decides what a player may read. |
| Voices (Kokoro) | Not asked for. Could be a later Minerva option. |

## 4. Minerva (A3), which has no single Ovid counterpart

Minerva describes and tells stories. From Ovid she takes the Describing job, the cost ledger and the rule that output
is a draft the GM accepts. Her inputs are SSE4's own: sifter hits (K8), explanation bundles (K4) and the active player
data view (K9), so she never writes a fact the audience may not see. V3's AI context fields (`seedText`, `style`,
`length`) carry over as her prompt notes (see `docs/v3-save-fields.md`).

## 5. For Alex to confirm

1. Relicensing each lifted Ovid file to Apache-2.0 at A1, with a `NOTICE` line. Confirmed by Alex, 21 Nov 2026.
2. A fourth job, Deciding, for cheap checks (for example "does this story hit fit this audience"). Recommend yes; it
   keeps Minerva's and Orac's expensive models for the work that needs them.
