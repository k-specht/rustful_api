use warp::Rejection;
use warp::Reply;
use warp::Filter;

use crate::routes::respond;

// DELETE <domain>/user/#
/// A function that returns a warp route for deleting a user.
pub(crate) fn delete_user() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("user" / u32)
        .and(warp::delete())
        .and_then(delete_retrieve)
        .and_then(delete_success)
}

/// Uses the id to make an SQL DELETE query.
async fn delete_retrieve(id: u32) -> Result<String, warp::reject::Rejection> {
    // An SQL query can be made here that safely deletes user data
    println!(
        "User #{} deleted",
        id
    );

    // The id is passed on for the hello world filter to consume
    Ok(id.to_string())
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
