<!-- The Switchboard as a view (PB2): it reads the table from the workspace the shell gives every view. -->
<script lang="ts">
  import Switchboard from "./Switchboard.svelte";
  import type { ViewContext } from "../../../ui-contract/src/index.ts";

  let { ctx }: { ctx: ViewContext } = $props();
  const table = $derived(ctx.workspace.table);
</script>

{#if table}
  <Switchboard {table} ws={ctx.workspace} session={ctx.session} />
{:else}
  <p class="wait" data-switchboard-waiting>The table opens with the campaign; one moment.</p>
{/if}

<style>
  .wait {
    padding: 24px;
    color: var(--sse-ink-muted);
  }
</style>
