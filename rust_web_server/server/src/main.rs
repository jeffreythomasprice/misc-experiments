mod user;

use std::{error::Error, net::IpAddr, str::FromStr, sync::Arc};

use rocket::Config;
use sqlx::sqlite::SqlitePoolOptions;
use user::UserService;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("db?mode=rwc")
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    _ = rocket::custom(Config {
        port: 8000,
        address: IpAddr::from_str("127.0.0.1").unwrap(),
        ..Config::debug_default()
    })
    .manage(Arc::new(UserService::new(db.clone())))
    .mount("/", routes![index])
    .mount("/users", user::endpoints::routes())
    .launch()
    .await?;

    Ok(())
}
