"""Optical constants of condensates for the renderer's bands, clouds, hazes and
ground stains (package MP33, asked by the giant-bands and plumes threads).

Shape: optical.refractive_index = {kind table, lambda_nm, n, k, src}; the
absorption coefficient follows as alpha = 4 pi k / lambda.

  titan-tholin   organic haze made by a DC discharge through 90 % N2 and
                 10 % CH4 at 0.2 mbar, room temperature (Khare et al. 1984,
                 Table I, 0.26 to 2.42 micron)

The Table I values were read on 24 Nov 2026 from the text layer of the
HITRAN aerosol archive's copy of the paper, which drops the minus sign of
each exponent: k is printed as a mantissa and an exponent b with k = a x 10^-b
(the paper's own text gives k near 4e-4 at 1.5 micron and a few hundredths in
the visible, which fixes the sign). Unverified until checked against the
page image (page 133).
"""

SOURCES = {
    "khare1984": ("Khare, B. N., Sagan, C., Arakawa, E. T., Suits, F., Callcott, T. A. and Williams, M. W. 1984, Icarus 60, 127: optical constants of organic tholins produced in a simulated Titanian atmosphere, from soft X-ray to microwave frequencies; Table I, n and k = a 10^b (n to +/-0.05, k to +/-30 %)", "10.1016/0019-1035(84)90142-8", "https://hitran.org/data/Aerosols/Aerosols%20data%20from%20previous%20HITRAN/Aerosols-2016/papers/khare_tholins_1984.pdf", "unverified"),
}

# Khare et al. 1984 Table I as read: (micron, k mantissa, k exponent b, n), k = a 10^-b.
KHARE = [
    (0.2638, 1.8, 1, 1.68), (0.2952, 1.5, 1, 1.66), (0.3263, 1.3, 1, 1.64), (0.3542, 1.1, 1, 1.63),
    (0.3874, 9.1, 2, 1.66), (0.4133, 7.6, 2, 1.69), (0.4428, 6.0, 2, 1.72), (0.5635, 2.3, 2, 1.70),
    (0.6888, 8.8, 3, 1.68), (0.8731, 2.4, 3, 1.66), (1.016, 1.0, 3, 1.65), (1.148, 6.4, 4, 1.65),
    (1.192, 5.2, 4, 1.65), (1.357, 4.2, 4, 1.64), (1.381, 4.1, 4, 1.64), (1.802, 4.0, 4, 1.64),
    (1.813, 4.2, 4, 1.64), (1.823, 4.4, 4, 1.64), (1.873, 5.2, 4, 1.63), (2.019, 7.2, 4, 1.63),
    (2.214, 8.9, 4, 1.63), (2.393, 8.0, 4, 1.62), (2.403, 8.5, 4, 1.62), (2.422, 1.2, 3, 1.62),
]


def apply(mixtures):
    """Return the new mixture rows."""
    U = "unverified"
    lam = [um * 1000.0 for um, _, _, _ in KHARE]
    assert lam == sorted(lam)
    k = [float("%.3g" % (a * 10.0 ** -b)) for _, a, b, _ in KHARE]
    # The visible absorbs far more than the near infrared: that is what makes the haze orange.
    assert k[6] > 50 * k[15]  # 443 nm against 1802 nm
    tholin = {
        "id": "titan-tholin",
        "names": {"physics": "Titan tholin (N2-CH4 discharge organics)", "plain": "Titan's orange haze stuff", "story": "tholin"},
        "recipe": {}, "by": "mass",
        "note": "a laboratory stand-in for Titan's organic haze and for the red-brown stains on icy worlds; real hazes differ with how they were made",
        "optical": {"refractive_index": {
            "kind": "table", "lambda_nm": [round(x, 1) for x in lam], "n": [n for _, _, _, n in KHARE], "k": k,
            "src": "khare1984", "check": U, "at_k": 300.0,
            "note": "thin films and pellets of tholin from 0.9 N2 / 0.1 CH4 at 0.2 mbar, room temperature; alpha = 4 pi k / lambda",
        }},
    }
    assert all(m["id"] != tholin["id"] for m in mixtures)
    return [tholin]
