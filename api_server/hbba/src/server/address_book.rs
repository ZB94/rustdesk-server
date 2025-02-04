use crate::database::model::AddressBook;
use crate::server::jwt::Claims;
use crate::server::{user, Response};
use crate::DbPool;
use axum::{Extension, Json};
use serde_with::json::JsonString;

#[instrument(skip(pool))]
pub async fn get_address_book(
    _data: Json<serde_json::Value>,
    claims: Claims,
    pool: Extension<DbPool>,
) -> Response<AddressBookData> {
    debug!("get address book");
    if let Err(e) = user::check_perm(&claims, None) {
        return e;
    }

    match pool.get_address_book(&claims.username).await {
        Ok(data) => Response::ok(AddressBookData { data }),
        Err(sqlx::Error::RowNotFound) => Response::error("未找到地址簿信息，请联系管理员"),
        Err(e) => {
            warn!(error = %e, "获取地址簿时出现异常");
            Response::error("获取地址簿失败，请联系管理员")
        }
    }
}

#[instrument(skip(pool))]
pub async fn update_address_book(
    Json(AddressBookData { data }): Json<AddressBookData>,
    claims: Claims,
    pool: Extension<DbPool>,
) -> Response<()> {
    debug!("update address book");
    if let Err(e) = user::check_perm(&claims, None) {
        return e;
    }

    match pool
        .update_address_book(&claims.username, &data.tags, &data.peers)
        .await
    {
        Ok(()) => Response::ok(()),
        Err(e) => {
            warn!(error = %e, "更新地址簿失败");
            Response::error("更新失败，请重试")
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct AddressBookData {
    #[serde_as(as = "JsonString")]
    pub data: AddressBook,
}
