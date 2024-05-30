use core::f32::consts::PI;

use libm::{fabsf, floorf, remainderf, Libm};

// pub fn sign()
// use pico::Pico;

// pub fn tile_at(celeste: &Celeste, x: f32, y: f32) -> bool {
//     return celeste.mem.mget()
// }

pub trait LibmExt {
    fn floor(self) -> f32;
    fn abs(self) -> f32;
    fn signum(self) -> f32;
    fn rem_euclid(self, x: f32) -> f32;
}

impl LibmExt for f32 {
    fn floor(self) -> f32 {
        Libm::<f32>::floor(self)
    }

    fn abs(self) -> f32 {
        Libm::<f32>::fabs(self)
    }

    fn signum(self) -> f32 {
        if self < 0.0 {
            -1.0
        } else if self == 0.0 {
            0.0
        } else {
            1.0
        }
    }

    fn rem_euclid(self, x: f32) -> f32 {
        Libm::<f32>::remainder(x, self)
    }
}

pub fn min(v1: f32, v2: f32) -> f32 {
    f32::min(v1, v2)
}
pub fn sin(percentage: f32) -> f32 {
    // p8's trig is weird asf
    Libm::<f32>::sin(percentage * -2.0 * PI)
}
pub fn cos(percentage: f32) -> f32 {
    // p8's trig is weird asf
    Libm::<f32>::cos(percentage * -2.0 * PI)
}
pub fn sign(v: f32) -> f32 {
    if v != 0f32 {
        v.signum()
    } else {
        0f32
    }
}
pub fn max(v1: f32, v2: f32) -> f32 {
    f32::max(v1, v2)
}
pub fn appr(val: f32, target: f32, amount: f32) -> f32 {
    if val > target {
        max(val - amount, target)
    } else {
        min(val + amount, target)
    }
}
pub fn mid(v1: f32, v2: f32, v3: f32) -> f32 {
    return v1.max(v2).min(v3);
}
