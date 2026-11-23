# MP22: nine sse-atoms copies read the sheet

Kernel thread, 23 Nov 2026, agreed with the data sheet thread. The ledger goes from 12 lines to 3.

## What reads the sheet now
- `air.rs` `N_LOSCHMIDT` = ATM / (K_B T0C). It moves in the last bits (2.686780111e25 to 2.6867801118e25).
- `heat.rs` `AL26_TAU_MYR` = the sheet's Al-26 half-life over ln 2. The half-life is in seconds, so it moves in the last bits.
- `layers.rs`:
  - `NACL_MOLAR_MASS` = (Na + Cl) atomic weights / 1000, which is 0.0584398 rather than 0.05844;
  - `ICE_K` = the sheet's ice Ih conductivity fit (the same 651).
- `nebula.rs`:
  - `E_ION_H` = R∞ h c m_p / (m_p + m_e), 13.598 eV, from the sheet's constants;
  - `H_ALPHA_J` = E_ION_H (1/4 − 1/9). This is the vacuum line, 656.47 nm; 656.28 nm was the value in air;
  - `M_H` = H's atomic weight × M_U, 1.6738e-27 kg rather than 1.6735e-27;
  - `MU_H` = 1 / X of the protosolar mixture, 1.420 rather than 1.4.

## The generator
`scripts/build-rust-reference.py` now also writes two things into `sse_core::sheet`:
- `isotope_half_life_s`, `isotope_decay_heat_w_per_kg`, `isotope_share_at_sun_birth` and `isotope_share_today`, by isotope;
- `protosolar::{X, Y, Z}`, worked out from every element's protosolar abundance times its atomic weight. This gives X 0.704, Y 0.279 and Z 0.017 (Lodders 2021).

## Left on the ledger
These three wait for the data sheet thread's next sheet PR:
- `AL26_H0`, which needs the CI-chondrite Al mass fraction;
- `WATER_KF`, which will be worked out as R T_f² M / ΔH_fus once water's latent heat of fusion is on the sheet;
- `OMEGA_SUN`, which needs the Sun's sidereal rotation P_rot_sun.

## Goldens re-blessed on purpose
- `tests/golden/nebulae.json`: μ_H, m_H and the ionisation energy.
- `tests/golden/cosmos.json` and `tests/fixtures/cosmos/solar-patch.json`: the expected planetary nebulae, through the nebula atoms.
- `tests/golden/contract.jsonl`, two lines: `nebulae_read`, and `air_optics_read` through Loschmidt's number.
