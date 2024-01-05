use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use bcrypt::{hash, verify, DEFAULT_COST};
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use regex::Regex;

const DB_NAME: &str = "myApp";
const COLL_NAME: &str = "users";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

//function for hashing password
fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Failed to hash password")
}

//function for verifying password while authenticating
fn verify_password(password: &str, hash: &str) -> bool {
    verify(password, hash).expect("Failed to verify password")
}


// Function to validate email format using a regular expression
fn is_valid_email(email: &str) -> bool {
    lazy_static::lazy_static! {
        static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    }
    EMAIL_REGEX.is_match(email)
}

/// Adds a new user to the "users" collection in the database.
#[post("/add_user")]
async fn add_user(client: web::Data<Client>, form: web::Form<User>) -> HttpResponse {
    // Validate the email address
    if !is_valid_email(&form.email) {
        return HttpResponse::BadRequest().body("Invalid email format");
    }
    // Check if passwords match
    if form.password != form.confirm_password {
        return HttpResponse::BadRequest().body("Passwords do not match");
    }
    // Hash the new password before updating the user
    let hashed_password = hash_password(&form.password);

    // Create a new User with the hashed password
    let user = User {
        first_name: form.first_name.clone(),
        last_name: form.last_name.clone(),
        username: form.username.clone(),
        email: form.email.clone(),
        password: hashed_password,
        confirm_password: String::new(), // Set to an empty string or handle it as needed
    };

    let collection = client.database(DB_NAME).collection(COLL_NAME);
    let result = collection.insert_one(user, None).await;
    // Validate the email format before proceeding
    match result {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Gets the user with the supplied username.
#[get("/get_user/{username}")]
async fn get_user(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
    let username = username.into_inner();
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    match collection
        .find_one(doc! { "username": &username }, None)
        .await
    {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => {
            HttpResponse::NotFound().body(format!("No user found with username {username}"))
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Gets the user with the supplied username.
#[get("/delete_user/{username}")]
async fn delete_user(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
    let username = username.into_inner();
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    match collection
        .delete_one(doc! { "username": &username }, None)
        .await
    {    
        Ok(result) => {
            if result.deleted_count > 0 {
                HttpResponse::Ok().body("User deleted successfully")
            } else {
                HttpResponse::NotFound().body(format!("No user found with username {}", username))
            }
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}


//Get User Sign In
#[get("/sign_in_user/{username}/{password}")]
async fn sign_in_user(client: web::Data<Client>, path: web::Path<(String, String)>) -> HttpResponse {
    let (username, password)  = path.into_inner();
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    match collection
    .find_one(doc! {"username" : &username}, None)
    .await {
        Ok(Some(user)) => {
            if verify_password(&password, &user.password) {
                HttpResponse::Ok().body(format!("SignIn Succesfull Welcome User {username}"))
            }else {
                HttpResponse::NotFound().body(format!("Incorrect Password"))
            }
        }
        Ok(None) => {
            HttpResponse::NotFound().body(format!("No user found with username {username}"))
        }
        Err(_) => todo!(),
    }
}
//edits User Profile
#[post("/edit_user/{username}")]
async fn edit_user(client: web::Data<Client>, username: web::Path<String>, user: web::Form<User>) -> HttpResponse {
    let username = username.into_inner();
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    // Create a filter based on the username
    let filter = doc! { "username": &username };

    if !is_valid_email(&user.email) {
        return HttpResponse::BadRequest().body("Invalid email format");
    }

    // Check if passwords match
    if user.password != user.confirm_password {
        return HttpResponse::BadRequest().body("Passwords do not match");
    }

    // Hash the new password before updating the user
    let hashed_password = hash_password(&user.password);
    // Create an update document with the new user data
    let update_doc = doc! {
        "$set": {
            "first_name": &user.first_name,
            "last_name": &user.last_name,
            "username": &user.username,
            "email": &user.email,
            "password" : &hashed_password
        }
    };

    // Use the update_one method to update the user
    match collection.update_one(filter, update_doc, None).await {
        Ok(result) => {
            if result.modified_count > 0 {
                HttpResponse::Ok().body("User updated successfully")
            } else {
                HttpResponse::NotFound().body(format!("No user found with username {}", username))
            }
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Creates an index on the "username" field to force the values to be unique.
async fn create_username_index(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(options)
        .build();
    client
        .database(DB_NAME)
        .collection::<User>(COLL_NAME)
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set in the .env file");

    let client = Client::with_uri_str(uri).await.expect("failed to connect");
    create_username_index(&client).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(add_user)
            .service(get_user)
            .service(delete_user)
            .service(edit_user)
            .service(sign_in_user)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}