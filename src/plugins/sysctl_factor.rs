use serde::Deserialize;
use std::process::exit;

use crate::config::PluginConfig;
use crate::plugin;
use crate::utils;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub factor: f64,
}

impl plugin::PluginExecImplementation for Settings {
    type PluginState = plugin::EmptyState;

    fn pre(instance: &str, conf: &PluginConfig<Self>, targets: &[String]) {
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
            let raw = utils::sysctl::get_string(target).unwrap_or_else(|_| {
                eprintln!("error: cannot read sysctl key '{}'", target);
                exit(1);
            });
            let raw_int: i64 = raw.parse().unwrap_or_else(|_| {
                eprintln!("error: cannot parse sysctl key '{}' as integer", raw);
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
        "sysctl_factor"
    }

    fn desc() -> &'static str {
        "
        Read a integer value from sysctl to which an optional factor can be applied.
        For instance, say you have a sysctl for temperature given in m°C, where a value
        of 32128 would correspond to a temperature of 32.128°C. Then you can use this
        plugin with a factor of 0.001.

        Another example, suppose you have a sysctl that gives the number of memory pages.
        You can use a factor of 4096 (page size) to get amount of memory in bytes.
        "
    }
}
