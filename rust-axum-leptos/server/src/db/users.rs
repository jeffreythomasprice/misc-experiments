use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};

#[derive(Debug)]
pub struct User {
    pub username: String,
    pub is_admin: bool,
}

pub fn check_password(conn: &Connection, username: &str, password: &str) -> Result<Option<User>> {
    Ok(conn
        .query_row(
            "SELECT username, is_admin FROM users WHERE username = ? AND password = ?",
            [username, password],
            |row| {
                Ok(User {
                    username: row.get(0)?,
                    is_admin: row.get(1)?,
                })
            },
        )
        .optional()?)
}
