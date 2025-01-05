#[cfg(feature = "axum")]
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};
use uuid::Uuid;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct JWTClaims {
    pub id: Uuid,
    pub srv: String,
    pub exp: u64,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct JWTClaimsv2 {
    pub sub: String,
    pub exp: u64,
}

pub struct JWTKeys {
    pub private: Option<Vec<u8>>,
    pub public: Vec<u8>,
}

pub enum JWTKeyError {
    FileNotFound,
    FailedRead,
}

impl JWTKeys {
    async fn load_file(filename: &str) -> Result<Vec<u8>, JWTKeyError> {
        let mut key_file = match File::open(filename).await {
            Ok(key_file) => key_file,
            Err(_) => return Err(JWTKeyError::FileNotFound),
        };

        let mut key = Vec::new();

        match key_file.read_to_end(&mut key).await {
            Ok(_) => Ok(key),
            Err(_) => Err(JWTKeyError::FailedRead),
        }
    }

    pub async fn load(private_filename: &str, public_filename: &str) -> Self {
        let private_key = match JWTKeys::load_file(private_filename).await {
            Ok(key) => key,
            Err(error) => match error {
                JWTKeyError::FileNotFound => panic!("File {} not found!", private_filename),
                JWTKeyError::FailedRead => panic!("Failed to read file {}!", private_filename),
            },
        };

        let public_key = match JWTKeys::load_file(public_filename).await {
            Ok(key) => key,
            Err(error) => match error {
                JWTKeyError::FileNotFound => panic!("File {} not found!", public_filename),
                JWTKeyError::FailedRead => panic!("Failed to read file {}!", public_filename),
            },
        };

        JWTKeys {
            private: Some(private_key),
            public: public_key,
        }
    }

    pub async fn load_public(public_filename: &str) -> Self {
        let public_key = match JWTKeys::load_file(public_filename).await {
            Ok(key) => key,
            Err(error) => match error {
                JWTKeyError::FileNotFound => panic!("File {} not found!", public_filename),
                JWTKeyError::FailedRead => panic!("Failed to read file {}!", public_filename),
            },
        };

        JWTKeys {
            private: None,
            public: public_key,
        }
    }
}

pub trait GetJWTKeys {
    fn get_jwt_keys(&self) -> impl AsRef<JWTKeys>;
}

#[cfg(feature = "axum")]
pub async fn axum_middleware<S>(
    headers: HeaderMap,
    State(state): State<S>,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode>
where
    S: GetJWTKeys,
{
    if !headers.contains_key("authorization") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let header_token = headers
        .get("authorization")
        .unwrap()
        .to_str()
        .unwrap()
        .split(" ")
        .collect::<Vec<&str>>();
    if header_token.len() < 2 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match jsonwebtoken::decode::<JWTClaims>(
        header_token[1],
        &jsonwebtoken::DecodingKey::from_rsa_pem(&state.get_jwt_keys().as_ref().public).unwrap(),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS512),
    ) {
        Ok(token) => {
            request.extensions_mut().insert(token);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
