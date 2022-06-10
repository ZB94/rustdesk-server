use model::{AddressBook, Peer, Permission, User};
use sqlx::Result;
use sqlx::SqlitePool;

pub mod model;

#[derive(Clone)]
pub struct DbPool {
    pool: SqlitePool,
}

/// 初始化
impl DbPool {
    /// 连接到数据库并进行初始化
    pub async fn new(url: &str) -> Result<Self> {
        let pool = Self {
            pool: SqlitePool::connect(url).await?,
        };
        pool.init().await?;
        Ok(pool)
    }

    async fn init(&self) -> Result<()> {
        sqlx::query(
            r#"
create table if not exists user
(
    username text     not null,
    password text     not null,
    perm     interage not null,
    disabled boolean  not null default false,
    primary key (username, perm)
);
create table if not exists address_book
(
    username text not null,
    updated_at datetime not null,
    tags text not null default '[]',
    peers text not null default '[]'
);
"#,
        )
        .execute(&self.pool)
        .await?;

        let _ = self
            .create_user("admin", "admin", Permission::Admin, false)
            .await;
        let _ = self
            .create_user("admin", "admin", Permission::User, false)
            .await;

        Ok(())
    }
}

/// 用户操作
impl DbPool {
    pub async fn query_user(
        &self,
        username: &str,
        password: &str,
        perm: Permission,
    ) -> Result<User> {
        sqlx::query_as("select * from user where username = ? and password = ? and perm = ?")
            .bind(username)
            .bind(password)
            .bind(perm)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
        perm: Permission,
        disabled: bool,
    ) -> Result<()> {
        sqlx::query("insert into user values (?, ?, ?, ?);")
            .bind(username)
            .bind(password)
            .bind(perm)
            .bind(disabled)
            .execute(&self.pool)
            .await?;

        if perm == Permission::User {
            let _ = sqlx::query("insert into address_book(username, updated_at) values (?, ?)")
                .bind(username)
                .bind(chrono::Local::now())
                .execute(&self.pool)
                .await;
        }

        Ok(())
    }

    pub async fn delete_user(&self, username: &str, perm: Permission) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        if perm == Permission::User {
            sqlx::query("delete from address_book where username = ?")
                .bind(username)
                .execute(&mut tx)
                .await?;
        }
        sqlx::query("delete from user where username = ? and perm = ?")
            .bind(username)
            .bind(perm)
            .execute(&mut tx)
            .await?;

        tx.commit().await
    }

    pub async fn update_user_password(
        &self,
        username: &str,
        old_password: &str,
        new_password: &str,
        perm: Permission,
    ) -> Result<()> {
        sqlx::query("update user set password = ? where username = ? and password = ? and perm = ?")
            .bind(new_password)
            .bind(username)
            .bind(old_password)
            .bind(perm)
            .execute(&self.pool)
            .await
            .and_then(|r| {
                if r.rows_affected() == 1 {
                    Ok(())
                } else {
                    Err(sqlx::Error::RowNotFound)
                }
            })
    }

    pub async fn get_users(&self) -> Result<Vec<User>> {
        sqlx::query_as("select * from user")
            .fetch_all(&self.pool)
            .await
    }
}

/// 地址簿操作
impl DbPool {
    pub async fn get_address_book(&self, username: &str) -> Result<AddressBook> {
        sqlx::query_as("select updated_at, tags, peers from address_book where username = ?")
            .bind(username)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn update_address_book(
        &self,
        username: &str,
        tags: &[String],
        peers: &[Peer],
    ) -> Result<()> {
        sqlx::query(
            "update address_book set updated_at = ?, tags = ?, peers = ? where username = ?",
        )
        .bind(chrono::Utc::now())
        .bind(sqlx::types::Json(tags))
        .bind(sqlx::types::Json(peers))
        .bind(username)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
