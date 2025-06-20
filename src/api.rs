use crate::{
    state::AppState,
    types::{Arrival, ArrivalRequest, ArrivalResponse, PresenceResponse, Response},
    util::Zeroizing,
};
use poem::{IntoResponse, web::Data};
use poem_openapi::{
    ApiResponse, OpenApi, ResponseContent, SecurityScheme, Tags,
    auth::Bearer,
    payload::{Json, PlainText},
};
use time::OffsetDateTime;

macro_rules! verify_auth {
    ($state:ident, $bearer:ident) => {{
        if let Err(error) = $state.oidc_service.is_allowed(&$bearer).await {
            error!(?error);
            return GeneralResponse::Empty;
        }
    }};
}

macro_rules! dispatch_attempt {
    ($attempt:expr, |$var_name:ident| $logic:block) => {{
        match { $attempt } {
            Ok($var_name) => (async move |$var_name| $logic)($var_name).await,
            Err(error) => {
                error!(?error);
                GeneralResponse::Error
            }
        }
    }};
}

#[derive(Tags)]
enum ApiTags {
    Arrival,
    Health,
    Panic,
    Presence,
}

#[derive(ApiResponse)]
enum GeneralResponse<T = PlainText<&'static str>>
where
    T: IntoResponse + ResponseContent + Send,
{
    #[oai(status = "200")]
    Ok(T),

    #[oai(status = "204")]
    Empty,

    #[oai(status = "500")]
    Error,
}

pub struct ArrivalApi;

#[OpenApi(prefix_path = "/arrival", tag = "ApiTags::Arrival")]
impl ArrivalApi {
    /// Remove your announced arrival
    #[oai(path = "/", method = "delete")]
    async fn delete(
        &self,
        GeneralAuth(bearer): GeneralAuth,
        Data(state): Data<&AppState>,
    ) -> GeneralResponse {
        verify_auth!(state, bearer);
        dispatch_attempt!(
            state.oidc_service.load_username(&bearer).await,
            |username| {
                state.arrivals.remove(&username).await;
                GeneralResponse::Empty
            }
        )
    }

    /// Get all announced arrivals
    #[oai(path = "/", method = "get")]
    async fn get(
        &self,
        GeneralAuth(bearer): GeneralAuth,
        Data(state): Data<&AppState>,
    ) -> GeneralResponse<Json<ArrivalResponse>> {
        verify_auth!(state, bearer);

        let users = state
            .arrivals
            .iter()
            .map(|(name, value)| ((*name).clone(), value))
            .collect();

        GeneralResponse::Ok(Json(ArrivalResponse { users }))
    }

    /// Announce a new arrival
    #[oai(path = "/", method = "put")]
    async fn put(
        &self,
        GeneralAuth(bearer): GeneralAuth,
        Json(data): Json<ArrivalRequest>,
        Data(state): Data<&AppState>,
    ) -> GeneralResponse {
        verify_auth!(state, bearer);
        dispatch_attempt!(
            state.oidc_service.load_username(&bearer).await,
            |username| {
                state
                    .arrivals
                    .insert(
                        username,
                        Arrival {
                            arrival_type: data.arrival_type,
                            when: data.when,
                            edited_at: OffsetDateTime::now_utc(),
                        },
                    )
                    .await;

                GeneralResponse::Empty
            }
        )
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
#[oai(ty = "openid_connect")]
#[oai(openid_connect_url = "https://auth.openlab-augsburg.de/application/o/presence/")]
struct GeneralAuth(Bearer);

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
    async fn delete(
        &self,
        GeneralAuth(bearer): GeneralAuth,
        Data(state): Data<&AppState>,
    ) -> GeneralResponse {
        verify_auth!(state, bearer);
        dispatch_attempt!(
            state.oidc_service.load_username(&bearer).await,
            |username| {
                state.presence.remove(&username).await;
                GeneralResponse::Empty
            }
        )
    }

    /// Get all current presence entries
    #[oai(path = "/", method = "get")]
    async fn get(
        &self,
        GeneralAuth(bearer): GeneralAuth,
        Data(state): Data<&AppState>,
    ) -> GeneralResponse<Json<PresenceResponse>> {
        verify_auth!(state, bearer);

        let users = state
            .presence
            .iter()
            .map(|(name, value)| ((*name).clone(), value))
            .collect();

        GeneralResponse::Ok(Json(PresenceResponse { users }))
    }

    /// Announce a new presence
    #[oai(path = "/", method = "put")]
    async fn put(
        &self,
        GeneralAuth(bearer): GeneralAuth,
        Data(state): Data<&AppState>,
    ) -> GeneralResponse {
        verify_auth!(state, bearer);
        dispatch_attempt!(
            state.oidc_service.load_username(&bearer).await,
            |username| {
                state
                    .presence
                    .insert(username, OffsetDateTime::now_utc())
                    .await;

                GeneralResponse::Empty
            }
        )
    }
}
