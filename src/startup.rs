use std::net::TcpListener;
use crate::routes::{health_check,subscribe};
use actix_web::{HttpServer, App, web,dev::Server};
use sqlx::PgPool;
// use actix_web::middleware::Logger;
use tracing_actix_web::TracingLogger;


pub fn run(listener:TcpListener, db_pool:PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move|| {
        App::new()
        // .wrap(Logger::default())
        .wrap(TracingLogger::default())
            .route("/health_check",web::get().to(health_check))
            .route("/subscription", web::post().to(subscribe))
            .app_data(db_pool.clone())
        })
    .listen(listener)?
    .run();
    
    Ok(server)
}