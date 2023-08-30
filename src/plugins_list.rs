use crate::config::Config;
use crate::plugins;
use crate::plugin;

/**
Load all the plugins found in the configuration file
and return them as a vector of executable instances.
*/
pub fn load_plugins<'a>(config: Config, hostname: &str, interval: &str) -> Vec<Box<dyn plugin::ExecutablePlugin>> {
    let mut plugins: Vec<Box<dyn plugin::ExecutablePlugin>> = vec![];

    if let Some(plugin) = config.sysctl {
        for (instance_name, instance_config) in &plugin {
            let plugin_instance: plugin::PluginInstance<plugins::sysctl::Settings, plugin::EmptyState> = plugin::PluginInstance::new(
               instance_config.to_owned(),
               hostname.to_owned(),
               instance_name.to_owned(),
               interval.to_owned(),
            );

            plugins.push(Box::new(plugin_instance));
        }
    }

    plugins
}