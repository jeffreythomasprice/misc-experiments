use std::sync::Arc;

use base64::Engine;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    Catcher, State,
};

use crate::{
    errors::Error,
    user::{models::User, Service},
};

#[derive(Debug, Clone)]
pub struct Authenticated(User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r Authenticated {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let result = request
            .local_cache_async(async { authenticate(request).await })
            .await;
        match result {
            Ok(result) => Outcome::Success(result),
            Err(e) => Outcome::Failure((Status::Unauthorized, e.clone())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IsAdmin(User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IsAdmin {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        request.guard::<&Authenticated>().await.and_then(|auth| {
            if auth.0.is_admin {
                Outcome::Success(IsAdmin(auth.0.clone()))
            } else {
                Outcome::Failure((Status::Forbidden, Error::Forbidden))
            }
        })
    }
}

pub fn catchers() -> Vec<Catcher> {
    catchers![unauthorized]
}

#[catch(401)]
fn unauthorized() -> Error {
    Error::Unauthorized
}

const BASIC_AUTH_PREFIX: &str = "Basic ";

async fn authenticate(request: &Request<'_>) -> Result<Authenticated, Error> {
    let service = request
        .guard::<&State<Arc<Service>>>()
        .await
        .success_or_else(|| {
            error!("failed to get user service");
            Error::Unauthorized
        })?;

    let header = &request.headers().get("Authorization").collect::<Vec<_>>();
    if header.len() != 1 {
        debug!("expected one auth header, got {}", header.len());
        return Err(Error::Unauthorized);
    }
    let header = header[0];

    if let Ok(result) = jwt_auth(service, header).await {
        return Ok(result);
    }

    if let Ok(result) = basic_auth(service, header).await {
        return Ok(result);
    }

    debug!("no authenticate method succeeded");
    Err(Error::Unauthorized)
}

async fn basic_auth(service: &Service, header: &str) -> Result<Authenticated, Error> {
    trace!("trying basic auth");

    // strip off prefix
    if !header.starts_with(BASIC_AUTH_PREFIX) {
        debug!("auth header doesn't look like basic auth");
        return Err(Error::Unauthorized);
    }
    let header = &header[BASIC_AUTH_PREFIX.len()..];

    // get the username and password components
    let header = base64::engine::general_purpose::URL_SAFE
        .decode(header)
        .or_else(|e| {
            debug!("auth header isn't base64-encoded: {e}");
            Err(Error::Unauthorized)
        })?;
    let header = std::str::from_utf8(header.as_slice()).or_else(|e| {
        debug!("auth header is base64 encoded, but encoded value isn't a utf8-string: {e}");
        Err(Error::Unauthorized)
    })?;
    let (name, password) = header.split_once(":").ok_or_else(|| {
        debug!("auth header doesn't have a ':' delimeter");
        Error::Unauthorized
    })?;

    // get user
    let user = service.get_by_name(name).await.or_else(|e| {
        debug!("error finding user: {e:?}");
        Err(e)
    })?;

    // check that password matches
    if user.password == password {
        debug!("authenticated {name}");
        Ok(Authenticated(user))
    } else {
        debug!("password mismatch for {name}");
        Err(Error::Unauthorized)
    }
}

async fn jwt_auth(service: &Service, header: &str) -> Result<Authenticated, Error> {
    trace!("trying jwt auth");

    // TODO look for 'Bearer: <jwt>'
    Err(Error::Unauthorized)
}
