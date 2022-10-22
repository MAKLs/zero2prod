#![forbid(unsafe_code)]
use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run<A>(addr: A) -> std::io::Result<Server>
where
    A: std::net::ToSocketAddrs,
{
    let listener = TcpListener::bind(addr)?;
    run_with_listener(listener)
}

pub fn run_with_listener(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(|| App::new().route("/health", web::get().to(health_check)))
        .listen(listener)?
        .run();

    Ok(server)
}
