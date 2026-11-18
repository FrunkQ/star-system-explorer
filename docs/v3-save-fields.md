# V3 save-file fields for the importer (P3)

Status: K0 output, read from V3.1 `src/lib/types.ts` and `src/lib/io/saveShape.spec.ts`. P3 owns the importer; this
list says what it reads and what it does with each field. Real V3 saves for P3's tests come from the Explorers site and
stay outside the repo.

The rule that decides every row: **V4 stores authored facts only.** V3 wrote many derived values back into the save
(its own fault 8). The importer takes those as a check, never as truth: it recomputes them with V4 flows and lists
every difference in the import report, as the P3 check requires.

Classes:

- **A, authored**: becomes an authored event (source `authored`, or `authored-impossible` when V3 marked an override or
  anomaly).
- **C, catalogued**: a real-sky value from an import; becomes a `catalogued` event with its V3 provenance.
- **D, derived**: not imported as state; recomputed by V4 and compared in the report.
- **P, presentation**: goes to view settings or the GM's workspace, not the chronicle.
- **X, dropped**: read and ignored, with a line in the report when present.

## 1. Containers and formats

| Field | Class | Notes |
|---|---|---|
| `.sse.zip` with `bundleFormat` as the first line of `starmap.json` or `system.json` | — | The importer accepts plain JSON or the zip. It refuses an unknown `bundleFormat` politely. Two canonical V3 archives are pinned in V3's `tests/fixtures/creator-hub-*.sse.zip`; use copies outside this repo. |
| `ATTRIBUTIONS.md` in the zip | A | Credit chain, kept on the imported events. |
| GM registries only (B112, G89): custom calendars, tag categories, moved switches | A | Become the GM's own pack in the stack. Shipped items that V3 left out stay out. |

## 2. Starmap

| Field | Class | Notes |
|---|---|---|
| `id`, `name`, `description`, `gmNotes` | A | Universe identity and notes. `gmNotes` never enters a player data view. |
| `systems[]` (StarSystemNode) | A | See section 3. |
| `routes[]`: `id`, `sourceSystemId`, `targetSystemId`, `distance`, `unit`, `lineStyle`, `name` | A, P | Route relation is authored; `lineStyle` is presentation; `distance` is D (recomputed from positions unless the GM authored it). |
| `activeJourneys[]`, `adriftConstructs[]` | A | Scheduled intents and coasting states at the import time. |
| `mapMode`, `invertDisplay`, `scale`, `mapGrid`, `distanceUnit`, `unitIsPrefix`, `ignoreZForDistances`, `measurementUnits`, `temperatureUnit`, `unitPrefs`, `systemEdgeAu` | P | View settings. |
| `temporal` | A | Calendars and the time anchor; mapped onto V4 seconds from the origin. |
| `playerPresets`, `playerAssets`, `contentCredits`, `coverAssetId`, `mapBackground` | P | Player views and assets. Presets map to the nearest V4 default view and theme. |
| `rulePackOverrides` | A | Into the GM's own pack. |
| `travellerMetadata`, `importedSubsectors` | A | Into the Traveller pack's records (X3a). |
| `appVersion`, `baseMapVersion`, `baseMapUpgradeDeclined`, `baseMapUpgradeDismissed`, `keptCopyForVersion`, `broadcastId`, `gridCenterId` | X | V3 bookkeeping. `appVersion` goes in the report header. |
| `generationEngine` | X | Dead in V3 already. |

## 3. Systems

| Field | Class | Notes |
|---|---|---|
| StarSystemNode `id`, `name`, `isNameUserDefined`, `position {x, y, z?}`, `subsectorId` | A | Position becomes the system's authored place; a missing `z` is 0 and flagged flat. |
| StarSystemNode `viewport`, `time.displayTimeSec` | P | |
| System `id`, `name`, `seed`, `epochT0`, `age_Gyr`, `ageEstimated`, `ageBandGyr` | A | V3 `seed` is kept as provenance; V4 draws from its own keyed seed, so a V3 seed does not reproduce V3 generation. Age is authored. |
| System `rulePackId`, `rulePackVersion` | X | Replaced by the stack hash. Report names the V3 pack. |
| System `tags`, `notes`, `gmNotes`, `credits` | A | |
| System `visualScalingMultiplier`, `toytownFactor` | P | |
| System `nodes[]` | A | Bodies and barycentres, section 4. |

## 4. Nodes (CelestialBody and Barycenter)

Common (NodeBase): `id`, `name`, `parentId`, `ui_parentId` (P), `placement`, `tags`, `notes`, `gmNotes`,
`description`, `object_playerhidden`, `description_playerhidden`. All A except `ui_parentId`. The two `playerhidden`
flags become entries in the default player data view.

Tags (`key`, `value`, `ns`, `origin`, `override`, `secret`, `disclosure`): `origin` decides the class. `physics`,
`rule` and `derived` tags are D (V4 re-emits them and the report compares); `authored`, `manual` and `inherited` are A;
`override` makes an A tag replace a physics one; `secret` and `disclosure` (hidden, anonymous, open) go to the default
data view.

Barycenter: `memberIds`, `orbit`, `coOrbital`, `circumbinary` are A; `effectiveMassKg` is D.

Orbit: `hostId`, `elements` (Kepler `a_AU`, `e`, `i_deg`, `Omega_deg`, `omega_deg`, `M0_rad`), `t0`, `resonance`,
`isRetrogradeOrbit`, `frame` are A; `hostMu`, `n_rad_per_s` are D; `seed`, `lastEditedT0` are X. A `coOrbital`
(Lagrange point) makes the orbit D, as in V3.

CelestialBody, authored or catalogued (A, or C when its tags show a real-sky import):

- identity and role: `kind`, `roleHint`, `classes` (a pinned class is A), `stellarType`, `autoClassify`, `figureSources`
- primary physical inputs: `massKg`, `radiusKm` (`radiusInnerKm`, `radiusOuterKm` for belts and rings),
  `rotation_period_hours`, `axial_tilt_deg` and `obliquity_deg`, `makeup`, `atmosphere` (`name`, `main`,
  `pressure_bar`, `composition`), `hydrosphere` (`coverage`, `depth_m`, `composition`, `layers`), `biosphere`,
  `magnetism` when hand-set, `volatiles`
- GM intent: `overrides` and `anomalies` (become `authored-impossible`), `densityHold`, `evolveAtmosphere`,
  `atmosphere0`
- stars: `temperatureK`, `flareActivity`, `isSelfLuminous`, `selfLuminousTeffK`, `internalLuminositySolar`
- traveller block: `traveller` (UWP and the rest), into the Traveller pack (X3a)
- art: `image`, `apparentColorHex` when hand-picked (P)

CelestialBody, derived (D, recomputed and compared): `temperatureRangeK`, `temperatureProfile`, `surfaceTempK`,
`equilibriumTempK`, `pressureBar`, `densityGcm3`, `calculatedGravity_ms2`, `distanceToHost_km`, `orbitalBoundaries`,
`albedo`, `albedoBreakdown`, `gasThermalInflation`, `radiogenicHeatK`, `internalHeatK`, `magneticFieldGauss`,
`magnetic_field`, `magnetosphere`, `astrosphereAu`, `geoActivity`, `irradiationDose`, `orbitalRadiation`,
`surfaceRadiation`, `totalIncidentFlux` (and min, max), `starlightFlux`, `photonRadiation`, `particleRadiation`,
`radiationShieldingAtmo`, `radiationShieldingMag`, `starlightDimming`, `receivedFrac`, `occluders`, `apparentColor`,
`surfaceSpectrum`, `vegetation`, `oblateness`, `tidallyLocked`, `starTidallyLocked`, `beltInnerEdgeRadii`,
`habitabilityBreakdown` (`factors`, `surfaceScore`, `modifiers`, `finalScore`, `tier`), `classification` when not
pinned, `auroraEmitters`, `typicalForClass`.

Constructs (`kind: construct`): `physical_parameters` (`dimensionsM`, `massKg`, `spinRadiusM`, `cargoCapacity_tonnes`,
`can_aerobrake`, `thermal_protection_type`, `aerobrake_limit_kms`), `systems` (`power_plants`, `life_support`,
`modules`), `crew`, `engines`, `fuel_tanks`, `sensors`, `model`, `IsTemplate`, `cargoDescription`,
`current_cargo_tonnes`, `current_crew_count`, `megaType`, `megaParams`, `requires`, `explain`, `artificial`,
`constructChrome` are A (X3d). Motion: `vector_position_au`, `vector_velocity_ms`, `vector_epoch_ms`, `flight_state`,
`attachedTo`, `surface_anchor`, `draft_transit_plan`, `scheduled_journeys`, `autopilot` are A as the state and intents
at import time. `flight_log` becomes chronicle history events dated in the past. `autopilotStuckReason`,
`placementHealCount` are X.

AI context (`seedText`, `tags`, `style`, `length`, `lastPrompt`): A, kept for Minerva (A3) as the GM's prompt notes.

## 5. What P3 must test

- Both V3 bundled starmaps and at least three GM-made maps from the Explorers site import without loss of any A field
  (round trip of A fields compared field by field).
- The report lists every D field whose V4 value differs from V3 beyond the flow's stated tolerance, with the flow that
  produced the V4 value, so each difference is explained.
- A V3 override or anomaly arrives as `authored-impossible` and shows its honesty mark on the GM view.
- `secret` tags and `gmNotes` never appear in the default player data view.
