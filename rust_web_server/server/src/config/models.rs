use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Config {
    pub key: String,
    pub value: String,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}
