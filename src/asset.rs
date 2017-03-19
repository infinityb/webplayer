// because having an intermediate function call tricks the linter
#![allow(unmounted_route)]


use rocket::Route;
use rocket::response::content::JavaScript;

pub fn statics() -> Vec<Route> {
    routes![
        jquery_js,
        jquery_js_map,
    ]
}

#[get("/js/jquery.min.map")]
fn jquery_js() -> JavaScript<&'static str> {
    JavaScript(include_str!("../assets/js/jquery-3.1.1.min.map"))
}

#[get("/js/jquery.js")]
fn jquery_js_map() -> JavaScript<&'static str> {
    JavaScript(include_str!("../assets/js/jquery-3.1.1.min.js"))
}
