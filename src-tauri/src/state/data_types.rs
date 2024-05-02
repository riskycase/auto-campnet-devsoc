extern crate timer;

use tauri::{AppHandle, Manager};
use timer::Guard;

use crate::{
    networking::credentials::CredentialManager, Credentials, NotificationState, TrafficStats,
    TrafficUnits, state::tray::TrayManager
};

#[derive(Clone)]
pub struct RunningState {
    pub login_guard: Option<Guard>,
    pub traffic_guard: Option<Guard>,
}

impl RunningState {
    pub fn default() -> RunningState {
        RunningState {
            login_guard: Option::None,
            traffic_guard: Option::None,
        }
    }
}

#[derive(Clone)]
pub struct UserState {
    pub credential_manager: CredentialManager,
    pub login_endpoint: String,
    pub credentials: Credentials,
}

impl UserState {
    pub fn default(app: AppHandle, login_endpoint: String) -> UserState {
        let credential_manager = CredentialManager::new(app.app_handle());
        UserState {
            credential_manager: credential_manager.to_owned(),
            login_endpoint,
            credentials: Credentials::default(),
        }
    }
}

#[derive(Clone)]
pub struct TrafficState {
    pub portal_endpoint: String,
    pub cookie: String,
    pub csrf: String,
    pub traffic: TrafficStats,
    pub traffic_units: TrafficUnits,
    pub last_notification_state: NotificationState,
    pub tray_manager: TrayManager,
}

impl TrafficState {
    pub fn default(app: AppHandle, portal_endpoint: String) -> TrafficState {
        TrafficState {
            portal_endpoint,
            cookie: "".to_string(),
            csrf: "".to_string(),
            traffic: TrafficStats::default(),
            traffic_units: TrafficUnits::default(),
            last_notification_state: NotificationState::None,
            tray_manager: TrayManager::new(app),
        }
    }
}
