use std::collections::HashMap;
use warp::Rejection;
use warp::Reply;
use warp::Filter;
use rustract::types::DataTypeValue;

use crate::ErrorType;
use crate::AppError;
use crate::Check;
use crate::routes::extract;
use crate::routes::respond;
use crate::routes::with_json_body;

// POST <domain>/api/test/hello
/// A function that returns a warp route for adding a new user.
pub(crate) fn post_user() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("register")
        .and(warp::post())
        .and(with_json_body())
        .and_then(extract)
        .and_then(insert)
        .and_then(say_hello)
}

/// Uses the fields to create some query or handle some type of custom logic.
/// 
/// The `req` variable now has all the data specified by the `FieldDesign`.
/// Nothing here can error due to previous checks during extraction, but error handling is included anyway.
/// For cleaner code, error handling can be left out of this section.
pub(crate) async fn insert(req: HashMap<String, DataTypeValue>) -> Result<String, warp::reject::Rejection> {
    // name
    let value = req.get("name").check()?;
    let name = match value {
        DataTypeValue::String(data) => data,
        _ => Err(AppError {
            err_type: ErrorType::Internal,
            message: format!("err: wrong type; expected String, found other; JSON: \"{}\"", value),
        })?
    };

    // email
    let value = req.get("email").check()?;
    let email = match value {
        DataTypeValue::String(data) => data,
        _ => Err(AppError {
            err_type: ErrorType::Internal,
            message: format!("err: wrong type; expected String, found other; JSON: \"{}\"", value),
        })?
    };

    // registered
    let value = req.get("registered").check()?;
    let registered = match value {
        DataTypeValue::String(data) => data,
        _ => Err(AppError {
                err_type: ErrorType::Internal,
                message: format!("err: wrong type; expected String, found other; JSON: \"{}\"", value),
            })?
    };

    // type
    let value = req.get("type").check()?;
    let type_field = match value {
        DataTypeValue::Enum(data) => data,
        _ => return Err(AppError {
            err_type: ErrorType::Internal,
            message: format!("err: wrong type; expected Enum, found other; JSON: \"{}\"", value)
        })?
    };

    // twofa
    let value = req.get("twofa").check()?;
    let twofa= match value {
        DataTypeValue::Byte(data) => data,
        _ => return Err(AppError {
            err_type: ErrorType::Internal,
            message: format!("err: wrong type; expected Byte, found other; JSON: \"{}\"", value)
        })?
    };

    // friends
    let value = req.get("friends").check()?;
    let friends = match value {
        DataTypeValue::Byte(data) => data,
        _ => return Err(AppError {
            err_type: ErrorType::Internal,
            message: format!("err: wrong type; expected Byte, found other; JSON: \"{}\"", value)
        })?
    };

    // An SQL query can be made here that safely inserts the verified data
    print!(
        "Found User: {{ name: {}, email: {}, registered: {}, type: {}, twofa: {}, friends: {} }}",
        name, email, registered, type_field, twofa, friends
    );

    // The name is passed on for the hello world filter to consume
    Ok(name.to_string())
}

/// Creates a hello response for warp to reply with.
pub(crate) async fn say_hello(user_name: String) -> Result<impl Reply, Rejection> {
    respond(
        Ok(format!(
            "Welcome, {}! If this was hooked up to a database, you would be added.",
            user_name)
        ),
        warp::http::StatusCode::ACCEPTED
    )
}
