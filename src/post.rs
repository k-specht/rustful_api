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

// POST <domain>/api
/// A function that returns a warp route for adding a new user.
pub(crate) fn post_user() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("")
        .and(warp::post())
        .and(with_json_body())
        .and_then(post_extract)
        .and_then(post_insert)
        .and_then(post_success)
}

/// Uses the fields to create a POST query.
/// 
/// The `req` variable now has all the data specified by the `FieldDesign`.
/// Nothing in this section will error unless the developer uses the wrong type here,
/// but error handling is included for that instance.
/// 
/// Alternatively, you could use the global's field info to avoid possible developer error!
async fn post_insert(mut req: HashMap<String, DataTypeValue>) -> Result<String, warp::reject::Rejection> {
    // name
    let value = req.remove("name").check()?;
    let name = match value {
        DataTypeValue::String(data) => data,
        _ => Err(AppError {
            err_type: ErrorType::Internal,
            message: format!("err: wrong type; expected String, found other; JSON: \"{}\"", value),
        })?
    };

    // email
    let value = req.remove("email").check()?;
    let email = match value {
        DataTypeValue::String(data) => data,
        _ => Err(AppError {
            err_type: ErrorType::Internal,
            message: format!("err: wrong type; expected String, found other; JSON: \"{}\"", value),
        })?
    };

    // registered
    let value = req.remove("registered");
    let registered: Option<String> = if let Some(value) = value {
            match value {
                DataTypeValue::String(data) => Some(data),
                _ => Err(AppError {
                        err_type: ErrorType::Internal,
                        message: format!("err: wrong type; expected String, found other; JSON: \"{}\"", value),
                    })?
            }
    } else { None };

    // type
    let value = req.remove("type").check()?;
    let type_field = match value {
        DataTypeValue::Enum(data) => data,
        _ => return Err(AppError {
            err_type: ErrorType::Internal,
            message: format!("err: wrong type; expected Enum, found other; JSON: \"{}\"", value)
        })?
    };

    // An SQL query can be made here that safely inserts the verified data
    let mut output = format!("Found User: {{ name: {}, email: {}, ", name, email);
    if let Some(v) = registered { output += format!("registered: {}, ", v).as_str() };
    output += format!("type: {} }}", type_field).as_str();
    println!(
        "{}",
        output
    );

    // The name is passed on for the hello world filter to consume
    Ok(name.to_string())
}

/// Extracts the data from the request body and verifies it in the process.
/// 
/// This function will require all required fields, so it is best used for POST requests.
async fn post_extract(body: serde_json::Value) -> Result<HashMap<String, DataTypeValue>, warp::reject::Rejection> {
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
            } else if field.required && !field.generated {
                Err(AppError {
                    err_type: ErrorType::BadRequest,
                    message: format!("field {} is listed as required, but was not included in the request body", &field.field_design_title),
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

/// Replies with a success code and POST-related message.
async fn post_success(user_name: String) -> Result<impl Reply, Rejection> {
    respond(
        Ok(format!(
            "Welcome, {}! If this was hooked up to a database, you would be added.",
            user_name)
        ),
        warp::http::StatusCode::ACCEPTED
    )
}
