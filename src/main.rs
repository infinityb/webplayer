#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(conservative_impl_trait)]
#![feature(fnbox)]

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
extern crate url;

use std::path::PathBuf;
use std::io::{self, Read};
use std::fs::File;
use rocket::{Response, State};
use rocket_contrib::{JSON as Json};
use rocket::response::content::{
    JSON as JsonResp,
    HTML as HtmlResp,
};
use rocket::response::{Responder, Failure};
use rocket::http::{Status, ContentType};
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
mod config;
mod webby;

use self::config::{AppConfig, VfsBackend};

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
use self::database::SongQuery;

const ENABLE_CORS: bool = true;

const ALLOW_ORIGINS: &'static [&'static str] = &[
    "http://music-dev.yshi.org",
];

#[options("/blob/<id>")]
fn blob_obj_options(/* request: &rocket::Request, */ id: BlobId) -> impl Responder<'static> {
    let mut builder = Response::build();

    // webby::Cors {
    //     allow_origins: ALLOW_ORIGINS,
    //     allow_methods: &["GET", "POST"],
    //     allow_headers: &["Content-Type", "Authorization"],
    //     expose_headers: &[],
    // }
    //     .set_headers(&request, &mut builder)
    //     .map_err(|()| Failure(Status::Forbidden))
    //     ?;

    builder.finalize()
}

#[get("/blob/<id>")]
fn blob_obj_get(config: State<AppConfig>, id: BlobId) -> impl Responder<'static> {
    // auth: AuthTokenBlob, 
    // let user_id = try!(config.validate_auth(&auth));
    // if !auth.is_valid(config.secret.as_bytes()) {
    //     return Err(Failure(Status::Forbidden));
    // }
    let vfs = config.vfs_driver.boxed();
    let stream = match vfs.open_read(&id) {
        Ok(stream) => stream,
        Err(err) => {
            println!("error opening blob: {}", err);
            return Err(Failure(Status::InternalServerError));
        }
    };
    Ok(wrap_blob(stream))
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

#[options("/songs")]
fn songs_options() -> impl Responder<'static> {
    cors_options()
}


#[get("/songs")]
fn songs_get(config: State<AppConfig>, auth: AuthTokenBlob) -> impl Responder<'static> {
    // let user_id = try!(config.validate_auth(&auth));
    if false && !auth.is_valid(config.secret.as_bytes()) {
        // XXX: richer errors
        return Err(Failure(Status::Forbidden));
    }

    let conn = database::drivers::get_driver(config.database.read_url())
        .map_err(|e| {
            println!("error: {:?}", e);
            Failure(Status::InternalServerError)
        })?;

    let songs = conn.get_songs(&SongQuery{})
        .map_err(|e| {
            println!("error: {:?}", e);
            Failure(Status::InternalServerError)
        })?;

    Ok(wrap_json(&rpc::SongSetResponse { results: songs}))
}

// Access-Control-Allow-Origin: *
// Access-Control-Allow-Methods: POST
// Access-Control-Allow-Headers: Content-Type

#[options("/login")]
fn login_options() -> impl Responder<'static> {
    cors_options()
}

#[post("/login", format="application/json", data="<login>")]
fn login_post(config: State<AppConfig>, login: Json<rpc::LoginRequest>) -> impl Responder<'static> {
    let Json(login) = login;

    let mut conn = database::drivers::get_driver(config.database.read_url())
        .map_err(|e| {
            println!("error: {:?}", e);
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

    let account = conn.find_or_create_user(&auth_data)
        .map_err(|e| {
            println!("error: {}", e);
            Failure(Status::Forbidden)
        })?;

    println!("auth_data = {:?}", auth_data);
    let ainfo = AuthTokenInfo::new(account.get_user_id());
    let token = AuthTokenBlob::sign(config.secret.as_bytes(), &ainfo).into_inner();

    Ok(wrap_json(&rpc::LoginResponse {
        access_token: token,
    }))
}

fn cors_options() -> impl Responder<'static> {
    let mut builder = Response::build();
    if ENABLE_CORS {
        builder.raw_header("Access-Control-Allow-Origin", "*");
        builder.raw_header("Access-Control-Allow-Methods", "GET, POST");
        builder.raw_header("Access-Control-Allow-Headers", "Content-Type, Authorization");
    }
    builder.finalize()
}

fn wrap_json<T: serde::Serialize>(ser: &T) -> impl Responder<'static> {
    let body = serde_json::to_vec(ser).unwrap();

    let mut builder = Response::build();
    builder.status(Status::Ok);
    builder.header(ContentType::JSON);
    if ENABLE_CORS {
        builder.raw_header("Access-Control-Allow-Origin", "*");
        builder.raw_header("Access-Control-Allow-Methods", "GET, POST");
        builder.raw_header("Access-Control-Allow-Headers", "Content-Type, Authorization");
    }
    builder.sized_body(io::Cursor::new(body));
    builder.finalize()
}

fn wrap_blob<T: 'static + Read>(rr: T) -> impl Responder<'static> {
    let mut builder = Response::build();
    builder.status(Status::Ok);
    builder.header(ContentType::JSON);
    if ENABLE_CORS {
        builder.raw_header("Access-Control-Allow-Origin", "*");
        builder.raw_header("Access-Control-Allow-Methods", "GET, POST");
        builder.raw_header("Access-Control-Allow-Headers", "Content-Type, Authorization");
    }
    builder.chunked_body(rr, 32 * 1024);
    builder.finalize()
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


            blob_obj_get,
            blob_obj_options,
            tracks_search_get,
            login_post,
            login_options,
            songs_get,
            songs_options,
        ])
        .manage(app)
        .launch()
}
