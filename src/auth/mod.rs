use uuid::Uuid;
use std::str::FromStr;
use rocket::request::FromRequest;
use rocket::{Request, Outcome};
use rocket::http::Status;
use bincode::{serialize, deserialize, SizeLimit};
use ::util::{dehex, hex};
use crypto::hmac::Hmac;
use crypto::sha2::Sha256;
use crypto::util::fixed_time_eq;


#[derive(Serialize, Deserialize)]
pub struct AuthTokenInfo {
    id: [u8; 16],
    exp: i64,
}

fn now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    dur.as_secs() as i64
}

impl AuthTokenInfo {
    pub fn is_valid(&self) -> bool {
        now() < self.exp
    }

    pub fn new(user_id: Uuid) -> AuthTokenInfo {
        AuthTokenInfo {
            id: *user_id.as_bytes(),
            exp: now() + 12 * 3600,
        }
    }
}

#[derive(Debug)]
pub struct AuthTokenBlob(pub String);

impl AuthTokenBlob {
    pub fn is_valid(&self, secret: &[u8]) -> bool {
        match self.decode(secret) {
            Ok(info) => info.is_valid(),
            Err(()) => false,
        }
    }

    pub fn sign(secret: &[u8], info: &AuthTokenInfo) -> AuthTokenBlob {
        let mut sig = [0; 32];
        let env_data = serialize(&info, SizeLimit::Bounded(256)).unwrap();
        env_secret_sig(secret, &env_data, &mut sig);
        let envelope = SigEnvelope {
            ver: 1,
            sig: sig,
            data: env_data,
        };
        let out = serialize(&envelope, SizeLimit::Bounded(256)).unwrap();
        AuthTokenBlob(hex(&out).unwrap())
    }

    pub fn decode(&self, secret: &[u8]) -> Result<AuthTokenInfo, ()> {
        let envelope_data: Vec<u8> = dehex(&self.0).map_err(|e| {
            println!("error dehexing: {:?}", e);
            ()
        })?;
        println!("envelope_data = {:?}", envelope_data);
        let envelope: SigEnvelope = deserialize(&envelope_data).map_err(|e| {
            println!("error deserializing envelope: {}", e);
            ()
        })?;
        let data = validate_sig(secret, &envelope)?;
        deserialize(data).map_err(|e| {
            println!("error deserializing info: {}", e);
            ()
        })
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl FromStr for AuthTokenBlob {
    type Err = ();
    
    fn from_str(val: &str) -> Result<Self, Self::Err> {
        Ok(AuthTokenBlob(val.to_owned()))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AuthTokenBlob {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, (Status, Self::Error), ()> {
        for token in request.headers().get("Authorization") {
            println!("token = {:?}", token);
            if let Some(bearer) = deprefix("bearer ", token) {
                println!("bearer token = {:?}", bearer);
                if let Ok(tok) = bearer.parse() {
                    println!("bearer token parsed = {:?}", tok);
                    return Outcome::Success(tok);
                }
            }
        }
        Outcome::Failure((Status::Forbidden, ()))
    }
}

fn deprefix<'a>(prefix: &'static str, value: &'a str) -> Option<&'a str> {
    if value.starts_with(prefix) {
        Some(&value[prefix.len()..])
    } else {
        None
    }
}

#[derive(Serialize, Deserialize)]
struct SigEnvelope {
    ver: i32,
    sig: [u8; 32],
    data: Vec<u8>,
}

fn validate_sig<'a>(secret: &[u8], env: &'a SigEnvelope) -> Result<&'a [u8], ()> {
    match env.ver {
        0 => validate_sig_v0(secret, env),
        1 => validate_sig_v1(secret, env),
        _ => Err(()),
    }
}

fn validate_sig_v0<'a>(secret: &[u8], env: &'a SigEnvelope) -> Result<&'a [u8], ()> {
    Ok(&env.data)
}

fn validate_sig_v1<'a>(secret: &[u8], env: &'a SigEnvelope) -> Result<&'a [u8], ()> {
    let mut desired_sig = [0; 32];
    env_secret_sig(secret, &env.data, &mut desired_sig);

    if !fixed_time_eq(&env.sig, &desired_sig) {
        return Err(());
    }

    Ok(&env.data)
}

fn env_secret_sig(secret: &[u8], data: &[u8], sig: &mut [u8; 32]) {
    use crypto::mac::Mac;
    let mut hmac = Hmac::new(Sha256::new(), &secret);
    hmac.input(&data);
    hmac.raw_result(sig);
}