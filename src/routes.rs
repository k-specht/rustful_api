use std::collections::HashMap;
use warp::Rejection;
use warp::Reply;
use warp::Filter;
use rustract::types::DataTypeValue;

use crate::ErrorType;
use crate::DB_DESIGN;
use crate::AppError;
use crate::Check;
use crate::post::post_user;

/// Returns the route tree to be served.
pub fn gen_routes() -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone  {
    // <domain>/api/
    warp::path!("api" / ..)
        .and(post_user()) // Create
}

/// Extracts the data from the request body and verifies it in the process.
pub(crate) async fn extract(body: serde_json::Value) -> Result<HashMap<String, DataTypeValue>, warp::reject::Rejection> {
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

/// Uses warp to respond to the client.
/// 
/// Status is the status code on success.
pub(crate) fn respond<T: serde::Serialize>(result: Result<T, AppError>, status: warp::http::StatusCode) -> Result<impl Reply, Rejection> {
    match result {
        Ok(response) => 
            Ok(warp::reply::with_status(warp::reply::json(&response), status)),
        Err(err) => 
            Err(warp::reject::custom(err))
    }
}

/// Ensures that the request contains JSON within the size limit.
pub(crate) fn with_json_body() -> impl Filter<Extract = (serde_json::Value,), Error = Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
