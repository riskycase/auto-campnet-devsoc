use super::data_types::{NotificationState, TrafficStats, TrafficUnits};
use crate::AppState;
use regex::Regex;
use std::sync::{Arc, Mutex};
use tauri::{api::notification::Notification, Manager};

fn get_cookie(app: tauri::AppHandle) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let app_state = app.state::<Arc<Mutex<AppState>>>();
    let credentials = app_state.lock().unwrap().credentials.clone();
    let body: String = format!(
        "mode=451&json=%7B%22username%22%3A%22{}%22%2C%22password%22%3A%22{}%22%2C%22languageid%22%3A%221%22%2C%22browser%22%3A%22Chrome_109%22%7D&t={}",
        credentials.username,
        credentials.password,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let response = client
        .post(app_state.lock().unwrap().portal_endpoint.to_owned() + "/userportal/Controller")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send();
    if response.is_ok() {
        app_state.lock().unwrap().cookie = response
            .unwrap()
            .headers()
            .get(reqwest::header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .split(";")
            .into_iter()
            .nth(0)
            .unwrap()
            .to_string();
        Ok(())
    } else {
        Err(response.err().unwrap())
    }
}

fn get_csrf(app: tauri::AppHandle) -> Result<(), ()> {
    let client = reqwest::blocking::Client::new();
    let app_state = app.state::<Arc<Mutex<AppState>>>();
    let cookie = app_state.lock().unwrap().cookie.to_string();
    let response = client
        .get(
            app_state.lock().unwrap().portal_endpoint.to_string()
                + "/userportal/webpages/myaccount/index.jsp",
        )
        .header(reqwest::header::COOKIE, cookie.to_string())
        .header(
            reqwest::header::USER_AGENT,
            format!("AutoCampnetRuntime/{}", app.package_info().version),
        )
        .send();
    if response.is_ok() {
        let regex = Regex::new(r"k3n = '(.+)'").unwrap();
        let body = response.unwrap().text().unwrap();
        let matches = regex.captures(body.as_str());
        if matches.is_some() {
            app_state.lock().unwrap().csrf = matches
                .unwrap()
                .get(0)
                .unwrap()
                .as_str()
                .split("'")
                .into_iter()
                .nth(1)
                .unwrap()
                .to_string();
            Ok(())
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

pub fn get_remaining_data(app: tauri::AppHandle, initial_run: bool) {
    if !initial_run {
        let app_state = app.state::<Arc<Mutex<AppState>>>();
        app_state.lock().unwrap().traffic_guard = Option::None;
        let client = reqwest::blocking::Client::new();
        let campnet_status = client
            .head(app_state.lock().unwrap().login_endpoint.to_owned())
            .send();
        if campnet_status.is_ok() {
            let cookie_result = get_cookie(app.app_handle());
            if cookie_result.is_ok() {
                let csrf_result = get_csrf(app.app_handle());
                if csrf_result.is_ok() {
                    let cookie = app_state.lock().unwrap().cookie.to_string();
                    let csrf = app_state.lock().unwrap().csrf.to_string();
                    let portal_endpoint = app_state.lock().unwrap().portal_endpoint.to_string();
                    let data_result = client
                        .get(
                            portal_endpoint.to_string()
                                + "/userportal/webpages/myaccount/AccountStatus.jsp",
                        )
                        .query(&[
                            ("popup", "0"),
                            (
                                "t",
                                format!(
                                    "{}",
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis()
                                )
                                .as_str(),
                            ),
                        ])
                        .header("X-CSRF-Token", csrf)
                        .header(reqwest::header::COOKIE, cookie)
                        .header(
                            reqwest::header::USER_AGENT,
                            format!("AutoCampnetRuntime/{}", app.package_info().version),
                        )
                        .header(
                            reqwest::header::REFERER,
                            portal_endpoint.to_string()
                                + "/userportal/webpages/myaccount/login.jsp",
                        )
                        .send();
                    if data_result.is_ok() {
                        let body_text = data_result.unwrap().text().unwrap();
                        let dom =
                            tl::parse(body_text.as_str(), tl::ParserOptions::default()).unwrap();
                        let parser = dom.parser();
                        let element = dom
                            .get_element_by_id("content3")
                            .expect("")
                            .get(parser)
                            .unwrap();
                        let table_text = element.inner_html(parser).to_string();
                        let sub_dom =
                            tl::parse(table_text.as_str(), tl::ParserOptions::default()).unwrap();
                        let sub_parser = sub_dom.parser();
                        let mut data_vector: Vec<f32> = Vec::new();
                        let mut unit_vector: Vec<String> = Vec::new();
                        let datas = sub_dom.query_selector("td.tabletext").unwrap();
                        datas.for_each(|data| {
                            data_vector.push(
                                data.get(sub_parser)
                                    .unwrap()
                                    .inner_text(sub_parser)
                                    .trim()
                                    .replace("&nbsp;", "")
                                    .parse::<f32>()
                                    .unwrap(),
                            );
                            unit_vector.push(
                                data.get(sub_parser)
                                    .unwrap()
                                    .children()
                                    .unwrap()
                                    .all(sub_parser)
                                    .get(1)
                                    .unwrap()
                                    .outer_html(sub_parser)
                                    .to_string()
                                    .split(".")
                                    .nth(1)
                                    .unwrap()
                                    .split("\"")
                                    .nth(0)
                                    .unwrap()
                                    .to_string(),
                            );
                        });
                        let traffic = TrafficStats {
                            total: data_vector[6],
                            last: data_vector[7],
                            current: data_vector[8],
                            used: data_vector[9],
                            remaining: data_vector[10],
                        };
                        app_state.lock().unwrap().traffic = traffic.clone();
                        let data_usage = traffic.used / traffic.total;
                        let current_notification_state = if data_usage < 0.5 {
                            NotificationState::None
                        } else if data_usage < 0.9 {
                            NotificationState::Used50
                        } else if data_usage < 1.0 {
                            NotificationState::Used90
                        } else {
                            NotificationState::Used100
                        };
                        let traffic_units = TrafficUnits {
                            total: unit_vector[6].to_string(),
                            last: unit_vector[7].to_string(),
                            current: unit_vector[8].to_string(),
                            used: unit_vector[9].to_string(),
                            remaining: unit_vector[10].to_string(),
                        };
                        app_state.lock().unwrap().traffic_units = traffic_units.clone();
                        if app_state.lock().unwrap().last_notification_state
                            != current_notification_state
                        {
                            if current_notification_state == NotificationState::Used50 {
                                Notification::new("com.riskycase.autocampnet")
                                    .title("50% data warning!")
                                    .body("Consider slowing down")
                                    .show()
                                    .unwrap();
                            } else if current_notification_state == NotificationState::Used90 {
                                Notification::new("com.riskycase.autocampnet")
                                    .title("90% data warning!")
                                    .body("Tread the interwebs slowly")
                                    .show()
                                    .unwrap();
                            }
                            app_state.lock().unwrap().last_notification_state =
                                current_notification_state
                        }
                        app.get_window("main")
                            .unwrap()
                            .emit("traffic", traffic.clone())
                            .unwrap();
                        app.get_window("main")
                            .unwrap()
                            .emit("traffic_units", traffic_units.clone())
                            .unwrap();
                    }
                }
            }
        }
        let app_handle_next = app.app_handle();
        let callback_timer = timer::Timer::new();
        let callback_gaurd =
            callback_timer.schedule_with_delay(chrono::Duration::seconds(45), move || {
                get_remaining_data(app_handle_next.app_handle(), false);
            });
        app_state.lock().unwrap().traffic_guard = Option::Some(callback_gaurd.to_owned());
        std::thread::sleep(std::time::Duration::from_secs(55));
    } else {
        let app_handle_next = app.app_handle();
        let app_state = app.state::<Arc<Mutex<AppState>>>();
        let callback_timer = timer::Timer::new();
        let callback_gaurd =
            callback_timer.schedule_with_delay(chrono::Duration::zero(), move || {
                get_remaining_data(app_handle_next.app_handle(), false);
            });
        app_state.lock().unwrap().traffic_guard = Option::Some(callback_gaurd.to_owned());
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
