use std::net::SocketAddr;

use axum::response::IntoResponse;
use axum::routing::post;
use axum::Extension;
use serde::Serialize;

pub mod address_book;
pub mod jwt;
pub mod manage;
pub mod user;

use crate::database::DbPool;

pub async fn start(bind: &SocketAddr, pool: DbPool) -> Result<(), axum::BoxError> {
    let router = axum::Router::new()
        .route("/api/login", post(user::login))
        .route("/api/logout", post(user::logout))
        .route("/api/currentUser", post(user::current_user))
        .route("/api/ab", post(address_book::update_address_book))
        .route("/api/ab/get", post(address_book::get_address_book))
        .route("/manage/login", post(manage::login))
        .route("/manage/change_password", post(manage::change_password))
        .layer(Extension(pool));

    axum::Server::bind(bind)
        .serve(router.into_make_service())
        .await?;

    Ok(())
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
