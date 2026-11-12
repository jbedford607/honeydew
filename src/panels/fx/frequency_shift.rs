//! Metal Shift native touch panel.
use honeydew_audio::frequency_shift as f;
use crate::panels::build::*;
pub(in crate::panels) fn panel()->Panel{
 Panel::new(vstack([
  hstack([knob(1,"Shift Hz").p(f::SHIFT),knob(2,"Stereo").p(f::STEREO),
   knob(3,"Feedback").p(f::FEEDBACK),knob(4,"Mix").p(f::MIX)]),
  hstack([list(5,"Up · Down",Bar::Left).p(f::DIRECTION).w(Size::U(4.0))]),
 ]))
}
