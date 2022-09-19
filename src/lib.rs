use obs_wrapper::{
    prelude::*,
    source::*,
    properties::*,
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

enum FilterType {
    LowPass,
    HighPass,
}

impl FilterType {
    fn convert(s: String) -> Self {
        match s.as_str() {
            "low_pass" => Self::LowPass,
            "high_pass" => Self::HighPass,
            _ => panic!("Unexpected filter type")
        }
    }
}

struct BiquadFilter {
    sample_rate: usize,
    channels: usize,
    filter_type: FilterType,
    coeffs: Coeffs,
    old_values: [OldValues; 2],
    cutoff_freq: f32,
    q: f32,
}

impl BiquadFilter {
    // Low pass filter from Audio EQ Cookbook.
    fn calc_low_pass(sample_rate: usize, freq: f32, q: f32) -> Coeffs {
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

    // High pass filter from Audio EQ Cookbook.
    fn calc_high_pass(sample_rate: usize, freq: f32, q: f32) -> Coeffs {
        let w0 = 2.0 * PI * freq as f32 / sample_rate as f32;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q);

        let b0 = (1.0 + cos_w0) / 2.0;
        let b1 = -1.0 - cos_w0;
        let b2 = b0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        Coeffs {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    fn calc_coeffs(filter_type: &FilterType, sample_rate: usize, cutoff_freq: f32, q: f32,) -> Coeffs {
        match filter_type {
            FilterType::LowPass => BiquadFilter::calc_low_pass(sample_rate, cutoff_freq, q),
            FilterType::HighPass => BiquadFilter::calc_high_pass(sample_rate, cutoff_freq, q)
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

    fn create(create: &mut CreatableSourceContext<Self>, _source: SourceContext) -> Self {
        let (sample_rate, channels) =
            create.with_audio(|audio| (audio.output_sample_rate(), audio.output_channels()));
        let settings = &create.settings;
        let filter_type = if let Some(filter_type) = settings.get::<std::borrow::Cow<'_, str>, _>(obs_string!("filter_type")) {
            FilterType::convert(filter_type.to_string())
        } else {
            FilterType::LowPass
        };

        let cutoff_freq = settings.get(obs_string!("cutoff_freq")).unwrap_or(200.0);
        let q = settings.get(obs_string!("q")).unwrap_or(0.7);
        let coeffs = BiquadFilter::calc_coeffs(&filter_type, sample_rate, cutoff_freq, q);
        Self {
            sample_rate,
            channels,
            filter_type,
            coeffs,
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
            cutoff_freq,
            q,
        }
    }
}

impl GetNameSource for BiquadFilter {
    fn get_name() -> ObsString {
        obs_string!("Biquad filter")
    }
}

impl GetPropertiesSource for BiquadFilter {
    fn get_properties(&mut self) -> Properties {
        let mut properties = Properties::new();
        let list_prop = &mut properties
            .add_list(
                obs_string!("filter_type"),
                obs_string!("Filter type"),
                false
            );
        list_prop.insert(0, obs_string!("Low pass"), obs_string!("low_pass"));
        list_prop.insert(1, obs_string!("High pass"), obs_string!("high_pass"));
        properties
            .add(
                obs_string!("cutoff_freq"),
                obs_string!("Cutoff frequency"),
                NumberProp::new_float(1.0 as f32)
                    .with_range(1.0..=10000.0)
                    .with_slider(),
            )
            .add(
                obs_string!("q"),
                obs_string!("Q"),
                NumberProp::new_float(0.01 as f32)
                    .with_range(0.01..=1.0) // Q must not include zero, which causes divide by zero error.
                    .with_slider(),
            );
        properties
    }
}
impl UpdateSource for BiquadFilter {
    fn update(&mut self, settings: &mut DataObj, _context: &mut GlobalContext) {
        if let Some(filter_type) = settings.get::<std::borrow::Cow<'_, str>, _>(obs_string!("filter_type")) {
            self.filter_type = FilterType::convert(filter_type.to_string())
        }
        if let Some(cutoff_freq) = settings.get::<f32, _>(obs_string!("cutoff_freq")) {
            self.cutoff_freq = cutoff_freq;
        }
        if let Some(q) = settings.get::<f32, _>(obs_string!("q")) {
            self.q = q;
        }

        self.coeffs = BiquadFilter::calc_coeffs(&self.filter_type, self.sample_rate, self.cutoff_freq, self.q);
    }
}

impl FilterAudioSource for BiquadFilter {
    fn filter_audio(&mut self, audio: &mut audio::AudioDataContext) {
        for channel in 0..self.channels {
            let buffer = audio.get_channel_as_mut_slice(channel).unwrap();
            for output in buffer.iter_mut() {
                let sample = *output;
                let result = BiquadFilter::apply_filter(
                    &self.coeffs,
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
            .enable_get_properties()
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