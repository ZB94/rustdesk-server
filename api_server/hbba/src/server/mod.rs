use std::net::SocketAddr;
use std::sync::Arc;

use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Extension;
use serde::Serialize;

pub mod address_book;
pub mod jwt;
pub mod manage;
pub mod user;

use crate::database::DbPool;

pub async fn start(
    bind: &SocketAddr,
    pool: DbPool,
    static_dir: Option<String>,
    server_address: ServerAddress,
) -> Result<(), axum::BoxError> {
    let mut router = axum::Router::new()
        .route(
            "/server_address",
            get(|sa: Extension<Arc<ServerAddress>>| async move {
                let sa: ServerAddress = (&*sa.0).clone();
                Response::ok(sa)
            }),
        )
        .layer(Extension(Arc::new(server_address)));

    if let Some(d) = static_dir {
        debug!("static dir: {}", &d);
        let static_dir = axum_extra::routing::SpaRouter::new("/static", d);
        router = router.merge(static_dir).route(
            "/",
            get(|| async { axum::response::Redirect::permanent("/static/") }),
        );
    }

    router = router
        .route("/api/login", post(user::login))
        .route("/api/logout", post(user::logout))
        .route("/api/currentUser", post(user::current_user))
        .route("/api/ab", post(address_book::update_address_book))
        .route("/api/ab/get", post(address_book::get_address_book))
        .route("/manage/login", post(manage::login))
        .route("/manage/change_password", post(manage::change_password))
        .route("/manage/user", get(manage::get_users))
        .route("/manage/user", post(manage::crate_user))
        .route("/manage/user", delete(manage::delete_user))
        .route("/manage/user", put(manage::update_user))
        .layer(Extension(pool));

    axum::Server::bind(bind)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

#[derive(Debug, Serialize, Clone)]
pub struct ServerAddress {
    pub id_server: SocketAddr,
    pub reply_server: SocketAddr,
    pub api_server: SocketAddr,
    pub pubkey: String,
}

impl ServerAddress {
    pub fn new(
        id_server: SocketAddr,
        reply_server: Option<SocketAddr>,
        api_server: Option<SocketAddr>,
        pubkey: Option<String>,
    ) -> Self {
        let ip = id_server.ip();
        let reply_server = reply_server.unwrap_or_else(|| SocketAddr::new(ip, 21117));
        let api_server = api_server.unwrap_or_else(|| SocketAddr::new(ip, 21114));
        let pubkey = pubkey.unwrap_or_else(|| {
            std::fs::read_to_string("id_ed25519.pub")
                .expect("读取公钥文件失败")
                .trim()
                .to_string()
        });
        Self {
            id_server,
            reply_server,
            api_server,
            pubkey,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Response<T> {
    pub error: Option<String>,
    #[serde(flatten)]
    pub data: Option<T>,
}

impl<T> Response<T> {
    #[inline]
    pub fn ok(data: T) -> Self {
        Self {
            error: None,
            data: Some(data),
        }
    }

    #[inline]
    pub fn error<S: ToString>(error: S) -> Self {
        Self {
            error: Some(error.to_string()),
            data: None,
        }
    }
}

impl<T: Serialize> IntoResponse for Response<T> {
    fn into_response(self) -> axum::response::Response {
        axum::Json::into_response(axum::Json(self))
    }
}
