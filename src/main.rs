use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::exit;
use std::{thread, time};

mod config;
mod plugin;
mod plugins;
mod plugins_list;
mod utils;

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
    let (hostname, interval) = match (env::var("COLLECTD_HOSTNAME"), env::var("COLLECTD_INTERVAL"))
    {
        (Ok(hostname), Ok(interval)) => (hostname, interval),
        _ => {
            println!("error: cannot read env variable COLLECTD_HOSTNAME and COLLECTD_INTERVAL");
            println!(
                "error: these should be set either by collectd or by yourself if testing the probe"
            );
            exit(1);
        }
    };

    let sleep_duration =
        time::Duration::from_secs_f64(interval.parse::<f64>().unwrap_or_else(|_| {
            println!(
                "error: cannot parse COLLECTD_INTERVAL='{}' as an integer",
                interval
            );
            exit(1);
        }));

    // from the configuration, we assemble a vector of plugin instance to execute
    let mut plugin_instances = plugins_list::load_plugins(config, &hostname, &interval);

    loop {
        for plugin_instance in plugin_instances.iter_mut() {
            plugin_instance.exec();
        }

        // flush after executing all plugins
        io::stdout().flush().unwrap();
        thread::sleep(sleep_duration);
    }
}
