use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{Request, Response};
use shared::JsonResponse;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> status::Custom<String> {
    status::Custom(Status::ImATeapot, "Hello, World!".into())
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
    rocket::custom(rocket::Config {
        address: "127.0.0.1".parse().unwrap(),
        port: 8001,
        ..Default::default()
    })
    .attach(Cors)
    .mount("/", routes![index, json_example])
}
