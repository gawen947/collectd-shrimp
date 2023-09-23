use serde::Deserialize;

use crate::config::PluginConfig;
use crate::plugin;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {}

impl plugin::PluginExecImplementation for Settings {
    type PluginState = plugin::EmptyState;

    fn pre(instance: &str, conf: &PluginConfig<Self>, targets: &[String]) {
        conf.check_no_setting_required(instance);
        conf.check_no_target_required(instance, targets);
    }

    fn exec<'a>(
        _instance: &str,
        _conf: &PluginConfig<Self>,
        _state: &mut Self::PluginState,
        _targets: &'a [String],
    ) -> Vec<plugin::PluginResult<'a>> {
        vec![plugin::PluginResult {
            time: plugin::now(),
            value: "0".to_owned(),
            target: None,
            type_instance: None,
        }]
    }

    fn name() -> &'static str {
        "null"
    }

    fn desc() -> &'static str {
        "The null plugin is always enabled. Its main goal is to avoid deadcode warnings \
        when compiling with no feature enabled."
    }
}
