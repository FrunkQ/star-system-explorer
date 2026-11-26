"""How long sunlight lets a molecule last (stray-numbers sweep, 26 Nov 2026).

Each gas row may carry `photochemistry.lifetime_1au`: the time, in seconds,
before light from a quiet Sun at 1 au breaks the molecule up or ionises it
(every destruction channel together). It grows as the square of the distance
from the star. Read by:

  - the ice ground block (`sse.surface.open_ice_loss`, via sse-surface): the
    share of a sublimed water molecule's hops that sunlight breaks up;
  - the comets of sse-small: each coma gas's scale length and the ion tail.

Before this both readers typed their own copy (a rate of 1.2e-5 per second in
sse-atoms, lifetimes in packs/base-small-bodies). The numbers are the
quiet-Sun values as the planetary literature quotes them from Huebner, Keady
and Lyon 1992, at two figures; they were recalled, not read from the paper's
tables (paper search is closed until 1 Dec 2026), so each is unverified.
Water's 8.3e4 s is the inverse of the old 1.2e-5 per second rounded the same
way (1 / 1.2e-5 = 8.33e4 s), so the ice ground's rate moves by 0.4 %.
"""

SOURCES = {
    "huebner1992": ("Huebner, W. F., Keady, J. J. and Lyon, S. P. 1992, Astrophysics and Space Science 195, 1: solar photo rates for planetary atmospheres and atmospheric pollutants (quiet- and active-Sun rate coefficients at 1 au for every photodissociation and photoionisation branch)", "10.1007/BF00644558", None, "unverified"),
}

# id, lifetime at 1 au under the quiet Sun (s), note.
LIFETIMES = [
    ("h2o", 8.3e4, "quiet Sun, all branches (mostly H2O to H + OH); about a day. The active Sun's is about half"),
    ("co2", 5.0e5, "quiet Sun, all branches (mostly CO2 to CO + O)"),
    ("co", 1.3e6, "quiet Sun, all branches (dissociation and ionisation to CO+)"),
]


def apply(substances, val):
    by_id = {s["id"]: s for s in substances}
    for sid, life, note in LIFETIMES:
        row = by_id[sid]
        row.setdefault("photochemistry", {})["lifetime_1au"] = val(
            life, "s", "huebner1992", "unverified", at_place="1 au from a quiet Sun; scales as the distance squared", note=note)
