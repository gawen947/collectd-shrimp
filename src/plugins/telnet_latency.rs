use std::error::Error;
use std::time;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::time::Duration;
use serde::Deserialize;

use crate::config::PluginConfig;
use crate::plugin;

type ReadFn = fn(TcpStream, Option<&str>) -> Result<bool, Box<dyn Error>>;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// If specified, this text is sent before expecting a response.
    pub query: Option<String>,

    /// The text that is expected, otherwise returns -2
    pub expect: Option<String>,

    /// Maximum time for the query, otherwise returns the configured timeout value.
    pub timeout: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct State {
    timeout_duration: Duration,
    read_fn: ReadFn,
}

impl plugin::State<Settings> for State {
    fn new(_instance: &str, conf: &PluginConfig<Settings>, _targets: &[String]) -> Self {
        let settings = conf.settings.to_owned().unwrap_or(Settings {
            query: None,
            expect: None,
            timeout: None,
        });

        let timeout_value: f32 = settings.timeout.unwrap_or(f32::INFINITY);
        let read_fn = match settings.expect {
            Some(_) => read_expected,
            None => read_onebyte
        };

        Self {
            timeout_duration: Duration::from_secs_f32(timeout_value),
            read_fn
        }
    }
}

/**
Read enough bytes to compare to the expected string.
Returns true if both string match.
*/
fn read_expected(mut stream: TcpStream, expected: Option<&str>) -> Result<bool, Box<dyn Error>> {
    let expected = expected.unwrap();
    let mut buf = vec!(0u8; expected.as_bytes().len());

    stream.read_exact(&mut buf)?;

    let received = std::str::from_utf8(&buf)?;

    Ok(expected == received)
}

/// Read only one byte and returns.
fn read_onebyte(mut stream: TcpStream, _: Option<&str>) -> Result<bool, Box<dyn Error>> {
    // we also use read_exact() instead of read() here because it automatically ignores ErrorKind::Interrupted
    stream.read_exact(&mut [0u8; 1])?;
    Ok(true)
}


/**
Connect using TCP to the target and read either the first character
or enough to compare to a given string. Returns true if string match
or any character was read.
*/
fn query_tcp(target: &str,
             timeout: Duration,
             read_fn: ReadFn,
             query: Option<&str>,
             expected: Option<&str>) -> Result<bool, Box<dyn Error>> {
    let mut stream = TcpStream::connect(target)?;

    stream.set_write_timeout(Some(timeout))?;
    stream.set_read_timeout(Some(timeout))?;

    if let Some(query) = query {
        stream.write_all(query.as_bytes())?;
    }

    read_fn(stream, expected)
}

impl plugin::PluginExecImplementation for Settings {
    type PluginState = State;

    fn pre(instance: &str, conf: &PluginConfig<Self>, targets: &[String]) {
        conf.check_target_required(instance, targets);
    }

    fn exec<'a>(
        _instance: &str,
        conf: &PluginConfig<Self>,
        state: &mut Self::PluginState,
        targets: &'a [String],
    ) -> Vec<plugin::PluginResult<'a>> {
        let mut results: Vec<plugin::PluginResult> = Vec::with_capacity(targets.len());

        for target in targets {
            let measurement_time = plugin::now();
            let start = time::Instant::now();
            let query_response = query_tcp(
                target,
                state.timeout_duration,
                state.read_fn,
                conf.settings.as_ref().and_then(|s| s.query.as_deref()),
                conf.settings.as_ref().and_then(|s| s.expect.as_deref())
            );

            let value = match query_response {
                Ok(true) => {
                    let duration = start.elapsed();
                    duration.as_secs_f32().to_string()
                }
                Ok(false) => "-2".to_string(),
                Err(_) => "-1".to_string()
            };

            results.push(plugin::PluginResult {
                time: measurement_time,
                value,
                target: Some(target),
                type_instance: None,
            });
        }

        results
    }


    fn name() -> &'static str {
        "telnet_latency"
    }

    fn desc() -> &'static str {
        "
        Connect to a specified host and port and returns the time required to
        receive a response. It's possible to specify a timeout or an expected value
        so that if the query returns something different than expected, the plugin
        returns an error value. If no expected value is provided, it measures the
        time to the first received byte.

        Multiple errors value can be returned as negative numbers:

          * -1: Connection or IO error.
          * -2: (if configured) The value is different from the configured expected value.
        "
    }
}