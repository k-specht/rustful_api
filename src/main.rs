extern crate warp;
extern crate tokio;
extern crate lazy_static;
extern crate rustract;

use warp::Filter;
use warp::hyper::{header, Method};
use warp::reject::Reject;
use lazy_static::lazy_static;
use std::convert::Infallible;

use rustract::db::Database;
use rustract::error::RustractError;
use rustract::init;

mod routes;
mod post;
mod get;
mod patch;
mod delete;

// Allows the database design to be used as a global.
// This is important because Warp's closures cannot take ownership of a non-static reference to the database.
lazy_static! {
    pub static ref DB_DESIGN: Database = init(Some("./config.json"), Some("./db_dump.sql"), true)
        .expect("failed to start example");
}

/// An environment config struct that stores server and database-related information.
/// 
/// This is not actually compatible with npm `.env` files due to being stored as JSON.
/// To get around this, you could use a custom `.env` serde (de/)serializer,
/// or use external crates instead.
/// 
/// Currently, database-related information is stored in DB_DESIGN's config.
/// TODO: Update rustract to support .env-stored data.
#[derive(serde::Serialize, serde::Deserialize)]
struct DotEnv {
    port: u16,
}

/// Entry point into the server.
/// 
/// TODO: Improve dotenv loading after adding support to rustract.
#[tokio::main]
async fn main() {
    let dotenv = load_env("./.env").await.expect("config file should load");
    start(dotenv.port).await.expect("server stopped, exiting app");
}

/// Loads the dotenv or config file from the filesystem.
async fn load_env(path: &str) -> Result<DotEnv, AppError> {
    let source = std::fs::read_to_string(path)?;
    let dotenv = serde_json::from_str::<DotEnv>(source.as_str())?;
    Ok(dotenv)
}

/// Serves the warp server on localhost, port 3030.
async fn start(port: u16) -> Result<(), RustractError> {
    // Configure CORS to allow any origin
    let cors = warp::cors()
        .allow_methods(&[Method::GET, Method::POST, Method::DELETE])
        .allow_headers(&[header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_any_origin();

    // Start the server
    println!("server started on port 3030");
    warp::serve(
        routes::gen_routes()
            .recover(handle_rejection)
            .with(cors)
        )
        .run(([127, 0, 0, 1], port))
        .await;

    // Once the thread-blocking await stops, the server has as well
    println!("server stopped");
    Ok(())
}

/// An error type `enum` representing the ways a client request could cause an error in the server logic.
#[derive(Debug)]
pub enum ErrorType {
    NotFound,
    Internal,
    BadRequest,
}

/// A custom error struct for making custom Warp `Rejection` replies.
#[derive(Debug)]
pub struct AppError {
    pub err_type: ErrorType,
    pub message: String,
}

impl AppError {
    /// Constructs a new error from the provided information.
    pub fn new(err_type: ErrorType, message: String) -> Self {
        AppError {
            err_type,
            message
        }
    }

    /// Creates a warp-compatible http error code from this error.
    pub fn to_http_status(&self) -> warp::http::StatusCode {
        match self.err_type {
            ErrorType::NotFound => warp::http::StatusCode::NOT_FOUND,
            ErrorType::Internal => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::BadRequest => warp::http::StatusCode::BAD_REQUEST,
        }
    }

    /// Wraps the error for warp's reject type for code readability.
    pub fn into_warp(self) -> warp::reject::Rejection {
        warp::reject::custom(self)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError {
            err_type: ErrorType::Internal,
            message: e.to_string()
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError {
            err_type: ErrorType::Internal,
            message: e.to_string()
        }
    }
}

/// A trait that allows calling "check" on `Option`s instead of unwrap.
/// 
/// This alternative to `unwrap` allows using the `?` operator instead of panicking.
/// It also provides a better error message.
pub(crate) trait Check<T> {
    /// Unwraps the provided `Option`, generating an appropriate error if needed.
    /// 
    /// This function exists as an alternative to `unwrap`.
    fn check(self) -> Result<T, AppError>;
}

impl<T> Check<T> for Option<T> {
    fn check(self) -> Result<T, AppError> {
        match self {
            Some(value) => Ok(value),
            None => Err(AppError {
                err_type: ErrorType::Internal,
                message: "err: value not found (internal)".to_string()
            })
        }
    }
}

/// Allows quickly generating internal errors as a replacement for `unwrap`.
impl<T> From<Option<T>> for AppError {
    fn from(_: Option<T>) -> Self {
        AppError {
            err_type: ErrorType::Internal,
            message: "err: option returned empty".to_string()
        }
    }
}

/// Allows the `AppError` struct to be used as a custom Warp Rejection.
impl Reject for AppError {}

/// An example of rejection handling.
pub async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message: String;

    // "Not Found" error
    if err.is_not_found() {
        code = warp::http::StatusCode::NOT_FOUND;
        message = "Not Found".to_string();

    // A custom error
    } else if let Some(app_err) = err.find::<AppError>() {
        code = app_err.to_http_status();
        message = app_err.message.clone();

    // "Invalid Body" error
    } else if err.find::<warp::filters::body::BodyDeserializeError>().is_some() {
        code = warp::http::StatusCode::BAD_REQUEST;
        message = "Invalid Body".to_string();
    
    // "Method Not Allowed" error
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = warp::http::StatusCode::METHOD_NOT_ALLOWED;
        message = "Method Not Allowed".to_string();

    // In case something was missed, logs and responds with 500
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("Unhandled rejection: {:?}", err);
    }

    // Constructs a JSON response with the error message
    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json, code))
}

/// An error-wrapping struct for replying to clients.
#[derive(serde::Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}
