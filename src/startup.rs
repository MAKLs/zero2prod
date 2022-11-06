use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::routes;

pub fn run<A>(addr: A, pool: PgPool) -> std::io::Result<Server>
where
    A: std::net::ToSocketAddrs,
{
    let listener = TcpListener::bind(addr)?;
    run_with_listener(listener, pool)
}

pub fn run_with_listener(listener: TcpListener, pool: PgPool) -> std::io::Result<Server> {
    let pool = web::Data::new(pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(pool.clone())
            .route("/health", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
