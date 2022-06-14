use std::future::ready;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, get_service, post, put};
use axum::Extension;
use serde::Serialize;

use crate::database::DbPool;

pub mod address_book;
pub mod jwt;
pub mod manage;
pub mod user;

pub async fn start(
    bind: &SocketAddr,
    pool: DbPool,
    static_dir: Option<String>,
    download_dir: Option<String>,
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
        let static_dir = tower_http::services::ServeDir::new(d);
        router = router
            .nest(
                "/static",
                get_service(static_dir).handle_error(|_| ready(StatusCode::INTERNAL_SERVER_ERROR)),
            )
            .route(
                "/",
                get(|| async { axum::response::Redirect::permanent("/static/") }),
            );
    }

    if let Some(d) = download_dir {
        debug!("download dir: {}", &d);
        let downloads = std::fs::read_dir(&d)
            .expect("遍历下载目录失败")
            .filter_map(|f| {
                let f = f.expect("获取下载目录文件信息失败");
                let path = f.path();
                if path.is_file() {
                    let name = path
                        .file_name()
                        .expect("获取下载目录文件名称失败")
                        .to_string_lossy()
                        .to_string();
                    Some(DownloadInfo {
                        url: format!("/download/{name}"),
                        name,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let download_dir = tower_http::services::ServeDir::new(d);
        router = router
            .nest(
                "/download",
                get_service(download_dir)
                    .handle_error(|_| ready(StatusCode::INTERNAL_SERVER_ERROR)),
            )
            .route(
                "/download_list",
                get(|dl: Extension<Arc<Vec<DownloadInfo>>>| async move {
                    let dl = (&*dl.0).clone();
                    Response::ok(serde_json::json!({ "links": dl }))
                }),
            )
            .layer(Extension(Arc::new(downloads)));
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
pub struct DownloadInfo {
    pub name: String,
    pub url: String,
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
