use sqlx::PgPool;
use std::net::TcpListener;
use tracing::subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zero2prod::configuration;
use zero2prod::startup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Allows all logs to be redirected to the tracing subscriber
    // This allows to catch actix-web logs as traces
    LogTracer::init().expect("Failed to set LogTracer");

    // Filters the spans based on their log level
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("INFO"));
    // Emmits the actual logs to stdout
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    // Defines the subscriber used to process spans
    subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let configuration = configuration::get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    startup::run(listener, connection_pool)?.await
}
