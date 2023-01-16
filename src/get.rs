use warp::Rejection;
use warp::Reply;
use warp::Filter;

use crate::routes::respond;

// GET <domain>/api/#
/// A function that returns a warp route for getting user info.
pub(crate) fn get_user() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("" / u32)
        .and(warp::get())
        .and_then(get_retrieve)
        .and_then(get_success)
}

/// Uses the id to create a GET query.
async fn get_retrieve(id: u32) -> Result<String, warp::reject::Rejection> {
    // An SQL query can be made here that safely updates any verified data
    // You could also error if nothing but the id is included
    println!(
        "User #{}: <SQL-retrieved stuff>",
        id
    );

    // The id is passed on for the hello world filter to consume
    Ok(id.to_string())
}

/// Replies with a success code and GET-related message.
async fn get_success(user_id: String) -> Result<impl Reply, Rejection> {
    respond(
        Ok(format!(
            "Welcome, User #{}! If this was hooked up to a database, your information would be retrieved.",
            user_id)
        ),
        warp::http::StatusCode::ACCEPTED
    )
}
