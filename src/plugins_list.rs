use std::process::exit;

use crate::config::Config;
use crate::plugin;
use crate::plugins;

/**
Load all the plugins found in the configuration file
and return them as a vector of executable instances.
*/
pub fn load_plugins(
    config: Config,
    hostname: &str,
    interval: &str,
) -> Vec<Box<dyn plugin::ExecutablePlugin>> {
    let mut plugins: Vec<Box<dyn plugin::ExecutablePlugin>> = vec![];

    if let Some(plugin) = config.sysctl {
        for (instance_name, instance_config) in &plugin {
            let plugin_instance: plugin::PluginInstance<plugins::sysctl::Settings> = plugin::PluginInstance::new(
                instance_config.to_owned(),
                hostname.to_owned(),
                instance_name.to_owned(),
                interval.to_owned(),
            );

            plugins.push(Box::new(plugin_instance));
        }
    }

    if plugins.is_empty() {
        println!("warning: no plugin configured");
        exit(1);
    }

    plugins
}
