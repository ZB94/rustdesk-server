use crate::database::AddressBook;
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
) -> Json<Response<AddressBookData>> {
    debug!("get address book");
    if let Err(e) = user::check_perm(&claims, None) {
        return Json(e);
    }

    let (error, data) = match pool.get_address_book(&claims.username).await {
        Ok(data) => (None, Some(AddressBookData { data })),
        Err(sqlx::Error::RowNotFound) => (Some("未找到地址簿信息，请联系管理员".to_string()), None),
        Err(e) => {
            warn!(error = %e, "获取地址簿时出现异常");
            (Some("获取地址簿失败，请联系管理员".to_string()), None)
        }
    };
    Json(Response { error, data })
}

#[instrument(skip(pool))]
pub async fn update_address_book(
    Json(AddressBookData { data }): Json<AddressBookData>,
    claims: Claims,
    pool: Extension<DbPool>,
) -> Json<Response<()>> {
    debug!("update address book");
    if let Err(e) = user::check_perm(&claims, None) {
        return Json(e);
    }

    let error = match pool
        .update_address_book(&claims.username, &data.tags, &data.peers)
        .await
    {
        Ok(()) => None,
        Err(e) => {
            warn!(error = %e, "更新地址簿失败");
            Some("更新失败，请重试".to_string())
        }
    };

    Json(Response { error, data: None })
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct AddressBookData {
    #[serde_as(as = "JsonString")]
    pub data: AddressBook,
}
