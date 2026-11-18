<!-- A region filled with hatching: uncertainty, unresolved clouds, unsurveyed territory. -->
<script lang="ts">
  let {
    prefix,
    d,
    pattern = "hatch",
    opacity = 1,
  }: { prefix: string; d: string; pattern?: "hatch" | "cross" | "stipple" | "band"; opacity?: number } = $props();
</script>

<path {d} fill="url(#{prefix}-{pattern})" fill-rule="evenodd" fill-opacity={opacity} aria-hidden="true" />
