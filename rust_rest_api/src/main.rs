#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
use rocket::{get, post, routes, Rocket, Request, Data, Response};
use rocket::http::Status;
use rocket::http::ContentType;
use rocket::response::Responder;

#[get("/hello")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();

}
