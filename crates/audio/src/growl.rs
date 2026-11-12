//! Aurora Growl Forge: dual moving formant filters, waveshaping and digital grit.
//! Native Honeydew effect: fixed state and no allocation in the audio callback.
use std::f32::consts::{PI, TAU};
pub const GROWL: u16 = 1009;
pub const DRIVE: usize = 1;
pub const VOWEL: usize = 2;
pub const RESONANCE: usize = 3;
pub const SPREAD: usize = 4;
pub const MOTION: usize = 5;
pub const DEPTH: usize = 6;
pub const FOLD: usize = 7;
pub const GRIT: usize = 8;
pub const MIX: usize = 9;
pub const PARAMS: usize = 10;
pub const DEFAULTS: [f32; PARAMS] = [1.0, 0.4, 0.53, 0.55, 0.5, 0.35, 0.45, 0.32, 0.12, 0.7];
#[derive(Clone, Copy, Default)]
struct Svf {
    i1: f32,
    i2: f32,
    a1: f32,
    a2: f32,
    a3: f32,
}
impl Svf {
    fn tune(&mut self, hz: f32, q: f32, rate: f32) {
        let g = (PI * (hz / rate).min(0.44)).tan();
        let k = 1.0 / q.max(0.55);
        self.a1 = 1.0 / (1.0 + g * (g + k));
        self.a2 = g * self.a1;
        self.a3 = g * self.a2;
    }
    fn band(&mut self, x: f32) -> f32 {
        let v3 = x - self.i2;
        let v1 = self.a1.mul_add(self.i1, self.a2 * v3);
        let v2 = self.i2 + self.a2 * self.i1 + self.a3 * v3;
        self.i1 = (2.0 * v1 - self.i1).clamp(-64.0, 64.0);
        self.i2 = (2.0 * v2 - self.i2).clamp(-64.0, 64.0);
        if self.i1.abs() < 1e-20 {
            self.i1 = 0.0;
        }
        if self.i2.abs() < 1e-20 {
            self.i2 = 0.0;
        }
        v1
    }
    fn clear(&mut self) {
        self.i1 = 0.0;
        self.i2 = 0.0;
    }
}
pub struct GrowlState {
    filters: [[Svf; 2]; 2],
    rate: f32,
    drive: f32,
    center: f32,
    q: f32,
    spread: f32,
    lfo_rate: f32,
    depth: f32,
    fold: f32,
    grit: f32,
    levels: f32,
    mix: f32,
    phase: f32,
    ticker: u8,
    // Avoid modifying the filter's tuning cadence on repeat automation writes.
    last_params: [f32; PARAMS],
}
impl GrowlState {
    pub fn new(params: &[f32], sample_rate: f32) -> Self {
        let mut s = Self {
            filters: [[Svf::default(); 2]; 2],
            rate: sample_rate,
            drive: 1.0,
            center: 400.0,
            q: 2.0,
            spread: 1.7,
            lfo_rate: 1.0,
            depth: 0.0,
            fold: 0.0,
            grit: 0.0,
            levels: 256.0,
            mix: 0.7,
            phase: 0.0,
            ticker: 0,
            last_params: [f32::NAN; PARAMS],
        };
        s.refresh(params, sample_rate);
        s
    }
    pub fn refresh(&mut self, p: &[f32], sr: f32) {
        let values: [f32; PARAMS] = std::array::from_fn(|i| {
            p.get(i).copied().unwrap_or(DEFAULTS[i]).clamp(0.0, 1.0)
        });
        if values == self.last_params && sr == self.rate {
            return;
        }
        self.last_params = values;
        let v = |i: usize| values[i];
        self.rate = sr;
        self.drive = 1.0 + 19.0 * v(DRIVE).powi(2);
        self.center = 80.0 * 22.0_f32.powf(v(VOWEL));
        self.q = 0.7 + 13.0 * v(RESONANCE).powi(2);
        self.spread = 1.2 + 1.4 * v(SPREAD);
        self.lfo_rate = 0.05 + 13.0 * v(MOTION).powi(2);
        self.depth = 1.6 * v(DEPTH);
        self.fold = v(FOLD);
        self.grit = v(GRIT);
        self.levels = 2.0_f32.powf(15.0 - 11.0 * self.grit);
        self.mix = v(MIX);
        self.ticker = 0;
        self.retune();
    }
    fn retune(&mut self) {
        let centre = self.center * 2.0_f32.powf((TAU * self.phase).sin() * self.depth);
        for (ch, fs) in self.filters.iter_mut().enumerate() {
            let stereo = if ch == 0 { 0.995 } else { 1.005 };
            fs[0].tune(centre * stereo, self.q, self.rate);
            fs[1].tune(centre * self.spread * stereo, self.q, self.rate);
        }
    }
    pub fn reset(&mut self) {
        for f in &mut self.filters {
            for b in f {
                b.clear();
            }
        }
        self.phase = 0.0;
        self.ticker = 0;
    }
    fn channel(&mut self, ch: usize, x: f32) -> f32 {
        let driven = (x * self.drive).tanh();
        let folded = (driven * (1.0 + 6.0 * self.fold)).sin();
        let shaped = driven + self.fold * (folded - driven);
        let vowel = self.filters[ch][0].band(shaped) + 0.75 * self.filters[ch][1].band(shaped);
        let wet = 0.3 * shaped + 1.6 * vowel;
        let crushed = (wet * self.levels).round() / self.levels;
        x + self.mix * (wet + self.grit * (crushed - wet) - x)
    }
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        if self.ticker == 0 {
            self.retune();
        }
        self.ticker = (self.ticker + 1) % 48;
        self.phase = (self.phase + self.lfo_rate / self.rate).fract();
        // Invalid upstream samples must not poison persistent filter state.
        let l = if l.is_finite() { l } else { 0.0 };
        let r = if r.is_finite() { r } else { 0.0 };
        (self.channel(0, l), self.channel(1, r))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bad_samples_cannot_poison_filter_state() {
        let mut synth = GrowlState::new(&DEFAULTS, 48000.0);
        let (left, right) = synth.process_frame(f32::NAN, f32::INFINITY);
        assert!(left.is_finite() && right.is_finite());
        for _ in 0..500 {
            let (l, r) = synth.process_frame(0.25, -0.25);
            assert!(l.is_finite() && r.is_finite());
        }
    }

    #[test]
    fn identical_automation_values_preserve_the_exact_audio_stream() {
        let mut plain = GrowlState::new(&DEFAULTS, 48000.0);
        let mut repeated = GrowlState::new(&DEFAULTS, 48000.0);
        for n in 0..12_000 {
            if n % 37 == 0 {
                repeated.refresh(&DEFAULTS, 48000.0);
                repeated.refresh(&DEFAULTS, 48000.0);
            }
            let x = (n as f32 * 0.017).sin() * 0.8;
            assert_eq!(
                plain.process_frame(x, -x),
                repeated.process_frame(x, -x),
                "sample {n}"
            );
        }
    }

    #[test]
    fn transparent_zero_mix() {
        let mut p = DEFAULTS;
        p[MIX] = 0.0;
        let mut s = GrowlState::new(&p, 48000.0);
        for _ in 0..100 {
            assert_eq!(s.process_frame(0.13, -0.27), (0.13, -0.27));
        }
    }
    #[test]
    fn finite_after_extreme_modulation() {
        let p = [1.0; PARAMS];
        let mut s = GrowlState::new(&p, 48000.0);
        for n in 0..100_000 {
            let (l, r) =
                s.process_frame((n as f32 * 0.17).sin() * 0.9, (n as f32 * 0.31).sin() * 0.9);
            assert!(l.is_finite() && r.is_finite() && l.abs() < 100.0 && r.abs() < 100.0);
        }
        s.reset();
        assert_eq!(s.process_frame(0.0, 0.0), (0.0, 0.0));
    }
}
