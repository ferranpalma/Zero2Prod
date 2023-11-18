use tracing::subscriber;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    // Filters the spans based on their log level
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    // Emmits the actual logs to stdout
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Allows all logs to be redirected to the tracing subscriber
    // This allows to catch actix-web logs as traces
    LogTracer::init().expect("Failed to set LogTracer");
    // Defines the subscriber used to process spans
    subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
