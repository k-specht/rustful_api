use warp::Rejection;
use warp::Reply;
use warp::Filter;

use crate::AppError;
use crate::post::post_user;
use crate::get::get_user;
use crate::patch::patch_user;
use crate::delete::delete_user;

/// Returns the route tree to be served.
/// 
/// Currently, CRUD is implemented for the user table.
pub fn gen_routes() -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone  {
    post_user() // Create
        .or(get_user()) // Read
        .or(patch_user()) // Update
        .or(delete_user()) // Delete
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
