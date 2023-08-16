use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::json::Json;
use rocket::{Request, Response};
use shared::JsonResponse;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> String {
    return "Hello, World!".into();
}

#[get("/json")]
fn json_example() -> Json<JsonResponse> {
    Json(JsonResponse::new("baz", 42))
}

struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
fn rocket() -> _ {
    let mut config = rocket::Config::default();
    config.address = "127.0.0.1".parse().unwrap();
    config.port = 8001;
    rocket::custom(config)
        .attach(Cors)
        .mount("/", routes![index, json_example])
}
