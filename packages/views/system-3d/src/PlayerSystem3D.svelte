<!--
  System 3D for a player screen: the bodies the audience has a distance for,
  on rings about their parent; bodies known only to exist are named in the
  note, never placed by guess. Players are not given phases, so bodies are
  spread round their rings for display (said in the note).
-->
<script lang="ts">
  import { subjectName, type PlayerContext, type PlayerScene } from "../../../ui-contract/src/index.ts";
  import System3D from "./System3D.svelte";
  import { fromPlayerScene } from "./bodies.ts";

  let { ctx, scene }: { ctx: PlayerContext; scene: PlayerScene } = $props();
  const bodies = $derived(fromPlayerScene(scene, String(ctx.params.distance_field ?? "orbit.semi_major_axis_m"), ctx.mark));
  const star = $derived(scene.subjects.find((s) => s.address === scene.scope));
</script>

<System3D
  {bodies}
  selection={ctx.selection()}
  onselect={(id) => ctx.select(id)}
  title={star ? subjectName(star) : "the system"}
  tier={String(ctx.params.tier ?? "auto")}
  labels={ctx.params.labels !== false}
  distanceNote=", positions round each ring spread for display"
/>
