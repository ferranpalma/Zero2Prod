use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::{connection_pool, Application};
use zero2prod::telemetry;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// This value is initialized only in the first access
static TRACING: Lazy<()> = Lazy::new(|| {
    let subsciber_name = String::from("test");
    let default_filter_level = String::from("info");
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subsciber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subsciber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Connect to postgres
    let mut connection = PgConnection::connect_with(&config.connect_without_db())
        .await
        .expect("Failed to connect to Postgres.");

    // Create database instance for testing purposes
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Open connection to freshly created database
    let connection_pool = PgPool::connect_with(config.connect_with_db())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut config = get_configuration().expect("Failed to read configuration.");
        config.database.database_name = Uuid::new_v4().to_string();
        // Let the OS choose a random port
        config.application.port = 0;
        config
    };

    // Create and migrate the database
    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let address = format!("http://127.0.0.1:{}", application.get_port());
    let _ = tokio::spawn(application.run());

    TestApp {
        address,
        db_pool: connection_pool(&configuration.database),
    }
}
