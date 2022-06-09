use std::net::SocketAddr;

use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use serde_with::json::JsonString;
use uuid::Uuid;

mod jwt;

use crate::database::{DbPool, Permission};
use crate::server::jwt::Claims;

pub async fn start(bind: &SocketAddr, pool: DbPool) -> Result<(), axum::BoxError> {
    let router = axum::Router::new()
        .route("/api/logout", axum::routing::post(logout))
        .route("/api/currentUser", axum::routing::post(current_user))
        .route("/api/ab/get", axum::routing::post(get_ab))
        .route("/api/ab", axum::routing::post(update_ab))
        .route("/api/login", axum::routing::post(login))
        .layer(Extension(pool));

    axum::Server::bind(bind)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

#[instrument]
async fn get_ab(claims: Claims) -> Json<Response<GetAddressBook>> {
    debug!("get address book");
    check_perm(&claims, None)
        .map(|_| {
            Json(Response {
                error: None,
                data: Some(GetAddressBook {
                    updated_at: 0,
                    data: Default::default(),
                }),
            })
        })
        .unwrap_or_else(Json)
}

#[instrument]
async fn update_ab(
    Json(UpdateAddressBook { data: _data }): Json<UpdateAddressBook>,
    claims: Claims,
) -> Json<Response<()>> {
    debug!("update address book");
    check_perm(&claims, None)
        .map(|_| {
            Json(Response {
                error: None,
                data: Some(()),
            })
        })
        .unwrap_or_else(Json)
}

#[instrument(skip(pool))]
async fn login(Json(login): Json<Login>, pool: Extension<DbPool>) -> Json<Response<LoginResponse>> {
    debug!("user login");
    let (error, data) = match pool
        .query_user(&login.username, &login.password, Permission::User)
        .await
    {
        Ok(user) => {
            if user.disabled {
                (Some("该账号已被禁用,请联系管理员".to_string()), None)
            } else {
                let access_token = Claims::gen_user_token(login.username, login.local_peer);
                (
                    None,
                    Some(LoginResponse {
                        access_token,
                        user: User {
                            name: user.username,
                        },
                    }),
                )
            }
        }
        Err(sqlx::Error::RowNotFound) => (Some("用户名或密码错误".to_string()), None),
        Err(e) => {
            warn!(login_user=?login, error=%e, "用户登录时发生异常错误");
            (Some("服务器发生错误".to_string()), None)
        }
    };

    Json(Response { error, data })
}

#[instrument]
async fn current_user(
    Json(lp): Json<LocalPeer>,
    claims: Claims,
) -> (StatusCode, Json<Response<User>>) {
    debug!("query current user");

    check_perm(&claims, Some(&lp))
        .map(|_| {
            (
                StatusCode::OK,
                Json(Response {
                    error: None,
                    data: Some(User { name: claims.iss }),
                }),
            )
        })
        .unwrap_or_else(|r| (StatusCode::UNAUTHORIZED, Json(r)))
}

#[instrument]
async fn logout(Json(_local_peer): Json<LocalPeer>, _claims: Claims) -> Json<Response<()>> {
    debug!("user logout");
    Json(Response {
        error: None,
        data: None,
    })
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct LocalPeer {
    pub id: String,
    #[serde(with = "ser_local_peer_uuid")]
    pub uuid: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
    #[serde(flatten)]
    pub local_peer: LocalPeer,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub user: User,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct Response<T> {
    pub error: Option<String>,
    #[serde(flatten)]
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AddressBook {
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub peers: Vec<Peer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct UpdateAddressBook {
    #[serde_as(as = "JsonString")]
    pub data: AddressBook,
}

#[serde_as]
#[derive(Debug, Serialize)]
pub struct GetAddressBook {
    pub updated_at: i64,
    #[serde_as(as = "JsonString")]
    pub data: AddressBook,
}

mod ser_local_peer_uuid {
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use uuid::Uuid;

    pub fn deserialize<'de, D>(de: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let b64str = String::deserialize(de)?;
        let b64 = base64::decode(b64str).map_err(Error::custom)?;
        let uuid_str = String::from_utf8(b64).map_err(Error::custom)?;
        Uuid::parse_str(&uuid_str).map_err(Error::custom)
    }

    pub fn serialize<S>(uuid: &Uuid, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        base64::encode(uuid.to_string()).serialize(ser)
    }
}

#[inline]
fn check_perm<T>(claims: &Claims, lp: Option<&LocalPeer>) -> Result<(), Response<T>> {
    if claims.perm == Permission::User && lp.map(|lp| lp == &claims.local_peer).unwrap_or(true) {
        Ok(())
    } else {
        Err(Response {
            error: Some("用户权限异常，请重新登录".to_string()),
            data: None,
        })
    }
}
