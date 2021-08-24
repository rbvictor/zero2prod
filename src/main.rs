//! src/main.rs

use {
    // env_logger::Env,
    sqlx::postgres::PgPoolOptions,
    std::net::TcpListener,
    zero2prod::{
        configuration::get_configuration,
        startup::run,
        telemetry::{
            get_subscriber,
            init_subscriber,
        }
    },
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(),
                                    "info".into(),
                                    std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;

    // Create DB connection
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&configuration.database.connection_string())
        .expect("Failed to connect to Postgres.");

    run(listener, connection_pool)?.await
}