use std::sync::Arc;

use diesel::*;
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json},
};

use crate::{db, AppState};
use tracing::*;

#[handler]
pub fn list_users(
    Data(state): Data<&Arc<AppState>>,
) -> Result<Json<Vec<shared::UserResponse>>, StatusCode> {
    use self::db::schema::users::dsl::*;

    let db = &mut state.db.get().unwrap();
    let results = users
        .select(db::models::User::as_select())
        .load(db)
        .map_err(|e| {
            error!("error selecting users: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_iter()
        .map(|x| x.into())
        .collect();
    Ok(Json(results))
}

#[handler]
pub fn create_user(
    Data(state): Data<&Arc<AppState>>,
    Json(request): Json<shared::CreateUserRequest>,
) -> Result<Json<shared::UserResponse>, StatusCode> {
    use crate::db::schema::users;

    let request: db::models::UserWithJustUsernameAndPassword = request.into();
    let db = &mut state.db.get().unwrap();
    let result: shared::UserResponse = diesel::insert_into(users::table)
        .values(&request)
        .returning(db::models::User::as_returning())
        .get_result(db)
        .map_err(|e| {
            error!("error inserting new user: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into();
    Ok(Json(result))
}

#[handler]
pub fn log_in(
    Data(state): Data<&Arc<AppState>>,
    Json(request): Json<shared::LogInRequest>,
) -> Result<Json<shared::UserResponse>, StatusCode> {
    use self::db::schema::users::dsl::*;

    let db = &mut state.db.get().unwrap();
    let results: Vec<db::models::User> = users
        .filter(
            username
                .eq(request.username)
                .and(password.eq(request.password)),
        )
        .limit(1)
        .select(db::models::User::as_select())
        .load(db)
        .map_err(|e| {
            error!("error checking user credentials: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    match &results[..] {
        [user] => {
            debug!("found user with correct credentials");
            Ok(Json((*user).clone().into()))
        }
        _ => {
            debug!("incorrect credentials");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
