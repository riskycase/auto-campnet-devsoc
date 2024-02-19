use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TrafficStats {
    pub total: f32,
    pub last: f32,
    pub current: f32,
    pub used: f32,
    pub remaining: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TrafficUnits {
    pub total: String,
    pub last: String,
    pub current: String,
    pub used: String,
    pub remaining: String,
}

#[derive(Clone, PartialEq, Copy)]
pub enum NotificationState {
    None,
    Used50,
    Used90,
    Used100,
}