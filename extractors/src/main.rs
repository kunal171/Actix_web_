//!Actix Web provides a facility for type-safe request information access called extractors (i.e., impl FromRequest)
//! An extractor can be accessed as an argument to a handler function. 
//! Actix Web supports up to 12 extractors per handler function. 
//! Argument position does not matter
use actix_web::{get, web, App, HttpServer, Result};
use serde::{Serialize, Deserialize};

//It is also possible to extract path information to a type that implements the Deserialize trait 
//from serde by matching dynamic segment names with field names. 
//Here is an equivalent example that uses serde instead of a tuple type.

#[derive(Deserialize)]
struct Info {
    user_id: u32,
    friend: String,
}

// this handler gets called if the query deserializes into `Info` successfully
// otherwise a 400 Bad Request error response is returned

/// extract path info from "/users/{user_id}/{friend}" url
/// {user_id} - deserializes to a u32
/// {friend} - deserializes to a String
/// 
/// extract path info using serde
///
#[derive(Deserialize)]
struct Info1 {
    username: String,
}

// this handler gets called if the query deserializes into `Info` successfully
// otherwise a 400 Bad Request error response is returned
#[get("/")]
async fn index1(info: web::Json<Info1>) -> String {
    format!("Welcome {}!", info.username)
}

#[get("/users/{user_id}/{friend}")] // <- define path parameters
async fn index(info: web::Path<Info>) -> Result<String> {
    Ok(format!(
        "Welcome {}, user_id {}!",
        info.friend, info.user_id
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index)
        .service(index1))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}