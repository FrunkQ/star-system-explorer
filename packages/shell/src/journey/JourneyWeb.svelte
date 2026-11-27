<!--
  The cosmic web on a journey (grand zoom GZ8, first part): the web cube's own view (Web3D, the same reads its screen
  makes), read centred on home and aimed by the journey. Home is found in the cube as the same catalogued halo moved
  by the cube's own Zel'dovich step (journey-play.ts `homeInCube`). Says ready once the cube and its galaxies are in.
-->
<script lang="ts" module>
  import type { ViewSession as Session, WebCube as Cube, WebGalaxies as Galaxies } from "../../../ui-contract/src/index.ts";
  type HomeGalaxy = Extract<Awaited<ReturnType<Session["homeGalaxy"]>>, { ok: true }>["home"];
  type Reads = { home: HomeGalaxy; cube: Cube; galaxies: Galaxies } | string;
  // One set of reads per session, shared by the web's layer and its backdrop (the kernel's answers are deterministic).
  const reads = new WeakMap<Session, Promise<Reads>>();
  // The cube's side: twice the web's stop, so its edges never show while the journey rests there.
  const EXTENT_MPC = 200;
  const CUBE_N = 48;
  function readWeb(s: Session): Promise<Reads> {
    let r = reads.get(s);
    if (!r) {
      r = (async (): Promise<Reads> => {
        const h = await s.homeGalaxy();
        if (!h.ok) return `home galaxy: ${h.message}`;
        const home = h.home;
        const centre: [number, number, number] = [...home.position_mpc];
        const c = await s.webCube({ centre, extent_mpc: EXTENT_MPC, n: CUBE_N });
        if (!c.ok) return `web cube: ${c.message}`;
        if (!s.webGalaxies) return "this seat does not read the web's galaxies";
        // The floor the web's own screen uses for a cube this big, but never above home, so home is lit in it.
        const floor = Math.max(1e8, Math.min(1e11 * (EXTENT_MPC / 100) ** 3, home.mass_msun * 0.5));
        const g = await s.webGalaxies({ centre, extent_mpc: EXTENT_MPC, n: CUBE_N, galaxy_floor_msun: floor });
        if (!g.ok) return `web galaxies: ${g.message}`;
        return { home, cube: c.web, galaxies: g.web };
      })();
      reads.set(s, r);
    }
    return r;
  }
</script>

<script lang="ts">
  import Web3D from "../../../views/foam-map/src/Web3D.svelte";
  import { homeInCube, sayNumber, webEye, WEB_CUBE_FOV, type JourneyCamera, type Vec3, type ViewSession, type WebCube, type WebGalaxies } from "../../../ui-contract/src/index.ts";

  let {
    session,
    cam,
    onready,
    inner = null,
  }: {
    session: ViewSession;
    cam: JourneyCamera | null;
    onready?: (r: { ok: boolean; note: string }) => void;
    /** Past the seam to the galaxies nearby: their reach, and how much of the light within it is still the web's. */
    inner?: { radius_mpc: number; alpha: number } | null;
  } = $props();

  let cube = $state.raw<WebCube | null>(null);
  let galaxies = $state.raw<WebGalaxies | null>(null);
  let target = $state<Vec3 | null>(null);
  $effect(() => {
    void readWeb(session).then((r) => {
      if (typeof r === "string") return onready?.({ ok: false, note: r });
      cube = r.cube;
      galaxies = r.galaxies;
      const found = homeInCube(r.galaxies.galaxies, r.home);
      const p = r.home.position_mpc;
      target = found ? found.position : [p[0], p[1], p[2]];
      onready?.({ ok: true, note: found ? `home moved ${sayNumber(found.moved_mpc, 2)} Mpc by the cube's step` : "home is under the cube's light floor: aimed at its catalogue place" });
    });
  });
  const aim = $derived(cam && target ? { eye: webEye(cam, target, WEB_CUBE_FOV).eye, target } : null);
</script>

{#if cube}
  <Web3D {cube} {galaxies} look="true" hud={false} {inner} aim={aim ?? { eye: [target?.[0] ?? 0, target?.[1] ?? 0, (target?.[2] ?? 0) + 300], target: target ?? [0, 0, 0] }} />
{/if}
