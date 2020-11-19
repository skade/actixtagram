use actix_web::{get, post, HttpResponse, Error, web};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use std::io::{Cursor, Write};
use sqlx::SqlitePool;
use image::io::Reader as ImageReader;
use blocking;
use tracing::{info, instrument};

#[get("/")]
pub async fn hello() -> HttpResponse {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
pub async fn echo(req_body: String) -> HttpResponse {
    HttpResponse::Ok().body(req_body)
}

#[instrument(name="image_processing",skip(payload,db_pool))]
pub async fn save_file(mut payload: Multipart,  db_pool: web::Data<SqlitePool>) -> Result<HttpResponse, Error> {

    info!("Starting to read multipart file");

    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let filepath = format!("./static/{}", &filename);

        // File::create is blocking operation, use threadpool
        let mut f = std::fs::File::create(filepath).unwrap();

        let mut buffer: Vec<u8> = Vec::new();
        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            
            buffer.extend_from_slice(&data);
            //blocking::unblock(|| f.write_all(&data));
        }


        let db_filename = String::from(filename);

        let db_pool = db_pool.as_ref().clone();

        actix_rt::spawn(
            create_unprocessed_upload(db_pool, db_filename)
        );

        blocking::unblock(move || {
            let mut reader =  ImageReader::new(Cursor::new(buffer));
            reader.set_format(image::ImageFormat::Png);

            // insert sql notification here

            let mut img = reader.decode().unwrap();

            let cropped_image = img.crop(0, 0, 250, 250);
            
            cropped_image.write_to(&mut f, image::ImageOutputFormat::Png).unwrap();
        }).await;

    }
    drop(payload);
    Ok(HttpResponse::Ok().into())
}

#[instrument(skip(pool))]
pub async fn create_unprocessed_upload(pool: SqlitePool, filename: String) {
    info!("Writing file info into database");
    
    let mut conn = pool.acquire().await.unwrap();

    let id = sqlx::query!(
        r#"
        INSERT INTO uploads ( filename, processed )
        VALUES ( ?1, false )
        "#,
        filename
    )
    .execute(&mut conn)
    .await.unwrap();  
}