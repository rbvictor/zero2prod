//! tests/common/mod.rs

use {
    once_cell::sync::Lazy,
    std::net::TcpListener,
    sqlx::{Connection, Executor, PgConnection, PgPool},
    tracing_subscriber::fmt::writer::BoxMakeWriter,
    uuid::Uuid,
    zero2prod::{
        configuration::{
            get_configuration,
            DatabaseSettings
        },
        startup::run,
        telemetry::{
            get_subscriber,
            init_subscriber
        }
    }
};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let sink = if std::env::var("TEST_LOG").is_ok() {
        BoxMakeWriter::new(std::io::stdout)
    }
    else {
        BoxMakeWriter::new(std::io::sink)
    };

    let subscriber = get_subscriber("test".into(), "debug".into(), sink);
    init_subscriber(subscriber);
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// The function is asynchronous now!
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone())
        .expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database

    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
