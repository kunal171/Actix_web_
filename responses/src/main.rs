use actix_web::{http::header::{ContentType, ContentEncoding}, HttpResponse};
use actix_web::{get, web, middleware, Responder, Result};
use serde::Serialize;

#[derive(Serialize)]
struct MyObj {
    name: String,
}

#[get("/")]
async fn index2() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .insert_header(("X-Hdr", "sample"))
        .body("data")
}


#[get("/a/{name}")]
async fn index(name: web::Path<String>) -> Result<impl Responder> {
    let obj = MyObj {
        name: name.to_string(),
    };
    Ok(web::Json(obj))
}

static HELLO_WORLD: &[u8] = &[
    0x1f, 0x8b, 0x08, 0x00, 0xa2, 0x30, 0x10, 0x5c, 0x00, 0x03, 0xcb, 0x48, 0xcd, 0xc9, 0xc9,
    0x57, 0x28, 0xcf, 0x2f, 0xca, 0x49, 0xe1, 0x02, 0x00, 0x2d, 0x3b, 0x08, 0xaf, 0x0c, 0x00,
    0x00, 0x00,
];

#[get("/gzip")]
async fn index3() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(ContentEncoding::Gzip)
        .body(HELLO_WORLD)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};

    HttpServer::new(|| App::new()
        .wrap(middleware::Compress::default())
        .service(index)
        .service(index2)
        .service(index3)
        )
        .bind(("127.0.0.1", 8081))?
        .run()
        .await
}

