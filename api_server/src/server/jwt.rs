use crate::database::Permission;
use crate::server::{LocalPeer, Response};
use axum::extract::{FromRequest, RequestParts};
use axum::http::StatusCode;
use axum::Json;
use jsonwebtoken::{Algorithm, Validation};

const SECRET: &[u8] = b"rustdesk api server";
const ALGORITHM: Algorithm = Algorithm::HS512;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub nbf: usize,
    pub local_peer: LocalPeer,
    pub perm: Permission,
}

impl Claims {
    pub fn gen_user_token(username: String, local_peer: LocalPeer) -> String {
        let current = chrono::Utc::now();
        let claims = Self {
            exp: (current + chrono::Duration::days(30)).timestamp() as usize,
            iat: current.timestamp() as usize,
            iss: username,
            nbf: current.timestamp() as usize,
            local_peer,
            perm: Permission::User,
        };

        let header = jsonwebtoken::Header::new(ALGORITHM);
        let key = jsonwebtoken::EncodingKey::from_secret(SECRET);
        jsonwebtoken::encode(&header, &claims, &key).unwrap()
    }
}

#[async_trait]
impl<B: Send> FromRequest<B> for Claims {
    type Rejection = (StatusCode, Json<Response<()>>);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        const ERROR_CODE: StatusCode = StatusCode::UNAUTHORIZED;
        const PREFIX: &str = "bearer";

        let header = req
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    ERROR_CODE,
                    Json(Response {
                        error: Some("输入token无效，请重新登录".to_string()),
                        data: None,
                    }),
                )
            })?;

        let mut iter = header.split_whitespace();
        iter.next()
            .and_then(|prefix| {
                if prefix.to_lowercase() == PREFIX {
                    Some(())
                } else {
                    None
                }
            })
            .and_then(|_| iter.next())
            .and_then(|token| {
                let key = jsonwebtoken::DecodingKey::from_secret(SECRET);
                let validation = Validation::new(ALGORITHM);
                jsonwebtoken::decode(token, &key, &validation).ok()
            })
            .map(|d| d.claims)
            .ok_or_else(|| {
                (
                    ERROR_CODE,
                    Json(Response {
                        error: Some("token格式错误，请重新登录".to_string()),
                        data: None,
                    }),
                )
            })
    }
}
