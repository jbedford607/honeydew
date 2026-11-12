pub mod chunk;
mod legacy;
mod read;
mod write;

pub use chunk::FlmError;
pub use legacy::{read_flgsynth, read_flms};
pub use read::{
    BUDGET_BASE, MAX_ID, preset_fits, preset_type, read, read_drumset, read_preset, read_rack,
    read_with_notices,
};
pub use write::{write, write_drumset, write_preset, write_rack};
pub mod sf2d;

#[cfg(test)]
mod aurora_roundtrip_tests {
    use super::{read, read_preset, write, write_preset};
    use crate::song::{ChannelKind, Song, SynthKind};
    use honeydew_audio::{aurora_synth, frequency_shift, growl};

    #[test]
    fn aurora_modules_and_custom_parameters_survive_project_reopen() {
        let mut song = Song::default();
        song.add_channel(ChannelKind::Synth(SynthKind::AuroraWavetable));
        let rack = &mut song.channels[1].rack;
        rack.modules[0].params[aurora_synth::WAVE_A] = 0.91;
        rack.modules[0].params[aurora_synth::UNISON] = 0.85;
        rack.add(growl::GROWL);
        rack.add(frequency_shift::METAL_SHIFT);
        rack.modules[1].params[growl::VOWEL] = 0.24;
        rack.modules[2].params[frequency_shift::SHIFT] = 0.77;
        let encoded = write(&song).expect("export project with Aurora");
        assert!(encoded.windows(4).any(|w| w == b"HDPm"), "native effects and synth must be wrapped for FLM compatibility");
        let decoded = read(&encoded).expect("reopen project with Aurora");
        let modules = &decoded.channels[1].rack.modules;
        assert_eq!(modules.iter().map(|m| m.type_id).collect::<Vec<_>>(),
            [aurora_synth::AURORA_SYNTH, growl::GROWL, frequency_shift::METAL_SHIFT]);
        assert_eq!(modules[0].params[aurora_synth::WAVE_A], 0.91);
        assert_eq!(modules[0].params[aurora_synth::UNISON], 0.85);
        assert_eq!(modules[1].params[growl::VOWEL], 0.24);
        assert_eq!(modules[2].params[frequency_shift::SHIFT], 0.77);
    }

    #[test]
    fn aurora_synth_preset_reopens_with_its_parameters() {
        let mut song = Song::default();
        song.add_channel(ChannelKind::Synth(SynthKind::AuroraWavetable));
        let module = &mut song.channels[1].rack.modules[0];
        module.params[aurora_synth::FM] = 0.83;
        module.params[aurora_synth::RELEASE] = 0.31;
        let preset = write_preset(module).expect("write Aurora preset");
        let decoded = read_preset(&preset, Some(aurora_synth::AURORA_SYNTH))
            .expect("read Aurora preset");
        assert_eq!(decoded.params[aurora_synth::FM], 0.83);
        assert_eq!(decoded.params[aurora_synth::RELEASE], 0.31);
    }
}
