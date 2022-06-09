use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::Json;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use serde_with::json::JsonString;
use std::net::SocketAddr;
use uuid::Uuid;

#[tokio::main(flavor = "multi_thread")]
pub async fn start(bind: &SocketAddr) -> Result<(), axum::BoxError> {
    let router = axum::Router::new()
        // .nest("/api/logout", axum::routing::any(any_route))
        .route("/api/logout", axum::routing::post(logout))
        .route("/api/currentUser", axum::routing::post(current_user))
        .route("/api/ab/get", axum::routing::post(get_ab))
        .route("/api/ab", axum::routing::post(update_ab))
        .route("/api/login", axum::routing::post(login));

    axum::Server::bind(bind)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn get_ab() -> Json<Response<GetAddressBook>> {
    debug!("get address book");
    Json(Response {
        error: None,
        data: Some(GetAddressBook {
            updated_at: 0,
            data: Default::default(),
        }),
    })
}

async fn update_ab(
    Json(UpdateAddressBook { data }): Json<UpdateAddressBook>,
) -> Json<Response<()>> {
    debug!("update: {:#?}", data);
    Json(Response {
        error: None,
        data: Some(()),
    })
}

async fn login(Json(user): Json<Login>) -> Json<Response<LoginResponse>> {
    debug!("user login: {:#?}", &user);
    Json(Response {
        error: None,
        data: Some(LoginResponse {
            access_token: "test".to_string(),
            user: User {
                name: "login".to_string(),
            },
        }),
    })
}

async fn current_user(Json(lp): Json<LocalPeer>) -> (StatusCode, Json<Response<User>>) {
    debug!("current user: {:?}", lp);
    (
        StatusCode::OK,
        Json(Response {
            error: None,
            data: Some(User {
                name: "current user".to_string(),
            }),
        }),
    )
}

async fn logout(Json(lp): Json<LocalPeer>) -> Json<Response<()>> {
    debug!("logout: {:?}", lp);
    Json(Response {
        error: None,
        data: None,
    })
}

async fn any_route(
    uri: Uri,
    data: Json<Value>,
    method: Method,
    headers: HeaderMap,
) -> Json<Response<()>> {
    debug!("method: {}", method);
    debug!("path: {}", uri.path());
    debug!("headers: {:#?}", headers);
    debug!("data: {:#?}", &data);
    Json(Response {
        error: None,
        data: None,
    })
}

#[derive(Debug, Deserialize)]
struct LocalPeer {
    pub id: String,
    #[serde(deserialize_with = "de_local_peer_uuid")]
    pub uuid: Uuid,
}

#[derive(Debug, Deserialize)]
struct Login {
    pub username: String,
    pub password: String,
    #[serde(flatten)]
    pub local_peer: LocalPeer,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    pub access_token: String,
    pub user: User,
}

#[derive(Debug, Serialize)]
struct User {
    pub name: String,
}

#[derive(Debug, Serialize)]
struct Response<T> {
    pub error: Option<String>,
    #[serde(flatten)]
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct AddressBook {
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub peers: Vec<Peer>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Peer {
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
struct UpdateAddressBook {
    #[serde_as(as = "JsonString")]
    pub data: AddressBook,
}

#[serde_as]
#[derive(Debug, Serialize)]
struct GetAddressBook {
    pub updated_at: i64,
    #[serde_as(as = "JsonString")]
    pub data: AddressBook,
}

fn de_local_peer_uuid<'de, D>(de: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    let b64str = String::deserialize(de)?;
    let b64 = base64::decode(b64str).map_err(Error::custom)?;
    let uuid_str = String::from_utf8(b64).map_err(Error::custom)?;
    Uuid::parse_str(&uuid_str).map_err(Error::custom)
}
