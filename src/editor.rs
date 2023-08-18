// TODO: ADD SECTION HEADERS IN GUI

use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::ArraxisParams;

#[derive(Lens)]
struct Data {
    params: Arc<ArraxisParams>,
    current_band: usize,
    active_bands: i32,
}

enum AppEvent {
    NextBand,
    PrevBand,
    SynchronizeWithMain,
}

impl Model for Data {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::NextBand => {
                self.current_band = (self.current_band + 1) % self.active_bands as usize;
            },
            AppEvent::PrevBand => {
                self.current_band = (self.current_band + self.active_bands as usize - 1) % self.active_bands as usize;
            },
            AppEvent::SynchronizeWithMain => {
                self.active_bands = self.params.active_bands.value();
                if self.current_band >= self.active_bands as usize {
                    self.current_band = self.active_bands as usize - 1;
                }
            }
        })
    }
}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (600, 800))
}

const DMMONO_REGULAR: &[u8] = include_bytes!("../assets/DMMono-Regular.ttf");
const DMMONO_MEDIUM: &[u8] = include_bytes!("../assets/DMMono-Medium.ttf");
const DMMONO_LIGHT: &[u8] = include_bytes!("../assets/DMMono-Light.ttf");

pub(crate) fn create(
    params: Arc<ArraxisParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        cx.add_fonts_mem(&[
            DMMONO_REGULAR,
            DMMONO_MEDIUM,
            DMMONO_LIGHT,
        ]);

        cx.set_default_font(&["DM Mono"]);
        cx.add_theme(include_str!("../assets/style.css"));

        Data {
            params: params.clone(),
            current_band: 0,
            active_bands: params.active_bands.value(),
        }
        .build(cx);
        
        // this spawns a thread that emits an event every fixed interval
        // i went with 33ms because iirc the gui is at 30fps, so this should be enough
        cx.spawn(|cx| { // wasted 2 hours on this
            loop {
                let _ = cx.emit(AppEvent::SynchronizeWithMain);
                thread::sleep(Duration::from_millis(33));
            }
        });

        ResizeHandle::new(cx);
        VStack::new(cx, |cx| {
            Label::new(cx, "Arraxis")
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            macro_rules! slider {
                ($cx:ident, $name:expr, $param:ident) => {
                    VStack::new($cx, |$cx| {
                        Label::new($cx, $name)
                            .height(Pixels(20.0))
                            .child_top(Pixels(10.0))
                            .child_bottom(Pixels(5.0));
                        ParamSlider::new($cx, Data::params, |params| &params.$param)
                            .class("paramslider");
                    })
                    .width(Pixels(200.0))
                    .child_left(Stretch(1.0))
                    .child_right(Stretch(1.0))
                }
            }

            macro_rules! crossover_slider {
                ($cx:ident, $name:expr, $param:ident) => {
                    VStack::new($cx, |$cx| {
                        ParamSlider::new($cx, Data::params, |params| &params.$param)
                            .width(Pixels(580.0))
                            .height(Pixels(20.0))
                            .class("crossoverslider");
                    })
                    .width(Pixels(600.0))
                    .child_left(Stretch(1.0))
                    .child_right(Stretch(1.0))
                }
            }

            macro_rules! button {
                ($cx:ident, $name:expr, $param:ident) => {
                    VStack::new($cx, |$cx| {
                        Label::new($cx, $name)
                            .height(Pixels(20.0))
                            .child_top(Pixels(10.0))
                            .child_bottom(Pixels(5.0));
                        ParamButton::new($cx, Data::params, |params| &params.$param)
                            .class("parambutton");
                    })
                    .width(Pixels(300.0))
                    .child_left(Stretch(1.0))
                    .child_right(Stretch(1.0))
                }
            }
            macro_rules! button_event_emitter {
                ($cx:ident, $name:expr, $event:expr) => {
                    HStack::new($cx, |$cx| {
                        Button::new($cx, |$cx| $cx.emit($event), |$cx| {
                            Label::new($cx, $name)
                                .height(Pixels(20.0))
                                .width(Pixels(180.0))
                                .child_top(Stretch(1.0))
                                .child_bottom(Stretch(1.0))
                                .class("textcolor")
                        })
                        .width(Pixels(180.0))
                        .height(Pixels(30.0))
                        .child_left(Stretch(1.0))
                        .child_right(Stretch(1.0))
                        .class("eventbutton");
                    })
                    .width(Pixels(200.0))
                    .child_left(Stretch(1.0))
                    .child_right(Stretch(1.0));
                }
            }

            macro_rules! section_header {
                ($cx:ident, $title:expr) => {
                    Label::new($cx, &format!("─────{}─────", $title))
                        .font_size(20.0)
                        .height(Pixels(35.0))
                        .child_top(Pixels(10.0))
                        .child_bottom(Pixels(10.0))
                        .child_left(Stretch(1.0))
                        .child_right(Stretch(1.0))
                        .width(Percentage(100.0));
                };
            }

            macro_rules! bandspecificparams {(
                $cx:ident,
                $name:expr,
                $active:ident,
                $gain:ident,
                $epsilon:ident,
                $distortion_type:ident,
                $distortion_mix:ident,
                $output_gain:ident) => {
                    VStack::new($cx, |$cx| {
                        HStack::new($cx, |$cx| {
                            slider!($cx, "Gain", $gain)
                            .child_top(Pixels(5.0))
                            .child_bottom(Pixels(5.0));
                            slider!($cx, "Output Gain", $output_gain)
                            .child_top(Pixels(5.0))
                            .child_bottom(Pixels(5.0));
                            slider!($cx, "Distortion Mix", $distortion_mix)
                            .child_top(Pixels(5.0))
                            .child_bottom(Pixels(5.0));
                        })
                        .width(Stretch(1.0))
                        .height(Pixels(20.0))
                        .child_left(Stretch(1.0))
                        .child_right(Stretch(1.0));
                        HStack::new($cx, |$cx| {
                            slider!($cx, "Epsilon", $epsilon)
                            .child_top(Pixels(5.0))
                            .child_bottom(Pixels(5.0));
                            slider!($cx, "Distortion Type", $distortion_type)
                            .child_top(Pixels(5.0))
                            .child_bottom(Pixels(5.0));
                        })
                        .width(Stretch(1.0))
                        .height(Pixels(20.0))
                        .child_left(Stretch(1.0))
                        .child_right(Stretch(1.0));
                    })
                };
            }

            section_header!(cx, "Global Parameters");

            HStack::new(cx, |cx| {
                slider!(cx, "Input Gain", gain);
                slider!(cx, "Output Gain", output_gain);
            })
            .width(Stretch(1.0))
            .height(Pixels(20.0));

            HStack::new(cx, |cx| {
                slider!(cx, "Dry/Wet Mix", distortion_mix);
                slider!(cx, "Band Mix", bands_mix);
            })
            .width(Stretch(1.0))
            .height(Pixels(20.0));

            HStack::new(cx, |cx| {
                slider!(cx, "Epsilon", epsilon);
                slider!(cx, "Distortion Type", distortion_type);
            })
            .width(Stretch(1.0))
            .height(Pixels(20.0))
            .child_bottom(Pixels(10.0));

            section_header!(cx, "Band Parameters");
            
            VStack::new(cx, |cx| {
                Binding::new(cx, Data::active_bands, |cx, active_bands_val| {
                    if active_bands_val.get(cx) == 1 {
                        return;
                    }
                    if active_bands_val.get(cx) >= 2 {
                        Label::new(cx, "Band Crossovers")
                            .font_size(15.0)
                            .height(Pixels(35.0))
                            .child_top(Pixels(5.0))
                            .child_bottom(Pixels(5.0))
                            .child_left(Stretch(1.0))
                            .child_right(Stretch(1.0))
                            .width(Percentage(100.0));
                        crossover_slider!(cx, "Crossover 1", band1_cutoff);
                    }
                    if active_bands_val.get(cx) >= 3 {
                        crossover_slider!(cx, "Crossover 2", band2_cutoff);
                    }
                    if active_bands_val.get(cx) >= 4 {
                        crossover_slider!(cx, "Crossover 3", band3_cutoff);
                    }
                    if active_bands_val.get(cx) >= 5 {
                        crossover_slider!(cx, "Crossover 4", band4_cutoff);
                    }
                })
            })
            .width(Stretch(1.0))
            .height(Pixels(0.0))
            .child_bottom(Pixels(0.0));

            HStack::new(cx, |cx| {
                slider!(cx, "Band Amount", active_bands);
            })
            .width(Stretch(1.0))
            .height(Pixels(20.0))
            .child_bottom(Pixels(5.0));

            //HStack::new(cx, |cx| {
            //    slider!(cx, "BLT Sandwich", band_to_listen_to);
            //})
            //.width(Stretch(1.0))
            //.height(Pixels(20.0))
            //.child_bottom(Pixels(5.0));

            section_header!(cx, "Band Specific Parameters");

            HStack::new(cx, |cx| {
                Binding::new(cx, Data::current_band, |cx, current_band| {
                    Label::new(cx, &format!("Band {}", current_band.get(cx) + 1))
                    .font_size(20.0)
                    .height(Pixels(35.0))
                    .child_top(Pixels(10.0))
                    .child_bottom(Pixels(10.0))
                    .child_left(Stretch(1.0))
                    .child_right(Stretch(1.0));
                })
            })
            .width(Stretch(1.0))
            .height(Pixels(20.0))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0));

            HStack::new(cx, |cx| {
                button_event_emitter!(cx, "Prev Band", AppEvent::PrevBand);
                button_event_emitter!(cx, "Next Band", AppEvent::NextBand);
            })
            .width(Stretch(1.0))
            .height(Pixels(20.0));

            Binding::new(cx, Data::current_band, |cx, current_band| {
                match &current_band.get(cx) {
                    0 => bandspecificparams!(cx, "Band 1", band1_active, band1_gain, band1_epsilon, band1_distortion_type, band1_distortion_mix, band1_output_gain),
                    1 => bandspecificparams!(cx, "Band 2", band2_active, band2_gain, band2_epsilon, band2_distortion_type, band2_distortion_mix, band2_output_gain),
                    2 => bandspecificparams!(cx, "Band 3", band3_active, band3_gain, band3_epsilon, band3_distortion_type, band3_distortion_mix, band3_output_gain),
                    3 => bandspecificparams!(cx, "Band 4", band4_active, band4_gain, band4_epsilon, band4_distortion_type, band4_distortion_mix, band4_output_gain),
                    4 => bandspecificparams!(cx, "Band 5", band5_active, band5_gain, band5_epsilon, band5_distortion_type, band5_distortion_mix, band5_output_gain),
                    _ => unreachable!()
                };
            })
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0))
        .class("fullbandparams");
    })
}
