use std::f32::consts::PI;
use nih_plug::prelude::Enum;

#[derive(PartialEq, Enum)]
pub enum DistFuncs {
    HardClip, // sign(x) * min(|x| * t, 1)
    SoftClip, // (1 - e^(-|3x| * t)) / (1 + e^(-|3x| * t))
    Tanh, // tanh(2 * x * t) 
    Sqrt, // sign(x) * sqrt(|x|) * t
    Bitcrush, // round(x * t) / t
    // more experimental
    Sine, // sign(x) * sin(x * t * PI)
    Wonky, // sin(x * t) * cos(x * t)
    SineSqr, // sign(x) * sin(x^2 * t * PI)
    SineSoftClip, // (1 - e^(-|3x| * t)) / (1 + e^(-|3x| * t)) + (sin(10 * x * t * PI) / (sign(x) * 5 * sqrt(|x|)))
}

impl DistFuncs {
    pub fn get_dist(&self, x: f32, t: f32) -> f32 {
        match self {
            DistFuncs::Sqrt => x.signum() * (x * t).abs().sqrt() ,
            DistFuncs::Sine => x.signum() * (x.abs() * t * PI).sin(),
            DistFuncs::Tanh => (2.0 * x * t).tanh(),
            DistFuncs::SineSqr => x.signum() * (x.abs().powi(2) * t * PI).sin(),
            DistFuncs::HardClip => (x * t).min(1.0).max(-1.0),
            DistFuncs::SoftClip => (1.0 - (-x * t * 3.0).exp()) / (1.0 + (-x * t * 3.0).exp()),
            DistFuncs::Bitcrush => {
                if t != 0.0 {
                    (x * t).round() / t
                } else {
                    0.0
                }
            },
            DistFuncs::Wonky => (x * t).sin() * (x * t).cos(),
            DistFuncs::SineSoftClip => {
                let softclip = (1.0 - (-x * t * 3.0).exp()) / (1.0 + (-x * t * 3.0).exp());
                let sine = ((10.0 * t * x * PI).sin() * x) / (x.signum() * 5.0 * x.abs().sqrt());
                softclip + sine
            },
        }
    }
}