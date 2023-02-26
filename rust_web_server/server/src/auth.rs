use std::sync::Arc;

use base64::Engine;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    Catcher, State,
};

use crate::{errors::Error, user::Service};

#[derive(Debug)]
pub struct Authenticated;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authenticated {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // TODO check header for Authorization
        match &request.headers().get("Authorization").collect::<Vec<_>>()[..] {
            &[header] => {
                async fn is_valid<'r>(
                    header: &str,
                    request: &'r Request<'_>,
                ) -> Result<Authenticated, Error> {
                    // strip off prefix
                    const BASIC_AUTH_PREFIX: &str = "Basic ";
                    if !header.starts_with(BASIC_AUTH_PREFIX) {
                        Err(Error::Unauthorized)?;
                    }
                    let header = &header[BASIC_AUTH_PREFIX.len()..];

                    // get the username and password components
                    let header = base64::engine::general_purpose::URL_SAFE
                        .decode(header)
                        .or(Err(Error::Unauthorized))?;
                    let header =
                        std::str::from_utf8(header.as_slice()).or(Err(Error::Unauthorized))?;
                    let (name, password) = header.split_once(":").ok_or(Error::Unauthorized)?;

                    // get user
                    let service = request
                        .guard::<&State<Arc<Service>>>()
                        .await
                        .success_or(Error::Unauthorized)?;
                    // TODO JEFF failure to find user 404, should 401
                    let user = service.get_by_name(name).await?;

                    // check that password matches
                    if user.password == password {
                        Ok(Authenticated)
                    } else {
                        Err(Error::Unauthorized)
                    }
                }

                match is_valid(header, request).await {
                    Ok(result) => Outcome::Success(result),
                    Err(e) => {
                        let (status, _) = e.to_response();
                        Outcome::Failure((status, e))
                    }
                }
            }
            _ => Outcome::Failure((Status::Unauthorized, Error::Unauthorized)),
        }
    }
}

pub fn catchers() -> Vec<Catcher> {
    catchers![unauthorized]
}

#[catch(401)]
fn unauthorized() -> Error {
    Error::Unauthorized
}
