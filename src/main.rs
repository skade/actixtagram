use actix_web::{web, App, HttpServer};
use actix_files as fs;


use actixtagram::{hello,echo,save_file};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::resource("/upload").route(web::post().to(save_file))
            )
            .service(hello)
            .service(echo)
            .service(fs::Files::new("/", "/static").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}