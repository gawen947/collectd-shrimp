use serde::Deserialize;
use std::process::exit;
use sysctl::Sysctl;

use crate::probe::{Probe, ProbeResult, now};

#[derive(Debug, Deserialize)]
pub struct ProbeSysctl {
    /// List of sysctl keys to probe
    pub keys: Vec<String>,
}

impl Probe for ProbeSysctl {
    fn execute(&self) -> Vec<ProbeResult> {
        self.keys
            .iter()
            .map(|key| {
                let ctl = sysctl::Ctl::new(key).unwrap_or_else(|_| {
                    println!("error: A cannot read sysctl key '{}'", key);
                    exit(1);
                });

                let val = ctl.value_string()
                    .unwrap_or_else(|_| {
                        println!("error: B cannot read sysctl key '{}'", key);
                        exit(1);
                    });
                
                ProbeResult {
                    time: now(),
                    name: key.to_owned(),
                    value: Box::new(val.to_owned()),
                }
            })
            .collect()
    }

    fn name(&self) -> &'static str {
        "sysctl"
    }
}
