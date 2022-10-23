use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};

use crate::routes;

pub fn run<A>(addr: A) -> std::io::Result<Server>
where
    A: std::net::ToSocketAddrs,
{
    let listener = TcpListener::bind(addr)?;
    run_with_listener(listener)
}

pub fn run_with_listener(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
