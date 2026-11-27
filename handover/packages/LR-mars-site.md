# LR fix: the run-through lands on a plain Beowulf can land on (26 Nov 2026)

## What broke

`tests/ui/run-through.e2e.mjs` failed on main: Beowulf's Mars landing came out "landed-on-a-bad-site" where it used to say "landed". CI did not run the test, so nothing went red.

## The cause

The test pressed the sixth landmark on Mars's Land screen. Mars's terrain is generated from its physics. PR 417 (25 Nov, SOL4) gave Mars its measured core share, 0.24 in place of the default 0.32. This was confirmed by taking only the `interior.core_mass_fraction` rows out of `packs/base-sol` on that commit: the old landmark came back. With the measured core, Mars's landmarks are different places. The sixth was Dribeque Planitia, which is fine to land on. It is now Drilotel Planitia, 5.2 m rough against the 5 m limit. The kernel judges that plain honestly, so the code was right and the test's fixed pick was the fault. Today's PRs 495, 591 and 594 were not involved: a fresh build of main from before PR 495 fails the same way.

Bisecting trap: `scripts/build-shell.sh` builds the kernel only when `packages/kernel-client/wasm/sse_wasm_bg.wasm` is missing, and that file is ignored by git. When you check out an older commit, delete it first, or the old screens run on the newer kernel.

## What changed

- The run-through now presses each plains landmark in turn and keeps the first one the kernel says Beowulf can land on. On main that is Torrasa Planitia, 2.3 m rough. The assertion that Beowulf lands is unchanged.
- `ci.yml` runs `run-through.e2e.mjs`. It takes about a minute. The coordinator decided this under Alex's 26 Nov word that the coordinator decides routine test changes.

## Found on main, not fixed here (each its own job)

1. **Sabe Planitia reads 174 m rough.** On the Solar System's Mars (main at 6cd856fc), the landmark list offers Sabe Planitia as "plains", but its site reads 173.9 m about the mean ground. Every other plain reads 2 to 6 m. A plain that rough is unlikely to be real. Either the landmark is put on the wrong ground or the site's roughness is misread there. Start with how plains landmarks are placed (the `landmarks` op) and `sse-surface/src/site.rs`.
2. **view-controls.e2e.mjs fails on main.** At line 105, Galaxies nearby (`cluster`) never reports `data-style="chart"` within 30 s after the galaxy chart was left in 3D. It failed the same way twice. It is not in CI. It is not linked to Mars.

The other 36 browser test files that CI skips all passed on main on 26 Nov. All 38 take about 16 minutes in a row.
