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
}

#[derive(Debug, Clone)]
pub struct State {
    pub scale: TemperatureScale,
    pub scale_fn: fn(&sysctl::Temperature) -> f32,
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

        Self { scale, scale_fn }
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
        conf.check_no_setting_required(instance);
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
                value: temp_value.to_string(),
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
