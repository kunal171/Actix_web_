use actix_web::{web, App, HttpServer, Responder, get, HttpResponse};
use std::sync::Mutex;

//This struct Represent State 
struct AppState {
    app_name: String,
}

struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

#[get("/")]
async fn index(data: web::Data<AppState>) -> impl Responder {
    let app_name = &data.app_name; // Get the app_name
    format!("Hello {app_name}!")
}

async fn index1(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard

    format!("Request number: {counter}") // <- response with count
}

// this function could be located in a different module
fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/test")
            .route(web::get().to(|| async { HttpResponse::Ok().body("test") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

// this function could be located in a different module
fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app")
            .route(web::get().to(|| async { HttpResponse::Ok().body("app") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // Note: web::Data created _outside_ HttpServer::new closure
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(AppState {
            app_name: String::from("Actix web"),
        }))
        .app_data(counter.clone())
        .route("/hey", web::get().to(index1))
        .service(index)
        .configure(config)
        .service(web::scope("/api").configure(scoped_config))
    })
    .bind(("127.0.0.1",8080))?
    .run()
    .await
}