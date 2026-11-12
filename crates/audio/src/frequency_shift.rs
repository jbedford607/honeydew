//! Aurora Metal Shift: 63-tap FIR Hilbert frequency shifter.
//! Unlike pitch shifting, adds a fixed frequency offset to every partial.
use std::f32::consts::{PI,TAU};
pub const METAL_SHIFT:u16=1010;
pub const SHIFT:usize=1;pub const DIRECTION:usize=2;pub const STEREO:usize=3;
pub const FEEDBACK:usize=4;pub const MIX:usize=5;pub const PARAMS:usize=6;
pub const DEFAULTS:[f32;PARAMS]=[1.0,0.49,0.0,0.3,0.0,0.7];
const TAPS:usize=63;const CENTRE:usize=31;
pub struct FrequencyShiftState{
 history:[[f32;TAPS];2],coefficients:[f32;TAPS],pos:usize,
 phase:[f32;2],step:[f32;2],previous:[f32;2],
 down:bool,feedback:f32,mix:f32,
}
impl FrequencyShiftState{
pub fn new(p:&[f32],sample_rate:f32)->Self{
 let mut coefficients=[0.0;TAPS];
 for (n,c) in coefficients.iter_mut().enumerate(){
  let k=n as i32-CENTRE as i32;
  if k!=0&&k%2!=0{
   let window=0.54-0.46*(TAU*n as f32/(TAPS-1) as f32).cos();
   *c=(2.0/(PI*k as f32))*window;
  }
 }
 let mut s=Self{history:[[0.0;TAPS];2],coefficients,pos:0,
 phase:[0.0;2],step:[0.0;2],previous:[0.0;2],
 down:false,feedback:0.0,mix:1.0};
 s.refresh(p,sample_rate);s
}
pub fn refresh(&mut self,p:&[f32],sr:f32){
 let v=|i:usize|p.get(i).copied().unwrap_or(DEFAULTS[i]).clamp(0.0,1.0);
 let hz=(0.25*4000.0_f32.powf(v(SHIFT))).min(sr*0.4);
 self.step=[TAU*hz/sr,TAU*(hz*(1.0+0.2*v(STEREO))).min(sr*0.4)/sr];
 self.down=v(DIRECTION)>=0.5;self.feedback=0.75*v(FEEDBACK);self.mix=v(MIX);
}
pub fn reset(&mut self){
 self.history=[[0.0;TAPS];2];self.phase=[0.0;2];
 self.previous=[0.0;2];self.pos=0;
}
fn channel(&mut self,ch:usize,x:f32)->f32{
 self.history[ch][self.pos]=(x+self.feedback*self.previous[ch]).clamp(-8.0,8.0);
 let delayed=self.history[ch][(self.pos+TAPS-CENTRE)%TAPS];
 let mut quadrature=0.0;
 for (age,&c) in self.coefficients.iter().enumerate(){
  if c!=0.0{quadrature+=c*self.history[ch][(self.pos+TAPS-age)%TAPS];}
 }
 let phase=self.phase[ch];
 let shifted=if self.down{delayed.mul_add(phase.cos(),quadrature*phase.sin())}
 else{delayed.mul_add(phase.cos(),-quadrature*phase.sin())};
 self.previous[ch]=shifted.clamp(-8.0,8.0);
 self.phase[ch]=(phase+self.step[ch]).rem_euclid(TAU);
 x+self.mix*(shifted-x)
}
pub fn process_frame(&mut self,l:f32,r:f32)->(f32,f32){
 let a=self.channel(0,l);let b=self.channel(1,r);
 self.pos=(self.pos+1)%TAPS;(a,b)
}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn mix_zero_passes_original(){
 let mut p=DEFAULTS;p[MIX]=0.0;
 let mut s=FrequencyShiftState::new(&p,48000.0);
 for n in 0..200{
 let x=n as f32/400.0-0.25;
 assert_eq!(s.process_frame(x,-x),(x,-x));
 }
 }
 #[test]fn bounded_and_resettable(){
 let p=[1.0;PARAMS];let mut s=FrequencyShiftState::new(&p,48000.0);
 for n in 0..50000{
 let x=(n as f32*0.1).sin();
 let (l,r)=s.process_frame(x,-x);
 assert!(l.is_finite()&&r.is_finite()&&l.abs()<100.0&&r.abs()<100.0);
 }
 s.reset();for _ in 0..100{assert_eq!(s.process_frame(0.0,0.0),(0.0,0.0));}
 }
}
