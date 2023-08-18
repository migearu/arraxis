pub fn mix_between(a: f32, b: f32, mix: f32) -> f32 {
    a * (1.0 - mix) + b * (mix)
}