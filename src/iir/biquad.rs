use std::f32::consts::PI;
use nih_plug::prelude::*;

// modified from https://github.com/renzol2/fx/blob/main/fx/src/biquad.rs

#[derive(Debug, PartialEq, Eq, Clone, Copy, Enum)]
pub enum BiquadFilterType {
    LowPass,
    HighPass,
    BandPass
}

#[derive(Copy, Clone)]
/// Biquad filter code from: https://www.earlevel.com/main/2012/11/26/biquad-c-source-code/
pub struct BiquadFilter {
    // Filter type & coefficients
    filter_type: BiquadFilterType,
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
    pub b1: f32,
    pub b2: f32,

    // Filter parameters
    fc: f32,
    q: f32,
    peak_gain: f32,

    // Unit delays
    z1: f32,
    z2: f32,

    // sample rate
    sample_rate: f32,
}

impl BiquadFilter {
    pub fn new() -> BiquadFilter {
        let mut bqf = BiquadFilter {
            filter_type: BiquadFilterType::LowPass,
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            b1: 0.0,
            b2: 0.0,
            fc: 0.5,
            q: 0.707,
            peak_gain: 0.0,
            z1: 0.0,
            z2: 0.0,
            sample_rate: 44100.0,
        };
        bqf.set_filter_type(BiquadFilterType::LowPass);
        bqf
    }

    /// Sets filter type and recalculates coefficients.
    pub fn set_filter_type(&mut self, filter_type: BiquadFilterType) {
        self.filter_type = filter_type;
        self.calculate_biquad_coefficients();
    }

    /// Sets Q value and recalculates coefficients.
    pub fn set_q(&mut self, q: f32) {
        self.q = q;
        self.calculate_biquad_coefficients();
    }

    /// Sets center frequency and recalculates coefficients.
    pub fn set_fc(&mut self, fc: f32) {
        self.fc = fc;
        self.calculate_biquad_coefficients();
    }

    /// Sets peak gain and recalculates coefficients.
    pub fn set_peak_gain(&mut self, peak_gain: f32) {
        self.peak_gain = peak_gain;
        self.calculate_biquad_coefficients();
    }

    /// Sets all the filter's parameters.
    pub fn set_biquad(&mut self, filter_type: BiquadFilterType, fc: f32, q: f32, peak_gain: f32, sample_rate: f32) {
        self.filter_type = filter_type;
        self.q = q;
        self.fc = fc;
        self.sample_rate = sample_rate;
        self.peak_gain = peak_gain;
    }

    pub fn get_cutoff(&self) -> f32 {
        self.fc * self.sample_rate
    }

    /// Recalculates coefficients according to the filter's current parameters.
    pub fn calculate_biquad_coefficients(&mut self) {
        let v = 10.0_f32.powf(self.peak_gain.abs() / 20.0);
        let k = (PI * self.fc / self.sample_rate).tan();
        let norm = (1.0 + k / self.q + k * k).recip();

        match self.filter_type {
            BiquadFilterType::LowPass => {
                let omega = std::f32::consts::TAU * self.fc;
                let cos_omega = omega.cos();
                let alpha = omega.sin() / (2.0 * self.q);

                let b0 = 1.0 + alpha;
                self.a0 = ((1.0 - cos_omega) / 2.0) / b0;
                self.a1 = (1.0 - cos_omega) / b0;
                self.a2 = ((1.0 - cos_omega) / 2.0) / b0;
                self.b1 = (-2.0 * cos_omega) / b0;
                self.b2 = (1.0 - alpha) / b0;
            }
            BiquadFilterType::HighPass => {
                let omega = std::f32::consts::TAU * self.fc;
                let cos_omega = omega.cos();
                let alpha = omega.sin() / (2.0 * self.q);

                let b0 = 1.0 + alpha;
                self.a0 = ((1.0 + cos_omega) / 2.0) / b0;
                self.a1 = -(1.0 + cos_omega) / b0;
                self.a2 = ((1.0 + cos_omega) / 2.0) / b0;
                self.b1 = (-2.0 * cos_omega) / b0;
                self.b2 = (1.0 - alpha) / b0;
            }
            BiquadFilterType::BandPass => {
                self.a0 = k / self.q * norm;
                self.a1 = 0.0;
                self.a2 = -self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            }
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = input * self.a0 + self.z1;
        self.z1 = input * self.a1 - self.b1 * output + self.z2;
        self.z2 = input * self.a2 - self.b2 * output;
        output
    }

    pub fn reset_unit_delays(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}