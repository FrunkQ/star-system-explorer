//! Dust between a star and the observer dims and reddens it (night sky
//! design 4.3, stage Y2, light side). One block: the Cardelli, Clayton and
//! Mathis extinction curve, A_λ / A_V as a function of wavelength and the
//! dust's R_V. The amount of dust along a sight line (A_V) comes from the
//! patch dust model; this block only says how that dimming spreads across
//! wavelengths.

use crate::common::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

const CCM89: &str = "Cardelli, J. A., Clayton, G. C. and Mathis, J. S. 1989, The relationship between infrared, optical, and ultraviolet extinction, ApJ 345, 245: A_λ/A_V = a(x) + b(x)/R_V, x = 1/λ in µm⁻¹, valid 0.3 to 10 µm⁻¹; R_V = 3.1 for the diffuse interstellar medium. doi:10.1086/167900";

/// Extinction at wavelength λ (m) relative to the V band, A_λ / A_V, for
/// dust with total-to-selective ratio R_V (CCM89). Beyond the fitted range
/// it is held at the nearest end in the far ultraviolet and falls to zero
/// as x^1.61 in the far infrared, as the infrared form does.
pub fn ccm_ratio(lambda_m: f64, r_v: f64) -> f64 {
    let x = (1e-6 / lambda_m).clamp(0.0, 10.0);
    let (a, b) = if x < 1.1 {
        let p = sse_core::math::pow(x, 1.61);
        (0.574 * p, -0.527 * p)
    } else if x < 3.3 {
        let y = x - 1.82;
        let a = 1.0
            + y * (0.17699
                + y * (-0.50447
                    + y * (-0.02427
                        + y * (0.72085 + y * (0.01979 + y * (-0.77530 + y * 0.32999))))));
        let b = y
            * (1.41338
                + y * (2.28305
                    + y * (1.07233
                        + y * (-5.38434 + y * (-0.62251 + y * (5.30260 + y * -2.09002))))));
        (a, b)
    } else if x < 8.0 {
        let (fa, fb) = if x >= 5.9 {
            let z = x - 5.9;
            (
                -0.04473 * z * z - 0.009779 * z * z * z,
                0.2130 * z * z + 0.1207 * z * z * z,
            )
        } else {
            (0.0, 0.0)
        };
        let a = 1.752 - 0.316 * x - 0.104 / ((x - 4.67) * (x - 4.67) + 0.341) + fa;
        let b = -3.090 + 1.825 * x + 1.206 / ((x - 4.62) * (x - 4.62) + 0.263) + fb;
        (a, b)
    } else {
        let z = x - 8.0;
        let a = -1.073 - 0.628 * z + 0.137 * z * z - 0.070 * z * z * z;
        let b = 13.670 + 4.257 * z - 0.420 * z * z + 0.374 * z * z * z;
        (a, b)
    };
    (a + b / r_v).max(0.0)
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    vec![Def {
        id: "sse.light.dust_extinction",
        version: "1.0.0",
        inputs: vec![port("wavelength", "light.wavelength", D::LENGTH, Some((1e-7, 1e-3)))],
        outputs: vec![port("ratio", "dust.extinction_ratio", D::NONE, None)],
        params: vec![param("r_v", D::NONE, 3.1, (2.0, 6.0), "R_V, total over selective extinction: 3.1 for diffuse dust, up to about 5 in dense clouds with larger grains")],
        cost: CostClass::Trivial,
        references: &[CCM89],
        curious: "Dust dims blue light more than red, so a star seen through dust looks redder as well as fainter. In infrared light the same dust is nearly clear, which is how we see the galaxy's centre.",
        science: "The extinction curve A_λ/A_V of Galactic dust follows one family set by R_V: polynomial fits in inverse wavelength across the infrared, optical and ultraviolet, with the 217.5 nm bump.",
        formula: "A_λ / A_V = a(x) + b(x) / R_V,  x = 1/λ (µm⁻¹)",
        f: |i, p| Ok(vec![ccm_ratio(positive(i[0], "wavelength")?, p[0])]),
    }
    .build()]
}
