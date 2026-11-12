//! Aurora Wavetable: an original Android-native hybrid synth for Honeydew.
//! Two morphable band-limited oscillators, oscillator FM, 8-voice unison,
//! sub, filter, ADSR, LFO, drive and stereo. No work allocates in audio callback.
use crate::{ProcessContext, instrument::Instrument};
use std::f32::consts::{PI, TAU};
use std::sync::LazyLock;

// Interpolated sine table eliminates repeated transcendentals per unison voice.
// Force its one-time initialization when creating the instrument, not on the audio thread.
const SINE_POINTS: usize = 2048;
static SINE_TABLE: LazyLock<[f32; SINE_POINTS + 1]> =
    LazyLock::new(|| std::array::from_fn(|i| (TAU * i as f32 / SINE_POINTS as f32).sin()));
#[inline]
fn fast_sine(phase: f32) -> f32 {
    let pos = phase * SINE_POINTS as f32;
    let idx = (pos as usize).min(SINE_POINTS - 1);
    let frac = pos - idx as f32;
    SINE_TABLE[idx] + frac * (SINE_TABLE[idx + 1] - SINE_TABLE[idx])
}

pub const AURORA_SYNTH: u16 = 1011;
pub const WAVE_A: usize = 1;
pub const WAVE_B: usize = 2;
pub const OSC_MIX: usize = 3;
pub const UNISON: usize = 4;
pub const DETUNE: usize = 5;
pub const WIDTH: usize = 6;
pub const CUTOFF: usize = 7;
pub const RESONANCE: usize = 8;
pub const ATTACK: usize = 9;
pub const DECAY: usize = 10;
pub const SUSTAIN: usize = 11;
pub const RELEASE: usize = 12;
pub const LFO_RATE: usize = 13;
pub const LFO_WAVE: usize = 14;
pub const LFO_FILTER: usize = 15;
pub const SUB: usize = 16;
pub const DRIVE: usize = 17;
pub const LEVEL: usize = 18;
pub const FM: usize = 19;
pub const FILTER_ENV: usize = 20;
pub const PARAMS: usize = 21;
pub const DEFAULTS: [f32; PARAMS] = [
    1.0, 0.65, 0.0, 0.35, 0.0, 0.2, 0.6, 0.75, 0.15, 0.03, 0.2, 0.72, 0.2, 0.45, 0.0, 0.0, 0.12,
    0.12, 0.65, 0.05, 0.0,
];
const POLY: usize = 12;
const UNISON_MAX: usize = 8;
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Stage {
    #[default]
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}
#[derive(Clone, Copy, Default)]
struct Env {
    state: Stage,
    value: f32,
    release_step: f32,
}
impl Env {
    fn on(&mut self) {
        *self = Self {
            state: Stage::Attack,
            value: 0.0,
            release_step: 0.0,
        };
    }
    fn off(&mut self, release: f32, sr: f32) {
        if self.state != Stage::Off && self.state != Stage::Release {
            self.release_step = self.value / (release.max(0.001) * sr).max(1.0);
            self.state = Stage::Release;
        }
    }
    fn tick(&mut self, a: f32, d: f32, s: f32, sr: f32) -> f32 {
        match self.state {
            Stage::Off => self.value = 0.0,
            Stage::Attack => {
                self.value += 1.0 / (a * sr).max(1.0);
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.state = Stage::Decay;
                }
            }
            Stage::Decay => {
                self.value -= (1.0 - s) / (d * sr).max(1.0);
                if self.value <= s {
                    self.value = s;
                    self.state = Stage::Sustain;
                }
            }
            Stage::Sustain => self.value = s,
            Stage::Release => {
                self.value -= self.release_step;
                if self.value <= 0.0 {
                    *self = Self::default();
                }
            }
        }
        self.value.clamp(0.0, 1.0)
    }
}
#[derive(Clone, Copy, Default)]
struct Filter {
    i1: f32,
    i2: f32,
    g: f32,
    k: f32,
    a1: f32,
}
impl Filter {
    fn tune(&mut self, hz: f32, res: f32, sr: f32) {
        self.g = (PI * (hz / sr).clamp(0.00001, 0.44)).tan();
        self.k = 2.0 - 1.8 * res;
        self.a1 = 1.0 / (1.0 + self.g * (self.g + self.k));
    }
    fn low(&mut self, x: f32) -> f32 {
        let v3 = x - self.i2;
        let v1 = (self.i1 + self.g * v3) * self.a1;
        let v2 = self.i2 + self.g * v1;
        self.i1 = 2.0 * v1 - self.i1;
        self.i2 = 2.0 * v2 - self.i2;
        if self.i1.abs() < 1e-20 {
            self.i1 = 0.0;
        }
        if self.i2.abs() < 1e-20 {
            self.i2 = 0.0;
        }
        v2
    }
}
#[derive(Clone, Copy, Default)]
struct Voice {
    active: bool,
    key: u8,
    slide: [f32; 3],
    vel: f32,
    pan: f32,
    age: u64,
    env: Env,
    a: [f32; UNISON_MAX],
    b: [f32; UNISON_MAX],
    sub: f32,
    fl: Filter,
    fr: Filter,
    ticker: u8,
}
struct Settings {
    count: usize,
    wave_a: f32,
    wave_b: f32,
    mix: f32,
    detune: [f32; UNISON_MAX],
    pan: [f32; UNISON_MAX],
    norm: f32,
    cutoff: f32,
    res: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    lfo_rate: f32,
    lfo_wave: f32,
    lfo_filter: f32,
    sub: f32,
    drive: f32,
    level: f32,
    fm: f32,
    env_filter: f32,
}
impl Settings {
    fn of(p: &[f32; PARAMS]) -> Self {
        let times = |v: f32, lo: f32, hi: f32| lo * (hi / lo).powf(v);
        let count = (1.0 + 7.0 * p[UNISON]).round() as usize;
        let mut detune = [1.0; UNISON_MAX];
        let mut pan = [0.0; UNISON_MAX];
        let cents = 100.0 * p[DETUNE].powi(2);
        for i in 0..count {
            let off = if count == 1 {
                0.0
            } else {
                2.0 * i as f32 / (count - 1) as f32 - 1.0
            };
            detune[i] = 2.0_f32.powf(cents * off / 1200.0);
            pan[i] = off * 0.5 * p[WIDTH];
        }
        Self {
            count,
            wave_a: p[WAVE_A],
            wave_b: p[WAVE_B],
            mix: p[OSC_MIX],
            detune,
            pan,
            norm: 1.0 / (count as f32).sqrt(),
            cutoff: 30.0 * (17_000.0_f32 / 30.0).powf(p[CUTOFF]),
            res: p[RESONANCE],
            attack: times(p[ATTACK], 0.001, 5.0),
            decay: times(p[DECAY], 0.005, 10.0),
            sustain: p[SUSTAIN],
            release: times(p[RELEASE], 0.005, 10.0),
            lfo_rate: times(p[LFO_RATE], 0.03, 20.0),
            lfo_wave: p[LFO_WAVE],
            lfo_filter: p[LFO_FILTER],
            sub: p[SUB],
            drive: 1.0 + 12.0 * p[DRIVE].powi(2),
            level: p[LEVEL],
            fm: p[FM],
            env_filter: p[FILTER_ENV],
        }
    }
}
/// PolyBLEP correction for saw and pulse discontinuities.
#[inline]
fn blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let x = t / dt;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}
#[inline]
fn shape(kind: usize, t: f32, dt: f32) -> f32 {
    match kind {
        0 => fast_sine(t),
        1 => 2.0 * t - 1.0 - blep(t, dt),
        2 => {
            let y = if t < 0.5 { 1.0 } else { -1.0 };
            y + blep(t, dt) - blep((t + 0.5).fract(), dt)
        }
        _ => 2.0 * (2.0 * t - 1.0).abs() - 1.0,
    }
}
#[inline]
fn oscillator(morph: f32, phase: f32, step: f32) -> f32 {
    let x = morph * 3.0;
    let first = (x.floor() as usize).min(3);
    let blend = (x - first as f32).clamp(0.0, 1.0);
    let a = shape(first, phase, step);
    if blend == 0.0 {
        return a;
    }
    a + blend * (shape((first + 1).min(3), phase, step) - a)
}
pub struct AuroraSynth {
    sr: f32,
    p: [f32; PARAMS],
    voices: [Voice; POLY],
    serial: u64,
    lfo: f32,
    bend: f32,
}
impl AuroraSynth {
    pub fn new(sr: f32) -> Self {
        LazyLock::force(&SINE_TABLE);
        Self {
            sr,
            p: DEFAULTS,
            voices: [Voice::default(); POLY],
            serial: 0,
            lfo: 0.0,
            bend: 0.0,
        }
    }
    fn render(&mut self, cfg: &Settings, freqs: &[f32; POLY], lfo: f32) -> (f32, f32) {
        let sr = self.sr;
        let wa = (cfg.wave_a + 0.5 * cfg.lfo_wave * lfo).clamp(0.0, 1.0);
        let wb = (cfg.wave_b + 0.5 * cfg.lfo_wave * lfo).clamp(0.0, 1.0);
        let mut left = 0.0;
        let mut right = 0.0;
        for (i, v) in self.voices.iter_mut().enumerate() {
            if !v.active {
                continue;
            }
            let env = v.env.tick(cfg.attack, cfg.decay, cfg.sustain, sr);
            if v.env.state == Stage::Off {
                v.active = false;
                continue;
            }
            let base = freqs[i];
            let mut l = 0.0;
            let mut r = 0.0;
            for unison in 0..cfg.count {
                let step = (base * cfg.detune[unison] / sr).min(0.44);
                let b = oscillator(wb, v.b[unison], step);
                let phase = (v.a[unison] + cfg.fm * 0.2 * b).rem_euclid(1.0);
                let a = oscillator(wa, phase, step);
                let sample = (1.0 - cfg.mix) * a + cfg.mix * b;
                let pan = (v.pan + v.slide[1] + cfg.pan[unison]).clamp(0.0, 1.0);
                l += sample * (1.0 - pan).sqrt();
                r += sample * pan.sqrt();
                v.a[unison] = (v.a[unison] + step).fract();
                v.b[unison] = (v.b[unison] + step).fract();
            }
            l *= cfg.norm;
            r *= cfg.norm;
            let sub = fast_sine(v.sub) * cfg.sub * 0.5;
            v.sub = (v.sub + base * 0.5 / sr).fract();
            let sub_pan = (v.pan + v.slide[1]).clamp(0.0, 1.0);
            l += sub * (1.0 - sub_pan).sqrt();
            r += sub * sub_pan.sqrt();
            if v.ticker == 0 {
                let cutoff = (cfg.cutoff
                    * 2.0_f32.powf(lfo * cfg.lfo_filter * 4.0 + env * cfg.env_filter * 4.0))
                .clamp(30.0, 0.44 * sr);
                v.fl.tune(cutoff, cfg.res, sr);
                v.fr.tune(cutoff, cfg.res, sr);
            }
            v.ticker = (v.ticker + 1) % 32;
            let vol = env * (v.vel + v.slide[2]).clamp(0.0, 1.0) * cfg.level * 0.16;
            left += v.fl.low((l * cfg.drive).tanh()) * vol;
            right += v.fr.low((r * cfg.drive).tanh()) * vol;
        }
        (left, right)
    }
}
impl Instrument for AuroraSynth {
    fn set_parameter(&mut self, index: usize, value: f32) {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return;
        }
        if let Some(p) = self.p.get_mut(index) {
            *p = value;
        }
        // Switching an instrument off must not leave notes sounding when re-enabled.
        if index == 0 && value < 0.5 {
            self.reset();
        }
    }
    fn note_on(&mut self, key: u8, velocity: f32, pan: f32) {
        if self.p[0] < 0.5 || !velocity.is_finite() || velocity <= 0.0 || !pan.is_finite() {
            return;
        }
        self.serial = self.serial.wrapping_add(1);
        let index = self
            .voices
            .iter()
            .position(|v| !v.active)
            .or_else(|| {
                // Reuse the quietest releasing note before stealing a held note.
                self.voices
                    .iter()
                    .enumerate()
                    .filter(|(_, voice)| voice.env.state == Stage::Release)
                    .min_by(|(_, a), (_, b)| a.env.value.total_cmp(&b.env.value))
                    .map(|(i, _)| i)
            })
            .unwrap_or_else(|| {
                self.voices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, v)| v.age)
                    .map_or(0, |(i, _)| i)
            });
        let mut v = Voice {
            active: true,
            key,
            vel: velocity.clamp(0.0, 1.0),
            pan: pan.clamp(0.0, 1.0),
            age: self.serial,
            ..Voice::default()
        };
        v.env.on();
        for i in 0..UNISON_MAX {
            v.a[i] = (i as f32 * 0.618_034 + key as f32 * 0.013).fract();
            v.b[i] = (i as f32 * 0.381_966 + key as f32 * 0.017 + 0.25).fract();
        }
        self.voices[index] = v;
    }
    fn note_off(&mut self, key: u8) {
        let release = Settings::of(&self.p).release;
        for v in &mut self.voices {
            if v.active && v.key == key {
                v.env.off(release, self.sr);
            }
        }
    }
    fn all_notes_off(&mut self) {
        let release = Settings::of(&self.p).release;
        for v in &mut self.voices {
            if v.active {
                v.env.off(release, self.sr);
            }
        }
    }
    fn reset(&mut self) {
        self.voices = [Voice::default(); POLY];
        self.lfo = 0.0;
        self.bend = 0.0;
    }
    fn process(&mut self, left: &mut [f32], right: &mut [f32], ctx: ProcessContext) {
        if self.p[0] < 0.5 {
            return;
        }
        let cfg = Settings::of(&self.p);
        let mut freqs = [0.0; POLY];
        for (i, v) in self.voices.iter().enumerate() {
            if v.active {
                freqs[i] =
                    440.0 * 2.0_f32.powf((v.key as f32 - 69.0 + self.bend + v.slide[0]) / 12.0);
            }
        }
        if ctx.playing {
            self.lfo = (ctx.start_beat as f32 * 60.0 / ctx.tempo_bpm.max(1.0) * cfg.lfo_rate)
                .rem_euclid(1.0);
        }
        // Silent channels are common on Android; avoid processing 12 empty voices per sample.
        if self.voices.iter().all(|voice| !voice.active) {
            let frames = left.len().min(right.len()) as f32;
            self.lfo = (self.lfo + frames * cfg.lfo_rate / self.sr).fract();
            return;
        }
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let modulation = (TAU * self.lfo).sin();
            let (a, b) = self.render(&cfg, &freqs, modulation);
            *l += a;
            *r += b;
            self.lfo = (self.lfo + cfg.lfo_rate / self.sr).fract();
        }
    }
    fn set_controller(&mut self, index: usize, value: f32) {
        if index == 4 && value.is_finite() {
            self.bend = 4.0 * value.clamp(0.0, 1.0) - 2.0;
        }
    }
    fn set_slide(&mut self, pitch: f32, pan: f32, velocity: f32) {
        if pitch.is_finite() && pan.is_finite() && velocity.is_finite() {
            for voice in &mut self.voices {
                voice.slide = [
                    pitch.clamp(-48.0, 48.0),
                    pan.clamp(-1.0, 1.0),
                    velocity.clamp(-1.0, 1.0),
                ];
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn render(s: &mut AuroraSynth, n: usize) -> Vec<f32> {
        let mut l = vec![0.0; n];
        let mut r = l.clone();
        s.process(
            &mut l,
            &mut r,
            ProcessContext {
                sample_rate: 48000.0,
                tempo_bpm: 120.0,
                start_beat: 0.0,
                playing: false,
            },
        );
        l
    }
    #[test]
    fn audible_and_finite() {
        let mut s = AuroraSynth::new(48000.0);
        s.note_on(69, 1.0, 0.5);
        let l = render(&mut s, 4096);
        assert!(l.iter().any(|x| x.abs() > 0.0001));
        assert!(l.iter().all(|x| x.is_finite() && x.abs() < 10.0));
    }
    #[test]
    fn release_and_retrigger() {
        let mut s = AuroraSynth::new(48000.0);
        s.set_parameter(RELEASE, 0.0);
        s.note_on(60, 1.0, 0.5);
        render(&mut s, 1000);
        s.note_off(60);
        render(&mut s, 1000);
        assert!(s.voices.iter().all(|v| !v.active));
    }
    #[test]
    fn slide_offsets_apply_to_active_voices_not_new_ones() {
        let mut s = AuroraSynth::new(48000.0);
        s.note_on(60, 1.0, 0.5);
        s.set_slide(12.0, 0.25, -0.4);
        let first = s.voices.iter().find(|v| v.active && v.key == 60).unwrap();
        assert_eq!(first.slide, [12.0, 0.25, -0.4]);
        s.note_on(64, 1.0, 0.5);
        let second = s.voices.iter().find(|v| v.active && v.key == 64).unwrap();
        assert_eq!(second.slide, [0.0; 3]);
        s.set_slide(0.0, 1.0, -1.0);
        assert!(render(&mut s, 512).iter().all(|&x| x == 0.0));
    }

    #[test]
    fn interpolated_sine_matches_reference() {
        AuroraSynth::new(48000.0);
        for i in 0..4096 {
            let phase = (i as f32 + 0.37) / 4096.0;
            let expected = (TAU * phase).sin();
            assert!((fast_sine(phase) - expected).abs() < 0.000005);
        }
    }

    #[test]
    fn extreme_polyphonic_render_stays_finite_across_sample_rates() {
        for sample_rate in [44100.0, 48000.0, 96000.0] {
            let mut synth = AuroraSynth::new(sample_rate);
            for i in 1..PARAMS {
                synth.set_parameter(i, 1.0);
            }
            for key in 48..60 {
                synth.note_on(key, 1.0, 0.5);
            }
            let result = render(&mut synth, 4096);
            assert!(
                result.iter().all(|x| x.is_finite()),
                "sample rate {sample_rate}"
            );
            synth.all_notes_off();
            render(&mut synth, 4096);
            synth.reset();
            assert!(synth.voices.iter().all(|v| !v.active));
        }
    }

    #[test]
    fn disabled_instrument_never_outputs_audio_or_retains_stuck_notes() {
        let mut synth = AuroraSynth::new(48000.0);
        synth.note_on(69, 1.0, 0.5);
        assert!(render(&mut synth, 1024).iter().any(|x| x.abs() > 0.0001));
        synth.set_parameter(0, 0.0);
        assert!(synth.voices.iter().all(|v| !v.active));
        assert!(render(&mut synth, 1024).iter().all(|&x| x == 0.0));
        synth.note_on(69, 1.0, 0.5);
        assert!(synth.voices.iter().all(|v| !v.active));
        synth.set_parameter(0, 1.0);
        synth.note_on(69, 1.0, 0.5);
        assert!(render(&mut synth, 1024).iter().any(|x| x.abs() > 0.0001));
    }

    #[test]
    fn releasing_voice_is_stolen_before_held_notes() {
        let mut synth = AuroraSynth::new(48000.0);
        for key in 60..72 {
            synth.note_on(key, 1.0, 0.5);
        }
        render(&mut synth, 128);
        synth.note_off(61);
        synth.note_on(80, 1.0, 0.5);
        assert!(synth.voices.iter().any(|v| v.active && v.key == 80));
        assert!(synth.voices.iter().all(|v| !v.active || v.key != 61));
        assert!(synth.voices.iter().any(|v| v.active && v.key == 60));
    }

    #[test]
    fn bounded_polyphony() {
        let mut s = AuroraSynth::new(48000.0);
        for k in 0..40 {
            s.note_on(k, 1.0, 0.5);
        }
        assert_eq!(s.voices.iter().filter(|v| v.active).count(), POLY);
    }
}
