// Copyright (c) 2021 Björn Ottosson

// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is furnished to do
// so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![allow(
    non_snake_case,
    clippy::many_single_char_names,
    clippy::cast_possible_truncation
)]

use std::f32::consts::{PI, TAU};
#[derive(Clone, Debug)]
pub struct Lab {
    pub L: f32,
    pub a: f32,
    pub b: f32,
}

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

// Alternative representation of (L_cusp, C_cusp)
// Encoded so S = C_cusp/L_cusp and T = C_cusp/(1-L_cusp)
// The maximum value for C in the triangle is then found as fmin(S*L, T*(1-L)), for a given L
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct ST {
    pub s: f32,
    pub T: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct LinRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<&Rgb> for LinRgb {
    fn from(rgb: &Rgb) -> Self {
        fn to_linear_srgb(x: u8) -> f32 {
            let x = f32::from(x) / 255.0;
            if x >= 0.003_130_8 {
                (1.055) * x.powf(1.0 / 2.4) - 0.055
            } else {
                12.92 * x
            }
        }

        Self {
            r: to_linear_srgb(rgb.r),
            g: to_linear_srgb(rgb.g),
            b: to_linear_srgb(rgb.b),
        }
    }
}

impl From<&LinRgb> for Rgb {
    fn from(rgb: &LinRgb) -> Self {
        #[allow(clippy::cast_sign_loss)]
        fn from_linear_srgb(x: f32) -> u8 {
            let y = if x >= 0.04045 {
                ((x + 0.055) / (1.0 + 0.055)).powf(2.4)
            } else {
                x / 12.92
            };
            (y * 255.0).round() as u8
        }

        Self {
            r: from_linear_srgb(rgb.r),
            g: from_linear_srgb(rgb.g),
            b: from_linear_srgb(rgb.b),
        }
    }
}

// toe function for L_r
fn toe(x: f32) -> f32 {
    let k_1: f32 = 0.206;
    let k_2: f32 = 0.03;
    let k_3: f32 = (1.0 + k_1) / (1.0 + k_2);
    0.5 * (k_3 * x - k_1
        + (k_3 * x - k_1)
            .mul_add(k_3 * x - k_1, 4.0 * k_2 * k_3 * x)
            .sqrt())
}

// inverse toe function for L_r
fn toe_inv(x: f32) -> f32 {
    let k_1: f32 = 0.206;
    let k_2: f32 = 0.03;
    let k_3: f32 = (1.0 + k_1) / (1.0 + k_2);
    x.mul_add(x, k_1 * x) / (k_3 * (x + k_2))
}

impl From<&LC> for ST {
    fn from(cusp: &LC) -> Self {
        let L = cusp.L;
        let C = cusp.C;
        Self {
            s: C / L,
            T: C / (1.0 - L),
        }
    }
}

impl From<&Hsv> for Lab {
    fn from(hsv: &Hsv) -> Self {
        let Hsv { h, s, v } = hsv;

        let a_ = (TAU * h).cos();
        let b_ = (TAU * h).sin();

        let cusp = find_cusp(a_, b_);
        let ST_max: ST = (&cusp).into();
        let S_max = ST_max.s;
        let T_max = ST_max.T;
        let S_0 = 0.5;
        let k = 1.0 - S_0 / S_max;

        // first we compute L and V as if the gamut is a perfect triangle:

        // L, C when v==1:
        let L_v = 1.0 - s * S_0 / (S_0 + T_max - T_max * k * s);
        let C_v = s * T_max * S_0 / (S_0 + T_max - T_max * k * s);

        let mut L = v * L_v;
        let mut C = v * C_v;

        // then we compensate for both toe and the curved top part of the triangle:
        let L_vt = toe_inv(L_v);
        let C_vt = C_v * L_vt / L_v;

        let L_new = toe_inv(L);
        C = C * L_new / L;
        L = L_new;

        let rgb_scale = LinRgb::from(&Self {
            L: L_vt,
            a: a_ * C_vt,
            b: b_ * C_vt,
        });

        let scale_L = (1.0 / rgb_scale.r.max(rgb_scale.g).max(rgb_scale.b).max(0.0)).cbrt();

        L *= scale_L;
        C *= scale_L;

        Self {
            L,
            a: C * a_,
            b: C * b_,
        }
    }
}

impl From<&Lab> for Hsv {
    fn from(lab: &Lab) -> Self {
        let C = lab.a.hypot(lab.b);
        let a_ = lab.a / C;
        let b_ = lab.b / C;

        let mut L = lab.L;
        let h = 0.5 + 0.5 * (-lab.b).atan2(-lab.a) / PI;

        let cusp = find_cusp(a_, b_);
        let ST_max = ST::from(&cusp);
        let S_max = ST_max.s;
        let T_max = ST_max.T;
        let S_0 = 0.5;
        let k = 1.0 - S_0 / S_max;

        // first we find L_v, C_v, L_vt and C_vt

        let t = T_max / L.mul_add(T_max, C);
        let L_v = t * L;
        let C_v = t * C;

        let L_vt = toe_inv(L_v);
        let C_vt = C_v * L_vt / L_v;

        let rgb_scale = LinRgb::from(&Lab {
            L: L_vt,
            a: a_ * C_vt,
            b: b_ * C_vt,
        });
        let scale_L = (1.0 / rgb_scale.r.max(rgb_scale.g).max(rgb_scale.b).max(0.0)).cbrt();

        L /= scale_L;
        // C = C / scale_L;

        // C = C * toe(L) / L;
        L = toe(L);

        let v = L / L_v;
        let s = (S_0 + T_max) * C_v / T_max.mul_add(S_0, T_max * k * C_v);

        Self { h, s, v }
    }
}

impl From<&Lab> for LinRgb {
    fn from(color: &Lab) -> Self {
        let l_ = 0.215_803_76_f32.mul_add(color.b, 0.396_337_78_f32.mul_add(color.a, color.L));
        let m_ = color.L - 0.105_561_346 * color.a - 0.063_854_17 * color.b;
        let s_ = color.L - 0.089_484_18 * color.a - 1.291_485_5 * color.b;

        let l = l_.powi(3);
        let m = m_.powi(3);
        let s = s_.powi(3);

        Self {
            r: 0.230_759_05_f32.mul_add(s, 4.076_724_5 * l - 3.307_217 * m),
            g: (-1.268_143_8_f32).mul_add(l, 2.609_332_3 * m) - 0.341_134_43 * s,
            b: 1.706_862_6_f32.mul_add(s, -0.004_111_988_5 * l - 0.703_476_3 * m),
        }
    }
}

impl From<&LinRgb> for Lab {
    fn from(color: &LinRgb) -> Self {
        let r = color.r;
        let g = color.g;
        let b = color.b;

        let l_ = 0.051_457_565_f32.mul_add(b, 0.412_165_6_f32.mul_add(r, 0.536_275_2 * g));
        let m_ = 0.107_406_58_f32.mul_add(b, 0.211_859_1_f32.mul_add(r, 0.680_718_96 * g));
        let s_ = 0.630_261_36_f32.mul_add(b, 0.088_309_795_f32.mul_add(r, 0.281_847_42 * g));

        let l = l_.cbrt();
        let m = m_.cbrt();
        let s = s_.cbrt();

        Self {
            L: 0.210_454_26_f32.mul_add(l, 0.793_617_8 * m) - 0.004_072_047 * s,
            a: 0.450_593_7_f32.mul_add(s, 1.977_998_5 * l - 2.428_592_2 * m),
            b: 0.025_904_037_f32.mul_add(l, 0.782_771_77 * m) - 0.808_675_77 * s,
        }
    }
}

// finds L_cusp and C_cusp for a given hue
// a and b must be normalized so a^2 + b^2 == 1
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LC {
    pub L: f32,
    pub C: f32,
}
// Finds the maximum saturation possible for a given hue that fits in sRgb
// Saturation here is defined as S = C/L
// a and b must be normalized so a^2 + b^2 == 1

fn compute_max_saturation(a: f32, b: f32) -> f32 {
    // Max saturation will be when one of r, g or b goes below zero.
    // Select different coefficients depending on which component goes below zero first
    let k0: f32;
    let k1: f32;
    let k2: f32;
    let k3: f32;
    let k4: f32;
    let wl: f32;
    let wm: f32;
    let ws: f32;
    if -1.881_703_3_f32 * a - 0.809_364_9_f32 * b > 1_f32 {
        // Red component
        k0 = 1.190_862_8_f32;
        k1 = 1.765_767_3_f32;
        k2 = 0.596_626_4_f32;
        k3 = 0.755_152_f32;
        k4 = 0.567_712_4_f32;
        wl = 4.076_741_7_f32;
        wm = -3.307_711_6_f32;
        ws = 0.230_969_94_f32;
    } else if 1.814_441_1_f32 * a - 1.194_452_8_f32 * b > 1_f32 {
        // Green component
        k0 = 0.739_565_13_f32;
        k1 = -0.459_544_03_f32;
        k2 = 0.082_854_27_f32;
        k3 = 0.125_410_7_f32;
        k4 = 0.145_032_03_f32;
        wl = -1.268_438_f32;
        wm = 2.609_757_4_f32;
        ws = -0.341_319_38_f32;
    } else {
        // Blue component
        k0 = 1.357_336_5_f32;
        k1 = -0.009_157_99_f32;
        k2 = -1.151_302_1_f32;
        k3 = -0.505_596_04_f32;
        k4 = 0.006_921_67_f32;
        wl = -0.004_196_086_4_f32;
        wm = -0.703_418_6_f32;
        ws = 1.707_614_7_f32;
    }
    // Approximate max saturation using a polynomial:
    let mut S: f32 = (k4 * a).mul_add(b, (k3 * a).mul_add(a, k2.mul_add(b, k1.mul_add(a, k0))));
    // Do one step Halley's method to get closer
    // this gives an error less than 10e6, except for some blue hues where the dS/dh is close to infinite
    // this should be sufficient for most applications, otherwise do two/three steps
    let k_l: f32 = 0.396_337_78_f32.mul_add(a, 0.215_803_76_f32 * b);
    let k_m: f32 = -0.105_561_346_f32 * a - 0.063_854_17_f32 * b;
    let k_s: f32 = -0.089_484_18_f32 * a - 1.291_485_5_f32 * b;
    let l_: f32 = S.mul_add(k_l, 1.0_f32);
    let m_: f32 = S.mul_add(k_m, 1.0_f32);
    let s_: f32 = S.mul_add(k_s, 1.0_f32);
    let l: f32 = l_ * l_ * l_;
    let m: f32 = m_ * m_ * m_;
    let s: f32 = s_ * s_ * s_;
    let l_dS: f32 = 3.0_f32 * k_l * l_ * l_;
    let m_dS: f32 = 3.0_f32 * k_m * m_ * m_;
    let s_dS: f32 = 3.0_f32 * k_s * s_ * s_;
    let l_dS2: f32 = 6.0_f32 * k_l * k_l * l_;
    let m_dS2: f32 = 6.0_f32 * k_m * k_m * m_;
    let s_dS2: f32 = 6.0_f32 * k_s * k_s * s_;
    let f: f32 = ws.mul_add(s, wl.mul_add(l, wm * m));
    let f1: f32 = ws.mul_add(s_dS, wl.mul_add(l_dS, wm * m_dS));
    let f2: f32 = ws.mul_add(s_dS2, wl.mul_add(l_dS2, wm * m_dS2));
    S -= f * f1 / (f1 * f1 - 0.5_f32 * f * f2);
    S
}

fn find_cusp(a: f32, b: f32) -> LC {
    // First, find the maximum saturation (saturation S = C/L)
    let S_cusp = compute_max_saturation(a, b);

    // Convert to linear sRgb to find the first point where at least one of r,g or b >= 1:
    let rgb_at_max = LinRgb::from(&Lab {
        L: 1.0,
        a: S_cusp * a,
        b: S_cusp * b,
    });
    let L_cusp = (1.0 / rgb_at_max.r.max(rgb_at_max.g).max(rgb_at_max.b)).cbrt();
    let C_cusp = L_cusp * S_cusp;

    LC {
        L: L_cusp,
        C: C_cusp,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_round_trip() {
        let expected = Rgb {
            r: 80,
            g: 160,
            b: 240,
        };

        let actual = Rgb::from(&LinRgb::from(&Lab::from(&Hsv::from(&Lab::from(
            &LinRgb::from(&expected),
        )))));
        assert_eq!(expected, actual);
    }
}
