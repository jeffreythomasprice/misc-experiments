mod auth;
mod db;
mod errors;
mod user;

use std::{error::Error, net::IpAddr, str::FromStr, sync::Arc};

use db::create_db;
use errors::catchers;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{Header, Status},
    Request, Response,
};

#[macro_use]
extern crate rocket;

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

    let jwt_key = auth::jwt::Key::new()?;

    _ = rocket::custom(rocket::Config {
        port: 8001,
        address: IpAddr::from_str("127.0.0.1").unwrap(),
        ..rocket::Config::debug_default()
    })
    .manage(Arc::new(user::Service::new(db.clone())))
    .manage(jwt_key)
    .register("/", catchers())
    .mount("/api/login", auth::routes())
    .mount("/api/users", user::routes())
    .attach(Cors)
    .mount("/", routes![all_options])
    .launch()
    .await?;

    Ok(())
}

// https://stackoverflow.com/a/64904947/9290998

// TODO move me
struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new(
            "Access-Control-Allow-Origin",
            "http://localhost:8000",
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

// TODO move me
#[options("/<_..>")]
fn all_options() -> Status {
    Status::Ok
}
