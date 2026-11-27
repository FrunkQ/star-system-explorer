#!/usr/bin/env python3
"""Write packs/base-physics/formulas/coorbital.json: Trojans (bodies sharing a
planet's orbit at its L4 and L5 points) as formula blocks, for any planet
round any star. Package CO1 (thread "Add co-orbital asteroids as general
physics", 26 Nov 2026).

Each block is a visible Equation with its source, its fitted numbers as
Model numbers and worked examples worked out here by the same formula.
Formation (crates/sse-formation/src/coorbital.rs) runs these blocks; it
types no number of its own.

Run: python3 scripts/build-coorbital-blocks.py           (write)
     python3 scripts/build-coorbital-blocks.py --check   (fail if stale)
"""
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "..", "packs", "base-physics", "formulas", "coorbital.json")

# Round figures for the worked examples only (the blocks read nothing typed
# here; the sheet's own values never appear as typed copies).
M_SUN = 2.0e30
M_JUP = 1.9e27
M_NEP = 1.02e26
M_URA = 8.7e25
M_SAT = 5.7e26
M_EARTH = 6.0e24

MD99 = "Murray, C. D. and Dermott, S. F. 1999, Solar System Dynamics, Cambridge University Press: section 3.8, the stability of the Lagrangian points (L4 and L5 are linearly stable for mu < 0.0385, the Gascheau and Routh condition 27 mu (1 - mu) < 1). doi:10.1017/CBO9781139174817"
DM81 = "Dermott, S. F. and Murray, C. D. 1981, The dynamics of tadpole and horseshoe orbits I. Theory, Icarus 48, 1: tadpoles within (8 mu / 3)^(1/2) of the shared orbit. doi:10.1016/0019-1035(81)90147-0"
LH10 = "Lykawka, P. S. and Horner, J. 2010, The capture of Trojan asteroids by the giant planets during planetary migration, MNRAS 405, 1375 (arXiv:1003.2137), abstract: a disc planetesimal is captured as a Trojan with probability several times 1e-6 to 1e-5 at Jupiter, below 1e-6 to 1e-5 at Saturn, 1e-5 to 1e-4 at Uranus and 1e-4 to 1e-3 at Neptune; the bulk of the captured objects are unstable and decay over the age of the system. doi:10.1111/j.1365-2966.2010.16538.x"
NVM13 = "Nesvorny, D., Vokrouhlicky, D. and Morbidelli, A. 2013, Capture of Trojans by jumping Jupiter, ApJ 768, 45: Jupiter's present Trojans are about 6e-7 to 8e-7 of the outer planetesimal disc. doi:10.1088/0004-637X/768/1/45"
SCHWAMB26 = "Schwamb, M. E. et al. 2026, NSF-DOE Vera C. Rubin Observatory's first view of the Neptune Trojan L5 (trailing) cloud: two new long-term stable Trojans, ApJL 1010, L36, section 1: Neptune's Trojans are estimated at 4 to 10 times as many as Jupiter's; long-lived ones are primordial, stable for gigayears. doi:10.3847/2041-8213/aead9a"
DLFM24 = "de la Fuente Marcos, R. et al. 2024, Dynamics of 2023 FW14, the second L4 Mars trojan, A&A 683, L14 (arXiv:2403.04061): Mars keeps probably primordial Trojans at L5 (the Eureka family); passing Mars-crossers are captured as temporary Trojans for millions of years. doi:10.1051/0004-6361/202449688"
EMERY24 = "Emery, J. P. et al. 2024, Surface compositions of Trojan asteroids, Space Science Reviews 220, 28, section 1: Jupiter's Trojans are stable over the age of the Solar System (Levison et al. 1997); capture in the Nice model's migration (Morbidelli et al. 2005, Nesvorny et al. 2013) empties and refills the Lagrange regions. doi:10.1007/s11214-024-01060-7"


def model(v, grade, src):
    return {"value": v, "grade": grade, "source": src}


def inp(kind, unit, name, valid=None):
    d = {"kind": kind, "unit": unit, "name": name}
    if valid is not None:
        d["valid"] = valid
    return d


# ------------------------------------------------------------ the zone


def zone(m, M):
    mu = m / (M + m)
    stable = 1 if 27 * mu * (1 - mu) < 1 else 0
    width = math.sqrt(8 * mu / 3)
    return {"mu": mu, "stable": stable, "width": width}


def zone_atom():
    exs = []
    for m, M in [(M_JUP, M_SUN), (M_NEP, M_SUN), (M_EARTH, M_SUN), (1.0e29, 1.0e30)]:
        r = zone(m, M)
        exs.append({"m": m, "M": M, **r})
    return {
        "id": "sse.coorbital.trojan_zone",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "planet_mass", "letter": "m"},
            {"port": "host_mass", "letter": "M"},
        ],
        "outputs": [
            {"port": "mass_ratio", "letter": "mu"},
            {"port": "points_stable", "letter": "stable"},
            {"port": "tadpole_width", "letter": "width"},
        ],
        "positive": [
            {"port": "planet_mass", "says": "planet mass"},
            {"port": "host_mass", "says": "host mass"},
        ],
        "references": [MD99, DM81],
        "shown": "μ = m / (M + m);  L4, L5 hold if 27 μ (1 − μ) < 1;  Δa/a = (8μ/3)^½",
        "equation": {
            "name": "Whether a world's leading and trailing points can hold Trojans, and how wide the swarm is",
            "formula": "mu = m / (M + m)\n"
                       "stable = if 27 * mu * (1 - mu) < 1 then 1 else 0\n"
                       "width = sqrt(8 * mu / 3)",
            "inputs": {
                "m": inp("body.mass", "kg", "mass of the world"),
                "M": inp("orbit.host_mass", "kg", "mass of what it circles"),
            },
            "outputs": {
                "mu": inp("orbit.mass_ratio", "1", "the world's share of the pair's mass"),
                "stable": inp("orbit.lagrange_stable", "1", "1 if the points 60 degrees ahead and behind hold things, 0 if not"),
                "width": inp("orbit.tadpole_width", "1", "how far a Trojan's orbit size swings, as a share of the world's"),
            },
            "source": {
                "doi": "10.1017/CBO9781139174817",
                "text": "Gascheau 1843 and Routh 1875, in Murray and Dermott 1999: the points 60 degrees ahead and behind a world are stable while 27 mu (1 - mu) < 1, about mu < 0.0385 (one part in 26). Dermott and Murray 1981: tadpole orbits stay within (8 mu / 3)^(1/2) of the shared orbit",
                "where": "Murray and Dermott 1999 section 3.8; Dermott and Murray 1981 section 4",
            },
            "explain": {
                "curious": "Any world drags two quiet spots round its star with it, 60 degrees ahead and 60 degrees behind. Small bodies caught there stay for billions of years, as long as the world is under about a 26th of its star's mass. Heavier worlds get a wider swarm.",
                "science": "In the restricted three-body problem L4 and L5 are linearly stable for 27 mu (1 - mu) < 1 (Gascheau, Routh). Tadpole orbits librate about them with a maximum half-width in semi-major axis of (8 mu / 3)^(1/2) a (Dermott and Murray 1981). Jupiter: about 0.05; Neptune: about 0.012; Earth: about 0.003.",
            },
            "examples": exs,
        },
    }


# ------------------------------------------------------------ capture


K_CAPTURE = 0.027
H_HANDOFF = 8.0


def capture(width, steps, k=K_CAPTURE, h=H_HANDOFF):
    return {"share": k * width * h ** (-steps)}


def capture_atom():
    exs = []
    for w, s in [(0.0117, 0), (0.0108, 1), (0.0276, 2), (0.0503, 3)]:
        exs.append({"width": w, "steps": s, **capture(w, s)})
    return {
        "id": "sse.coorbital.capture_share",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "tadpole_width", "letter": "width"},
            {"port": "worlds_between", "letter": "steps"},
        ],
        "outputs": [{"port": "captured_share", "letter": "share"}],
        "models": {
            "k": model(K_CAPTURE, "fitted", LH10),
            "h": model(H_HANDOFF, "fitted", LH10),
        },
        "references": [LH10, NVM13, EMERY24],
        "shown": "share = k · (Δa/a) · h^(−n)",
        "equation": {
            "name": "Share of a stirred belt a world catches into its leading and trailing points",
            "formula": "share = k * width * h ^ (-steps)",
            "inputs": {
                "width": inp("orbit.tadpole_width", "1", "how wide the world's Trojan swarm is"),
                "steps": inp("orbit.giants_between", "1", "how many giant worlds sit between this world and the stirred belt", [0, 20]),
                "k": inp("model.trojan_capture_per_width", "1", "catch per unit swarm width, for the world nearest the belt"),
                "h": inp("model.trojan_capture_handoff", "1", "how much the flow of scattered bodies thins past each giant"),
            },
            "outputs": {
                "share": inp("formation.captured_share", "1", "share of the stirred belt caught as Trojans"),
            },
            "source": {
                "doi": "10.1111/j.1365-2966.2010.16538.x",
                "text": "Lykawka and Horner 2010: all four giants catch Trojans from the disc their migration scatters, Neptune most (1e-4 to 1e-3 of the disc), then Uranus (1e-5 to 1e-4), Saturn (below 1e-6 to 1e-5) and Jupiter (several times 1e-6 to 1e-5)",
                "where": "Lykawka and Horner 2010, abstract",
            },
            "explain": {
                "curious": "When a giant ploughs through a belt, every world nearby can catch some of the scattered bodies into its quiet points. The world nearest the belt sees the most of them; each giant further in sees fewer, because the ones before it have already flung most away.",
                "science": "The catch is the swarm's width times the flow of scattered bodies reaching the world. The flow thins by a factor h past each giant between the world and the belt. k and h are fitted to Lykawka and Horner 2010's four ranges: Neptune 3.2e-4, Uranus 3.6e-5, Saturn 1.2e-5 (just above their range) and Jupiter 2.7e-6. Our simplification: fudge register CO-capture.",
            },
            "examples": exs,
        },
    }


# ------------------------------------------------------------ survival


C_KEEP = 277.0


def keep(mu, disturbed, c=C_KEEP):
    return {"share": 0.0 if disturbed > 0.5 else min(1.0, c * mu)}


def keep_atom():
    exs = []
    for mu, d in [(9.55e-4, 0), (5.15e-5, 0), (2.86e-4, 1), (1.0e-2, 0)]:
        exs.append({"mu": mu, "disturbed": d, **keep(mu, d)})
    return {
        "id": "sse.coorbital.long_lived_share",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "mass_ratio", "letter": "mu"},
            {"port": "disturbed", "letter": "disturbed"},
        ],
        "outputs": [{"port": "long_lived_share", "letter": "share"}],
        "models": {"c": model(C_KEEP, "fitted", NVM13)},
        "references": [NVM13, LH10, SCHWAMB26, EMERY24],
        "shown": "share = min(1, c μ), or 0 if a neighbour disturbs",
        "equation": {
            "name": "Share of a world's caught Trojans that stay for billions of years",
            "formula": "share = if disturbed > 1 / 2 then 0 else min(1, c * mu)",
            "inputs": {
                "mu": inp("orbit.mass_ratio", "1", "the world's share of the pair's mass"),
                "disturbed": inp("orbit.coorbital_disturbed", "1", "1 if a neighbour shakes the swarm loose, 0 if not", [0, 1]),
                "c": inp("model.trojan_keep_per_mass_ratio", "1", "share kept per unit mass ratio"),
            },
            "outputs": {
                "share": inp("formation.long_lived_share", "1", "share of the catch still there after billions of years"),
            },
            "source": {
                "doi": "10.1088/0004-637X/768/1/45",
                "text": "Lykawka and Horner 2010: most caught Trojans are unstable and leak away. Nesvorny et al. 2013: Jupiter keeps about 7e-7 of the disc. Schwamb et al. 2026: Neptune's long-lived swarm is 4 to 10 times Jupiter's",
                "where": "Lykawka and Horner 2010 abstract; Nesvorny et al. 2013 abstract; Schwamb et al. 2026 section 1",
            },
            "explain": {
                "curious": "Bodies are caught with a spread of swings. Only the ones whose swing fits inside the quiet zone stay; a heavier world has a roomier zone, so it keeps more. A world shaken by a heavier neighbour keeps none for long.",
                "science": "The stable region is (8 mu / 3)^(1/2) wide in orbit size and about as wide in eccentricity, so its share of a catch spread over a fixed range grows as mu. c is fitted to Jupiter alone (2.7e-6 caught, 7e-7 kept: 0.26 at mu = 9.5e-4). Neptune then keeps 1.4% of 3.2e-4, about 6.5 times Jupiter's swarm, inside Schwamb et al.'s 4 to 10. Our simplification: fudge register CO-keep.",
            },
            "examples": exs,
        },
    }


# ------------------------------------------------------------ neighbours


NEAR = 0.03
HEAVY = 1.0


def disturbs(ratio, p, q, mn, mh, near=NEAR, heavy=HEAVY):
    offset = abs(ratio * q / p - 1)
    d = 1 if (offset < near and mn >= heavy * mh) else 0
    return {"offset": offset, "disturbed": d}


def disturbs_atom():
    exs = []
    # Saturn by Jupiter (near 5:2), Uranus by Neptune (near 2:1),
    # Neptune by Uranus (lighter), Mars by Earth (no near resonance).
    for ratio, p, q, mn, mh in [
        (2.484, 5, 2, M_JUP, M_SAT),
        (1.961, 2, 1, M_NEP, M_URA),
        (1.961, 2, 1, M_URA, M_NEP),
        (1.881, 2, 1, M_EARTH, 6.4e23),
    ]:
        exs.append({"ratio": ratio, "p": p, "q": q, "mn": mn, "mh": mh, **disturbs(ratio, p, q, mn, mh)})
    return {
        "id": "sse.coorbital.neighbour_disturbs",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "period_ratio", "letter": "ratio"},
            {"port": "resonance_outer", "letter": "p"},
            {"port": "resonance_inner", "letter": "q"},
            {"port": "neighbour_mass", "letter": "mn"},
            {"port": "planet_mass", "letter": "mh"},
        ],
        "outputs": [
            {"port": "resonance_offset", "letter": "offset"},
            {"port": "disturbed", "letter": "disturbed"},
        ],
        "models": {
            "near": model(NEAR, "assumed", LH10),
            "heavy": model(HEAVY, "assumed", LH10),
        },
        "positive": [
            {"port": "period_ratio", "says": "period ratio"},
            {"port": "resonance_outer", "says": "outer resonance number"},
            {"port": "resonance_inner", "says": "inner resonance number"},
            {"port": "planet_mass", "says": "planet mass"},
        ],
        "references": [LH10, EMERY24, DLFM24],
        "shown": "disturbed if |P₂/P₁ · q/p − 1| < near and m_n ≥ heavy · m",
        "equation": {
            "name": "Whether a neighbour shakes a world's Trojans loose",
            "formula": "offset = abs(ratio * q / p - 1)\n"
                       "disturbed = if offset < near and mn >= heavy * mh then 1 else 0",
            "inputs": {
                "ratio": inp("orbit.period_ratio", "1", "the outer world's year over the inner world's"),
                "p": inp("orbit.resonance_p", "1", "the nearest simple resonance, outer number", [1, 20]),
                "q": inp("orbit.resonance_q", "1", "the nearest simple resonance, inner number", [1, 20]),
                "mn": inp("body.mass", "kg", "mass of the neighbour"),
                "mh": inp("body.mass", "kg", "mass of the world holding the Trojans"),
                "near": inp("model.coorbital_near_resonance", "1", "how close to a simple resonance counts as near"),
                "heavy": inp("model.coorbital_heavy_neighbour", "1", "how heavy, against the world, a neighbour must be to disturb"),
            },
            "outputs": {
                "offset": inp("orbit.resonance_offset", "1", "how far the year ratio is from the resonance, as a share"),
                "disturbed": inp("orbit.coorbital_disturbed", "1", "1 if this neighbour shakes the swarm loose, 0 if not"),
            },
            "source": {
                "doi": "10.1111/j.1365-2966.2010.16538.x",
                "text": "Only Jupiter and Neptune of the four giants hold long-lived swarms, though all four catch them (Lykawka and Horner 2010). Mars keeps a primordial L5 group (de la Fuente Marcos et al. 2024). Our rule: a swarm is shaken loose when an adjacent world at least as heavy sits within 3% of a resonance of order 3 or less",
                "where": "judged by this package against the Solar System; fudge register CO-neighbour",
            },
            "explain": {
                "curious": "A heavier neighbour whose year nearly fits a simple ratio with this world's keeps tugging on the swarm at the same place, orbit after orbit, until the Trojans fall out.",
                "science": "Saturn sits 0.6% from 5:2 with Jupiter, 3.3 times heavier; Uranus 2% from 2:1 with Neptune, 1.2 times heavier; Venus 1.6% from 8:5 with Earth; Mercury 2.2% from 5:2 with Venus. All four lose their swarms. Jupiter, Neptune, Earth and Mars have no heavier neighbour so near, and keep theirs. A judged rule, not a fit: fudge register CO-neighbour.",
            },
            "examples": exs,
        },
    }


def build():
    return {
        "format": "sse.formulas/1",
        "family": "sse.coorbital",
        "about": "Trojans: bodies sharing a world's orbit at its L4 and L5 points, for any world round any star (package CO1). Which worlds can hold them, how wide the swarm is, how much a migration catches and how much stays.",
        "atoms": [zone_atom(), capture_atom(), keep_atom(), disturbs_atom()],
    }


def main():
    text = json.dumps(build(), indent=2, ensure_ascii=False) + "\n"
    if "--check" in sys.argv:
        with open(OUT, encoding="utf-8") as f:
            if f.read() != text:
                sys.exit("coorbital.json is stale: run python3 scripts/build-coorbital-blocks.py")
        return
    with open(OUT, "w", encoding="utf-8") as f:
        f.write(text)


if __name__ == "__main__":
    main()
