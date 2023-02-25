use std::sync::Arc;

use rocket::{http::Status, response::status, serde::json::Json, Route, State};

use shared::user::{CreateUserRequest, UserResponse};

use crate::{
    responses::{get_json_result, JsonResult},
    user::{models::User, service::Service},
};

pub fn routes() -> Vec<Route> {
    routes![list, get_by_name, create]
}

#[get("/")]
async fn list(service: &State<Arc<Service>>) -> JsonResult<Vec<UserResponse>> {
    get_json_result(service.list().await.and_then(|results| {
        Ok(results
            .iter()
            .map(|user| user.into())
            .collect::<Vec<UserResponse>>())
    }))
}

#[get("/<name>")]
async fn get_by_name(service: &State<Arc<Service>>, name: &str) -> JsonResult<UserResponse> {
    get_json_result(service.get_by_name(name).await)
}

#[post("/", data = "<request>")]
async fn create(
    service: &State<Arc<Service>>,
    request: Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, status::Custom<String>> {
    match service
        .create(&User {
            name: request.name.clone(),
            password: request.password.clone(),
            is_admin: request.is_admin,
        })
        .await
    {
        Ok(_) => match service.get_by_name(&request.name).await {
            Ok(result) => Ok(Json(UserResponse {
                name: result.name,
                is_admin: result.is_admin,
            })),
            Err(e) => Err(status::Custom(
                Status::InternalServerError,
                format!("{e:?}"),
            )),
        },
        Err(e) => Err(status::Custom(
            Status::InternalServerError,
            format!("{e:?}"),
        )),
    }
}

// TODO update

// TODO delete
