//! Deals with initialising logging

use crate::config::{self, Logger};
use crate::{Config, EnvFilter, logging::preinit_logger};
use std::fs::File;
use anyhow::Context;
use tracing::Subscriber;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{Layer, layer::SubscriberExt};

/// Small convenience wrapper around [`EnvFilter::parse`].
/// 
/// If `str` is [`None`], propogates it. If there was an error in parsing, logs.
/// in the preinit logs.
fn parse_envfilter_logging(str: Option<&str>) -> Option<EnvFilter> {
    str?.parse()
        .inspect_err(|e| {
            preinit_logger::warn(&format!("Error reading envfilter \"{e}\", skipping"));
        })
        .ok()
}

/// Initialises an individual logger from its config, and then returns the layer corresponding to
/// it.
/// 
/// Logs with the preinit logger when the file at the set logger path is not there (with the `info`
/// log level).
fn init_logger<S>(
    config: &config::Logger,
) -> anyhow::Result<Box<dyn Layer<S> + Send + Sync + 'static>>
where
    for<'span> S: Subscriber + LookupSpan<'span>,
{
    match config {
        Logger::Stdout {
            level,
            use_ansi,
            env_filter,
        } => Ok(tracing_subscriber::fmt::layer()
            .with_ansi(*use_ansi)
            .with_filter(LevelFilter::from_level(*level))
            .with_filter(parse_envfilter_logging(env_filter.as_deref()))
            .boxed()),

        Logger::File {
            level,
            path,
            use_ansi,
            env_filter,
        } => {
            if !path.exists() {
                preinit_logger::info(&format!(
                    "File at {} doesn't exist, creating",
                    path.display()
                ));
            }

            let file = File::create(path)
                .context(format!("Error creating logfile at path {}", path.display()))?;

            Ok(tracing_subscriber::fmt::layer()
                .with_ansi(*use_ansi)
                .with_writer(file)
                .with_filter(LevelFilter::from_level(*level))
                .with_filter(parse_envfilter_logging(env_filter.as_deref()))
                .boxed())
        }
    }
}

/// Initialises all the tracing loggers defined in a config.
/// 
/// # Logging
/// 
/// This logs when
/// - The file at the set logger path is not there (`info`)
/// - [`init_logger`] returned an error (`warn` + skips the logger)
/// - When there was an error running `tracing::set_global_default` (`error` and kills hte program)
pub fn init_loggers(config: &Config) {
    let loggers: Vec<_> = config
        .log
        .iter()
        .filter_map(|(k, config)| match init_logger(config) {
            Ok(config) => {
                preinit_logger::info(&format!("Inited logger {k}"));
                Some(config)
            }
            Err(e) => {
                preinit_logger::warn(&format!("Error initing logger {k}: {e}, skipping"));
                None
            }
        })
        .collect();

    let registry = tracing_subscriber::registry().with(loggers);

    if let Err(e) = tracing::subscriber::set_global_default(registry) {
        preinit_logger::error(&format!("Error starting tracing loggers: {e:?}"));
        std::process::exit(-1); // Can't think of another sane thing to do here that wouldn't leave
        // the app with no logs at all
    }

    tracing::info!("Inited loggers in config");
}
