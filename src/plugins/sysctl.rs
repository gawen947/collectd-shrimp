use serde::Deserialize;
use std::process::exit;

use crate::config::PluginConfig;
use crate::plugin;
use crate::utils;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {}

impl plugin::PluginExecImplementation for Settings {
    type PluginState = plugin::EmptyState;

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
        _state: &mut Self::PluginState,
        targets: &'a [String],
    ) -> Vec<plugin::PluginResult<'a>> {
        let mut results: Vec<plugin::PluginResult> = Vec::with_capacity(targets.len());

        for target in targets {
            results.push(plugin::PluginResult {
                time: plugin::now(),
                value: utils::sysctl::get_string(target).unwrap_or_else(|_| {
                    eprintln!("error: cannot read sysctl key '{}'", target);
                    exit(1);
                }),
                target: Some(target),
                type_instance: None,
            });
        }

        results
    }

    fn name() -> &'static str {
        "sysctl"
    }

    fn desc() -> &'static str {
        "Read raw values from sysctl."
    }
}
