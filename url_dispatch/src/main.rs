use actix_web::{web, App, HttpResponse, HttpServer, HttpRequest, Result, get, guard, http::header, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct Info {
    username: String,
}

// extract path info using serde
#[get("/{username}/index.html")] // <- define path parameters
async fn index1(info: web::Path<Info>) -> Result<String> {
    Ok(format!("Welcome {}!", info.username))
}

#[get("/a/{v1}/{v2}/")]
async fn index(req: HttpRequest) -> Result<String> {
    let v1: &str = req.match_info().get("v1").unwrap();
    let v2: u8 = req.match_info().query("v2").parse().unwrap();
    let (v3, v4): (&str, u8) = req.match_info().load().unwrap();
    Ok(format!("Values {} {} {} {}", v1, v2, v3, v4))
}

#[get("/show")]
async fn show_users() -> HttpResponse {
    HttpResponse::Ok().body("Show users")
}

#[get("/show/{id}")]
async fn user_detail(path: web::Path<(u32,)>) -> HttpResponse {
    HttpResponse::Ok().body(format!("User detail: {}", path.into_inner().0))
}

#[get("/test/")]
async fn index2(req: HttpRequest) -> Result<HttpResponse> {
    let url = req.url_for("kunal", ["1", "2", "3"])?; // <- generate url for "foo" resource

    Ok(HttpResponse::Found()
        .insert_header((header::LOCATION, url.as_str()))
        .finish())
}

#[get("/external")]
async fn index3(req: HttpRequest) -> impl Responder {
    let url = req.url_for("youtube", ["oHg5SJYRHA0"]).unwrap();
    assert_eq!(url.as_str(), "https://youtube.com/watch/oHg5SJYRHA0");

    url.to_string()
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index3)
            .external_resource("youtube", "https://youtube.com/watch/{video_id}")
                .service(
            web::scope("")
                .service(index)
                .service(index1)
                .service(
                    web::scope("users")
                    .service(show_users)
                    .service(user_detail),
                )
                .service(
                    web::resource("/test/{a}/{b}/{c}")
                    .name("kunal") // <- set resource name, then it could be used in `url_for`
                    .guard(guard::Get())
                    .to(HttpResponse::Ok),
                )
                .service(index2)
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
