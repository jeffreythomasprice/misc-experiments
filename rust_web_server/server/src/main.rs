mod auth;
mod db;
mod errors;
mod user;

use std::{error::Error, net::IpAddr, str::FromStr, sync::Arc};

use db::create_db;
use errors::catchers;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[main]
async fn main() -> Result<(), Box<dyn Error>> {
    let colors = fern::colors::ColoredLevelConfig::default();
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{: <5}][{}] {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%z"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .level_for("async_std", log::LevelFilter::Error)
        .level_for("async_io", log::LevelFilter::Error)
        .level_for("polling", log::LevelFilter::Error)
        .level_for("mio", log::LevelFilter::Error)
        .chain(std::io::stdout())
        .apply()?;

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
