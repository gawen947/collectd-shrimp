mod config;
mod probe;
mod probes;

use std::path::PathBuf;
use std::process::exit;
use std::{thread, time};

fn main() {
    let config_path = match std::env::args().nth(1) {
        Some(path) => PathBuf::from(path),
        None => {
            let mut etc: PathBuf = match std::env::consts::OS {
                "freebsd" => "/usr/local/etc".into(),
                _ => "/etc".into(),
            };
            etc.push("collectd-shrimp.toml");
            etc
        }
    };
    let config = config::config(&config_path).unwrap_or_else(|_| {
        println!(
            "error: cannot load configuration file '{}'",
            config_path.display()
        );
        exit(1);
    });

    let sleep_duration = time::Duration::from_secs(config.interval as u64);
    let hostname = gethostname::gethostname().into_string().unwrap();
    let interval = config.interval.to_string();
    loop {
        thread::sleep(sleep_duration);

        if let Some(probe_sysctl) = &config.probe_sysctl {
            probe::execute_probe(&hostname,
                                 &interval,
                                 probe_sysctl);
        }
    }
}
