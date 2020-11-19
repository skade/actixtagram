use actix_web::{web, App, HttpServer};
use actix_files as fs;
use sqlx::SqlitePool;
use std::env;

use actixtagram::{hello,echo,save_file, processor, ProcessingRequest, AppData};
use tracing::{info, Level};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::INFO)
        .init();

    info!("Starting actixtagram");

    // DATABASE_URL: "sqlite:uploads.db"
    let sqlite = SqlitePool::connect(&env::var("DATABASE_URL").unwrap()).await.unwrap();

    let (sender, receiver) = async_channel::bounded::<ProcessingRequest>(50);

    for _ in 1..=8 {
        actix_rt::spawn(
            processor(receiver.clone())
        );
    }

    HttpServer::new(move || {
        let app_data = AppData { pool: sqlite.clone(), sender: sender.clone() };

        App::new()
            .data(app_data)
            .service(
                web::resource("/upload").route(web::post().to(save_file))
            )
            .service(hello)
            .service(echo)
            .service(fs::Files::new("/", "static").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}