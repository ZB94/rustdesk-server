#[macro_use]
extern crate tracing;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate async_trait;

use crate::database::DbPool;

mod database;
mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("hbba=info,warn")),
        )
        .init();

    let pool = DbPool::new("sqlite://./db_v2.sqlite3")
        .await
        .expect("数据库连接失败");

    server::start(&"0.0.0.0:21114".parse().unwrap(), pool)
        .await
        .unwrap();
}
