use sqlx::FromRow;
use sqlx::Result;
use sqlx::SqlitePool;
use sqlx::Type;

#[derive(Clone)]
pub struct DbPool {
    pool: SqlitePool,
}

impl DbPool {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = Self {
            pool: SqlitePool::connect(url).await?,
        };
        pool.init_table().await?;
        Ok(pool)
    }

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

    async fn init_table(&self) -> Result<()> {
        sqlx::query(
            r#"create table if not exists user
(
    username text     not null,
    password text     not null,
    perm     interage not null,
    disabled boolean  not null default false,
    primary key (username, perm)
);"#,
        )
        .execute(&self.pool)
        .await?;

        let _ = sqlx::query("insert into user values ('admin', 'admin', 0, false);")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("insert into user values ('admin', 'admin', 1, false);")
            .execute(&self.pool)
            .await;

        Ok(())
    }
}

#[derive(Type)]
#[repr(u8)]
pub enum Permission {
    Admin = 0,
    User,
}

#[derive(FromRow)]
pub struct User {
    pub username: String,
    pub password: String,
    pub perm: Permission,
    pub disabled: bool,
}
