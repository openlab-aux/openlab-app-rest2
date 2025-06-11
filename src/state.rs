use crate::{TTL, types::Arrival, util::Zeroizing};
use moka::future::Cache;
use std::{ops::Deref, sync::Arc};
use time::OffsetDateTime;

pub struct AppStateInner {
    pub arrivals: Cache<Zeroizing<String>, Arrival>,
    pub panic_key: Zeroizing<String>,
    pub presence: Cache<Zeroizing<String>, OffsetDateTime>,
}

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AppState {
    pub fn new(panic_key: Zeroizing<String>) -> Self {
        Self(Arc::new(AppStateInner {
            arrivals: Cache::builder().time_to_live(TTL).build(),
            panic_key,
            presence: Cache::builder().time_to_live(TTL).build(),
        }))
    }
}
