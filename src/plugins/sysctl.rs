use serde::Deserialize;
use std::process::exit;

use crate::config::PluginConfig;
use crate::plugin;
use crate::utils;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {}

impl plugin::PluginExecImplementation for Settings {
    type PluginState = plugin::EmptyState;

    fn exec<'a>(
        &self,
        _conf: &PluginConfig<Self>,
        _state: &mut Self::PluginState,
        targets: &'a [String],
    ) -> Vec<plugin::PluginResult<'a>> {
        let mut results: Vec<plugin::PluginResult> = Vec::with_capacity(targets.len());

        for target in targets {
            results.push(plugin::PluginResult {
                time: plugin::now(),
                value: utils::sysctl::get_string(target).unwrap_or_else(|_| {
                    println!("error: cannot read sysctl key '{}'", target);
                    exit(1);
                }),
                target: Some(target),
                type_instance: None,
            });
        }

        results
    }

    fn name(&self) -> &'static str {
        "sysctl"
    }
}
