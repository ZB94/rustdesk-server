#[macro_use]
extern crate tracing;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate async_trait;

use crate::database::DbPool;
use clap::Parser;
use std::net::SocketAddr;

mod database;
mod server;

#[tokio::main]
async fn main() {
    let args: Args = Parser::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("hbba=info,warn")),
        )
        .init();

    let pool = DbPool::new("sqlite://./db_v2.sqlite3")
        .await
        .expect("数据库连接失败");

    let server_address = server::ServerAddress::new(
        args.id_server,
        args.reply_server,
        args.api_server,
        args.pubkey,
    );

    server::start(
        &args.bind,
        pool,
        args.static_dir,
        args.download_dir,
        server_address,
    )
    .await
    .unwrap();
}

#[derive(Debug, Parser)]
#[clap(author, version)]
pub struct Args {
    /// ID服务器的公网地址
    pub id_server: SocketAddr,
    /// 中继服务器的公网地址。如未设置，将设置为与`id_server`相同IP，端口号为`21117`
    pub reply_server: Option<SocketAddr>,
    /// API服务器的公网地址。如未设置，将设置为与`id_server`相同IP，端口号为`21114`
    pub api_server: Option<SocketAddr>,
    /// 公钥。如为设置，将在启动时读取运行目录的`id_ed25519.pub`的内容
    pub pubkey: Option<String>,

    /// 服务监听地址
    #[clap(long, short, default_value = "0.0.0.0:21114")]
    pub bind: SocketAddr,
    /// UI资源目录。设置时将将指定目录的内容挂在到`/static`下
    #[clap(long, short)]
    pub static_dir: Option<String>,
    /// 设置客户端下载目录。设置时将指定目录的所有文件都改在到`/download`下
    #[clap(long, short)]
    pub download_dir: Option<String>,
}
