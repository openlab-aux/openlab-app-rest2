#[macro_use]
extern crate tracing;

use clap::Parser;
use color_eyre::{Section, eyre};
use openlab_app_rest::{Args, oidc::OidcService, state::AppState};
use poem::listener::TcpListener;
use std::io;
use tracing::Instrument;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                EnvFilter::builder()
                    .with_default_directive(tracing::Level::INFO.into())
                    .from_env_lossy(),
            ),
        )
        .with(tracing_error::ErrorLayer::default())
        .init();

    let args = Args::parse();

    if !args.no_mlock {
        // mlock everything. to prevent swap snooping of presence data.
        unsafe {
            let ret = libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE);
            if ret == -1 {
                let error = eyre::Report::from(io::Error::last_os_error())
                        .note("have you set the system limits to allow us to lock the entire virtual address space?");
                return Err(error);
            }
        }
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async move {
        let config = openlab_app_rest::config::load(args.config).await?;

        info!(addr = ?config.server.addr, "booting up..");

        let oidc_service = OidcService::from_config(&config.oidc).await?;
        let exit_notify = async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("received shutdown notification");
        }
        .instrument(info_span!("exit_notify"));

        let server_fut = poem::Server::new(TcpListener::bind(config.server.addr))
            .run_with_graceful_shutdown(
                openlab_app_rest::routes(AppState::new(config.api.panic_key, oidc_service)),
                exit_notify,
                None,
            );

        tokio::spawn(server_fut).await??;

        Ok(())
    })
}
