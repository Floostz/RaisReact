use actix_web::{web, App, HttpServer, HttpResponse, Responder, post};
use actix_cors::Cors;
use serde::{Serialize, Deserialize};
use sqlx::{PgPool, Row};
use std::env;
use actix_web::middleware::Logger;
use std::sync::Mutex;


#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
}
#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}


#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

//async fn

async fn login_user(
    pool: web::Data<PgPool>,
    item: web::Json<LoginRequest>,
) -> impl Responder {
    let query = "SELECT password FROM users WHERE email = $1";
    let result = sqlx::query(query)
        .bind(&item.email)
        .fetch_optional(pool.get_ref())
        .await;

    match result {
        Ok(Some(record)) => {
            // Compare the provided password with the stored one
            if record.get::<String, _>("password") == item.password {
                HttpResponse::Ok().json(ApiResponse {
                    message: "Login successful!".to_string(),
                })
            } else {
                HttpResponse::Unauthorized().json(ApiResponse {
                    message: "Invalid email or password.".to_string(),
                })
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(ApiResponse {
            message: "Invalid email or password.".to_string(),
        }),
        Err(err) => {
            eprintln!("Database error: {}", err);
            HttpResponse::InternalServerError().json(ApiResponse {
                message: "Internal server error.".to_string(),
            })
        }
    }
}


async fn register_user(
    pool: web::Data<PgPool>,
    item: web::Json<RegisterRequest>,
) -> impl Responder {
    // Insert user into the database (you should hash the password before storing it)
    let query = "INSERT INTO users (email, password) VALUES ($1, $2)";
    let _ = sqlx::query(query)
        .bind(&item.email)
        .bind(&item.password)
        .execute(pool.get_ref())
        .await;

    HttpResponse::Ok().json(ApiResponse {
        message: "User registered successfully!".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create a database pool
    let pool = PgPool::connect(&database_url).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Cors::permissive()) // Use Cors::permissive() for a permissive CORS policy
            .wrap(Logger::default())
            .route("/register", web::post().to(register_user))
            .route("/login", web::post().to(login_user)) // Add the login route
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
