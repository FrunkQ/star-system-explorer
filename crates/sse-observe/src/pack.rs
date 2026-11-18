//! The campaign's observation setup as pack content: `provides.other.observe`
//! = `{ disclosure, views, campaign }`. The highest pack in the stack that
//! provides one wins whole (a campaign pack replaces the base setup; U5 edits
//! it there).

use crate::view::{Campaign, DataView, Disclosure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ObserveSetup {
    pub disclosure: Disclosure,
    pub views: Vec<DataView>,
    pub campaign: Campaign,
}

impl ObserveSetup {
    /// Read from the stack (bottom to top) via each pack's
    /// `provides.other.observe`, given as JSON values; `None` if no pack has one.
    pub fn from_stack(observe: &[(String, serde_json::Value)]) -> Result<Option<Self>, String> {
        let Some((pack, v)) = observe.last() else {
            return Ok(None);
        };
        let s: ObserveSetup =
            serde_json::from_value(v.clone()).map_err(|e| format!("{pack}: observe: {e}"))?;
        s.validate().map_err(|e| format!("{pack}: observe: {e}"))?;
        Ok(Some(s))
    }

    pub fn views_by_id(&self) -> BTreeMap<String, DataView> {
        self.views
            .iter()
            .map(|v| (v.id.clone(), v.clone()))
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        let ids = self.views_by_id();
        if ids.len() != self.views.len() {
            return Err("a data view id is defined twice".into());
        }
        for (what, id) in [
            ("default_view", &self.campaign.default_view),
            ("before", &self.campaign.before),
            ("after", &self.campaign.after),
        ] {
            if !ids.contains_key(id) {
                return Err(format!(
                    "campaign {what} names data view {id:?}, which is not defined"
                ));
            }
        }
        Ok(())
    }
}
