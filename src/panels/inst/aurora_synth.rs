//! Aurora Wavetable native touch panel.
use honeydew_audio::aurora_synth as a;
use crate::panels::build::*;
pub fn panel()->Option<Panel>{
 Some(Panel::new(vstack([
  tabs(0,"OSC · FILTER · AMP · MOD · FX").h(Size::U(1.25)),
  hstack([
   v_slider(1001,"Wave A").p(a::WAVE_A),v_slider(1002,"Wave B").p(a::WAVE_B),
   knob(1003,"Osc Mix").p(a::OSC_MIX),knob(1004,"Unison").p(a::UNISON),
   knob(1005,"Detune").p(a::DETUNE),knob(1006,"Width").p(a::WIDTH),
  ]).show(Show::Tab(0)),
  hstack([
   knob(2001,"Cutoff").p(a::CUTOFF),knob(2002,"Resonance").p(a::RESONANCE),
   knob(2003,"Envelope").p(a::FILTER_ENV),
  ]).show(Show::Tab(1)),
  hstack([
   v_slider(3001,"Attack").p(a::ATTACK),v_slider(3002,"Decay").p(a::DECAY),
   v_slider(3003,"Sustain").p(a::SUSTAIN),v_slider(3004,"Release").p(a::RELEASE),
  ]).show(Show::Tab(2)),
  hstack([
   knob(4001,"LFO Rate").p(a::LFO_RATE),knob(4002,"Wave Mod").p(a::LFO_WAVE),
   knob(4003,"Filter Mod").p(a::LFO_FILTER),
  ]).show(Show::Tab(3)),
  hstack([
   knob(5001,"Sub").p(a::SUB),knob(5002,"Drive").p(a::DRIVE),
   knob(5003,"FM Amount").p(a::FM),knob(5004,"Volume").p(a::LEVEL),
  ]).show(Show::Tab(4)),
 ])))
}
