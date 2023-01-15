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

// DELETE <domain>/api/test/unsubscribe
/// A function that returns a warp route for deleting a user.
pub(crate) fn delete_user() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("login")
        .and(warp::delete())
        .and(with_json_body())
        .and_then(delete_extract)
        .and_then(delete_retrieve)
        .and_then(delete_success)
}

/// Uses the fields to create a DELETE query.
/// 
/// The `req` variable now has some of the data specified by the `FieldDesign`.
/// Since not all of it is included, error handling is more important here.
/// 
/// You could also use the global's field info to avoid possible developer error!
async fn delete_retrieve(mut req: HashMap<String, DataTypeValue>) -> Result<String, warp::reject::Rejection> {
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
            message: format!("err: no id; the id field is required for DELETE requests")
        })?
    };

    // An SQL query can be made here that safely deletes user data
    println!(
        "User #{} deleted",
        id
    );

    // The id is passed on for the hello world filter to consume
    Ok(id.to_string())
}

/// Extracts the data from the request body and verifies it in the process.
async fn delete_extract(body: serde_json::Value) -> Result<HashMap<String, DataTypeValue>, warp::reject::Rejection> {
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
                    message: format!("id field is required for DELETE, but was not included in the request body"),
                })?
            }
        }

    Ok(map)
}

/// Replies with a success code and DELETE-related message.
async fn delete_success(user_id: String) -> Result<impl Reply, Rejection> {
    respond(
        Ok(format!(
            "Goodbye, User #{}. If this was hooked up to a database, your information would be deleted.",
            user_id)
        ),
        warp::http::StatusCode::ACCEPTED
    )
}
