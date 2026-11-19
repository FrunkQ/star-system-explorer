<!-- Time ruler view: scene events plus the kernel's history of this system. -->
<script lang="ts">
  import type { HistoryEvent, Json, SystemScene, ViewContext } from "../../../ui-contract/src/index.ts";
  import { TimeRuler, type RulerEvent } from "../../../chart-grammar/src/index.ts";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  let history = $state<HistoryEvent[]>([]);

  $effect(() => {
    void ctx.workspace.revision;
    ctx.session.history(scene.system).then((h) => (history = h));
  });

  const events = $derived<RulerEvent[]>([
    ...(params.scene_events ? scene.events : []),
    ...(params.edits === false ? [] : history).map((e) => ({ id: e.id, at_yr: e.at_yr, label: `Authored edit (${e.subject.join(", ")})`, tone: "authored" as const })),
  ]);
</script>

<TimeRuler from={scene.span_yr[0]} to={scene.span_yr[1]} now={ctx.workspace.now_yr} {events} label="System time" onchange={(yr) => ctx.workspace.setTime(yr)} />
