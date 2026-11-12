//! Aurora Growl Forge's automatable parameter registry.
use honeydew_audio::growl::{self as g,DEFAULTS,PARAMS};
use super::{Display,ModuleSpec,ParamSpec as P};
static PARAM_SPECS:[P;PARAMS]={
 let mut p=[P::ENABLE;PARAMS];
 p[g::DRIVE]=P::plain("Drive",DEFAULTS[g::DRIVE]);
 p[g::VOWEL]=P::cont("Formant",DEFAULTS[g::VOWEL],Display::Freq{min:80.0,max:1760.0,hz_decimals:0,khz_decimals:1});
 p[g::RESONANCE]=P::plain("Resonance",DEFAULTS[g::RESONANCE]);
 p[g::SPREAD]=P::plain("Formant Spread",DEFAULTS[g::SPREAD]);
 p[g::MOTION]=P::plain("Motion Rate",DEFAULTS[g::MOTION]);
 p[g::DEPTH]=P::plain("Motion Depth",DEFAULTS[g::DEPTH]);
 p[g::FOLD]=P::plain("Wavefold",DEFAULTS[g::FOLD]);
 p[g::GRIT]=P::plain("Digital Grit",DEFAULTS[g::GRIT]);
 p[g::MIX]=P::cont("Wet Mix",DEFAULTS[g::MIX],Display::linear(0.0,100.0,0," %"));
 p
};
pub static MODULE:ModuleSpec=ModuleSpec::effect(g::GROWL,"Aurora Growl Forge","Aurora Growl Forge",false,&PARAM_SPECS,&[]);
