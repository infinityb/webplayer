#[derive(Serialize, Deserialize, Debug)]
pub struct LoginRequest {
    // Foreign Account Provider name
    pub fap: String,
    // Foreign Account Access Token
    pub faat: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    pub access_token: String,
}