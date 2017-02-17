#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(conservative_impl_trait)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket_contrib;
extern crate uuid;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate postgres;
extern crate rocket;
extern crate hyper;
extern crate hyper_native_tls;
extern crate bincode;
extern crate crypto;
extern crate toml;

use std::io::{self, Read};
use std::fs::File;
use rocket::State;
use rocket_contrib::{JSON as Json};
use rocket::response::content::{
    JSON as JsonResp,
    HTML as HtmlResp,
};
use rocket::response::{Responder, Failure};
use rocket::http::Status;
use rocket::response::Stream;
use postgres::{Connection, TlsMode};

mod util;
mod blob;
mod asset;
mod database;
mod rpc;
mod foreign_auth;
mod auth;
mod model;

use self::blob::BlobId;
use self::auth::{
    AuthTokenBlob,
    AuthTokenInfo,
};
use self::foreign_auth::{
    ForeignAuthProvider,
    GoogleAuthProvider,
    GoogleAuthToken
};

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[get("/hi")]
fn hi_get() -> &'static str {
    "Hello, world!"
}

#[get("/blob/<id>")]
fn blob_obj_get(config: State<AppConfig>, auth: AuthTokenBlob, id: BlobId) -> impl Responder<'static> {
    // let user_id = try!(config.validate_auth(&auth));
    if !auth.is_valid(config.secret.as_bytes()) {
        return Err(Failure(Status::Forbidden));
    }

    let target: BlobId = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse().unwrap();
    if id != target {
        return Err(Failure(Status::NotFound));
    }

    let out: Vec<u8> = (0..64 * 1024).map(|by| {
        let by = by as i64;
        ((by * 524189) % 256) as u8
    }).collect();

    Ok(rocket::response::Stream::from(io::Cursor::new(out)))
}

#[post("/blob")]
fn blob_obj_post(config: State<AppConfig>, auth: AuthTokenBlob) -> impl Responder<'static> {
    // let user_id = try!(config.validate_auth(&auth));
    if !auth.is_valid(config.secret.as_bytes()) {
        return Err(Failure(Status::Forbidden));
    }

    let out = Vec::new();
    Ok(rocket::response::Stream::from(io::Cursor::new(out)))
}

#[derive(FromForm, Debug)]
struct Search {
   q: String,
}

#[get("/tracks/search?<search>")]
fn tracks_search_get(config: State<AppConfig>, auth: AuthTokenBlob, search: Search) -> impl Responder<'static> {
    // let user_id = try!(config.validate_auth(&auth));
    if !auth.is_valid(config.secret.as_bytes()) {
        return Err(Failure(Status::Forbidden));
    }

    Ok(format!("{:?}", search))
}

#[get("/songs")]
fn songs_get(config: State<AppConfig>, auth: AuthTokenBlob) -> impl Responder<'static> {
    // let user_id = try!(config.validate_auth(&auth));
    if !auth.is_valid(config.secret.as_bytes()) {
        // XXX: richer errors
        return Err(Failure(Status::Forbidden));
    }

    let conn = database::get_conn(config.database.read_url()).unwrap();
    let songs = database::get_songs(&conn)
        .map_err(|_| Failure(Status::InternalServerError))?
        .collect::<Vec<_>>();

    Ok(Json(songs))
}

#[get("/player")]
fn player_get() -> impl Responder<'static> {
    HtmlResp(include_str!("../template/player/index.html"))
}

#[get("/login")]
fn login_get(config: State<AppConfig>) -> impl Responder<'static> {
    const template: &'static str = include_str!("../template/login/index.html");
    HtmlResp(template.replace("__GOOGLE_AUDIENCE__", &config.google_auth.audience))
}

#[post("/login", format="application/json", data="<login>")]
fn login_post(config: State<AppConfig>, login: Json<rpc::LoginRequest>) -> impl Responder<'static> {
    let Json(login) = login;

    let conn = database::get_conn(config.database.write_url())
        .map_err(|e| {
            println!("postgres connection failure: {}", e);
            Failure(Status::InternalServerError)
        })?;

    let mut auth_data = None;
    if login.fap == "google" {
        let token = GoogleAuthToken(login.faat.clone());
        let prov = GoogleAuthProvider::new(&config.google_auth.audience);
        let auth = try!(prov.authenticate(&token)
            .map_err(|e| {
                println!("auth error: {:?}", e);
                Failure(Status::Forbidden)
            }));
        auth_data = Some(auth);
    }

    if auth_data.is_none() {
        return Err(Failure(Status::Forbidden));
    }
    let auth_data = auth_data.unwrap();

    let account = database::find_or_create_user(&conn, &auth_data)
        .map_err(|e| {
            println!("error: {}", e);
            Failure(Status::Forbidden)
        })?;

    println!("auth_data = {:?}", auth_data);
    let ainfo = AuthTokenInfo::new(account.get_user_id());
    
    // access_token = config.create_auth_token(&user_id);
    Ok(Json(rpc::LoginResponse {
        access_token: AuthTokenBlob::sign(config.secret.as_bytes(), &ainfo).into_inner(),
    }))
}

fn main() {
    let config_file = std::env::args_os().nth(1).expect("arg0: config.toml");
    let mut config = File::open(&config_file).unwrap();
    let mut config_str = String::new();
    config.read_to_string(&mut config_str).unwrap();
    let app: AppConfig = toml::from_str(&mut config_str).expect("error reading toml");

    rocket::ignite()
        .mount("/static", asset::statics())
        .mount("/", routes![
            hello,
            hi_get,
            blob_obj_get,
            tracks_search_get,
            login_get,
            login_post,
            songs_get,
            player_get,
        ])
        .manage(app)
        .launch()
}

#[derive(Deserialize)]
struct AppConfig {
    secret: String,
    google_auth: GoogleAuthConfig,
    database: DatabaseConfig,
}

#[derive(Deserialize)]
struct GoogleAuthConfig {
    audience: String,
}

#[derive(Deserialize)]
struct DatabaseConfig {
    read_url: Option<String>,
    write_url: String,
}

impl DatabaseConfig {
    fn read_url(&self) -> &str {
        if let Some(ref url) = self.read_url {
            return url;
        }
        &self.write_url
    }

    fn write_url(&self) -> &str {
        &self.write_url
    }
}

