//! Aurora Wavetable instrument, outside the imported FLM instrument table.
use super::{ModuleSpec, ParamSpec as P};
use honeydew_audio::aurora_synth::{AURORA_SYNTH, DEFAULTS, PARAMS};

static PARAMS_SPEC: [P; PARAMS] = {
    let mut p = [P::ENABLE; PARAMS];
    p[1] = P::plain("Wavetable A", DEFAULTS[1]);
    p[2] = P::plain("Wavetable B", DEFAULTS[2]);
    p[3] = P::plain("Oscillator Mix", DEFAULTS[3]);
    p[4] = P::plain("Unison Voices", DEFAULTS[4]);
    p[5] = P::plain("Unison Detune", DEFAULTS[5]);
    p[6] = P::plain("Stereo Spread", DEFAULTS[6]);
    p[7] = P::plain("Filter Cutoff", DEFAULTS[7]);
    p[8] = P::plain("Filter Resonance", DEFAULTS[8]);
    p[9] = P::plain("Amp Attack", DEFAULTS[9]);
    p[10] = P::plain("Amp Decay", DEFAULTS[10]);
    p[11] = P::plain("Amp Sustain", DEFAULTS[11]);
    p[12] = P::plain("Amp Release", DEFAULTS[12]);
    p[13] = P::plain("LFO Rate", DEFAULTS[13]);
    p[14] = P::plain("LFO Wave Amount", DEFAULTS[14]);
    p[15] = P::plain("LFO Filter Amount", DEFAULTS[15]);
    p[16] = P::plain("Sub Oscillator", DEFAULTS[16]);
    p[17] = P::plain("Drive", DEFAULTS[17]);
    p[18] = P::plain("Output Level", DEFAULTS[18]);
    p[19] = P::plain("FM Amount", DEFAULTS[19]);
    p[20] = P::plain("Filter Envelope Amount", DEFAULTS[20]);
    p
};
pub static MODULE: ModuleSpec = ModuleSpec::instrument(
    AURORA_SYNTH,
    "Aurora Wavetable",
    "Aurora Wavetable",
    &PARAMS_SPEC,
    &["OSC", "FILTER", "AMP", "MOD", "FX"],
);
