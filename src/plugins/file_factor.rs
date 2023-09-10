use serde::Deserialize;
use std::process::exit;

use crate::config::PluginConfig;
use crate::plugin;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub factor: f64,
}

impl plugin::PluginExecImplementation for Settings {
    type PluginState = plugin::EmptyState;

    fn pre(
        instance: &str,
        conf: &PluginConfig<Self>,
        _state: &mut Self::PluginState,
        targets: &[String],
    ) {
        conf.check_setting_required(instance);
        conf.check_target_required(instance, targets);
    }

    fn exec<'a>(
        _instance: &str,
        conf: &PluginConfig<Self>,
        _state: &mut Self::PluginState,
        targets: &'a [String],
    ) -> Vec<plugin::PluginResult<'a>> {
        let mut results: Vec<plugin::PluginResult> = Vec::with_capacity(targets.len());
        let factor = conf.settings.as_ref().unwrap().factor;

        for target in targets {
            let raw = std::fs::read_to_string(target).unwrap_or_else(|_| {
                println!("error: cannot file '{}'", target);
                exit(1);
            });
            let raw_int: i64 = raw.trim().parse().unwrap_or_else(|_| {
                println!("error: cannot parse raw value '{}' as integer", raw);
                exit(1);
            });
            let result = (raw_int as f64) * factor;

            results.push(plugin::PluginResult {
                time: plugin::now(),
                value: result.to_string(),
                target: Some(target),
                type_instance: None,
            });
        }

        results
    }

    fn name() -> &'static str {
        "file_factor"
    }

    fn desc() -> &'static str {
        "
        Read a integer value from a file to which an optional factor can be applied.\
        For instance, say you have a file in /sys that reports a temperature given in m°C,
        where a value of 32128 would correspond to a temperature of 32.128°C. Then you can
        use this plugin with a factor of 0.001.
        "
    }
}
