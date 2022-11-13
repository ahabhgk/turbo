#![feature(future_join)]
#![feature(min_specialization)]

use anyhow::Result;
#[cfg(feature = "cli")]
use clap::Parser;

#[global_allocator]
static ALLOC: turbo_malloc::TurboMalloc = turbo_malloc::TurboMalloc;

#[cfg(not(feature = "cli"))]
fn main() -> Result<()> {
    unimplemented!("Cannot run binary without CLI feature enabled");
}

#[tokio::main(flavor = "current_thread")]
#[cfg(feature = "cli")]
async fn main() -> Result<()> {
    // let options = next_dev::devserver_options::DevServerOptions::parse();
    let options = next_dev::devserver_options::DevServerOptions {
        dir: Some(std::path::PathBuf::from("demo")),
        root: None,
        port: 3000,
        hostname: std::net::IpAddr::from([0, 0, 0, 0]),
        eager_compile: false,
        no_open: true,
        log_level: None,
        show_all: true,
        log_detail: true,
        is_next_dev_command: false,
        allow_retry: false,
        dev: true,
        server_components_external_packages: Vec::new(),
    };

    next_dev::start_server(&options).await
}
