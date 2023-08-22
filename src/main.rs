use std::path::PathBuf;
use std::process::exit;
use std::{thread, time};
use std::io::{self, Write};
use std::env;

mod config;
mod probe;
mod probes;

fn main() {
    // find the config file according to the OS
    let config_path = match env::args().nth(1) {
        Some(path) => PathBuf::from(path),
        None => {
            let mut etc: PathBuf = match env::consts::OS {
                "freebsd" => "/usr/local/etc".into(),
                _ => "/etc".into(),
            };
            etc.push("collectd-shrimp.toml");
            etc
        }
    };

    // load/parse the config
    let config = config::config(&config_path).unwrap_or_else(|_| {
        println!(
            "error: cannot load configuration file '{}'",
            config_path.display()
        );
        exit(1);
    });

    // fetch expected env variables
    let (hostname, interval) = match (env::var("COLLECTD_HOSTNAME"), env::var("COLLECTD_INTERVAL")) {
        (Ok(hostname), Ok(interval)) => (hostname, interval),
        _ => {
            println!("error: cannot read env variable COLLECTD_HOSTNAME and COLLECTD_INTERVAL");
            println!("error: these should be set either by collectd or by yourself if testing the probe");
            exit(1);
        }
    };

    let sleep_duration = time::Duration::from_secs_f64(interval.parse::<f64>().unwrap_or_else(|_| {
        println!("error: cannot parse COLLECTD_INTERVAL='{}' as an integer", interval);
        exit(1);
    }));

    loop {
        if let Some(probe_sysctl) = &config.probe_sysctl {
            probe::execute_probe(&hostname,
                                 &interval,
                                 probe_sysctl);
        }

        // flush after executing all probes
        io::stdout().flush().unwrap();
        thread::sleep(sleep_duration);
    }
}
