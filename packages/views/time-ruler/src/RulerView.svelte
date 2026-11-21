<!-- Time ruler view: scene events plus the kernel's history of this system, and the campaign's clock (U11). -->
<script lang="ts">
  import { KIND_WORDS, type HistoryEvent, type Json, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { TimeRuler, defaultCalendar, fromPackCalendar, type RulerEvent } from "../../../chart-grammar/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  let history = $state<HistoryEvent[]>([]);

  $effect(() => {
    void ctx.workspace.revision;
    ctx.session.history(scene.system).then((h) => (history = h));
  });

  const events = $derived<RulerEvent[]>([
    ...(params.scene_events ? scene.events : []),
    ...(params.edits === false ? [] : history).flatMap((e): RulerEvent[] => {
      // A kept look-ahead's named transitions read as themselves; its epoch steps and bookkeeping stay off the ruler.
      if (e.kind === "evolve.transition" || e.kind === "commit.branch") return [];
      if (e.kind === "time.request") return [{ id: e.id, at_yr: e.at_yr, label: "Look-ahead kept", tone: "authored" }];
      if (e.kind.startsWith("time.")) {
        const k = e.kind.slice(5);
        const who = e.subject.map(name).filter((n) => n !== scene.title);
        return [{ id: e.id, at_yr: e.at_yr, label: `${KIND_WORDS[k] ?? k}${who.length && (k === "engulfed" || k === "oceans_lost") ? `: ${who.join(", ")}` : ""}`, tone: e.at_yr > ctx.workspace.now_yr ? "future" : "past" }];
      }
      return [{ id: e.id, at_yr: e.at_yr, label: `Authored edit (${e.subject.map(name).join(", ")})`, tone: "authored" }];
    }),
  ]);
  function name(address: string): string {
    return scene.subjects.find((s) => s.address === address)?.label ?? address;
  }
</script>

<TimeRuler from={scene.span_yr[0]} to={scene.span_yr[1]} now={ctx.workspace.now_yr} campaignNow={params.campaign_clock === false ? undefined : ctx.workspace.campaign_now_yr} calendar={ctx.workspace.pack_calendar ? fromPackCalendar(ctx.workspace.pack_calendar) : defaultCalendar(ctx.workspace.campaign_epoch_yr ?? ctx.workspace.campaign_now_yr ?? ctx.workspace.now_yr)} {events} label="System time" onchange={(yr) => ctx.workspace.setTime(yr)} onmakenow={(yr) => ctx.workspace.setCampaignNow?.(yr)} />
