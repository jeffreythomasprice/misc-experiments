use std::sync::Arc;

use rocket::{serde::json::Json, Route, State};

use shared::user::{CreateUserRequest, UpdateUserRequest, UserResponse};

use crate::{responses::Error, user::service::Service};

pub fn routes() -> Vec<Route> {
    routes![list, get_by_name, create, update, delete_by_name]
}

#[get("/")]
async fn list(service: &State<Arc<Service>>) -> Result<Json<Vec<UserResponse>>, Error> {
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
async fn get_by_name(
    service: &State<Arc<Service>>,
    name: &str,
) -> Result<Json<UserResponse>, Error> {
    Ok(Json(service.get_by_name(name).await?.into()))
}

#[post("/", data = "<request>")]
async fn create(
    service: &State<Arc<Service>>,
    request: Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, Error> {
    service.create(&request).await?;
    Ok(Json(service.get_by_name(&request.name).await?.into()))
}

#[put("/<name>", data = "<request>")]
async fn update(
    service: &State<Arc<Service>>,
    name: &str,
    request: Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, Error> {
    service.update(name, &request).await?;
    Ok(Json(service.get_by_name(name).await?.into()))
}

#[delete("/<name>")]
async fn delete_by_name(service: &State<Arc<Service>>, name: &str) -> Result<(), Error> {
    service.delete_by_name(name).await?;
    Ok(())
}
