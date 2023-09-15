use serde::Deserialize;
use std::process::exit;

use crate::config::PluginConfig;
use crate::plugin;
use crate::utils;

#[derive(Debug, Clone, Deserialize)]
pub enum TemperatureScale {
    Kelvin,
    Celsius,
    Fahrenheit, // don't use this ;)
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub scale: Option<TemperatureScale>,
    pub precision: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct State {
    pub scale: TemperatureScale,
    pub scale_fn: fn(&sysctl::Temperature) -> f32,
    pub format_fn: fn(f32) -> String,
}

impl plugin::State<Settings> for State {
    fn new(_instance: &str, conf: &PluginConfig<Settings>, _targets: &[String]) -> Self {
        // unless otherwise specified the default scale is Celsius (don't complain)
        let scale = if let Some(settings) = &conf.settings {
            if let Some(scale) = &settings.scale {
                scale.to_owned()
            } else {
                TemperatureScale::Celsius
            }
        } else {
            TemperatureScale::Celsius
        };

        let scale_fn = match scale {
            TemperatureScale::Kelvin => sysctl::Temperature::kelvin,
            TemperatureScale::Celsius => sysctl::Temperature::celsius,
            TemperatureScale::Fahrenheit => sysctl::Temperature::fahrenheit,
        };

        /*
        Originally this code used a boxed closure.
        But it is probably slightly more performant and
        also more simple to use function pointers instead.
        */
        let format_fn = if let Some(settings) = &conf.settings {
            if let Some(precision) = &settings.precision {
                match precision {
                    0 => |v| format!("{:.0}", v),
                    1 => |v| format!("{:.1}", v),
                    2 => |v| format!("{:.2}", v),
                    3 => |v| format!("{:.3}", v),
                    4 => |v| format!("{:.4}", v),
                    5 => |v| format!("{:.5}", v),
                    6 => |v| format!("{:.6}", v),
                    7 => |v| format!("{:.7}", v),
                    _ => |v| format!("{:.8}", v),
                }
            } else {
                |v: f32| v.to_string()
            }
        } else {
            |v: f32| v.to_string()
        };

        Self { scale, scale_fn, format_fn }
    }
}

impl plugin::PluginExecImplementation for Settings {
    type PluginState = State;

    fn pre(
        instance: &str,
        conf: &PluginConfig<Self>,
        _state: &mut Self::PluginState,
        targets: &[String],
    ) {
        conf.check_target_required(instance, targets);
    }

    fn exec<'a>(
        _instance: &str,
        _conf: &PluginConfig<Self>,
        state: &mut Self::PluginState,
        targets: &'a [String],
    ) -> Vec<plugin::PluginResult<'a>> {
        let mut results: Vec<plugin::PluginResult> = Vec::with_capacity(targets.len());

        for target in targets {
            let raw = utils::sysctl::get(target).unwrap_or_else(|_| {
                println!("error: cannot read sysctl key '{}'", target);
                exit(1);
            });
            let temp = raw.as_temperature().unwrap_or_else(|| {
                println!(
                    "error: cannot parse key '{}' with value '{}' as a temperature",
                    target, raw
                );
                exit(1);
            });
            let temp_value = (state.scale_fn)(temp); // we scale the temperature according to what has been stored in state

            results.push(plugin::PluginResult {
                time: plugin::now(),
                value: (state.format_fn)(temp_value),
                target: Some(target),
                type_instance: None,
            });
        }

        results
    }

    fn name() -> &'static str {
        "sysctl_temp"
    }

    fn desc() -> &'static str {
        "
        Read a temperature value from sysctl and transform it in either °K, °C or °F scale.
        If it is not possible to parse the sysctl as a temperature, the plugin fails.
        "
    }
}
