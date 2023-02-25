use sqlx::{FromRow, Pool, Sqlite};

#[derive(FromRow)]
pub struct User {
    pub name: String,
    pub password: String,
    pub is_admin: bool,
}

pub struct UserService {
    db: Pool<Sqlite>,
}

impl UserService {
    pub fn new(db: Pool<Sqlite>) -> UserService {
        UserService { db }
    }

    pub async fn list(&self) -> Result<Vec<User>, sqlx::Error> {
        Ok(
            sqlx::query_as::<_, User>("SELECT name, password, is_admin FROM users")
                .fetch_all(&self.db)
                .await?,
        )
    }

    pub async fn get_by_name(&self, name: &str) -> Result<User, sqlx::Error> {
        Ok(
            sqlx::query_as::<_, User>("SELECT name, password, is_admin FROM users WHERE name = ?")
                .bind(name)
                .fetch_one(&self.db)
                .await?,
        )
    }

    pub async fn create(
        &self,
        user: &User,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        Ok(
            sqlx::query("INSERT INTO users (name, password, is_admin) VALUES (?, ?, ?)")
                .bind(user.name.as_str())
                .bind(user.password.as_str())
                .bind(user.is_admin)
                .execute(&self.db)
                .await?,
        )
    }
}

pub mod endpoints {
    use std::sync::Arc;

    use rocket::{http::Status, response::status, serde::json::Json, Route, State};

    use shared::user::{CreateUserRequest, UserResponse};

    use super::*;

    pub fn routes() -> Vec<Route> {
        routes![list, create]
    }

    #[get("/")]
    async fn list(
        service: &State<Arc<UserService>>,
    ) -> Result<Json<Vec<UserResponse>>, status::Custom<String>> {
        match service.list().await {
            Ok(results) => Ok(Json(
                results
                    .iter()
                    .map(|user| UserResponse {
                        name: user.name.clone(),
                        is_admin: user.is_admin,
                    })
                    .collect(),
            )),
            Err(e) => Err(status::Custom(
                Status::InternalServerError,
                format!("{e:?}"),
            )),
        }
    }

    // TODO get

    #[post("/", data = "<request>")]
    async fn create(
        service: &State<Arc<UserService>>,
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
}
