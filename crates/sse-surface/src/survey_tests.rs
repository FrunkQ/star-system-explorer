use crate::atlas::TileKey;
use crate::scenario::{fixtures, settings};
use crate::survey::{survey_world, SurveyOptions};
use crate::SurveyState;
use sse_core::draw::Seed;

#[test]
fn every_fixture_surveys() {
    let mut s = settings();
    s.plates.cells_per_face = 16;
    s.atlas.tile_samples = 32;
    let seed = Seed::from_text("sse4-survey-tests");
    for f in fixtures() {
        let b = &f.body;
        let sv = survey_world(&s, b, &seed, b.age_gyr, &SurveyOptions::default())
            .unwrap_or_else(|e| panic!("{}: {e}", b.label));
        let t = sv
            .atlas
            .tile(TileKey::new(0, 0, 0, 0).unwrap(), SurveyState::Surveyed)
            .unwrap_or_else(|e| panic!("{}: {e:?}", b.label));
        assert!(t.samples > 0, "{}", b.label);
        let giant = sv.envelope.kind == "cloud-deck";
        assert_eq!(sv.register.is_none(), giant, "{}", b.label);
        assert_eq!(sv.atlas.layers().is_none(), giant, "{}", b.label);
        eprintln!(
            "{:10} {:12} stages {} features {} band {}",
            b.label,
            sv.envelope.kind,
            sv.stages.len(),
            sv.register.as_ref().map(|r| r.features.len()).unwrap_or(0),
            sv.aerostat.is_ok()
        );
    }
}
