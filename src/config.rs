use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

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
    pub sysctl: Plugin<plugins::sysctl::Settings>,
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

pub fn config(path: &PathBuf) -> Result<Config, Box<dyn Error>> {
    let config_str = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_str)?;

    Ok(config)
}
