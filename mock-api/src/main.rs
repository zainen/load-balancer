use std::{thread::sleep, time::Duration};

use axum::{http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};

use rand::Rng;
use serde::Serialize;

mod constants;

use constants::PORT;

enum ApiError {
    Unhealthy,
}

#[derive(Serialize)]
struct ErrorResponse {
    pub error: String
}

impl PartialEq for ApiError {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other),
        (Self::Unhealthy, Self::Unhealthy))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
           Self::Unhealthy => (StatusCode::BAD_REQUEST, "Api unhealthy") 
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });

        (status, body).into_response()
    }
}

async fn work() -> String {
    println!("Incoming request");
    let mut rng = rand::thread_rng();
    let random_number = rng.gen_range(10..=50);
    sleep(Duration::from_millis(random_number * 100));
    format!("hello from server {}", PORT.to_string())
}

async fn home() -> String {

    sleep(Duration::from_millis(10));
    format!("Hit home: {}", PORT.to_string())
}

async fn health_check() -> Result<impl IntoResponse, ApiError> {
    Ok("healthy")
}

#[tokio::main]
async fn main() {

    let app = Router::new()
        .route("/", get(home))
        .route("/work", get(work))
        .route("/work", post(work))
        .route("/health_check", get(health_check));

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", PORT.to_string())).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
