use obs_wrapper::{
    prelude::*,
    source::*,
    obs_register_module,
    obs_string,
};

struct BiquadFilter {

}

impl Sourceable for BiquadFilter {
    fn get_id() -> ObsString {
        obs_string!("biquad_filter")
    }

    fn get_type() -> SourceType {
        SourceType::FILTER
    }

    fn create(create: &mut CreatableSourceContext<Self>, source: SourceContext) -> Self {
        Self {

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
        todo!();
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