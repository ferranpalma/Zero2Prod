use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration;
use zero2prod::email_client::EmailClient;
use zero2prod::startup;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = telemetry::get_subscriber(
        String::from("zero2prod"),
        String::from("info"),
        std::io::stdout,
    );
    telemetry::init_subscriber(subscriber);

    // Expected to return a Settings instance that contains ApplicationSettings and DatabaseSettings
    let configuration = configuration::get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect_lazy_with(configuration.database.connect_with_db());

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let base_url = reqwest::Url::parse(&configuration.email_client.base_url)
        .unwrap_or_else(|_| panic!("Can't parse {} as url", configuration.email_client.base_url));
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port,
    );
    let listener = TcpListener::bind(address)?;
    startup::run(listener, connection_pool, email_client)?.await
}
