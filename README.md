# OBS Biquad filter plugin

This is an experimental plugin, which implements biquad filter shown in [Audio EQ Cookbook](https://webaudio.github.io/Audio-EQ-Cookbook/Audio-EQ-Cookbook.txt).

# Requirements

This plugin is implemented in Rust. You need [Rust](https://www.rust-lang.org/) to build the plugin.

# Build & Install

```
% git clone https://github.com/yoshinorisano/obs-biquad-filter-plugin.git
% cd obs-biquad-filter-plugin
% cargo build
```

After build successfully, copy ``` biquadfilter.dll ``` to the OBS's plugin directory.