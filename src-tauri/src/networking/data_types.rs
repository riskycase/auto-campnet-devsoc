use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn default() -> Credentials {
        Credentials {
            username: "".to_owned(),
            password: "".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TrafficStats {
    pub total: f32,
    pub last: f32,
    pub current: f32,
    pub used: f32,
    pub remaining: f32,
}

impl TrafficStats {
    pub fn default() -> TrafficStats {
        TrafficStats {
            total: 0.0,
            last: 0.0,
            current: 0.0,
            used: 0.0,
            remaining: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TrafficUnits {
    pub total: String,
    pub last: String,
    pub current: String,
    pub used: String,
    pub remaining: String,
}

impl TrafficUnits {
    pub fn default() -> TrafficUnits {
        TrafficUnits {
            total: "".to_string(),
            last: "".to_string(),
            current: "".to_string(),
            used: "".to_string(),
            remaining: "".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Copy)]
pub enum NotificationState {
    None,
    Used50,
    Used90,
    Used100,
}
