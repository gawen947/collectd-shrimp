use std::error::Error;
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Interval of time (in seconds) between executing the probes for sampling data.
    pub interval: usize,

    pub probe_sysctl: Option<ProbeSysctl>
}

#[derive(Debug, Deserialize)]
pub struct ProbeSysctl {
    /// List of sysctl keys to probe
    pub keys: Vec<String>
}

pub fn config(path: &PathBuf) -> Result<Config, Box<dyn Error>> {
    let config_str = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_str)?;

    Ok(config)
}