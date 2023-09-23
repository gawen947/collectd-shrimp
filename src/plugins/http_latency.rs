use serde::Deserialize;
use std::time;

use ureq::Error;

use crate::config::PluginConfig;
use crate::plugin;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// The text that is expected, otherwise returns -2
    pub expected: Option<String>,

    /// Maximum time for the query, otherwise returns the configured timeout value.
    pub timeout: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct State {
    agent: ureq::Agent,
    timeout: f32,
    result_fn: fn(ureq::Response, &Settings, time::Duration) -> String,
}

impl plugin::State<Settings> for State {
    fn new(_instance: &str, conf: &PluginConfig<Settings>, _targets: &[String]) -> Self {
        let settings = conf.settings.as_ref().unwrap();

        let mut timeout_value: f32 = f32::INFINITY;
        let mut builder = ureq::AgentBuilder::new();
        if let Some(timeout) = settings.timeout {
            builder = builder.timeout(time::Duration::from_secs_f32(timeout));
            timeout_value = timeout;
        }

        let result_fn: fn(ureq::Response, &Settings, time::Duration) -> String =
            if settings.expected.is_some() {
                |response, settings, duration| {
                    if let Ok(response_str) = response.into_string() {
                        if &response_str == settings.expected.as_ref().unwrap() {
                            duration.as_secs_f32().to_string()
                        } else {
                            "-2".to_owned() // unexpected response
                        }
                    } else {
                        "-3".to_owned() // cannot parse body
                    }
                }
            } else {
                |_, _, duration| duration.as_secs_f32().to_string()
            };

        Self {
            agent: builder.build(),
            timeout: timeout_value,
            result_fn,
        }
    }
}

impl plugin::PluginExecImplementation for Settings {
    type PluginState = State;

    fn pre(instance: &str, conf: &PluginConfig<Self>, targets: &[String]) {
        conf.check_setting_required(instance);
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
            let settings = conf.settings.as_ref().unwrap();

            // fetch the target
            let measurement_time = plugin::now();
            let start = time::Instant::now();
            let call_response = state.agent.get(target).call();
            let duration = start.elapsed();

            // compute the result
            let result: String = if duration.as_secs_f32() > state.timeout {
                state.timeout.to_string()
            } else {
                match call_response {
                    Ok(response) => (state.result_fn)(response, settings, duration),
                    Err(err) => match err {
                        Error::Status(code, _) => (-(code as i32)).to_string(),
                        Error::Transport(_) => "-1".to_owned(), // transport error
                    },
                }
            };

            results.push(plugin::PluginResult {
                time: measurement_time,
                value: result,
                target: Some(target),
                type_instance: None,
            });
        }

        results
    }

    fn name() -> &'static str {
        "http_latency"
    }

    fn desc() -> &'static str {
        "
        Fetch a URL using HTTP and returns the time required to issue the query.
        It's possible to specify a timeout or an expected value so that if the
        query returns something different than expected, the plugin returns an
        error value.

        Multiple errors value can be returned as negative numbers:

          * -1: Transport error.
          * -2: (if configured) The value is different from the configured expected value.
          * -3: Parse error while decoding the body.
          * -xxx: Error status code such as 404, 503, ...
        "
    }
}
