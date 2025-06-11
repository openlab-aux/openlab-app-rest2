#[macro_use]
extern crate tracing;

use self::{
    api::{ArrivalApi, HealthApi, PanicApi, PresenceApi},
    state::AppState,
    util::Zeroizing,
};
use clap::Parser;
use poem::{
    EndpointExt,
    middleware::{Compression, Cors, Tracing},
};
use poem_openapi::OpenApiService;
use std::{path::PathBuf, time::Duration};

pub mod api;
pub mod config;
pub mod state;
pub mod types;
pub mod util;

/// SIX HOURS FUCK YOU BITCH
pub const TTL: Duration = Duration::from_secs(6 * 3600);

#[derive(Parser)]
#[clap(author, about, version)]
pub struct Args {
    /// Path to the configuration file
    #[clap(long, short, default_value = "config.yml")]
    pub config: PathBuf,

    /// Don't mlock the whole address space of the program
    #[clap(long)]
    pub no_mlock: bool,
}

pub fn routes(state: AppState) -> impl poem::IntoEndpoint {
    let api = (ArrivalApi, HealthApi, PanicApi, PresenceApi);
    let service = OpenApiService::new(api, "OpenLab API for the app :3", env!("CARGO_PKG_VERSION"));
    let ui = service.swagger_ui();

    poem::Route::new()
        .nest("/", service)
        .nest("/docs", ui)
        .with(Cors::new().allow_credentials(true))
        .with(Compression::new())
        .with(Tracing)
        .data(state)
}
