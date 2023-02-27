use rocket::{serde::json::Json, Route};
use shared::user::UserResponse;

use crate::errors::Error;

use super::guards::Authenticated;

pub fn routes() -> Vec<Route> {
    routes![login]
}

#[post("/")]
fn login(auth: &Authenticated) -> Result<Json<UserResponse>, Error> {
    let user = &auth.0;
    todo!("TODO JEFF generate new jwt for {:?}", user);
}
