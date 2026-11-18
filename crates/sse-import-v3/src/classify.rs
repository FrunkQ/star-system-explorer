//! Field classes, from docs/v3-save-fields.md (read from V3.1 `src/lib/types.ts`).
//! Fields handled specially by the importer (masses, radii, orbit, tags,
//! parents) never reach these tables.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    /// Walked by the importer itself (lists of systems and nodes).
    Structural,
    /// Kept as an authored `v3.<name>` fact.
    Authored,
    /// GM overrides and anomalies: authored-impossible, with an honesty mark.
    Impossible,
    /// View settings, not physics.
    Presentation,
    /// V3 bookkeeping.
    Dropped,
    /// V3 wrote a derived value. V4 recomputes it.
    Derived,
    /// Not in V3.1's types: kept as authored and reported.
    Unknown,
}

pub fn starmap_class(k: &str) -> Class {
    use Class::*;
    match k {
        "systems" => Structural,
        "id" | "name" | "description" | "gmNotes" | "routes" | "activeJourneys"
        | "adriftConstructs" | "temporal" | "rulePackOverrides" | "travellerMetadata"
        | "importedSubsectors" | "distanceUnit" | "unitIsPrefix" | "coiCategories" | "poiPacks"
        | "reasonsConfig" => Authored,
        "mapMode"
        | "invertDisplay"
        | "scale"
        | "mapGrid"
        | "ignoreZForDistances"
        | "measurementUnits"
        | "temperatureUnit"
        | "unitPrefs"
        | "systemEdgeAu"
        | "playerPresets"
        | "playerAssets"
        | "contentCredits"
        | "coverAssetId"
        | "mapBackground" => Presentation,
        "appVersion"
        | "bundleFormat"
        | "baseMapVersion"
        | "baseMapUpgradeDeclined"
        | "baseMapUpgradeDismissed"
        | "keptCopyForVersion"
        | "broadcastId"
        | "gridCenterId"
        | "generationEngine" => Dropped,
        _ => Unknown,
    }
}

pub fn system_class(k: &str) -> Class {
    use Class::*;
    match k {
        "nodes" => Structural,
        "id" | "name" | "seed" | "epochT0" | "age_Gyr" | "ageEstimated" | "ageBandGyr" | "tags"
        | "notes" | "gmNotes" | "credits" => Authored,
        "visualScalingMultiplier" | "toytownFactor" => Presentation,
        "rulePackId" | "rulePackVersion" | "bundleFormat" | "appVersion" => Dropped,
        _ => Unknown,
    }
}

pub fn body_class(k: &str) -> Class {
    use Class::*;
    match k {
        // identity, GM text and intent
        "id"
        | "name"
        | "placement"
        | "notes"
        | "gmNotes"
        | "description"
        | "object_playerhidden"
        | "description_playerhidden"
        | "kind"
        | "roleHint"
        | "classes"
        | "stellarType"
        | "autoClassify"
        | "figureSources" => Authored,
        // primary physical inputs kept raw until P4 maps them
        "makeup"
        | "atmosphere"
        | "hydrosphere"
        | "biosphere"
        | "magnetism"
        | "volatiles"
        | "densityHold"
        | "evolveAtmosphere"
        | "atmosphere0"
        | "flareActivity"
        | "isSelfLuminous"
        | "selfLuminousTeffK"
        | "internalLuminositySolar"
        | "traveller"
        | "megaParams"
        | "megaType"
        | "requires"
        | "explain"
        | "artificial"
        | "constructChrome" => Authored,
        // barycentres
        "memberIds" | "coOrbital" | "circumbinary" => Authored,
        // constructs
        "physical_parameters"
        | "systems"
        | "crew"
        | "engines"
        | "fuel_tanks"
        | "sensors"
        | "model"
        | "IsTemplate"
        | "cargoDescription"
        | "current_cargo_tonnes"
        | "current_crew_count"
        | "vector_position_au"
        | "vector_velocity_ms"
        | "vector_epoch_ms"
        | "flight_state"
        | "attachedTo"
        | "surface_anchor"
        | "draft_transit_plan"
        | "scheduled_journeys"
        | "autopilot"
        | "flight_log"
        | "dimensionsM"
        | "spinRadiusM"
        | "cargoCapacity_tonnes"
        | "can_aerobrake"
        | "thermal_protection_type"
        | "aerobrake_limit_kms" => Authored,
        // AI prompt notes, for Minerva
        "seedText" | "style" | "length" | "lastPrompt" | "aiContext" => Authored,
        "overrides" | "anomalies" => Impossible,
        "ui_parentId" | "image" => Presentation,
        "autopilotStuckReason" | "placementHealCount" => Dropped,
        "temperatureK"
        | "temperatureRangeK"
        | "temperatureProfile"
        | "surfaceTempK"
        | "equilibriumTempK"
        | "pressureBar"
        | "densityGcm3"
        | "calculatedGravity_ms2"
        | "distanceToHost_km"
        | "orbitalBoundaries"
        | "albedo"
        | "albedoBreakdown"
        | "gasThermalInflation"
        | "radiogenicHeatK"
        | "internalHeatK"
        | "magneticFieldGauss"
        | "magnetic_field"
        | "magnetosphere"
        | "astrosphereAu"
        | "geoActivity"
        | "irradiationDose"
        | "orbitalRadiation"
        | "surfaceRadiation"
        | "totalIncidentFlux"
        | "totalIncidentFluxMin"
        | "totalIncidentFluxMax"
        | "starlightFlux"
        | "photonRadiation"
        | "particleRadiation"
        | "radiationShieldingAtmo"
        | "radiationShieldingMag"
        | "starlightDimming"
        | "starName"
        | "receivedFrac"
        | "occluders"
        | "apparentColor"
        | "apparentColorHex"
        | "surfaceSpectrum"
        | "vegetation"
        | "oblateness"
        | "tidallyLocked"
        | "starTidallyLocked"
        | "beltInnerEdgeRadii"
        | "habitabilityBreakdown"
        | "classification"
        | "auroraEmitters"
        | "typicalForClass"
        | "effectiveMassKg" => Derived,
        _ => Unknown,
    }
}
