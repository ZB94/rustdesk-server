use crate::database::model::Permission;
use crate::server::jwt::Claims;
use crate::server::Response;
use crate::DbPool;
use axum::http::StatusCode;
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

#[instrument(skip(pool))]
pub async fn get_users(claims: Claims, pool: Extension<DbPool>) -> (StatusCode, Response<Users>) {
    debug!("get users");
    if let Err(e) = check_admin(&claims) {
        return e;
    }

    match pool.get_users().await {
        Ok(users) => {
            debug!(users = ?&users, "用户列表");
            (
                StatusCode::OK,
                Response::ok(Users {
                    users: users
                        .into_iter()
                        .map(|u| User {
                            username: u.username,
                            perm: u.perm,
                            disabled: u.disabled,
                        })
                        .collect(),
                }),
            )
        }
        Err(e) => {
            warn!(error = %e, "获取用户列表时出现异常");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Response::error("获取用户列表时出现错误，请重试或联系管理员"),
            )
        }
    }
}

#[instrument(skip(pool))]
pub async fn crate_user(
    Json(user): Json<crate::database::model::User>,
    claims: Claims,
    pool: Extension<DbPool>,
) -> (StatusCode, Response<()>) {
    if let Err(e) = check_admin(&claims) {
        return e;
    }

    match pool
        .create_user(&user.username, &user.password, user.perm, user.disabled)
        .await
    {
        Ok(_) => (StatusCode::OK, Response::ok(())),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Response::error("已存在相同用户名与权限用户"),
        ),
    }
}

#[instrument(skip(pool))]
pub async fn delete_user(
    Json(user): Json<DeleteUser>,
    claims: Claims,
    pool: Extension<DbPool>,
) -> (StatusCode, Response<()>) {
    if let Err(e) = check_admin(&claims) {
        return e;
    }

    match pool.delete_user(&user.username, user.perm).await {
        Ok(_) => (StatusCode::OK, Response::ok(())),
        Err(e) => {
            warn!(error = %e, "删除用户时发生异常");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Response::error("删除用户时发生错误，请重试或联系管理员"),
            )
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteUser {
    pub username: String,
    pub perm: Permission,
}

#[derive(Debug, Serialize)]
pub struct Users {
    pub users: Vec<User>,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub username: String,
    pub perm: Permission,
    pub disabled: bool,
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

#[inline]
fn check_admin<T>(claims: &Claims) -> Result<(), (StatusCode, Response<T>)> {
    if claims.perm == Permission::Admin {
        Ok(())
    } else {
        Err((StatusCode::UNAUTHORIZED, Response::error("权限不足")))
    }
}
