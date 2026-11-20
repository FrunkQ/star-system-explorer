use super::*;
use crate::hurley::Fits;
use crate::lifeline::LifeLine;
use crate::stars::{stability, BinaryOrbit};
use sse_core::Address;

fn b(id: &str, m: f64) -> Node {
    Node::body(id, m)
}

#[test]
fn chain_and_tree_agree() {
    let fits = Fits::new(0.0142);
    let addr = |i| {
        "u:t/sys:0"
            .parse::<Address>()
            .unwrap()
            .child("star", i)
            .unwrap()
    };
    let star = |i: u64, mass: f64, orbit: Option<(f64, f64, f64)>| Star {
        address: addr(i),
        mass,
        life: LifeLine::new(&fits, mass).unwrap(),
        orbit: orbit.map(|(a, e, inner)| BinaryOrbit {
            period_days: 0.0,
            semi_major_axis_au: a,
            eccentricity: e,
            inner_mass: inner,
        }),
    };
    let chain = [
        star(0, 1.1, None),
        star(1, 0.9, Some((23.0, 0.52, 1.1))),
        star(2, 0.12, Some((8700.0, 0.5, 2.0))),
    ];
    let tree = from_chain(&chain).unwrap();
    assert_eq!(tree.label(), "(0+1)+2");
    assert_eq!((tree.count(), tree.depth()), (3, 2));
    let c = check(&tree);
    let s = stability(&chain);
    // Root pair is the outer orbit: same ratio and critical value as the chain's tag.
    let tag = s[2].as_ref().unwrap();
    assert_eq!(c[0].ratio, Some(tag.ratio));
    assert_eq!(c[0].critical, Some(tag.critical));
    assert!(c[0].stable && tag.stable);
    assert_eq!(c[1].path, "a");
    assert!(c[1].ratio.is_none());
}

#[test]
fn pairs_of_pairs_nest_to_any_depth() {
    // A 2+2 quadruple: two close pairs 100 au apart is stable.
    let quad = Node::pair(
        Node::pair(b("Aa", 1.0), b("Ab", 0.8), 1.0, 0.1),
        Node::pair(b("Ba", 0.9), b("Bb", 0.7), 1.5, 0.2),
        100.0,
        0.3,
    );
    assert_eq!(quad.label(), "(Aa+Ab)+(Ba+Bb)");
    let c = check(&quad);
    assert_eq!(c.len(), 3);
    assert!(c.iter().all(|p| p.stable), "{c:?}");
    // The worst inner pair is the wider one (1.5 au).
    assert!((c[0].ratio.unwrap() - 100.0 / 1.5).abs() < 1e-9);
    // Pull the pairs to 4 au apart: tagged unstable, not refused.
    let tight = Node::pair(
        Node::pair(b("Aa", 1.0), b("Ab", 0.8), 1.0, 0.1),
        Node::pair(b("Ba", 0.9), b("Bb", 0.7), 1.5, 0.2),
        4.0,
        0.3,
    );
    let t = check(&tight);
    assert!(
        !t[0].stable && t[0].why.contains("unstable"),
        "{}",
        t[0].why
    );
    // Castor-like sextuple: three close pairs, A and B 100 au apart, C at
    // 1000 au. Depth 3, six bodies, every level stable.
    let castor = Node::pair(
        Node::pair(
            Node::pair(b("Aa", 2.4), b("Ab", 0.3), 0.12, 0.5),
            Node::pair(b("Ba", 1.8), b("Bb", 0.3), 0.04, 0.0),
            104.0,
            0.34,
        ),
        Node::pair(b("Ca", 0.6), b("Cb", 0.6), 0.018, 0.0),
        1000.0,
        0.3,
    );
    assert_eq!((castor.count(), castor.depth()), (6, 3));
    let cc = check(&castor);
    assert_eq!(cc.len(), 5);
    assert!(cc.iter().all(|p| p.stable), "{cc:?}");
    assert_eq!(
        cc.iter().map(|p| p.path.as_str()).collect::<Vec<_>>(),
        ["", "a", "aa", "ab", "b"]
    );
    // Round-trips as data, so an authored system is just JSON.
    let j = serde_json::to_string(&castor).unwrap();
    assert_eq!(serde_json::from_str::<Node>(&j).unwrap(), castor);
}

#[test]
fn inclination_lowers_the_critical_ratio() {
    let mk = |i: f64| Node::Pair {
        a: Box::new(Node::pair(b("A", 1.0), b("B", 1.0), 1.0, 0.0)),
        b: Box::new(b("C", 1.0)),
        orbit: PairOrbit {
            semi_major_axis_au: 3.5,
            eccentricity: 0.0,
            inclination_deg: i,
        },
    };
    let flat = &check(&mk(0.0))[0];
    let tilted = &check(&mk(90.0))[0];
    assert!(tilted.critical.unwrap() < flat.critical.unwrap());
    assert!((tilted.critical.unwrap() / flat.critical.unwrap() - 0.85).abs() < 1e-9);
}
