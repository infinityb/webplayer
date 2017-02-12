use hyper;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

use serde_json;

use super::{
    ProviderId,
    ForeignAccount,
    ForeignAuthProvider,
    AuthError,
    AuthErrorKind,
};

pub struct GoogleAuthProvider {
    expect_audience: String,
}

impl GoogleAuthProvider {
    pub fn new(audience: &str) -> GoogleAuthProvider {
        GoogleAuthProvider {
            expect_audience: audience.into(),
        }
    }
}

impl GoogleAuthProvider {
    fn provider_id(&self) -> ProviderId {
        ProviderId("google")
    }
}

pub struct GoogleAuthToken(pub String);

#[derive(Serialize, Deserialize, Debug)]
struct GoogleAuthResponse {
    iss: String,
    aud: String,
    sub: String,
    given_name: String,
    family_name: String,
}

impl ForeignAuthProvider for GoogleAuthProvider {
    type Token = GoogleAuthToken;

    fn authenticate(&self, token: &Self::Token) -> Result<ForeignAccount, AuthError> {
        let prov_id = self.provider_id();

        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        let hycli = hyper::Client::with_connector(connector);
        
        let url = format!("https://www.googleapis.com/oauth2/v3/tokeninfo?id_token={}", token.0);
        let mut resp = try!(hycli.get(&url).send());
        if resp.status != hyper::Ok {
            return Err(AuthError {
                kind: AuthErrorKind::RemoteServiceError,
                message: format!("bad status code: {}", resp.status),
            });
        }
        let aresp: GoogleAuthResponse = try!(serde_json::from_reader(&mut resp)
            .map_err(|e| AuthError {
                kind: AuthErrorKind::RemoteServiceError,
                message: format!("deserialization error: {}", e),
            }));

        if aresp.aud != self.expect_audience {
            return Err(AuthError {
                kind: AuthErrorKind::InvalidToken,
                message: "Unexpected audience".into(),
            });
        }

        Ok(ForeignAccount {
            provider: prov_id,
            account_id: aresp.sub.into(),
        })
    }
}