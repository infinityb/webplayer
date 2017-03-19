use serde;
use serde_json;
use std::error::Error;
use std::io::{Read, Write};

use postgres::types::{FromSql, ToSql, IsNull, Type};

#[derive(Debug)]
pub struct JsonDocument(String);

impl JsonDocument {
    pub fn empty() -> JsonDocument {
        JsonDocument("{}".into())
    }
    
    pub fn into_inner(self) -> String {
        let JsonDocument(val) = self;
        val
    }

    pub fn deserialize<T>(&self) -> serde_json::Result<T>
        where T: serde::Deserialize
    {
        serde_json::from_str(&self.0)
    }
}

impl ::std::ops::Deref for JsonDocument {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl FromSql for JsonDocument {
    fn from_sql(ty: &Type, mut raw: &[u8])
        -> Result<JsonDocument, Box<Error + Sync + Send>>
    {
        if let Type::Jsonb = *ty {
            let mut b = [0; 1];
            try!(raw.read_exact(&mut b));
            // We only support version 1 of the jsonb binary format
            if b[0] != 1 {
                return Err("unsupported JSONB encoding version".into());
            }
        }
        let mut s = String::new();
        try!(raw.read_to_string(&mut s));
        Ok(JsonDocument(s))
    }

    fn accepts(ty: &Type) -> bool
    {
        match *ty {
            Type::Json | Type::Jsonb => true,
            _ => false,
        }
    }
}

impl ToSql for JsonDocument {
    fn to_sql(&self, ty: &Type, mut out: &mut Vec<u8>)
        -> Result<IsNull, Box<Error + Sync + Send>>
    {
        if let Type::Jsonb = *ty {
            out.push(1);
        }
        try!(write!(out, "{}", self.0));
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool
    {
        match *ty {
            Type::Json | Type::Jsonb => true,
            _ => false,
        }
    }

    to_sql_checked!();
}