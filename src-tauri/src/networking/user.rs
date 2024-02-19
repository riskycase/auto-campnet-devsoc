use super::data_types::Credentials;
use crate::AppState;
use reqwest::{
    blocking::{Client, Response},
    Error,
};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

pub fn login(creds: Credentials, login_endpoint: String) -> Result<Response, Error> {
    let body: String = format!(
        "mode=191&username={}&password={}&a={}&producttype=1",
        creds.username,
        creds.password,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let client = Client::new();
    client
        .post(login_endpoint.to_owned() + "/login.xml")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", body.chars().count())
        .body(body)
        .send()
}

pub fn logout(creds: Credentials, login_endpoint: String) -> Result<Response, Error> {
    let body: String = format!(
        "mode=193&username={}&a={}&producttype=1",
        creds.username,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let client = Client::new();
    client
        .post(login_endpoint.to_owned() + "/logout.xml")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", body.chars().count())
        .body(body)
        .send()
}

#[tauri::command]
pub fn credential_check(username: String, password: String, app: AppHandle) -> Result<(), String> {
    let app_state = app.state::<Arc<Mutex<AppState>>>();
    if logout(
        Credentials {
            username: username.to_owned(),
            password: password.to_owned(),
        },
        app_state.lock().unwrap().login_endpoint.to_owned(),
    )
    .is_ok()
    {
        let res = login(
            Credentials { username, password },
            app_state.lock().unwrap().login_endpoint.to_string(),
        );
        if res.is_ok() {
            let res_body: String = res.unwrap().text().unwrap();
            if res_body.contains("LIVE") || res_body.contains("exceeded") {
                Ok(())
            } else if res_body.contains("failed") {
                Err("INVALIDCRED".to_string())
            } else {
                Err("UNKNOWN".to_string())
            }
        } else {
            Err("UNKNOWN".to_string())
        }
    } else {
        Err("NOSOPHOS".to_string())
    }
}
