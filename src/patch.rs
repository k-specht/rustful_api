use std::collections::HashMap;
use warp::Rejection;
use warp::Reply;
use warp::Filter;
use rustract::types::DataTypeValue;

use crate::DB_DESIGN;
use crate::ErrorType;
use crate::AppError;
use crate::Check;
use crate::routes::respond;
use crate::routes::with_json_body;

// PATCH <domain>/api/test/update
/// A function that returns a warp route for adding a new user.
pub(crate) fn patch_user() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("update")
        .and(warp::patch())
        .and(with_json_body())
        .and_then(patch_extract)
        .and_then(patch_insert)
        .and_then(patch_success)
}

/// Uses the fields to create a PATCH query.
/// 
/// The `req` variable now has some of the data specified by the `FieldDesign`.
/// Since not all of it is included, error handling is more important here.
/// 
/// You could also use the global's field info to avoid possible developer error!
async fn patch_insert(mut req: HashMap<String, DataTypeValue>) -> Result<String, warp::reject::Rejection> {
    // These are all the possible parameters that can be updated
    let mut name: Option<String> = None;
    let mut email: Option<String> = None;
    let mut registered: Option<String> = None;
    let mut type_field: Option<u32> = None;

    // id
    // This is the only field that definitely needs to be here!
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

    // name
    if let Some(value) = req.remove("name") {
        name = Some(match value {
            DataTypeValue::String(data) => data,
            _ => Err(AppError {
                err_type: ErrorType::Internal,
                message: format!("err: wrong type; expected String, found other; JSON: \"{}\"", value),
            })?
        });
    }

    // email
    if let Some(value) = req.remove("email") {
        email = Some(match value {
            DataTypeValue::String(data) => data,
            _ => Err(AppError {
                err_type: ErrorType::Internal,
                message: format!("err: wrong type; expected String, found other; JSON: \"{}\"", value),
            })?
        });
    }

    // registered
    if let Some(value) = req.remove("registered") {
        registered = Some(match value {
            DataTypeValue::String(data) => data,
            _ => Err(AppError {
                    err_type: ErrorType::Internal,
                    message: format!("err: wrong type; expected String, found other; JSON: \"{}\"", value),
                })?
        });
    }

    // type
    if let Some(value) = req.remove("type") {
        type_field = Some(match value {
            DataTypeValue::Enum(data) => data,
            _ => return Err(AppError {
                err_type: ErrorType::Internal,
                message: format!("err: wrong type; expected Enum, found other; JSON: \"{}\"", value)
            })?
        });
    }

    // An SQL query can be made here that safely updates any verified data
    // You could also error if nothing but the id is included
    let mut query_string = "Updated User: {{ ".to_string();
    if let Some(value) = name {
        query_string += "name: ";
        query_string += value.as_str();
        query_string += ",";
    }
    if let Some(value) = email {
        query_string += "email: ";
        query_string += value.as_str();
        query_string += ",";
    }
    if let Some(value) = registered {
        query_string += "registered: ";
        query_string += value.as_str();
        query_string += ",";
    }
    if let Some(value) = type_field {
        query_string += "type: ";
        query_string += value.to_string().as_str();
    }
    print!(
        "{}",
        query_string
    );

    // The id is passed on for the hello world filter to consume
    Ok(id.to_string())
}

/// Extracts the data from the request body and verifies it in the process.
/// 
/// This function has custom requirements, so it is best used for PATCH requests.
async fn patch_extract(body: serde_json::Value) -> Result<HashMap<String, DataTypeValue>, warp::reject::Rejection> {
    // The map this function will extract from the JSON body
    let mut map: HashMap<String, DataTypeValue> = HashMap::new();

    // Checks to make sure the data exists/is structured properly
    if let Some(data_map) = body.as_object() {
        for key in DB_DESIGN.table("user").check()?.fields.keys() {
            let field = DB_DESIGN
                .table("user").check()?
                .field(key).check()?;

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
            } else if field.field_design_title.as_str() == "id" {
                Err(AppError {
                    err_type: ErrorType::BadRequest,
                    message: format!("id field is required for PATCH, but was not included in the request body"),
                })?
            }
        }

        Ok(map)
    } else {
        Err(AppError {
            err_type: ErrorType::BadRequest,
            message: format!("failed to parse JSON as object, JSON: \"{}\" (err: body should be a map)", body.to_string()),
        })?
    }
}

/// Replies with a success code and PATCH-related message.
async fn patch_success(user_name: String) -> Result<impl Reply, Rejection> {
    respond(
        Ok(format!(
            "Welcome, User #{}! If this was hooked up to a database, your information would be changed.",
            user_name)
        ),
        warp::http::StatusCode::ACCEPTED
    )
}
