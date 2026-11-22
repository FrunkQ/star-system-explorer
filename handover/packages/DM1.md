# DM1: the Cosmos group on the data sheet, and the "theoretical" mark

Track GH (dark matter and galaxy histories), design `handover/design/SSE4-galaxy-histories-design.md` section
3.1 to 3.3, accepted by Alex on 22 Nov 2026 13:15. Built by the master data sheet thread.

## What was built

- **The Cosmos group** on the Constants page (`packs/base-reference`, built by
  `scripts/build-base-reference.py`, `cosmos()`). Rows, all `group: "cosmos"`:

  | Row | Value | How |
  |---|---|---|
  | `H0` | 67.66 km/s/Mpc | Planck 2018 VI table 2 |
  | `H0_si` | 2.19e-18 per second | worked out: `H0 1000 / (1e6 pc)` |
  | `Omega_b` | 0.04897 | Planck's Ωb h² 0.02242 over h², to four figures (as G1 has always used it) |
  | `Omega_c` | 0.2607228, **theoretical** | Planck's Ωm less Ωb and Ων, so the three add to 0.3111 |
  | `Omega_nu` | 0.0014072 | one 0.06 eV neutrino, Ων h² = Σm / 93.14 eV |
  | `Omega_m` | 0.3111 | worked out: `Omega_b + Omega_c + Omega_nu` |
  | `dm_per_ordinary` | 5.32 | worked out: `Omega_c / Omega_b` |
  | `f_b` | 0.1574 | worked out: `Omega_b / Omega_m` |
  | `sigma8`, `n_s` | 0.8102, 0.9665 | Planck |
  | `T_cmb`, `N_eff` | 2.7255 K, 3.046 | Fixsen 2009, Mangano et al. 2005 |
  | `Omega_r` | 9.2e-5 | worked out from T_cmb and N_eff, the code's own formula |
  | `Omega_Lambda` | 0.68881, **theoretical** | worked out: `1 - Omega_m - Omega_r` (flat) |
  | `w0`, `wa` | -1 and 0, **theoretical** | the Chevallier-Polarski-Linder form |
  | `rho_Lambda` | 5.32e-10 J/m³ | worked out |
  | `m_wdm`, `sigma_dm`, `m_fdm` | 0 (cold, collisionless, off), **theoretical** | read from DM5 on |

  A worked-out row's typed copy is checked against its formula to 1e-9. Once any row it rests on is edited, at
  any depth, the worked-out value wins (the reader's one-pack check now reaches through derived rows).
- **The "theoretical" mark.** A constants row may carry `mark: "theoretical"` and must then carry a `mark_note`
  saying what would change the picture. It is the only row mark: authored and fictional still come from the
  packs. The reader refuses any other mark, and a theoretical row with no note.
- **Page groups.** `provides.reference.groups` names the Constants page's four groups (nature, convention,
  factor, cosmos), each with an `about`. Cosmos also carries four cited notes: the particle route and why
  (Clowe 2006, Boran 2018), the standard picture passing new tests and G not changing with time (DES 2025,
  Vijaykumar 2021, Sakstein and Desmond 2026), the check that haloes must reproduce (McGaugh 2016), and energy
  conservation per bound system. `read_matter` with no id returns them whole as `groups`. A constant's group
  must be one of them, and each note's sources must be on the sheet.
- **The cosmology reads the sheet.** `sse_core::constants` gained a `Cosmos` group in the one handle (ids
  `H0`, `Omega_b`, `Omega_c`, `Omega_nu`, `Omega_m`, `sigma8`, `n_s`, `T_cmb`, `N_eff`, `w0`, `wa`), set by
  the kernel from the stack's sheet for every call, as G is. `CosmologyPack::from_packs` takes the measured
  parameters from the handle. `packs/base-cosmology` keeps only `now_gyr`, a model setting. A pack that still
  sets `h0`, `omega_m` and the rest is refused, with a message naming the sheet row.
- **Reach.** `sse_atoms::reach::STAGES` declares the cosmos chain's stages beside the atoms:
  - follows: expansion, growth, power spectrum, web, epochs, halo counts, halo growth, baryon share;
  - estimated: major mergers (Fakhouri et al. 2010) and sub-haloes (Giocoli et al. 2008, Boylan-Kolchin et
    al. 2008), fits made in our universe and used unchanged;
  - cannot follow: the galaxy regulator and the ring yields, which are tuned to the observed population.

  `read_reach` takes any handle id. `uses_it` is true when a block reads the constant or a row worked out from
  it, so the expansion uses `Omega_c` through `Omega_m`.

## Not here

- **Where the mark shows.** The Reference screen's Constants page, at every depth, on phone and in Night chart,
  is the chart room thread's (MP10). The data is `row.mark` and `row.mark_note`, plus `groups.cosmos`.
- DM2 reads `w0` and `wa`; DM4 and DM5 read the dark matter rows. Until then the expansion uses w = -1, and
  the rows say so.
- The half-mode mass and the core size are worked out in code (DM4 and DM5), not as sheet formulas.

## Citations

Every source the rows and notes cite is in `docs/citations/matter-reference.md` section A5. Crossref was read
for 19 DOIs, and all of them match. Randall 2008 could not be fetched (rate limit), so it stays unverified.
Planck's numbers were read from the journal's own Table 2 page.

**One correction to the design.** Its Ωc of 0.2621 (Ωc h² 0.1200) is Planck's lensing column. The cosmology
has always used the column with BAO as well (Ωc h² 0.11933, H0 67.66, Ωm 0.3111). The sheet keeps that column,
which gives Ωc 0.2607 and a dark-to-ordinary ratio of 5.32, not 5.35.

## Checks

- `more_dark_matter_follows_through_the_cosmos_chain` (kernel):
  - with no sheet and with the unedited sheet, the galaxy chart is identical;
  - with twice the dark matter, the epochs and the history change;
  - the reach report lists the stages, and Ωm on the sheet moves to 0.5718;
  - the next kernel and the handle are back to Planck.
- `dark_matter_is_marked_theoretical_on_the_sheet` (kernel): the seven theoretical rows carry their note, the
  measured rows carry none, and the Cosmos notes come with the listing.
- `the_reach_report_lists_every_block_once` now covers stages as well as atoms.
- `cosmology_reads_the_cosmos_group` (sse-cosmos): the unedited handle gives the old parameters, an old pack
  is refused, and an edited handle reaches the cosmology.
- `the_theoretical_mark_and_page_groups_are_checked` and `an_edit_reaches_through_derived_rows`
  (sse-reference).
- `the_base_sheet_loads_clean`: every handle id, the Cosmos group included, equals the compiled value bit for
  bit.
- The typed-copies audit passes, with 615 left. No new copy was labelled.

Goldens: two lines of the contract golden change, on purpose. The `read_reach` case for G gains the stages, and the empty `read_matter` listing gains `"groups":{}`. Every other golden is
identical, which is DM1's acceptance test.
