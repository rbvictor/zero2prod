//! src/startup.rs

use {
    crate::routes::{
        health_check,
        subscriptions
    },
    actix_web::{
        App,
        dev::Server,
        HttpRequest,
        HttpServer,
        Responder,
        web::{
            self,
            Data,
        },
    },
    std::net::TcpListener,
    sqlx::PgPool,
    tracing_actix_web::TracingLogger,
};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the connection in a smart pointer
    let db_pool = Data::new(db_pool);

    let server = HttpServer::new(move || {
        // Capture `connection` from the surrounding environment
        App::new()
            // Middlewares are added using the `wrap` method on `App`
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check::health_check))
            // A new entry in our routing table for POST /subscriptions requests
            .route("/subscriptions", web::post().to(subscriptions::subscribe))
            .route("/{name}", web::get().to(greet))
            .route("/", web::get().to(greet))
            .app_data(db_pool.clone())
    })
        .listen(listener)?
        .run();

    Ok(server)
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}
