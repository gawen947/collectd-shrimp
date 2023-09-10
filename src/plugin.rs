use std::process::exit;
use std::time;

use crate::config::PluginConfig;

/// Plugin result for one target/type-instance of the plugin execution.
pub struct PluginResult<'a> {
    /// The time of measurement
    pub time: time::Duration,

    /// The result value of the plugin
    pub value: String,

    /**
    The target that was used to compute this result.
    If type_instance below is not specified, this will
    be used to compute the type_instance. If both target
    and type_instance are None, the type_instance will
    not be set.
    */
    pub target: Option<&'a str>,

    /**
    The type_instance to use in the result. If this is set
    it will be used in priority to set the type instance.
    However, only use this parameter when you really need
    a custom type_instance instead of just repeating the
    target reference because it means you have to create
    a new string on each execution.
    */
    pub type_instance: Option<String>,
}

/**
Trait that must be implemented by all plugins so they can be executed.
The idea of a plugin is roughly the same as collectd, although it is slightly more
coupled to the type of data you want to measure rather than the type of measurement.
*/
pub trait PluginExecImplementation: Sized {
    type PluginState: State<Self>;

    /// Executed before any execution to check the configuration and eventually initialize stuff.
    fn pre(
        instance: &str,
        conf: &PluginConfig<Self>,
        state: &mut Self::PluginState,
        targets: &[String],
    );

    /// Execute an instance of the plugin and return the results for each type-instance.
    fn exec<'a>(
        instance: &str,
        conf: &PluginConfig<Self>,
        state: &mut Self::PluginState,
        targets: &'a [String],
    ) -> Vec<PluginResult<'a>>;

    /// Specify the name of the plugin as used in the collectd identifier.
    fn name() -> &'static str;

    /// A way for plugins to provide a description of themselves. Not really used for now but might be in the future.
    fn desc() -> &'static str;
}

/// Each plugin/plugin-instance can have some state associated to it.
pub trait State<T> {
    fn new(instance: &str, conf: &PluginConfig<T>, targets: &[String]) -> Self;
}

/// Useful for plugins that don't need any particular state.
#[derive(Debug, Clone)]
pub struct EmptyState {}
impl<T> State<T> for EmptyState {
    fn new(_instance: &str, _conf: &PluginConfig<T>, _targets: &[String]) -> Self {
        Self {}
    }
}

/**
Contains global state used to execute a plugin.
Mostly stuff like string that could be pre-built once
instead of being reassembled at each plugin execution,
along with references to the plugin configuration.
*/
pub struct PluginInstance<T>
where
    T: PluginExecImplementation + ToOwned + Clone,
{
    config: PluginConfig<T>,
    state: T::PluginState,

    targets: Vec<String>,

    instance: String,
    interval: String,
    putval_base_str: String,
}

impl<T> PluginInstance<T>
where
    T: PluginExecImplementation + ToOwned + Clone,
{
    pub fn new(
        plugin_config: PluginConfig<T>,
        hostname: String,
        instance: String,
        interval: String,
    ) -> Self {
        let mut targets: Vec<String> = match plugin_config.targets.to_owned() {
            Some(targets) => targets,
            None => vec![],
        };

        if plugin_config.target.is_some() {
            targets.push(plugin_config.target.to_owned().unwrap());
        }

        // we precompute some of the string that we shall print on each execution
        let plugin_name = T::name();
        let type_name = &plugin_config.r#type;
        let putval_base_str = format!("PUTVAL {hostname}/{plugin_name}-{instance}/{type_name}");

        let mut state = T::PluginState::new(&instance, &plugin_config, &targets);
        T::pre(&instance, &plugin_config, &mut state, &targets);

        Self {
            config: plugin_config,
            state,
            targets,
            instance,
            interval,
            putval_base_str,
        }
    }

    /// Echo the putval command to stdout.
    fn putval(&self, type_instance: Option<&str>, time: &str, value: &str) {
        let putval_base_str = &self.putval_base_str;
        let interval_str = &self.interval;

        // FIXME: we should probably abstract that away with a macro
        match type_instance {
            Some(type_instance) => {
                println!("{putval_base_str}-\"{type_instance}\" interval={interval_str} {time}:{value}")
            }
            None => println!("{putval_base_str} interval={interval_str} {time}:{value}"),
        };
    }
}

/**
This is the trait that will actually be used to execute each plugin instance
in the main loop of the program. Since this will be called on dyn trait objects,
it will use a vtable indirection on each call. If you have a better idea, feel
free to propose one.
*/
pub trait ExecutablePlugin {
    /// Execute the plugin instance, printing it's value on stdout.
    fn exec(&mut self);
}

impl<T, S> ExecutablePlugin for PluginInstance<T>
where
    T: PluginExecImplementation<PluginState = S> + ToOwned + Clone,
    S: State<T> + Clone,
{
    fn exec(&mut self) {
        for result in T::exec(&self.instance, &self.config, &mut self.state, &self.targets) {
            let time = result.time.as_secs().to_string();

            if let Some(type_instance) = result.type_instance {
                self.putval(Some(&type_instance), &time, &result.value);
            } else {
                self.putval(result.target, &time, &result.value);
            }
        }
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
