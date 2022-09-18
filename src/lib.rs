use obs_wrapper::{
    prelude::*,
    source::*,
    obs_register_module,
    obs_string,
};

use std::f32::consts::PI;

struct Coeffs {
   b0: f32,
   b1: f32,
   b2: f32,
   a1: f32,
   a2: f32,
}

struct OldValues {
    x_n1: f32,
    x_n2: f32,
    y_n1: f32,
    y_n2: f32,
}

struct BiquadFilter {
    channels: usize,
    coeffs_low_pass: Coeffs,
    old_values: [OldValues; 2],
}

impl BiquadFilter {
    // Low pass filter from Audio EQ Cookbook.
    fn create_low_pass(sample_rate: usize, freq: f32, q: f32) -> Coeffs {
        let w0 = 2.0 * PI * freq as f32 / sample_rate as f32;

        let alpha = w0.sin() / (2.0 * q);
        let b1 = 1.0 - w0.cos();
        let b0 = b1 / 2.0;
        let b2 = b0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;

        Coeffs {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    #[inline]
    fn apply_filter(c: &Coeffs, x_n: f32, x_n1: f32, x_n2: f32, y_n1: f32, y_n2: f32) -> f32 {
        c.b0 * x_n + c.b1 * x_n1 + c.b2 * x_n2 - c.a1 * y_n1 - c.a2 * y_n2
    }
}

impl Sourceable for BiquadFilter {
    fn get_id() -> ObsString {
        obs_string!("biquad_filter")
    }

    fn get_type() -> SourceType {
        SourceType::FILTER
    }

    fn create(create: &mut CreatableSourceContext<Self>, source: SourceContext) -> Self {
        let (sample_rate, channels) =
            create.with_audio(|audio| (audio.output_sample_rate(), audio.output_channels()));
        let coeffs_low_pass = BiquadFilter::create_low_pass(sample_rate, 200.0, 0.7);
        Self {
            channels,
            coeffs_low_pass,
            old_values: [OldValues {
                x_n1: 0.0,
                x_n2: 0.0,
                y_n1: 0.0,
                y_n2: 0.0,
            }, OldValues {
                x_n1: 0.0,
                x_n2: 0.0,
                y_n1: 0.0,
                y_n2: 0.0,
            }],
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
        for channel in 0..self.channels {
            let buffer = audio.get_channel_as_mut_slice(channel).unwrap();
            for output in buffer.iter_mut() {
                let sample = *output;
                let result = BiquadFilter::apply_filter(
                    &self.coeffs_low_pass,
                    sample,
                    self.old_values[channel].x_n1,
                    self.old_values[channel].x_n2,
                    self.old_values[channel].y_n1,
                    self.old_values[channel].y_n2
                );
                *output = result;

                self.old_values[channel].y_n2 = self.old_values[channel].y_n1;
                self.old_values[channel].x_n2 = self.old_values[channel].x_n1;
                self.old_values[channel].y_n1 = result;
                self.old_values[channel].x_n1 = sample;
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