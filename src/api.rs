use crate::{
    state::AppState,
    types::{Arrival, ArrivalRequest, ArrivalResponse, Presence, PresenceResponse, Response},
    util::Zeroizing,
};
use poem::web::Data;
use poem_openapi::{ApiResponse, OpenApi, SecurityScheme, Tags, auth::Bearer, payload::Json};
use time::OffsetDateTime;

#[derive(Tags)]
enum ApiTags {
    Arrival,
    Health,
    Panic,
    Presence,
}

pub struct ArrivalApi;

#[OpenApi(prefix_path = "/arrival", tag = "ApiTags::Arrival")]
impl ArrivalApi {
    /// Remove your announced arrival
    #[oai(path = "/", method = "delete")]
    async fn delete(&self, Json(data): Json<Presence>, Data(state): Data<&AppState>) {
        state.arrivals.remove(&data.nickname).await;
    }

    /// Get all announced arrivals
    #[oai(path = "/", method = "get")]
    async fn get(&self, Data(state): Data<&AppState>) -> Json<ArrivalResponse> {
        let users = state
            .arrivals
            .iter()
            .map(|(name, value)| ((*name).clone(), value))
            .collect();

        Json(ArrivalResponse { users })
    }

    /// Announce a new arrival
    #[oai(path = "/", method = "put")]
    async fn put(&self, Json(data): Json<ArrivalRequest>, Data(state): Data<&AppState>) {
        state
            .arrivals
            .insert(
                data.nickname,
                Arrival {
                    arrival_type: data.arrival_type,
                    when: data.when,
                    edited_at: OffsetDateTime::now_utc(),
                },
            )
            .await;
    }
}

pub struct HealthApi;

#[OpenApi(prefix_path = "/health", tag = "ApiTags::Health")]
impl HealthApi {
    /// Health route
    #[oai(path = "/", method = "get")]
    async fn get(&self) -> Json<Response> {
        Json(Response {
            message: Zeroizing("Everything is working fine! :33".into()),
        })
    }
}

#[derive(SecurityScheme)]
#[oai(ty = "bearer")]
struct PanicAuth(Bearer);

#[derive(ApiResponse)]
enum PanicResponse {
    #[oai(status = "200")]
    Ok,

    #[oai(status = "401")]
    Unauthorized,
}

pub struct PanicApi;

#[OpenApi(prefix_path = "/panic", tag = "ApiTags::Panic")]
impl PanicApi {
    /// Clear any and all data from memory
    #[oai(path = "/", method = "post")]
    async fn post(
        &self,
        PanicAuth(bearer): PanicAuth,
        Data(state): Data<&AppState>,
    ) -> PanicResponse {
        if bearer.token != *state.panic_key {
            return PanicResponse::Unauthorized;
        }

        state.arrivals.invalidate_all();
        state.presence.invalidate_all();

        state.arrivals.run_pending_tasks().await;
        state.presence.run_pending_tasks().await;

        PanicResponse::Ok
    }
}

pub struct PresenceApi;

#[OpenApi(prefix_path = "/presence", tag = "ApiTags::Presence")]
impl PresenceApi {
    /// Delete a presence entry
    #[oai(path = "/", method = "delete")]
    async fn delete(&self, Json(data): Json<Presence>, Data(state): Data<&AppState>) {
        state.presence.remove(&data.nickname).await;
    }

    /// Get all current presence entries
    #[oai(path = "/", method = "get")]
    async fn get(&self, Data(state): Data<&AppState>) -> Json<PresenceResponse> {
        let users = state
            .presence
            .iter()
            .map(|(name, value)| ((*name).clone(), value))
            .collect();

        Json(PresenceResponse { users })
    }

    /// Announce a new presence
    #[oai(path = "/", method = "put")]
    async fn put(&self, Json(data): Json<Presence>, Data(state): Data<&AppState>) {
        state
            .presence
            .insert(data.nickname, OffsetDateTime::now_utc())
            .await;
    }
}
