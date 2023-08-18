/*
┌──────────────────────────────────────────────┐
│Hey, I originally started this as a project to│
│learn the Rust programming language, as well  │
│as a test to see if I can make a VST.         │
│                                              │
│I'll try to include as much information in the│
│comments as possible so that you don't have to│
│spend countless hours figuring things out.    │
│                                              │
│This is just your average multi-band distort- │
│ion plugin, it uses nih-plug, along with VIZIA│
│for the GUI.                                  │
│                                              │
│If you just want to use the plugin, compile   │
│using the provided release.cmd, or run        │
│cargo xtask bundle arraxis --release          │
│                                              │
│If you want to add functions, or develop a new│
│feature, go do it, and create a PR. If it     │
│works properly, then I'll merge it.           │
│                                              │
│Hopefully this ends up being useful for you   │
│                   MIGEARU                    │
└──────────────────────────────────────────────┘
23-08-17 11:10PM: linear phase crossover is beyond me, so you'll have to deal with minimum phase for now. i fucking give up.

* made using https://asciiflow.com/#/ it's pretty cool
PS: Again, I made this project to learn Rust, so if you see any bad code/practices, I guess just tell me or something so I can learn for next time.
 */

use nih_plug::prelude::*;
use satfuncs::DistFuncs;
use std::sync::Arc;
use params::ArraxisParams;

use iir::biquad;

// https://doc.rust-lang.org/rust-by-example/mod.html, to learn more about modules
mod funcs;
mod satfuncs;
// editor contains all the gui code
mod editor;
// params contains all the parameters (obviously lmao)
mod params;
mod iir;

// constants
const NUM_BANDS: usize = 5;

// this is the main struct where we'll implement the plugin
// generally a good idea to keep all parameters in a single struct so you can add new parameters to it easily
// however, some parameters that aren't editable by the user from the plugin's GUI can be kept in the main struct
// the filters are editable, but since they themselves aren't parameters, they're kept in the main struct
pub struct Arraxis {
    params: Arc<ArraxisParams>,
    // why NUM_BANDS + 1?
    // it's because while testing, i found that the last band would always let some frequencies that it should definitely not let through
    // so I added an extra band to store this unwanted signal, and then i just ignore it
    // some of this unwanted signal makes it through to the last band, but it's very little, around -41.5 LUFS, when the original sample is at -6.2 LUFS
    // it's also at 3 - 7 kHz, and it's very quiet, so it's passable for now, but i'll eventually try to fix it in a better way
    filters: [[[biquad::BiquadFilter; 2]; 2]; NUM_BANDS + 1],
    sample_rate: f32,
    buffer_config: BufferConfig,
}

// implement default, basically think of it as creating the variables we had earlier, as actual variables, and not concepts
// (i'm writing this at 12 am, so i'm not sure if this is the best way to explain it)
impl Default for Arraxis {
    fn default() -> Self {
        Self {
            params: Arc::new(ArraxisParams::default()),
            filters: [[[biquad::BiquadFilter::new(); 2]; 2]; NUM_BANDS + 1], // nah
            sample_rate: 0.0,
            buffer_config: BufferConfig {
                sample_rate: 0.0,
                min_buffer_size: None,
                max_buffer_size: 0,
                process_mode: ProcessMode::Realtime,
            },
        }
    }
}

// impl <MainStruct> isn't required, but it's useful to add so that you can make functions that use the main struct's variables
impl Arraxis {
    // this function is used to get the parameters for each band
    // i think there's a better way to do this, but this was easier to implement for testing
    // i might change this later if i find a better way to do it
    fn get_band_float_params(&self) -> [[&FloatParam; 4]; NUM_BANDS] {
        [
            [&self.params.band1_gain, &self.params.band1_output_gain, &self.params.band1_distortion_mix, &self.params.band1_epsilon],
            [&self.params.band2_gain, &self.params.band2_output_gain, &self.params.band2_distortion_mix, &self.params.band2_epsilon],
            [&self.params.band3_gain, &self.params.band3_output_gain, &self.params.band3_distortion_mix, &self.params.band3_epsilon],
            [&self.params.band4_gain, &self.params.band4_output_gain, &self.params.band4_distortion_mix, &self.params.band4_epsilon],
            [&self.params.band5_gain, &self.params.band5_output_gain, &self.params.band5_distortion_mix, &self.params.band5_epsilon],
        ]
    }
    fn get_band_distortion_functions(&self) -> [&EnumParam<DistFuncs>; NUM_BANDS] {
        [
            &self.params.band1_distortion_type,
            &self.params.band2_distortion_type,
            &self.params.band3_distortion_type,
            &self.params.band4_distortion_type,
            &self.params.band5_distortion_type,
        ]
    }

    // this function is used to update the filter's cutoff frequency, if it's changed
    // nothing else is changed, since the other parameters are constant
    // might add Linkwitz-Riley 48 dB/oct crossover later, but i think the 24 dB/oct is good enough for now
    fn update_filter(&mut self, sample_rate: f32) {
        let band_cutoffs = [
            if self.params.active_bands.value() >= 2 { self.params.band1_cutoff.value() } else { 20000.0 },
            if self.params.active_bands.value() >= 3 { self.params.band2_cutoff.value() } else { 20000.0 },
            if self.params.active_bands.value() >= 4 { self.params.band3_cutoff.value() } else { 20000.0 },
            if self.params.active_bands.value() >= 5 { self.params.band4_cutoff.value() } else { 20000.0 },
            20000.0,
        ];
        // the amount of filters will always be the number of bands - 1
        // at 1 band, there's no need for a crossover
        // at 2 bands, there's 1 crossover
        // at 3 bands, there's 2 crossovers
        // and so on...

        // each crossover consists of a lowpass and a highpass filter (i think a bandpass filter would work too, but i'm not sure how to implement it)
        for i in 0..NUM_BANDS {
            if band_cutoffs[i] != self.filters[i][0][0].get_cutoff() {
                for filter in &mut self.filters[i][0] {
                    filter.set_biquad(
                        biquad::BiquadFilterType::LowPass,
                        band_cutoffs[i] / sample_rate,
                        0.707,
                        0.0, // band gain will be applied later
                        self.sample_rate
                    );
                    filter.calculate_biquad_coefficients();
                    // silly story, i was trying to figure out why the filters weren't working, and i was trying to print the coefficients
                    // and i came up with this genius (sarcastic) line of code:
                    
                    // panic!("b0: {} b1: {} b2: {} a0: {} a1: {} frequency: {}", filter.a0, filter.a1, filter.a2, filter.b1, filter.b2, filter.get_cutoff());

                    // panic!() is crashes the program, and has an optional argument to print out a message
                    // so basically, i figured out that attaching a debugger, and forcing a crash with panic!(), it could print out the coefficients
                    // and i couldn't use debug mode because it always crashed for some reason
                }
                for filter in &mut self.filters[i][1] {
                    filter.set_biquad(
                        biquad::BiquadFilterType::HighPass,
                        band_cutoffs[i] / sample_rate,
                        0.707,
                        0.0, // band gain will be applied later
                        self.sample_rate
                    );
                    filter.calculate_biquad_coefficients();
                }
            }
        }
    }
    fn process_filter(&mut self, sample: f32) -> [f32; NUM_BANDS + 1] {
        // https://www.earlevel.com/main/2003/02/28/biquads/
        // 24 dB/oct Linkwitz-Riley crossover (NOT LINEAR PHASE)

        // small issue, when two filters end at the same cutoff, the area around 2x the cutoff is drastically reduced
        // try to avoid two filters coming too close (about two-fold or less) to each other
        // so if you have filter A at 1000 Hz, filter B should be at 2000 Hz or higher
        // if you have filter B at 5000 Hz, filter A should be at 2500 Hz or lower
        // i don't know how to fix this, but it's not a big deal, since two-fold increases are just an octave
        
        // if you want to fix this, just do a pr, and i'll merge it (IF IT WORKS!!!)
        let mut bands = [sample; NUM_BANDS + 1];
        // set the first band is just the input sample (you'll see why we do this)

        // process each band, only up to the number of active bands; the "bands" variable can have less bands than NUM_BANDS, but never more
        // i could've also used a vector to store bands, but that would be slower, and there's no possibility of having more bands than NUM_BANDS
        for i in 0..self.params.active_bands.value() as usize {
            // for the first loop, the first band contains our input sample
            let mut lowpassband = bands[i]; // set the lowpass band to the current band's signal
            let mut highpassband = bands[i]; // set the highpass band to the current band's signal
            for filter in &mut self.filters[i][0] {
                lowpassband = filter.process(lowpassband);
            }
            for filter in &mut self.filters[i][1] {
                highpassband = filter.process(highpassband);
            }
            bands[i] = lowpassband; // our current band will be in the lowpassed signal
            bands[i+1] = highpassband; // all the bands above the current band will be the in the highpassed signal
            // at the end of the loop, the last band will be the highpassed signal
        }
        bands
    }
    fn process_distortion(&mut self, band: f32, band_num: usize) -> f32 {
        let fp = self.get_band_float_params();
        let df = self.get_band_distortion_functions();
        let dry = band;
        let mut band = band;
        band *= fp[band_num][0].smoothed.next(); // apply the input gain
        band = df[band_num].value().get_dist(band, fp[band_num][3].smoothed.next()); // apply the distortion
        band *= fp[band_num][1].smoothed.next(); // apply the output gain
        band = funcs::mix_between(dry, band, fp[band_num][2].smoothed.next());  // apply the mix
        band
    }

    fn iir_minphase_process(&mut self, buffer: &mut Buffer) {
        for channel_samples in buffer.iter_samples() {
            let mut amplitude = 0.0;
            let num_samples = channel_samples.len();
            for (channel_idx, sample) in channel_samples.into_iter().enumerate() {
                // might wanna keep the dry sample
                let dry = *sample;
                let sm = self.process_filter(*sample);
                let mut processed = 0.0;
                // apply the distortion to each band, and combine it back into one sample for final distortion
                for i in 0..self.params.active_bands.value() as usize {
                    processed += self.process_distortion(sm[i], i);
                }
                *sample = processed;
                // apply the input gain
                *sample *= self.params.gain.smoothed.next();
                // apply the distortion
                *sample = self.params.distortion_type.value().get_dist(*sample, self.params.epsilon.smoothed.next());
                // apply the output gain
                *sample *= self.params.output_gain.smoothed.next();
                // mix the bands before and after final distortion
                *sample = funcs::mix_between(processed, *sample, self.params.bands_mix.smoothed.next());
                // mix the dry and wet signal
                *sample = funcs::mix_between(dry, *sample, self.params.distortion_mix.smoothed.next());
                // extra stuff, not important to audio processing
                amplitude += *sample;
            }

            // To save resources, a plugin can (and probably should!) only perform expensive
            // calculations that are only displayed on the GUI while the GUI is open
            if self.params.editor_state.is_open() {
                // nothing here yet
            }
        }
    }
}

impl Plugin for Arraxis {
    const NAME: &'static str = "Arraxis";
    const VENDOR: &'static str = "migearu";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "miguelenzoaruelo@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.update_filter(self.sample_rate);
        self.buffer_config = *buffer_config;
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.update_filter(self.sample_rate);
        context.set_latency_samples(0);
        self.iir_minphase_process(buffer);

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Arraxis {
    const CLAP_ID: &'static str = "com.your-domain.arraxis";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Distortion plugin, made it because I was bored.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Arraxis {
    const VST3_CLASS_ID: [u8; 16] = *b"VSTArraxisPlugin";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nih_export_clap!(Arraxis);
nih_export_vst3!(Arraxis);
