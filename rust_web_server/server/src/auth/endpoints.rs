use rocket::{serde::json::Json, Route, State};
use shared::auth::ResponseBody;

use crate::{auth::jwt::Claims, errors::Error};

use super::{guards::Authenticated, jwt::Service};

#[derive(Responder)]
struct Response {
    body: Json<ResponseBody>,
}

pub fn routes() -> Vec<Route> {
    routes![login]
}

#[post("/")]
fn login(auth: &Authenticated, service: &State<Service>) -> Result<Response, Error> {
    let user = auth.0.clone();

    let jwt = service.create_jwt(&Claims::new(&user.name))?;
    trace!("authenticated user {user:?} and produced new jwt {jwt}");

    Ok(Response {
        body: Json(ResponseBody { jwt }),
    })
}
