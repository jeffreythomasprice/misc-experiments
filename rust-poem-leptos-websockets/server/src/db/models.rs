use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
}

impl From<User> for shared::UserResponse {
    fn from(val: User) -> Self {
        shared::UserResponse {
            id: val.id,
            username: val.username,
        }
    }
}

#[derive(Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserWithJustUsernameAndPassword {
    pub username: String,
    pub password: String,
}

impl From<shared::CreateUserRequest> for UserWithJustUsernameAndPassword {
    fn from(value: shared::CreateUserRequest) -> Self {
        Self {
            username: value.username,
            password: value.password,
        }
    }
}
