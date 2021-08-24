//! src/telemetry.rs

use {
    tracing::{
        Subscriber,
        subscriber::set_global_default,
    },
    tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer},
    tracing_log::LogTracer,
    tracing_subscriber::{
        fmt::MakeWriter,
        layer::SubscriberExt,
        EnvFilter,
        Registry,
    },
};

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
pub fn get_subscriber(name: String,
                      env_filter: String,
                      // A function that returns a sink - a place we can write log to
                      sink: impl MakeWriter + Send + Sync + 'static,
) -> impl Subscriber + Send + Sync {

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(name,  sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}