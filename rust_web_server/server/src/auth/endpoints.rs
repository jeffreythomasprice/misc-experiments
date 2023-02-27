use rocket::{serde::json::Json, Route, State};
use shared::user::UserResponse;

use crate::{auth::jwt::Claims, errors::Error};

use super::{guards::Authenticated, jwt::Key};

pub fn routes() -> Vec<Route> {
    routes![login]
}

#[post("/")]
fn login(auth: &Authenticated, key: &State<Key>) -> Result<Json<UserResponse>, Error> {
    let user = &auth.0;

    let jwt = Claims {
        username: user.name.clone(),
    }
    .to_jwt(key)?;
    debug!("TODO JEFF jwt for user = {jwt}");

    todo!("TODO JEFF generate new jwt for {:?}", user);
}
