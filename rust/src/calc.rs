//! Physics for a single-layer air-core coil. Equations follow `docs/notes.md`.

use std::f64::consts::PI;

pub const MM_PER_IN: f64 = 25.4;

/// Medhurst H-factor vs l/d, from docs/notes.md. Clamped at its ends; values
/// for l/d < 1 are deliberately left out (they push Cs error beyond ~9%).
pub const H_TABLE: [(f64, f64); 5] = [
    (1.0, 0.46),
    (1.5, 0.47),
    (2.0, 0.50),
    (3.0, 0.61),
    (5.0, 0.81),
];

/// Gauges offered for the wire-diameter picker.
pub const AWG_RANGE: std::ops::RangeInclusive<u32> = 10..=36;

/// Coil geometry. All lengths in mm.
#[derive(Clone, Copy, Debug)]
pub struct Coil {
    pub d: f64,
    pub l: f64,
    pub n: f64,
    pub w: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Results {
    pub inductance_uh: f64,
    pub ratio: f64,
    pub h: f64,
    pub cs_pf: f64,
    pub srf_hz: f64,
    pub n_max: f64,
    /// mm
    pub pitch: f64,
    /// mm
    pub close_len: f64,
    /// mm, centre of wire, excluding leads
    pub wire_len: f64,
}

pub fn h_factor(ratio: f64) -> f64 {
    if ratio <= H_TABLE[0].0 {
        return H_TABLE[0].1;
    }
    for pair in H_TABLE.windows(2) {
        let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
        if ratio <= x1 {
            return y0 + (y1 - y0) * (ratio - x0) / (x1 - x0);
        }
    }
    H_TABLE[H_TABLE.len() - 1].1
}

/// Wheeler: L (μH) = d²n² / (18d + 40l), d and l in inches.
pub fn wheeler_uh(d_in: f64, l_in: f64, n: f64) -> f64 {
    (d_in * d_in * n * n) / (18.0 * d_in + 40.0 * l_in)
}

/// Wheeler transposed: n = √(L·(18d + 40l)) / d, d and l in inches.
pub fn turns_for_uh(l_uh: f64, d_in: f64, l_in: f64) -> f64 {
    (l_uh * (18.0 * d_in + 40.0 * l_in)).sqrt() / d_in
}

/// AWG bare-copper diameter in mm, rounded to 3 decimals:
/// d(mm) = 0.127 · 92^((36 − AWG) / 39)
pub fn awg_mm(gauge: u32) -> f64 {
    let mm = 0.127 * 92f64.powf((36.0 - gauge as f64) / 39.0);
    (mm * 1000.0).round() / 1000.0
}

/// The gauge whose diameter matches `w_mm`, if any.
pub fn awg_for(w_mm: f64) -> Option<u32> {
    AWG_RANGE
        .clone()
        .find(|&g| (awg_mm(g) - w_mm).abs() < 0.0005)
}

impl Coil {
    pub fn d_in(&self) -> f64 {
        self.d / MM_PER_IN
    }

    pub fn l_in(&self) -> f64 {
        self.l / MM_PER_IN
    }

    pub fn calc(&self) -> Results {
        let Coil { d, l, n, w } = *self;
        let inductance_uh = wheeler_uh(self.d_in(), self.l_in(), n);
        let ratio = l / d;
        let h = h_factor(ratio);
        let cs_pf = h * d / 10.0;
        let srf_hz = 1.0 / (2.0 * PI * (inductance_uh * 1e-6 * cs_pf * 1e-12).sqrt());
        Results {
            inductance_uh,
            ratio,
            h,
            cs_pf,
            srf_hz,
            n_max: d / w,
            pitch: l / n,
            close_len: n * w,
            wire_len: n * PI * (d + w),
        }
    }

    pub fn turns_for_uh(&self, l_uh: f64) -> f64 {
        turns_for_uh(l_uh, self.d_in(), self.l_in())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn wheeler_one_inch_coil() {
        assert!(close(wheeler_uh(1.0, 1.0, 10.0), 100.0 / 58.0));
    }

    #[test]
    fn turns_round_trip() {
        let coil = Coil {
            d: 20.0,
            l: 30.0,
            n: 17.5,
            w: 0.8,
        };
        let l = coil.calc().inductance_uh;
        assert!(close(coil.turns_for_uh(l), 17.5));
    }

    #[test]
    fn h_factor_interpolates_and_clamps() {
        assert!(close(h_factor(0.5), 0.46));
        assert!(close(h_factor(1.25), 0.465));
        assert!(close(h_factor(4.0), 0.71));
        assert!(close(h_factor(9.0), 0.81));
    }

    #[test]
    fn awg_table() {
        assert!(close(awg_mm(36), 0.127));
        assert!(close(awg_mm(10), 2.588));
        assert_eq!(awg_for(0.511), Some(24));
        assert_eq!(awg_for(0.8), None);
    }
}
