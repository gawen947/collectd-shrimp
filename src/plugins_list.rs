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

    // FIXME: refactor that with macros

    // null plugin
    if let Some(plugin) = config.null {
        for (instance_name, instance_config) in &plugin {
            let plugin_instance: plugin::PluginInstance<plugins::null::Settings> =
                plugin::PluginInstance::new(
                    instance_config.to_owned(),
                    hostname.to_owned(),
                    instance_name.to_owned(),
                    interval.to_owned(),
                );

            plugins.push(Box::new(plugin_instance));
        }
    }

    #[cfg(feature = "sysctl")]
    if let Some(plugin) = config.sysctl {
        for (instance_name, instance_config) in &plugin {
            let plugin_instance: plugin::PluginInstance<plugins::sysctl::Settings> =
                plugin::PluginInstance::new(
                    instance_config.to_owned(),
                    hostname.to_owned(),
                    instance_name.to_owned(),
                    interval.to_owned(),
                );

            plugins.push(Box::new(plugin_instance));
        }
    }

    #[cfg(feature = "sysctl_factor")]
    if let Some(plugin) = config.sysctl_factor {
        for (instance_name, instance_config) in &plugin {
            let plugin_instance: plugin::PluginInstance<plugins::sysctl_factor::Settings> =
                plugin::PluginInstance::new(
                    instance_config.to_owned(),
                    hostname.to_owned(),
                    instance_name.to_owned(),
                    interval.to_owned(),
                );

            plugins.push(Box::new(plugin_instance));
        }
    }

    #[cfg(all(target_os = "freebsd", feature = "sysctl_temp"))]
    if let Some(plugin) = config.sysctl_temp {
        for (instance_name, instance_config) in &plugin {
            let plugin_instance: plugin::PluginInstance<plugins::sysctl_temp::Settings> =
                plugin::PluginInstance::new(
                    instance_config.to_owned(),
                    hostname.to_owned(),
                    instance_name.to_owned(),
                    interval.to_owned(),
                );

            plugins.push(Box::new(plugin_instance));
        }
    }

    #[cfg(feature = "file")]
    if let Some(plugin) = config.file {
        for (instance_name, instance_config) in &plugin {
            let plugin_instance: plugin::PluginInstance<plugins::file::Settings> =
                plugin::PluginInstance::new(
                    instance_config.to_owned(),
                    hostname.to_owned(),
                    instance_name.to_owned(),
                    interval.to_owned(),
                );

            plugins.push(Box::new(plugin_instance));
        }
    }

    #[cfg(feature = "file_factor")]
    if let Some(plugin) = config.file_factor {
        for (instance_name, instance_config) in &plugin {
            let plugin_instance: plugin::PluginInstance<plugins::file_factor::Settings> =
                plugin::PluginInstance::new(
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
