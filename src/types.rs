use crate::Zeroizing;
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

#[derive(Clone, Deserialize, Enum, Serialize)]
pub enum ArrivalType {
    Connecten,
    Fokus,
    Gammeln,
}

#[derive(Clone, Deserialize, Object, Serialize)]
pub struct Arrival {
    pub arrival_type: ArrivalType,
    pub when: OffsetDateTime,
    pub edited_at: OffsetDateTime,
}

#[derive(Deserialize, Object, Serialize)]
pub struct PresenceResponse {
    pub users: HashMap<Zeroizing<String>, OffsetDateTime>,
}

#[derive(Deserialize, Object, Serialize)]
pub struct ArrivalResponse {
    pub users: HashMap<Zeroizing<String>, Arrival>,
}

#[derive(Deserialize, Object, Serialize)]
pub struct Response {
    pub message: Zeroizing<String>,
}

#[derive(Deserialize, Object, Serialize)]
pub struct Presence {
    pub nickname: Zeroizing<String>,
}

#[derive(Deserialize, Object, Serialize)]
pub struct ArrivalRequest {
    pub nickname: Zeroizing<String>,
    pub arrival_type: ArrivalType,
    pub when: OffsetDateTime,
}
