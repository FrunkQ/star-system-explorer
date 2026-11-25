//! The campaign's observation setup as pack content: `provides.other.observe`
//! = `{ disclosure, views, campaign }`. Views merge by id across the stack (a
//! higher pack's view replaces a lower one of the same id), so a game pack
//! adding its own views never drops the ones a campaign has already granted;
//! the disclosure rules and the campaign ladder come from the highest pack
//! that provides an observe setup (U5 edits it there).

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
        let mut merged: Option<ObserveSetup> = None;
        let mut top = "";
        for (pack, v) in observe {
            let s: ObserveSetup =
                serde_json::from_value(v.clone()).map_err(|e| format!("{pack}: observe: {e}"))?;
            s.validate().map_err(|e| format!("{pack}: observe: {e}"))?;
            top = pack;
            merged = Some(match merged {
                None => s,
                Some(mut below) => {
                    for view in s.views {
                        match below.views.iter_mut().find(|w| w.id == view.id) {
                            Some(w) => *w = view,
                            None => below.views.push(view),
                        }
                    }
                    ObserveSetup {
                        disclosure: s.disclosure,
                        views: below.views,
                        campaign: s.campaign,
                    }
                }
            });
        }
        if let Some(m) = &merged {
            m.validate()
                .map_err(|e| format!("{top}: observe (merged over the stack): {e}"))?;
        }
        Ok(merged)
    }

    /// Every view by id, with the built-in GM only view unless a pack
    /// defines its own of that id.
    pub fn views_by_id(&self) -> BTreeMap<String, DataView> {
        let mut m: BTreeMap<String, DataView> = self
            .views
            .iter()
            .map(|v| (v.id.clone(), v.clone()))
            .collect();
        m.entry(crate::known::GM_ONLY_VIEW.into())
            .or_insert_with(crate::known::gm_only_view);
        m
    }

    /// The campaign's three landmark views (SV1).
    pub fn landmarks(&self) -> crate::known::Landmarks {
        crate::known::Landmarks::resolve(&self.views, &self.campaign)
    }

    pub fn validate(&self) -> Result<(), String> {
        let ids: BTreeMap<&str, &DataView> =
            self.views.iter().map(|v| (v.id.as_str(), v)).collect();
        if ids.len() != self.views.len() {
            return Err("a data view id is defined twice".into());
        }
        for (what, id) in [
            ("default_view", &self.campaign.default_view),
            ("before", &self.campaign.before),
            ("after", &self.campaign.after),
        ] {
            if !ids.contains_key(id.as_str()) {
                return Err(format!(
                    "campaign {what} names data view {id:?}, which is not defined"
                ));
            }
        }
        if let Some(l) = &self.campaign.landmarks {
            for (what, id) in [
                ("gm_only", &l.gm_only),
                ("heard_of", &l.heard_of),
                ("known", &l.known),
            ] {
                if let Some(id) = id {
                    if id != crate::known::GM_ONLY_VIEW && !ids.contains_key(id.as_str()) {
                        return Err(format!(
                            "campaign landmarks.{what} names data view {id:?}, which is not defined"
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod merge_tests {
    use super::ObserveSetup;
    use std::path::PathBuf;

    fn observe(p: &str) -> serde_json::Value {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(p)).unwrap()).unwrap();
        v["provides"]["other"]["observe"].clone()
    }

    /// The base stack's player data (with "rumour") under the Traveller pack
    /// (which defines no "rumour"): the stacked setup keeps every view, so a
    /// body already granted "rumour" still reads (Run desk thread, 24 Nov).
    #[test]
    fn a_game_pack_adds_views_without_dropping_the_campaigns() {
        let base = observe("packages/shell/src/documents/kestrel.player-data.pack.json");
        let trav = observe("packs/traveller/pack.json");
        assert!(
            base.is_object() && trav.is_object(),
            "both packs carry an observe setup"
        );
        let s = ObserveSetup::from_stack(&[
            ("base".into(), base.clone()),
            ("traveller".into(), trav.clone()),
        ])
        .unwrap()
        .unwrap();
        let ids: Vec<&str> = s.views.iter().map(|v| v.id.as_str()).collect();
        for want in base["views"]
            .as_array()
            .unwrap()
            .iter()
            .chain(trav["views"].as_array().unwrap())
        {
            assert!(ids.contains(&want["id"].as_str().unwrap()), "{ids:?}");
        }
        assert!(ids.contains(&"rumour"), "{ids:?}");
        // The top pack's ladder and its own version of a shared id win.
        let top: ObserveSetup = serde_json::from_value(trav).unwrap();
        assert_eq!(s.campaign, top.campaign);
        assert_eq!(s.disclosure, top.disclosure);
        for v in &top.views {
            assert_eq!(s.views.iter().find(|w| w.id == v.id), Some(v));
        }
        // One pack alone reads as before.
        let alone = ObserveSetup::from_stack(&[("base".into(), base.clone())])
            .unwrap()
            .unwrap();
        assert_eq!(alone, serde_json::from_value::<ObserveSetup>(base).unwrap());
        assert_eq!(ObserveSetup::from_stack(&[]).unwrap(), None);
    }
}
