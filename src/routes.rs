use warp::Rejection;
use warp::Reply;
use warp::Filter;

use crate::AppError;
use crate::patch::patch_user;
use crate::post::post_user;

/// Returns the route tree to be served.
pub fn gen_routes() -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone  {
    // <domain>/api/
    warp::path!("api" / ..)
        .and(post_user()) // Create
        .or(patch_user()) // Update
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
