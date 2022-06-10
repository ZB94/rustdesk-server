use chrono::Utc;
use sqlx::sqlite::SqliteRow;
use sqlx::FromRow;
use sqlx::Type;
use sqlx::{Error, Result};
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct DbPool {
    pool: SqlitePool,
}

impl DbPool {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = Self {
            pool: SqlitePool::connect(url).await?,
        };
        pool.init().await?;
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

        let _ = self.create_user("admin", "admin", Permission::Admin).await;
        let _ = self.create_user("admin", "admin", Permission::User).await;

        Ok(())
    }

    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
        perm: Permission,
    ) -> Result<()> {
        sqlx::query("insert into user values (?, ?, ?, false);")
            .bind(username)
            .bind(password)
            .bind(perm)
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

#[derive(Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AddressBook {
    #[serde(default)]
    pub updated_at: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub peers: Vec<Peer>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Peer {
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

impl<'r> FromRow<'r, SqliteRow> for AddressBook {
    fn from_row(row: &'r SqliteRow) -> std::result::Result<Self, Error> {
        let updated_at: chrono::DateTime<Utc> = row.try_get("updated_at")?;
        let tags: sqlx::types::Json<Vec<String>> = row.try_get("tags")?;
        let peers: sqlx::types::Json<Vec<Peer>> = row.try_get("peers")?;
        Ok(Self {
            updated_at: Some(updated_at),
            tags: tags.0,
            peers: peers.0,
        })
    }
}
