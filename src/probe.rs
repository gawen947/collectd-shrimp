use std::process::exit;
use std::time;

pub struct ProbeResult {
    pub time: time::Duration,
    pub name: String,
    pub value: Box<dyn std::fmt::Display>,
}

/// Trait that must be implemented by the probes.
pub trait Probe {
    /// Execute the probe and return the results for each type/instance.
    fn execute(&self) -> Vec<ProbeResult>;

    /// Specify the probe name as it will be registered in collectd.
    fn name(&self) -> &'static str;
}

/**
Execute a probe and display its result on stdout in the collectd plain text protocol.
cf: https://collectd.org/wiki/index.php/Plain_text_protocol
 */
pub fn execute_probe(hostname: &str, interval: &str, probe: &impl Probe) {
    let plugin_name = probe.name();

    for r in probe.execute() {
        let name = r.name;
        let time = r.time.as_secs();
        let value = r.value.to_string();

        println!("PUTVAL \"{hostname}/{plugin_name}/{name}\" interval={interval} {time}:{value}");
    }
}

pub fn now() -> time::Duration {
    time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| {
            println!("error: howdy fellow time traveler!");
            exit(1);
        })
}
