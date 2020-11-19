use actix_web::{get, post, HttpResponse, Error, web};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use std::io::{Cursor, Write};
use sqlx::SqlitePool;
use image::io::Reader as ImageReader;
use blocking;
use tracing::{info, instrument};
use async_channel::{Sender, Receiver};
#[derive(Debug)]
pub struct AppData {
    pub pool: SqlitePool,
    pub sender: Sender<ProcessingRequest>,
}

#[derive(Debug)]
pub struct ProcessingRequest {
    filename: String,
    buffer: Vec<u8>,
}

#[get("/")]
pub async fn hello() -> HttpResponse {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
pub async fn echo(req_body: String) -> HttpResponse {
    HttpResponse::Ok().body(req_body)
}

#[instrument(name="image_processing",skip(payload,app_data))]
pub async fn save_file(mut payload: Multipart,  app_data: web::Data<AppData>) -> Result<HttpResponse, Error> {

    info!("Starting to read multipart file");

    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();

        let mut buffer: Vec<u8> = Vec::new();
        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            
            buffer.extend_from_slice(&data);
            //blocking::unblock(|| f.write_all(&data));
        }

        let db_filename = String::from(filename);

        let db_pool = app_data.pool.clone();

        actix_rt::spawn(
            create_unprocessed_upload(db_pool, db_filename)
        );

        app_data.sender.send(
            ProcessingRequest {
                filename: String::from(filename),
                buffer: buffer
            }
        ).await;

    }

    Ok(HttpResponse::Accepted().into())
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

pub async fn processor(mut receiver: Receiver<ProcessingRequest>) {
    while let Some(ProcessingRequest { filename, buffer}) = receiver.next().await {        
        blocking::unblock(move || { 
            let filepath = format!("./static/{}", &filename);
            let f = std::fs::File::create(filepath).unwrap();

            process_and_write_image(buffer, f) 
        }).await;
    }
}

fn process_and_write_image(buffer: Vec<u8>, mut to: std::fs::File) {
    let mut reader =  ImageReader::new(Cursor::new(buffer));
    reader.set_format(image::ImageFormat::Png);

    // insert sql notification here

    let mut img = reader.decode().unwrap();

    let cropped_image = img.crop(0, 0, 250, 250);
    
    cropped_image.write_to(&mut to, image::ImageOutputFormat::Png).unwrap();
} 