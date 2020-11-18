use actix_web::{App, Error};
use actix_web::dev::Service;

use actix_web::test;
use actixtagram::{hello};
use actix_rt;

#[actix_rt::test]
async fn test_index_ok()  -> Result<(), Error> {
    let app = App::new()
                .service(hello);

    let mut app = test::init_service(app).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = app.call(req).await.unwrap();

    assert_eq!(resp.status(), http::StatusCode::OK);

    let response_body = match resp.response().body().as_ref() {
        Some(actix_web::body::Body::Bytes(bytes)) => bytes,
        _ => panic!("Response error"),
    };

    assert_eq!(response_body, r##"Hello world!"##);

    Ok(())
}
