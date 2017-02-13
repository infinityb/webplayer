#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket_contrib;
extern crate uuid;
extern crate serde_json;
extern crate postgres;
extern crate rocket;
extern crate hyper;
extern crate hyper_native_tls;
extern crate bincode;
extern crate crypto;

use std::io::{self, Read};
use rocket_contrib::{JSON as Json};
use rocket::response::content::{
    JSON as JsonResp,
    HTML as HtmlResp,
};
use rocket::response::Failure;
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

const GOOGLE_AUDIENCE: &'static str = "969546834490-hcmh52us6p7eethdev97mq4bnr558qj9.apps.googleusercontent.com";

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[get("/hi")]
fn hi_get() -> &'static str {
    "Hello, world!"
}

#[get("/blob/<id>")]
fn blob_obj_get(auth: AuthTokenBlob, id: BlobId) -> Result<Stream<Box<Read>>, Failure> {
    if !auth.is_valid() {
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

    Ok(rocket::response::Stream::from(Box::new(io::Cursor::new(out))))
}

#[derive(FromForm, Debug)]
struct Search {
   q: String,
}

#[get("/tracks/search?<search>")]
fn tracks_search_get(auth: AuthTokenBlob, search: Search) -> Result<String, Failure> {
    if !auth.is_valid() {
        return Err(Failure(Status::Forbidden));
    }

    Ok(format!("{:?}", search))
}

#[get("/songs")]
fn songs_get(auth: AuthTokenBlob) -> Result<String, Failure> {
    if !auth.is_valid() {
        return Err(Failure(Status::Forbidden));
    }
    // use std::io::Write;

    // let conn = establish_connection().map_err(|_| Failure(Status::InternalServerError))?;

    // let results = song.load::<Song>(&conn)
    //     .map_err(|_| Failure(Status::InternalServerError))?;
    
    let mut out = io::Cursor::new(Vec::new());
    // for so in results.iter() {
    //     write!(&mut out, "{:?}\n", so);
    // }
    Ok(String::from_utf8(out.into_inner()).unwrap())
}

#[get("/player")]
fn player_get() -> HtmlResp<&'static str> {
    HtmlResp(include_str!("../template/player/index.html"))
}

#[get("/login")]
fn login_get() -> HtmlResp<&'static str> {
    HtmlResp(include_str!("../template/login/index.html"))
}

#[post("/login", format="application/json", data="<login>")]
fn login_post(login: Json<rpc::LoginRequest>) -> Result<Json<rpc::LoginResponse>, Failure> {
    let Json(login) = login;

    let conn = database::get_conn()
        .map_err(|e| {
            println!("postgres connection failure: {}", e);
            Failure(Status::InternalServerError)
        })?;

    let mut auth_data = None;
    if login.fap == "google" {
        let token = GoogleAuthToken(login.faat.clone());
        let prov = GoogleAuthProvider::new(GOOGLE_AUDIENCE);
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
    
    Ok(Json(rpc::LoginResponse {
        access_token: AuthTokenBlob::sign(&ainfo).into_inner(),
    }))
}

fn main() {
    // fail fast if we start poorly
    std::env::var("PG_DATABASE_URL").unwrap();

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
        ]).launch()
}