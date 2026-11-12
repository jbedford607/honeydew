//! Aurora Metal Shift's automatable parameter registry.
use honeydew_audio::frequency_shift::{self as f,DEFAULTS,PARAMS};
use super::{Display,ModuleSpec,ParamSpec as P};
static PARAM_SPECS:[P;PARAMS]={
 let mut p=[P::ENABLE;PARAMS];
 p[f::SHIFT]=P::cont("Shift",DEFAULTS[f::SHIFT],Display::Freq{min:0.25,max:1000.0,hz_decimals:1,khz_decimals:1});
 p[f::DIRECTION]=P::menu("Direction",DEFAULTS[f::DIRECTION],&["Up","Down"]);
 p[f::STEREO]=P::plain("Stereo Spread",DEFAULTS[f::STEREO]);
 p[f::FEEDBACK]=P::plain("Feedback",DEFAULTS[f::FEEDBACK]);
 p[f::MIX]=P::cont("Wet Mix",DEFAULTS[f::MIX],Display::linear(0.0,100.0,0," %"));
 p
};
pub static MODULE:ModuleSpec=ModuleSpec::effect(f::METAL_SHIFT,"Aurora Metal Shift","Aurora Metal Shift",false,&PARAM_SPECS,&[]);
