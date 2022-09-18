use obs_wrapper::{
    prelude::*,
    source::*,
    obs_register_module,
    obs_string,
};

struct BiquadFilter {
    channels: usize
}

impl Sourceable for BiquadFilter {
    fn get_id() -> ObsString {
        obs_string!("biquad_filter")
    }

    fn get_type() -> SourceType {
        SourceType::FILTER
    }

    fn create(create: &mut CreatableSourceContext<Self>, source: SourceContext) -> Self {
        let channels = create.with_audio(|audio| audio.output_channels());
        Self {
            channels
        }
    }
}

impl GetNameSource for BiquadFilter {
    fn get_name() -> ObsString {
        obs_string!("Biquad filter")
    }
}

impl UpdateSource for BiquadFilter {
    fn update(&mut self, _settings: &mut DataObj, context: &mut GlobalContext) {
        todo!();
    }
}

impl FilterAudioSource for BiquadFilter {
    fn filter_audio(&mut self, audio: &mut audio::AudioDataContext) {
        let gain = 0.03;
        for channel in 0..self.channels {
            let buffer = audio.get_channel_as_mut_slice(channel).unwrap();
            for output in buffer.iter_mut() {
                *output = *output * gain;
            }
        }
    }
}

struct BiquadFilterModule {
    context: ModuleContext
}

impl Module for BiquadFilterModule {
    fn new(context: ModuleContext) -> Self {
        Self { context }
    }

    fn get_ctx(&self) -> &ModuleContext {
        &self.context
    }

    fn load(&mut self, load_context: &mut LoadContext) -> bool {
        let source = load_context
            .create_source_builder::<BiquadFilter>()
            .enable_get_name()
            .enable_update()
            .enable_filter_audio()
            .build();

        load_context.register_source(source);

        true
    }

    fn description() -> ObsString {
        obs_string!("Apply biquad filter to audio") }

    fn name() -> ObsString {
        obs_string!("Biquad filter")
    }

    fn author() -> ObsString {
        obs_string!("Yoshinori Sano")
    }
}

obs_register_module!(BiquadFilterModule);