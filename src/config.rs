use crate::probes;
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Interval of time (in seconds) between executing the probes for sampling data.
    pub interval: usize,

    pub probe_sysctl: Option<probes::sysctl::ProbeSysctl>,
}

pub fn config(path: &PathBuf) -> Result<Config, Box<dyn Error>> {
    let config_str = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_str)?;

    Ok(config)
}
