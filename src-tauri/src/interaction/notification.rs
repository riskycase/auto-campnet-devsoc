use crate::constants::APP_IDENTIFIER;
use tauri::api::notification::Notification;

fn notify(title: &str, body: &str) {
    Notification::new(APP_IDENTIFIER)
        .title(title)
        .body(body)
        .show()
        .unwrap()
}

pub fn notify_logged_in() {
    notify(
        "Connected to Campnet!",
        "Logged in successfully to campus network",
    )
}

pub fn notify_incorrect_creds() {
    notify(
        "Could not connect to Campnet!",
        "Incorrect credentials were provided",
    )
}

pub fn notify_limit_exceeded() {
    notify(
        "Could not connect to Campnet!",
        "Daily data limit exceeded on credentials",
    )
}

pub fn notify_failure_generic() {
    notify(
        "Could not connect to Campnet!",
        "There was an issue with the login attempt",
    )
}

pub fn notify_logged_out() {
    notify(
        "Logged out of campnet!",
        "",
    )
}

pub fn notify_logout_error() {
    notify(
        "Unable to logout of campnet!",
        "",
    )
}
pub fn notify_creds_saved() {
    notify(
        "Credentials saved to disk",
        "App will try to login to campnet whenever available",
    )
}
