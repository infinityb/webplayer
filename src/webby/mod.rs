use std::io::Cursor;

use rocket::Request;
use rocket::http::Method;
use rocket::response::{Response, Responder, ResponseBuilder};
use rocket::http::{HeaderMap, ContentType};

pub struct Cors {
    pub allow_origins: &'static [&'static str],
    pub allow_methods: &'static [&'static str],
    // ingress allowed user-set headers
    pub allow_headers: &'static [&'static str],
    // egress allowed user-viewable headers
    pub expose_headers: &'static [&'static str],
}

fn is_actual_request(req: &Request) -> Result<bool, ()>
{
    let headers = req.headers();
    if !headers.contains("Origin") {
        return Err(());
    }
    if req.method() != Method::Options {
        return Ok(true);
    }
    Ok(req.headers().contains("Access-Control-Request-Method"))
}

fn set_headers_actual(conf: &Cors, req: &Request, builder: &mut ResponseBuilder) -> Result<(), ()>
{
    if conf.expose_headers.len() > 0 {
        builder.raw_header("Access-Control-Expose-Headers", conf.expose_headers.join(", "));
    }

    Ok(())
}

fn set_headers_preflight(conf: &Cors, req: &Request, builder: &mut ResponseBuilder) -> Result<(), ()>
{
    let headers = req.headers();
    let ac_req_method = headers.get_one("Access-Control-Request-Method").ok_or(())?;

    let mut ok_meth = false;
    for &meth in conf.allow_methods.iter() {
        if ac_req_method == meth {
            ok_meth = true;
        }
    }
    if !ok_meth {
        return Err(());
    }

    if conf.allow_methods.len() > 0 {
        builder.raw_header("Access-Control-Allow-Methods", conf.allow_methods.join(", "));
    }
    if conf.allow_headers.len() > 0 {
        builder.raw_header("Access-Control-Allow-Headers", conf.allow_headers.join(", "));
    }

    Err(())
}

fn set_headers_common_final(conf: &Cors, req: &Request, builder: &mut ResponseBuilder) -> Result<(), ()>
{
    let headers = req.headers();
    let mut found_origin: Option<&'static str> = None;
    if let Some(origin) = headers.get_one("Origin") {
        for &allowed_origin in conf.allow_origins.iter() {
            if allowed_origin == origin {
                found_origin = Some(allowed_origin);
            }
        }
    }

    let found_origin = found_origin.ok_or(())?;
    builder.raw_header("Access-Control-Allow-Origin", found_origin);
    Ok(())
}

impl Cors {
    pub fn set_headers<'h>(&self, req: &Request, builder: &mut ResponseBuilder) -> Result<(), ()>
    {
        let actual_req = is_actual_request(req)?;
        
        if actual_req {
            set_headers_actual(self, req, builder)?;
        } else {
            set_headers_preflight(self, req, builder)?;
        }
        set_headers_common_final(self, req, builder)?;

        Ok(())
    }
}
