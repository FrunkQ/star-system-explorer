//! The empty life suite of stage L0.

use crate::*;
use sse_suite::{AdvanceOutput, InputDataset, NullSuite, SuiteAdapter, SuiteError, SuiteManifest};

pub const NO_MODEL: &str = "no life model installed";

/// Installed, takes inputs, claims nothing, and says so when asked why.
#[derive(Clone, Debug, PartialEq)]
pub struct NoLifeModel {
    manifest: SuiteManifest,
}
impl Default for NoLifeModel {
    fn default() -> Self {
        NoLifeModel {
            manifest: NullSuite::life().manifest,
        }
    }
}

impl LifeSuite for NoLifeModel {
    fn id(&self) -> &str {
        "life"
    }
    fn evidence(&self) -> LifeEvidence {
        LifeEvidence::NoModel
    }
    fn envelope(
        &self,
        address: &Address,
        epoch: UniverseTime,
        profiles: &[LifeRequirementProfile],
    ) -> LifeEnvelope {
        LifeEnvelope {
            address: address.clone(),
            epoch,
            p_present: None,
            origin_time: None,
            energy_budget_w: None,
            biomass_class: None,
            capabilities: BTreeMap::new(),
            complexity: BTreeMap::new(),
            fluxes: Vec::new(),
            signatures: BTreeMap::new(),
            uncertainty: format!("{NO_MODEL}: nothing is known either way"),
            evidence: LifeEvidence::NoModel,
            inputs: ContentHash::of(&profiles.to_vec()).ok(),
        }
    }
    fn resolve(
        &self,
        _: &LifeEnvelope,
        _: &sse_core::Seed,
    ) -> Result<ResolvedBiosphere, LifeError> {
        Err(LifeError(NO_MODEL.into()))
    }
    fn query_feasible(&self, _: &LifeQuery, _: &[LifeRequirementProfile]) -> Feasibility {
        Feasibility {
            answer: None,
            reason: NO_MODEL.into(),
        }
    }
    fn explain(&self, _: &Address, _: &str) -> String {
        NO_MODEL.into()
    }
}

impl SuiteAdapter for NoLifeModel {
    fn manifest(&self) -> &SuiteManifest {
        &self.manifest
    }
    fn provide(&mut self, _: &sse_core::Seed, _: &InputDataset) -> Result<(), SuiteError> {
        Ok(())
    }
    fn advance(&mut self, _: UniverseTime) -> Result<AdvanceOutput, SuiteError> {
        Ok(AdvanceOutput::default())
    }
    fn explain(&self, _: &Address, _: &str) -> Option<String> {
        Some(NO_MODEL.into())
    }
}
