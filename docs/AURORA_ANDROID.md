# Honeydew Aurora — Android upgrade notes

This branch builds on the **original Honeydew Rust app** and its FLM-compatible project model. It does not substitute a standalone synth or a web wrapper.

## Added native modules
- **Aurora Wavetable** (instrument, type 1011): two morphable oscillators, FM, 1–8 unison, up to 12 notes, sub oscillator, ADSR, resonant filter, LFO, distortion and pitch/pan/velocity slides. Its oscillator morphs between sine, band-limited saw, pulse and triangle. It does **not** yet import Serum-style custom wavetable files.
- **Aurora Growl Forge** (effect, type 1009): dual moving formants, resonance, wavefolding, grit and wet/dry blend.
- **Aurora Metal Shift** (effect, type 1010): frequency (Hz) shifting with a 191-tap Hilbert filter, stereo offset, optional downward shifting and feedback.

All modules use Honeydew's existing automation parameters and module-rack controls; no separate application is required. Native instrument and effect processing uses Rust without allocations on the audio callback.

## User guide
1. Choose **Aurora Wavetable** from Honeydew's Synth list to add the instrument.
2. Add **Aurora Growl Forge** from the LEVEL effects category, or **Aurora Metal Shift** from MISC.
3. Save projects using Honeydew's existing project workflow. Existing Honeydew presets and project features remain in place.

## Compatibility and safeguards
- Native module IDs are written to Honeydew's `HDPm` extension chunks. FL Studio Mobile may ignore these extensions; never assume it can reproduce Aurora's sounds. **Keep backups of original projects** before editing or saving in a modified app.
- Dedicated regression tests cover save/reopen of a project with all three Aurora modules, Aurora preset save/reopen, non-finite DSP input, MIDI validation, muted notes, voice stealing, and maximum-polyphony finite processing.
- Sine waves use a 2048-point interpolated lookup table initialized before audio processing. Silent synth channels skip heavy DSP; released notes are preferentially reused to reduce voice-stealing clicks.
- Android CI builds debug APKs for ARMv7 and ARM64. Debug builds have their own package/signing configuration and may be installed separately from the developer's official application.
- Passing compilation and CI tests does not establish real-device audio latency, touch layout, or battery performance. Those need physical Android testing.

This experimental branch does not include a user-importable wavetable system, a factory preset pack, or a complete Serum 2 clone.
