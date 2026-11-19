//! F9: pinned worlds survive regeneration; reconstruction is labelled.
use crate::history::{How, Manifest};
use crate::pins::*;
use crate::pipeline::Block;

fn sunlike(n: u64) -> Manifest {
    let mut man = Manifest::new(
        "f9-pins",
        crate::scenario::system(n),
        crate::scenario::envelope(),
    );
    man.star_pins.primary_mass = Some(1.0);
    man.star_pins.count = Some(1);
    man.inherit_pins.age_gyr = Some(4.57);
    man
}

fn earth_and_jupiter() -> Pins {
    Pins {
        worlds: vec![
            WorldPin {
                label: "Earth".into(),
                mass_earth: 1.0,
                a_au: 1.0,
                e: Some(0.0167),
            },
            WorldPin {
                label: "Jupiter".into(),
                mass_earth: 317.8,
                a_au: 5.2,
                e: Some(0.049),
            },
        ],
        tries: 8,
        reconstruct: false,
    }
}

#[test]
fn pinned_worlds_survive_regeneration() {
    for n in 0..4 {
        let a = solve(sunlike(n), &earth_and_jupiter(), Block::Modern).unwrap();
        let b = solve(sunlike(n), &earth_and_jupiter(), Block::Modern).unwrap();
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        assert!(
            a.fit <= a.fit_first,
            "the search never does worse than the manifest's own seed"
        );
        for (i, pin) in earth_and_jupiter().worlds.iter().enumerate() {
            let mt = a.matches.iter().find(|x| x.pin == i).unwrap();
            let k = a
                .run
                .protoplanets
                .iter()
                .position(|p| p.address == mt.world)
                .expect("pinned world kept");
            let (p, w) = (&a.run.protoplanets[k], &a.run.worlds[k]);
            assert_eq!(p.a_au, pin.a_au);
            let total = p.mass + p.gas;
            assert!(
                (total / pin.mass_earth - 1.0).abs() < 1e-9,
                "{} mass {total}",
                pin.label
            );
            assert_eq!(w.address, p.address);
            assert_eq!(
                a.run.history.evidence.get(&format!("pins.worlds.{i}")),
                Some(&How::Authored)
            );
        }
        // Nothing generated sits on top of a pinned world.
        for w in &a.run.protoplanets {
            for mt in &a.matches {
                if w.address != mt.world {
                    let pin = &earth_and_jupiter().worlds[mt.pin];
                    assert!(
                        (w.a_au / pin.a_au - 1.0).abs() > 0.02,
                        "{} at {}",
                        w.address,
                        w.a_au
                    );
                }
            }
        }
        // The pinned Earth gets a heat budget like any solid world.
        let earth = &a.matches[0].world;
        assert!(a.run.heat.iter().any(|h| &h.address == earth));
        assert!(a.run.notes.iter().any(|n| n.contains("pinned")));
    }
}

#[test]
fn reconstruction_is_labelled() {
    let mut p = earth_and_jupiter();
    p.reconstruct = true;
    let a = solve(sunlike(0), &p, Block::Modern).unwrap();
    assert_eq!(
        a.run.history.evidence.get("history"),
        Some(&How::Reconstructed)
    );
}
