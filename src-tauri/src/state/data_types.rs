extern crate timer;

use timer::Guard;

use crate::{CredentialManager, Credentials, NotificationState, TrafficStats, TrafficUnits};

#[derive(Clone)]
pub struct RunningState {
    pub login_guard: Option<Guard>,
    pub traffic_guard: Option<Guard>,
}

#[derive(Clone)]
pub struct UserState {
    pub credential_manager: CredentialManager,
    pub login_endpoint: String,
    pub credentials: Credentials,
}

#[derive(Clone)]
pub struct TrafficState {
    pub portal_endpoint: String,
    pub cookie: String,
    pub csrf: String,
    pub traffic: TrafficStats,
    pub traffic_units: TrafficUnits,
    pub last_notification_state: NotificationState,
}
