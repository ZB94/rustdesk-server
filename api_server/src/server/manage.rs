use crate::database::model::Permission;
use crate::server::jwt::Claims;
use crate::server::Response;
use crate::DbPool;
use axum::{Extension, Json};
use sqlx::Error;

#[instrument(skip(pool))]
pub async fn login(Json(login): Json<Login>, pool: Extension<DbPool>) -> Response<LoginResponse> {
    debug!("user login");
    match pool
        .query_user(&login.username, &login.password, login.perm)
        .await
    {
        Ok(user) => Response::ok(LoginResponse {
            access_token: Claims::gen_manage_token(user.username, user.perm),
            perm: user.perm,
        }),
        Err(Error::RowNotFound) => Response::error("用户名或密码错误"),
        Err(e) => {
            warn!(error = %e, "登录时发生异常");
            Response::error("登录时发生错误，请重试或联系管理员")
        }
    }
}

#[instrument(skip(pool))]
pub async fn change_password(
    claims: Claims,
    Json(cp): Json<ChangePassword>,
    pool: Extension<DbPool>,
) -> Response<()> {
    debug!("user change password");
    match pool
        .update_user_password(
            &claims.username,
            &cp.old_password,
            &cp.new_password,
            claims.perm,
        )
        .await
    {
        Ok(()) => Response::ok(()),
        Err(Error::RowNotFound) => Response::error("旧密码错误"),
        Err(e) => {
            warn!(error = %e, "修改密码时发生错误");
            Response::error("修改密码时发生错误，请重试或联系管理员")
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChangePassword {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
    pub perm: Permission,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub perm: Permission,
}
