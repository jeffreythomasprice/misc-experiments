use async_std::io::Cursor;
use rocket::{
    http::{Cookie, Header},
    serde::json::Json,
    Route, State,
};
use serde::Serialize;
use shared::user::UserResponse;

use crate::{auth::jwt::Claims, errors::Error};

use super::{guards::Authenticated, jwt::Key};

#[derive(Serialize)]
struct ResponseBody {
    jwt: String,
}

#[derive(Responder)]
struct Response {
    body: Json<ResponseBody>,
}

pub fn routes() -> Vec<Route> {
    routes![login]
}

#[post("/")]
fn login(auth: &Authenticated, key: &State<Key>) -> Result<Response, Error> {
    let user = auth.0.clone();

    let jwt = Claims {
        username: user.name.clone(),
    }
    .to_jwt(key)?;
    trace!("authenticated user {user:?} and produced new jwt {jwt}");

    Ok(Response {
        body: Json(ResponseBody { jwt }),
    })
}
