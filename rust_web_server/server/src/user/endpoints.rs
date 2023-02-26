use std::sync::Arc;

use rocket::{serde::json::Json, Route, State};

use shared::user::{CreateUserRequest, UpdateUserRequest, UserResponse};

use crate::{
    auth::{Authenticated, IsAdmin},
    errors::Error,
    user::service::Service,
};

pub fn routes() -> Vec<Route> {
    routes![
        list,
        get_by_name_as_admin,
        get_by_name_as_user,
        create,
        update,
        delete_by_name
    ]
}

#[get("/")]
async fn list(
    service: &State<Arc<Service>>,
    _auth: IsAdmin,
) -> Result<Json<Vec<UserResponse>>, Error> {
    Ok(Json(
        service
            .list()
            .await?
            .into_iter()
            .map(|user| user.into())
            .collect::<Vec<UserResponse>>(),
    ))
}

#[get("/<name>")]
async fn get_by_name_as_admin(
    service: &State<Arc<Service>>,
    name: &str,
    _auth: IsAdmin,
) -> Result<Json<UserResponse>, Error> {
    Ok(Json(service.get_by_name(name).await?.into()))
}

#[get("/<name>", rank = 2)]
async fn get_by_name_as_user(
    service: &State<Arc<Service>>,
    name: &str,
    auth: &Authenticated,
) -> Result<Json<UserResponse>, Error> {
    if auth.0.name == name {
        Ok(Json(service.get_by_name(name).await?.into()))
    } else {
        Err(Error::Forbidden)
    }
}

// TODO add admin vs. non-admin below here

#[post("/", data = "<request>")]
async fn create(
    service: &State<Arc<Service>>,
    request: Json<CreateUserRequest>,
    _auth: &Authenticated,
) -> Result<Json<UserResponse>, Error> {
    service.create(&request).await?;
    Ok(Json(service.get_by_name(&request.name).await?.into()))
}

#[put("/<name>", data = "<request>")]
async fn update(
    service: &State<Arc<Service>>,
    name: &str,
    request: Json<UpdateUserRequest>,
    _auth: &Authenticated,
) -> Result<Json<UserResponse>, Error> {
    service.update(name, &request).await?;
    Ok(Json(service.get_by_name(name).await?.into()))
}

#[delete("/<name>")]
async fn delete_by_name(
    service: &State<Arc<Service>>,
    name: &str,
    _auth: &Authenticated,
) -> Result<(), Error> {
    service.delete_by_name(name).await?;
    Ok(())
}
