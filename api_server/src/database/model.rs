use chrono::Utc;
use sqlx::sqlite::SqliteRow;
use sqlx::{Error, FromRow, Row, Type};

#[derive(Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
#[repr(u8)]
pub enum Permission {
    Admin = 0,
    User,
}

#[derive(Debug, FromRow, Deserialize)]
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
