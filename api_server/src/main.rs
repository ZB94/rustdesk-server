#[macro_use]
extern crate tracing;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_with;

mod server;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("hbba=info")),
        )
        .init();

    server::start(&"0.0.0.0:21114".parse().unwrap()).unwrap();
}
