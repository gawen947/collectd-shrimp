use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::process::exit;

use crate::plugin;
use crate::plugins;

type Plugin<T> = Option<HashMap<String, PluginConfig<T>>>;

/**
Global configuration.
Contains an optional plugin section for each plugin instance.
A plugin section is a HashMap with instance name as key and
plugin settings as value.
*/
#[derive(Debug, Deserialize)]
pub struct Config {
    pub null: Plugin<plugins::null::Settings>,

    #[cfg(feature = "sysctl")]
    pub sysctl: Plugin<plugins::sysctl::Settings>,

    #[cfg(feature = "sysctl_factor")]
    pub sysctl_factor: Plugin<plugins::sysctl_factor::Settings>,

    #[cfg(all(target_os = "freebsd", feature = "sysctl_temp"))]
    pub sysctl_temp: Plugin<plugins::sysctl_temp::Settings>,

    #[cfg(feature = "file")]
    pub file: Plugin<plugins::file::Settings>,

    #[cfg(feature = "file_factor")]
    pub file_factor: Plugin<plugins::file_factor::Settings>,
}

/**
Configuration that are parsed for each plugin instance.
Note that among those some keys are optional.
It's up to the plugin to decide if it should fail
or not if some optional key is missing.
*/
#[derive(Debug, Clone, Deserialize)]
pub struct PluginConfig<T> {
    /**
    The type of data in the sense of collectd
    (that is something specified in types.db).
    */
    pub r#type: String,

    /**
    Optionally change the name of the plugin.
    For instance if you have a set of plugins that work together
    to report temperature from various systems, you might want to
    call them "temperature" instead of "sysctl_temp", "sysctl"
    and "file_factor".
    */
    pub name: Option<String>,

    /**
    Optionally change the interval for this plugin instance.
    This must be greater than the interval provided by collectd.
    Each time the plugin is executed, it checks if the last execution
    is older than the custom configured interval and skip execution otherwise.
    */
    pub interval: Option<f32>,

    /**
    The targets for the instance (if required by the plugin).
    If there is only one target, you could use "target" instead.
    If there is more than one target, if will be used as the "type" instance
    in the collectd identifier.
    */
    pub targets: Option<Vec<String>>,

    /**
    The target for the instance, if there is only one.
    Note that you cannot specify both "targets" and "target" at the same time.
    */
    pub target: Option<String>,

    /**
    The individual settings for the plugin.
    It is optional, but it is up to the plugin to check if the setting is missing or not.
    */
    pub settings: Option<T>,
}

impl<T> PluginConfig<T>
where
    T: plugin::PluginExecImplementation,
{
    /// Check if there is some settings configured.
    #[allow(dead_code)]
    pub fn check_setting_required(&self, instance: &str) {
        if self.settings.is_none() {
            println!(
                "warning: '{}:{}' plugin requires some setting(s)",
                T::name(),
                instance
            );
            exit(1);
        }
    }

    /// Check if there is no setting configured.
    pub fn check_no_setting_required(&self, instance: &str) {
        if self.settings.is_some() {
            println!(
                "warning: '{}:{}' plugin requires no setting",
                T::name(),
                instance
            );
            exit(1);
        }
    }

    /// Check if there are at least some target configured.
    #[allow(dead_code)]
    pub fn check_target_required(&self, instance: &str, targets: &[String]) {
        if targets.is_empty() {
            println!(
                "warning: no target specified for '{}:{}' plugin",
                T::name(),
                instance
            );
            exit(1);
        }
    }

    /// Check if there is no target configured.
    pub fn check_no_target_required(&self, instance: &str, targets: &[String]) {
        if !targets.is_empty() {
            println!(
                "warning: '{}:{}' plugin requires no target",
                T::name(),
                instance
            );
            exit(1);
        }
    }
}

pub fn config(path: &PathBuf) -> Result<Config, Box<dyn Error>> {
    let config_str = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_str)?;

    Ok(config)
}
