//! Growl Forge native touch panel.
use honeydew_audio::growl as g;
use crate::panels::build::*;
pub(in crate::panels) fn panel()->Panel{
 Panel::new(vstack([
  hstack([knob(1,"Drive").p(g::DRIVE),knob(2,"Formant").p(g::VOWEL),
   knob(3,"Resonance").p(g::RESONANCE),knob(4,"Spread").p(g::SPREAD),
   knob(5,"Motion").p(g::MOTION)]),
  hstack([knob(6,"Depth").p(g::DEPTH),knob(7,"Wavefold").p(g::FOLD),
   knob(8,"Grit").p(g::GRIT),knob(9,"Mix").p(g::MIX)]),
 ]))
}
