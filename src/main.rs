use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use mongodb::Client;

mod magic_crypt;
mod models;
mod routers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    println!("Starting API on {}", dotenvy::var("BIND_ADDR").unwrap());

    let uri = std::env::var("MONGODB").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let client = Client::with_uri_str(uri).await.expect("failed to connect");

    println!("Connected to the database");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(web::Data::new(client.clone()))
            .configure(routers::v1)
    })
    .bind(dotenvy::var("BIND_ADDR").unwrap())?
    .run()
    .await
}
