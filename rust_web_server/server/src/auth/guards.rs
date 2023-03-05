use std::sync::Arc;

use base64::Engine;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    State,
};

use crate::{
    auth::jwt::Claims,
    errors::Error,
    user::{models::User, Service as UserService},
};

use super::jwt::Service as JwtService;

#[derive(Debug, Clone)]
pub struct Authenticated(pub User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r Authenticated {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt_service = match request.guard::<&State<JwtService>>().await {
            Outcome::Success(service) => service,
            _ => {
                return Outcome::Failure((
                    Status::InternalServerError,
                    Error::InternalServerError("".to_string()),
                ));
            }
        };
        let result = request
            .local_cache_async(async { authenticate(request, &jwt_service).await })
            .await;
        match result {
            Ok(result) => Outcome::Success(result),
            Err(e) => {
                e.cache_on_request(request);
                Outcome::Failure((Status::Unauthorized, e.clone()))
            }
        }
        .forward_then(|_| Outcome::Forward(()))
    }
}

#[derive(Debug, Clone)]
pub struct IsAdmin(pub User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IsAdmin {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        request.guard::<&Authenticated>().await.and_then(|auth| {
            if auth.0.is_admin {
                Outcome::Success(IsAdmin(auth.0.clone()))
            } else {
                Outcome::Forward(())
            }
        })
    }
}

const BASIC_AUTH_PREFIX: &str = "Basic ";
const JWT_AUTH_PREFIX: &str = "Bearer ";

async fn authenticate(
    request: &Request<'_>,
    jwt_service: &JwtService,
) -> Result<Authenticated, Error> {
    let user_service = request
        .guard::<&State<Arc<UserService>>>()
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

    if let Ok(result) = jwt_auth(user_service, jwt_service, header).await {
        return Ok(result);
    }

    if let Ok(result) = basic_auth(user_service, header).await {
        return Ok(result);
    }

    debug!("no authenticate method succeeded");
    Err(Error::Unauthorized)
}

async fn basic_auth(user_service: &UserService, header: &str) -> Result<Authenticated, Error> {
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
    let user = user_service.get_by_name(name).await.or_else(|e| {
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

async fn jwt_auth(
    user_service: &UserService,
    jwt_service: &JwtService,
    header: &str,
) -> Result<Authenticated, Error> {
    trace!("trying jwt auth");

    // strip off prefix
    if !header.starts_with(JWT_AUTH_PREFIX) {
        debug!("auth header doesn't look like jwt auth");
        return Err(Error::Unauthorized);
    }
    let header = &header[JWT_AUTH_PREFIX.len()..];

    // get the jwt claims
    let claims = Claims::from_jwt_and_validate(jwt_service, header).or_else(|e| {
        debug!("jwt failed to parse or validate: {e:?}");
        Err(e)
    })?;
    trace!("claims {claims:?}");

    // get user
    let user = user_service
        .get_by_name(&claims.username)
        .await
        .or_else(|e| {
            debug!("error finding user: {e:?}");
            Err(e)
        })?;

    // success
    Ok(Authenticated(user))
}
