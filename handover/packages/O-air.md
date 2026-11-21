# Air optics (light track, for the renderer): Rayleigh scattering of planets' air in the kernel

Asked for by the renderer thread (R2a has a TypeScript copy in `packages/renderer/src/atmosphere.ts`), so the
renderer can read the kernel and delete its copy. Atoms `sse.air.*` in `sse-atoms::air`; gas table in
`packs/base-light` (`gases`); module `sse_light::air`; kernel op `read_air_optics`.

## What was built
- Five atoms: Rayleigh cross-section from refractivity and King factor (Lorentz-Lorenz form, Bodhaine et al.
  1999 eq. 22), dry-air refractivity (Peck and Reeder 1972), column optical depth τ = Pσ/(m̄g), scale height
  kT/(m̄g), scattering coefficient β = nσ.
- Gas table (pack data): dry air, N₂, O₂, Ar, CO₂, H₂, He, CH₄, H₂O with molar mass, n − 1 at 589.3 nm and 0 °C,
  King factor (Bodhaine's wavelength-dependent forms for N₂ and O₂).
- `optics(air, wavelengths, table)`: mean molecular mass, scale height, surface density, and at each wavelength
  and each visible slice centre: cross-section, β at the surface and τ straight up.
- Kernel op `read_air_optics { airs: [{ address, gases: [{ gas, fraction }], surface_pressure_pa,
  surface_temperature_k, gravity_m_s2 }], wavelengths_m? }` → `air_optics_read { airs }` (default 680, 550, 440 nm);
  error `bad_air`.

## Evidence (`cargo test -p sse-atoms` 1 test, `cargo test -p sse-light` 2 tests)
- Dry air at 550 nm: cross-section 4.5105×10⁻³¹ m² to 0.2 % (Bodhaine table 3), τ = 0.0971 to 0.5 % with g at the
  column height, scale height 8.43 km at 288.15 K.
- Earth from its gases (N₂, O₂, Ar, CO₂) and as dry air: β = 4.84, 11.49 and 28.6 per million metres at 680, 550 and
  440 nm to 1.5 % (Bucholtz 1995, the renderer's check numbers).
- Mars: scale height about 11 km, τ(550) between 0.001 and 0.006. Titan: over five times Earth's τ.
- Unknown gases and fractions that do not add to one are refused.

## Stand-ins (to add to docs/fudges.md)
- Gases other than dry air follow air's dispersion scaled to their 589 nm refractivity.
- King factors constant for Ar (1.00), CO₂ (1.15) and the rest (1.00).
- Scale height isothermal at the surface temperature; gravity at the surface, not the column height.
- Molecular scattering only (no haze, dust, cloud or absorption bands).
- The 589 nm refractivities (Kaye and Laby) are written from memory and need checking against the table.

## References
Bodhaine et al. 1999 (doi:10.1175/1520-0426(1999)016<1854:ORODC>2.0.CO;2, text checked 19 Nov); Peck and Reeder 1972
(doi:10.1364/JOSA.62.000958); Bucholtz 1995 (doi:10.1364/AO.34.002765, found by search 19 Nov); Kaye and Laby 2.5.7.
