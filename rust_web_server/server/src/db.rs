use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

pub async fn create_db() -> Result<Pool<Sqlite>, sqlx::Error> {
    let result = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("db?mode=rwc")
        .await?;
    common_db_init(&result).await?;
    Ok(result)
}

#[cfg(test)]
pub async fn create_db_for_test() -> Result<Pool<Sqlite>, sqlx::Error> {
    let result = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await?;
    common_db_init(&result).await?;
    Ok(result)
}

async fn common_db_init(db: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(db).await?;
    Ok(())
}
