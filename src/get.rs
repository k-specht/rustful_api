use std::collections::HashMap;
use warp::Rejection;
use warp::Reply;
use warp::Filter;
use rustract::types::DataTypeValue;

use crate::Check;
use crate::DB_DESIGN;
use crate::ErrorType;
use crate::AppError;
use crate::routes::respond;
use crate::routes::with_json_body;

// GET <domain>/api/test/login
/// A function that returns a warp route for logging in a user.
pub(crate) fn get_user() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("login")
        .and(warp::get())
        .and(with_json_body())
        .and_then(get_extract)
        .and_then(get_retrieve)
        .and_then(get_success)
}

/// Uses the fields to create a PATCH query.
/// 
/// The `req` variable now has some of the data specified by the `FieldDesign`.
/// Since not all of it is included, error handling is more important here.
/// 
/// You could also use the global's field info to avoid possible developer error!
pub(crate) async fn get_retrieve(mut req: HashMap<String, DataTypeValue>) -> Result<String, warp::reject::Rejection> {
    // id
    let id = match req.remove("id") {
        Some(value) => match value {
            DataTypeValue::Unsigned32(i) => i,
            _ => Err(AppError {
                err_type: ErrorType::Internal,
                message: format!("err: wrong type; expected Unsigned 32-bit integer, found other; JSON: \"{}\"", value)
            })?
        },
        None => Err(AppError {
            err_type: ErrorType::BadRequest,
            message: format!("err: no id; the id field is required for PATCH requests")
        })?
    };

    // An SQL query can be made here that safely updates any verified data
    // You could also error if nothing but the id is included
    println!(
        "User #{}: <SQL-retrieved stuff>",
        id
    );

    // The id is passed on for the hello world filter to consume
    Ok(id.to_string())
}

/// Extracts the data from the request body and verifies it in the process.
/// 
/// This function has custom requirements, so it is best used for PATCH requests.
pub(crate) async fn get_extract(body: serde_json::Value) -> Result<HashMap<String, DataTypeValue>, warp::reject::Rejection> {
    // The map this function will extract from the JSON body
    let mut map: HashMap<String, DataTypeValue> = HashMap::new();

    // Checks to make sure the data exists/is structured properly
    let field = DB_DESIGN.tables.get("user").check()?.field("id").check()?;
    if let Some(data_map) = body.as_object() {
            if let Some(data) = data_map.get(&field.field_design_title) {
                match field.extract(data) {
                    Ok(data_value) => {
                        map.insert(
                            field.field_design_title.to_string(),
                            data_value
                        );
                    },
                    Err(error) => {
                        Err(AppError {
                            err_type: ErrorType::BadRequest,
                            message: format!("field {} is not formatted properly: {}", &field.field_design_title, error.to_string())
                        })?
                    }
                }
            } else {
                Err(AppError {
                    err_type: ErrorType::BadRequest,
                    message: format!("id field is required for GET, but was not included in the request body"),
                })?
            }
        }

    Ok(map)
}

/// Replies with a success code and Update-related message.
pub(crate) async fn get_success(user_id: String) -> Result<impl Reply, Rejection> {
    respond(
        Ok(format!(
            "Welcome, User #{}! If this was hooked up to a database, your information would be retrieved.",
            user_id)
        ),
        warp::http::StatusCode::ACCEPTED
    )
}
