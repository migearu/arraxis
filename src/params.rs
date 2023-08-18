use nih_plug::prelude::*;
use satfuncs::DistFuncs;
use std::sync::Arc;
use nih_plug_vizia::ViziaState;

use crate::editor;
use crate::satfuncs;
use crate::biquad;
/// This is mostly identical to the gain example, minus some fluff, and with a GUI.

#[derive(Params)]
pub struct ArraxisParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "epsilon"]
    pub epsilon: FloatParam,

    #[id = "distortion_type"]
    pub distortion_type: EnumParam<DistFuncs>,

    #[id = "distortion_mix"]
    pub distortion_mix: FloatParam,

    #[id = "bands_mix"]
    pub bands_mix: FloatParam,

    #[id = "output_gain"]
    pub output_gain: FloatParam,

    // filter cutoffs
    // for example
    // the lowpass band cuts off at band1_cutoff
    // the first bandpass band cuts off the lows under band1_cutoff and the highs over band2_cutoff
    // the second bandpass band cuts off the lows under band2_cutoff and the highs over band3_cutoff
    // the third bandpass band cuts off the lows under band3_cutoff and the highs over band4_cutoff
    // the highpass band cuts off the lows under band5_cutoff
    // lp: 0 - b1
    // bp1: b1 - b2
    // bp2: b2 - b3
    // bp3: b3 - b4
    // hp: b4 - inf
    #[id = "band1_cutoff"]
    pub band1_cutoff: FloatParam,
    #[id = "band2_cutoff"]
    pub band2_cutoff: FloatParam,
    #[id = "band3_cutoff"]
    pub band3_cutoff: FloatParam,
    #[id = "band4_cutoff"]
    pub band4_cutoff: FloatParam,

    // BAND-SPECIFIC VARIABLES
    // there's definitely a better way to do this, but i don't know what it is

    // INPUT GAINS
    #[id = "band1_gain"]
    pub band1_gain: FloatParam,
    #[id = "band2_gain"]
    pub band2_gain: FloatParam,
    #[id = "band3_gain"]
    pub band3_gain: FloatParam,
    #[id = "band4_gain"]
    pub band4_gain: FloatParam,
    #[id = "band5_gain"]
    pub band5_gain: FloatParam,

    // EPSILONS
    #[id = "band1_epsilon"]
    pub band1_epsilon: FloatParam,
    #[id = "band2_epsilon"]
    pub band2_epsilon: FloatParam,
    #[id = "band3_epsilon"]
    pub band3_epsilon: FloatParam,
    #[id = "band4_epsilon"]
    pub band4_epsilon: FloatParam,
    #[id = "band5_epsilon"]
    pub band5_epsilon: FloatParam,

    // DISTORTIONS
    #[id = "band1_distortion_type"]
    pub band1_distortion_type: EnumParam<DistFuncs>,
    #[id = "band2_distortion_type"]
    pub band2_distortion_type: EnumParam<DistFuncs>,
    #[id = "band3_distortion_type"]
    pub band3_distortion_type: EnumParam<DistFuncs>,
    #[id = "band4_distortion_type"]
    pub band4_distortion_type: EnumParam<DistFuncs>,
    #[id = "band5_distortion_type"]
    pub band5_distortion_type: EnumParam<DistFuncs>,

    // DISTORTION MIXES
    #[id = "band1_distortion_mix"]
    pub band1_distortion_mix: FloatParam,
    #[id = "band2_distortion_mix"]
    pub band2_distortion_mix: FloatParam,
    #[id = "band3_distortion_mix"]
    pub band3_distortion_mix: FloatParam,
    #[id = "band4_distortion_mix"]
    pub band4_distortion_mix: FloatParam,
    #[id = "band5_distortion_mix"]
    pub band5_distortion_mix: FloatParam,

    // OUTPUT GAINS
    #[id = "band1_output_gain"]
    pub band1_output_gain: FloatParam,
    #[id = "band2_output_gain"]
    pub band2_output_gain: FloatParam,
    #[id = "band3_output_gain"]
    pub band3_output_gain: FloatParam,
    #[id = "band4_output_gain"]
    pub band4_output_gain: FloatParam,
    #[id = "band5_output_gain"]
    pub band5_output_gain: FloatParam,

    #[id = "active_bands"]
    pub active_bands: IntParam,
    #[id = "linear_phase"]
    pub linear_phase: BoolParam,

    //#[id = "band_to_listen_to"]
    //pub band_to_listen_to: IntParam,
}

impl Default for ArraxisParams {
    fn default() -> Self {
        macro_rules! frequency {
            ($name:expr, $default:expr) => {
                FloatParam::new(
                    stringify!($name),
                    $default,
                    FloatRange::Skewed {
                        min: 20.0,
                        max: 20000.0,
                        factor: FloatRange::skew_factor(-1.0),
                    },
                )
                .with_smoother(SmoothingStyle::Logarithmic(100.0))
                .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
                .with_string_to_value(formatters::s2v_f32_hz_then_khz())
            };
        }

        macro_rules! gain {
            ($name:expr, $default:expr) => {
                FloatParam::new(
                    stringify!($name),
                    util::db_to_gain($default),
                    FloatRange::Skewed {
                        min: util::db_to_gain(-30.0),
                        max: util::db_to_gain(30.0),
                        factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                    },
                )
                .with_smoother(SmoothingStyle::Logarithmic(50.0))
                .with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
            };
        }

        macro_rules! linear {
            ($name:expr, $default:expr, $unit:expr, $minval:expr, $maxval:expr) => {
                FloatParam::new(
                    stringify!($name),
                    $default,
                    FloatRange::Linear {
                        min: $minval,
                        max: $maxval,
                    },
                )
                .with_unit($unit)
                .with_value_to_string(formatters::v2s_f32_rounded(2))
            };
        }

        macro_rules! enumparameter {
            ($name:expr, $default:expr) => {
                EnumParam::new(
                    stringify!($name),
                    $default,
                )
            };
        }

        macro_rules! percentage {
            ($name:expr, $default:expr) => {
                FloatParam::new(
                    stringify!($name),
                    $default,
                    FloatRange::Linear {
                        min: 0.0,
                        max: 1.0,
                    },
                )
                .with_unit("%")
                .with_value_to_string(formatters::v2s_f32_percentage(2))
                .with_string_to_value(formatters::s2v_f32_percentage())
            };
        }

        macro_rules! discrete_integer {
            ($name:expr, $default:expr, $minval:expr, $maxval:expr) => {
                IntParam::new(
                    stringify!($name),
                    $default,
                    IntRange::Linear {
                        min: $minval,
                        max: $maxval,
                    },
                )
            };
        }
        Self {
            editor_state: editor::default_state(),

            // overall distortion; this might never be applied
            gain: gain!("Gain", 0.0),
            epsilon: linear!("Epsilon", 1.0, " ε", 0.0, 10.0),
            distortion_type: enumparameter!("Distortion Type", DistFuncs::HardClip),
            distortion_mix: percentage!("Distortion Mix", 1.0),
            bands_mix: percentage!("Band Mix", 1.0),
            output_gain: gain!("Output Gain", 0.0),

            // filter cutoffs https://www.teachmeaudio.com/mixing/techniques/audio-spectrum
            band1_cutoff: frequency!("Band 1 Cutoff", 250.0),
            band2_cutoff: frequency!("Band 2 Cutoff", 1000.0),
            band3_cutoff: frequency!("Band 3 Cutoff", 4000.0),
            band4_cutoff: frequency!("Band 4 Cutoff", 10000.0),
            // band 1 is sub-bass to bass
            // band 2 is low mids to somewhat upper mids
            // band 3 is upper mids to low presence
            // band 4 is high presence to low brilliance
            // band 5 is low brilliance to high brilliance

            // band-specific variables
            // gains
            band1_gain: gain!("Gain", 0.0),
            band2_gain: gain!("Gain", 0.0),
            band3_gain: gain!("Gain", 0.0),
            band4_gain: gain!("Gain", 0.0),
            band5_gain: gain!("Gain", 0.0),

            // epsilons
            band1_epsilon: linear!("Epsilon", 1.0, " ε", 0.0, 10.0),
            band2_epsilon: linear!("Epsilon", 1.0, " ε", 0.0, 10.0),
            band3_epsilon: linear!("Epsilon", 1.0, " ε", 0.0, 10.0),
            band4_epsilon: linear!("Epsilon", 1.0, " ε", 0.0, 10.0),
            band5_epsilon: linear!("Epsilon", 1.0, " ε", 0.0, 10.0),

            // distortion types
            band1_distortion_type: enumparameter!("Distortion Type", DistFuncs::HardClip),
            band2_distortion_type: enumparameter!("Distortion Type", DistFuncs::HardClip),
            band3_distortion_type: enumparameter!("Distortion Type", DistFuncs::HardClip),
            band4_distortion_type: enumparameter!("Distortion Type", DistFuncs::HardClip),
            band5_distortion_type: enumparameter!("Distortion Type", DistFuncs::HardClip),

            // distortion mixes
            band1_distortion_mix: percentage!("Distortion Mix", 1.0),
            band2_distortion_mix: percentage!("Distortion Mix", 1.0),
            band3_distortion_mix: percentage!("Distortion Mix", 1.0),
            band4_distortion_mix: percentage!("Distortion Mix", 1.0),
            band5_distortion_mix: percentage!("Distortion Mix", 1.0),

            // output gains
            band1_output_gain: gain!("Output Gain", 0.0),
            band2_output_gain: gain!("Output Gain", 0.0),
            band3_output_gain: gain!("Output Gain", 0.0),
            band4_output_gain: gain!("Output Gain", 0.0),
            band5_output_gain: gain!("Output Gain", 0.0),

            // active bands
            active_bands: discrete_integer!("Bands", 5, 1, 5),

            // linear phase
            linear_phase: BoolParam::new("Linear Phase", true),

            // band to listen to, only for debugging; this should be removed
            // band_to_listen_to: discrete_integer!("BLT Sandwich", 1, 1, 5),
        }
    }
}