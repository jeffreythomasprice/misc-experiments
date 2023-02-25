use shared::user::UserResponse;
use sqlx::FromRow;

#[derive(Debug, FromRow, PartialEq)]
pub struct User {
    pub name: String,
    pub password: String,
    pub is_admin: bool,
}

impl Into<UserResponse> for &User {
    fn into(self) -> UserResponse {
        UserResponse {
            name: self.name.clone(),
            is_admin: self.is_admin,
        }
    }
}

// TODO why both?
impl Into<UserResponse> for User {
    fn into(self) -> UserResponse {
        UserResponse {
            name: self.name.clone(),
            is_admin: self.is_admin,
        }
    }
}
