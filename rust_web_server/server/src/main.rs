mod auth;
mod db;
mod errors;
mod user;

use std::{error::Error, net::IpAddr, str::FromStr, sync::Arc};

use auth::catchers;
use db::create_db;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[main]
async fn main() -> Result<(), Box<dyn Error>> {
    simplelog::TermLogger::init(
        log::LevelFilter::Trace,
        simplelog::ConfigBuilder::new()
            .add_filter_ignore_str("async_std")
            .add_filter_ignore_str("async_io")
            .add_filter_ignore_str("polling")
            .add_filter_ignore_str("mio")
            .build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )?;

    let db = create_db().await?;

    _ = rocket::custom(rocket::Config {
        port: 8000,
        address: IpAddr::from_str("127.0.0.1").unwrap(),
        ..rocket::Config::debug_default()
    })
    .manage(Arc::new(user::Service::new(db.clone())))
    .register("/", catchers())
    .mount("/", routes![index])
    .mount("/users", user::routes())
    .launch()
    .await?;

    Ok(())
}
