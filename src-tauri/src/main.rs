// Do not show a console window on Windows
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use auto_launch::{AutoLaunch, AutoLaunchBuilder, Error};
use networking::credentials::CredentialManager;
use std::env::current_exe;
use std::sync::{Arc, Mutex};
use tauri::{api::notification::Notification, Manager};
extern crate chrono;
extern crate timer;

mod networking;
mod state;

use networking::data_types::{Credentials, NotificationState, TrafficStats, TrafficUnits};
use state::data_types::{RunningState, TrafficState, UserState};

pub struct AutoLaunchManager(AutoLaunch);

impl AutoLaunchManager {
    pub fn enable(&self) -> Result<(), Error> {
        self.0.enable()
    }

    pub fn disable(&self) -> Result<(), Error> {
        self.0.disable()
    }

    pub fn is_enabled(&self) -> Result<bool, Error> {
        self.0.is_enabled()
    }
}

fn connect_campnet(app: tauri::AppHandle, initial_run: bool) {
    if !initial_run {
        let running_state = app.state::<Arc<Mutex<RunningState>>>();
        running_state.lock().unwrap().login_guard = Option::None;
        let tray_handle = app.tray_handle();
        let resources_resolver = app.path_resolver();
        let active_icon_path = resources_resolver
            .resolve_resource("resources/icons/active.png")
            .unwrap();
        let used_50_icon_path = resources_resolver
            .resolve_resource("resources/icons/used_50.png")
            .unwrap();
        let used_90_icon_path = resources_resolver
            .resolve_resource("resources/icons/used_90.png")
            .unwrap();
        let inactive_icon_path = resources_resolver
            .resolve_resource("resources/icons/inactive.png")
            .unwrap();
        let user_state = app.state::<Arc<Mutex<UserState>>>();
        let credentials = user_state.lock().unwrap().credentials.to_owned();
        let client = reqwest::blocking::Client::new();
        let campnet_status = client
            .head(user_state.lock().unwrap().login_endpoint.to_owned())
            .send();
        if campnet_status.is_ok() {
            let login_status = client.head("https://www.google.com").send();
            if login_status.is_err() {
                let res = networking::user::login(
                    credentials,
                    user_state.lock().unwrap().login_endpoint.to_string(),
                );
                if res.is_ok() {
                    let traffic_state = app.state::<Arc<Mutex<TrafficState>>>();
                    let res_body: String = res.unwrap().text().unwrap();
                    if res_body.contains("LIVE") {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Connected to Campnet!")
                            .body("Logged in successfully to BPGC network")
                            .show()
                            .unwrap();
                        let current_notification_state =
                            traffic_state.lock().unwrap().last_notification_state;
                        if current_notification_state == NotificationState::None {
                            tray_handle
                                .set_icon(tauri::Icon::File(active_icon_path))
                                .unwrap();
                        } else if current_notification_state == NotificationState::Used50 {
                            tray_handle
                                .set_icon(tauri::Icon::File(used_50_icon_path))
                                .unwrap();
                        } else if current_notification_state == NotificationState::Used90 {
                            tray_handle
                                .set_icon(tauri::Icon::File(used_90_icon_path))
                                .unwrap();
                        }
                        let app_handle_next = app.app_handle();
                        let callback_timer = timer::Timer::new();
                        let callback_gaurd = callback_timer.schedule_with_delay(
                            chrono::Duration::milliseconds(2500),
                            move || {
                                connect_campnet(app_handle_next.app_handle(), false);
                            },
                        );
                        running_state.lock().unwrap().login_guard =
                            Option::Some(callback_gaurd.to_owned());
                        std::thread::sleep(std::time::Duration::from_secs(3));
                    } else if res_body.contains("failed") {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Could not connect to Campnet!")
                            .body("Incorrect credentials were provided")
                            .show()
                            .unwrap();
                        tray_handle
                            .set_icon(tauri::Icon::File(inactive_icon_path))
                            .unwrap();
                    } else if res_body.contains("exceeded") {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Could not connect to Campnet!")
                            .body("Daily data limit exceeded on credentials")
                            .show()
                            .unwrap();
                        tray_handle
                            .set_icon(tauri::Icon::File(inactive_icon_path))
                            .unwrap();
                    } else {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Could not to Campnet!")
                            .body("There was an issue with the login attempt")
                            .show()
                            .unwrap();
                        tray_handle
                            .set_icon(tauri::Icon::File(inactive_icon_path))
                            .unwrap();
                    }
                }
            } else {
                let app_handle_next = app.app_handle();
                let callback_timer = timer::Timer::new();
                let callback_gaurd = callback_timer.schedule_with_delay(
                    chrono::Duration::milliseconds(2500),
                    move || {
                        connect_campnet(app_handle_next.app_handle(), false);
                    },
                );
                running_state.lock().unwrap().login_guard = Option::Some(callback_gaurd.to_owned());
                tray_handle
                    .set_icon(tauri::Icon::File(active_icon_path))
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
        }
    } else {
        let app_handle_next = app.app_handle();
        let running_state = app.state::<Arc<Mutex<RunningState>>>();
        let callback_timer = timer::Timer::new();
        let callback_gaurd =
            callback_timer.schedule_with_delay(chrono::Duration::zero(), move || {
                connect_campnet(app_handle_next.app_handle(), false);
            });
        running_state.lock().unwrap().login_guard = Option::Some(callback_gaurd.to_owned());
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn auto_launch_check(app: tauri::AppHandle) {
    let window: tauri::Window = app.get_window("main").unwrap();
    window
        .emit(
            "autolaunch",
            app.state::<AutoLaunchManager>().is_enabled().unwrap(),
        )
        .unwrap();
}

fn main() {
    let tray_menu = tauri::SystemTrayMenu::new()
        .add_item(tauri::CustomMenuItem::new("show", "Show window"))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(tauri::CustomMenuItem::new("reconnect", "Force reconnect"))
        .add_item(tauri::CustomMenuItem::new("logout", "Logout"))
        .add_item(tauri::CustomMenuItem::new("delete", "Delete credentials"))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(tauri::CustomMenuItem::new("quit", "Quit"));
    let system_tray = tauri::SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        .setup(|app: &mut tauri::App| {
            let credential_manager = CredentialManager::new(app.app_handle());
            app.manage(Arc::new(Mutex::new(TrafficState {
                portal_endpoint: "https://campnet.bits-goa.ac.in:4443".to_string(),
                cookie: "".to_string(),
                csrf: "".to_string(),
                traffic: TrafficStats {
                    total: 0.0,
                    last: 0.0,
                    current: 0.0,
                    used: 0.0,
                    remaining: 0.0,
                },
                traffic_units: TrafficUnits {
                    total: "".to_string(),
                    last: "".to_string(),
                    current: "".to_string(),
                    used: "".to_string(),
                    remaining: "".to_string(),
                },
                last_notification_state: NotificationState::None,
            })));
            app.manage(Arc::new(Mutex::new(UserState {
                credential_manager: credential_manager.to_owned(),
                login_endpoint: "https://campnet.bits-goa.ac.in:8090".to_string(),
                credentials: Credentials {
                    username: "".to_string(),
                    password: "".to_string(),
                },
            })));
            app.manage(Arc::new(Mutex::new(RunningState {
                login_guard: Option::None,
                traffic_guard: Option::None,
            })));
            let creds = credential_manager.load();
            let app_handle_save = app.app_handle();
            app.listen_global("save", move |event: tauri::Event| {
                let creds: Credentials = serde_json::from_str(event.payload().unwrap()).unwrap();
                credential_manager.save(creds);
                let app_handle_thread = app_handle_save.app_handle();
                std::thread::spawn(move || {
                    connect_campnet(app_handle_thread.app_handle(), false);
                    networking::traffic::get_remaining_data(app_handle_thread.app_handle(), false);
                });
                Notification::new("com.riskycase.autocampnet")
                    .title("Credentials saved to disk")
                    .body("App will try to login to campnet whenever available")
                    .show()
                    .unwrap();
            });
            let app_handle_minimise = app.app_handle();
            app.listen_global("minimise", move |_event: tauri::Event| {
                app_handle_minimise
                    .get_window("main")
                    .unwrap()
                    .hide()
                    .unwrap();
            });
            let mut auto_launch_builder = AutoLaunchBuilder::new();
            auto_launch_builder.set_app_name(&app.package_info().name);
            let currnet_exe = current_exe();
            #[cfg(windows)]
            auto_launch_builder.set_app_path(&currnet_exe.unwrap().display().to_string());
            #[cfg(target_os = "macos")]
            {
                // on macOS, current_exe gives path to /Applications/Example.app/MacOS/Example
                // but this results in seeing a Unix Executable in macOS login items
                // It must be: /Applications/Example.app
                // If it didn't find exactly a single occurance of .app, it will default to
                // exe path to not break it.
                let exe_path = currnet_exe.unwrap().canonicalize()?.display().to_string();
                let parts: Vec<&str> = exe_path.split(".app/").collect();
                let app_path = if parts.len() == 2 {
                    format!("{}.app", parts.get(0).unwrap().to_string())
                } else {
                    exe_path
                };
                auto_launch_builder.set_app_path(&app_path);
            }
            #[cfg(target_os = "linux")]
            if let Some(appimage) = app
                .env()
                .appimage
                .and_then(|p| p.to_str().map(|s| s.to_string()))
            {
                auto_launch_builder.set_app_path(&appimage);
            } else {
                auto_launch_builder.set_app_path(&currnet_exe.unwrap().display().to_string());
            }

            app.manage(AutoLaunchManager(
                auto_launch_builder.build().map_err(|e| e.to_string())?,
            ));

            let app_handle_launch = app.app_handle();
            let _listen_global: tauri::EventHandler =
                app.listen_global("autolaunch", move |event: tauri::Event| {
                    if event.payload().unwrap().parse::<bool>().unwrap() {
                        app_handle_launch
                            .state::<AutoLaunchManager>()
                            .enable()
                            .unwrap();
                    } else {
                        app_handle_launch
                            .state::<AutoLaunchManager>()
                            .disable()
                            .unwrap();
                    }
                    auto_launch_check(app_handle_launch.app_handle());
                });
            let user_state = app.state::<Arc<Mutex<UserState>>>();
            let traffic_state = app.state::<Arc<Mutex<TrafficState>>>();
            if creds.is_ok() {
                user_state.lock().unwrap().login_endpoint =
                    String::from("https://campnet.bits-goa.ac.in:8090");
                traffic_state.lock().unwrap().portal_endpoint =
                    String::from("https://campnet.bits-goa.ac.in:4443");
                user_state.lock().unwrap().credentials = creds.unwrap();
                connect_campnet(app.app_handle(), true);
                networking::traffic::get_remaining_data(app.app_handle(), true);
            } else {
                app.get_window("main").unwrap().show().unwrap();
                auto_launch_check(app.app_handle());
            }
            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(|app: &tauri::AppHandle, event| match event {
            tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show" => {
                    auto_launch_check(app.app_handle());
                    let window: tauri::Window = app.get_window("main").unwrap();
                    window
                        .emit(
                            "credentials",
                            app.state::<Arc<Mutex<UserState>>>()
                                .lock()
                                .unwrap()
                                .credentials
                                .clone(),
                        )
                        .unwrap();
                    window
                        .emit(
                            "traffic",
                            app.state::<Arc<Mutex<TrafficState>>>()
                                .lock()
                                .unwrap()
                                .traffic
                                .clone(),
                        )
                        .unwrap();
                    window
                        .emit(
                            "traffic_units",
                            app.state::<Arc<Mutex<TrafficState>>>()
                                .lock()
                                .unwrap()
                                .traffic_units
                                .clone(),
                        )
                        .unwrap();
                    window.show().unwrap();
                    window.unminimize().unwrap();
                    window.set_focus().unwrap();
                }
                "logout" => {
                    let running_state = app.state::<Arc<Mutex<RunningState>>>();
                    let user_state = app.state::<Arc<Mutex<UserState>>>();
                    running_state.lock().unwrap().login_guard = Option::None;
                    let credentials = user_state.lock().unwrap().credentials.to_owned();
                    let login_endpoint = user_state.lock().unwrap().login_endpoint.to_owned();
                    if networking::user::logout(credentials, login_endpoint).is_ok() {
                        app.tray_handle()
                            .set_icon(tauri::Icon::File(
                                app.path_resolver()
                                    .resolve_resource("resources/icons/inactive.png")
                                    .unwrap(),
                            ))
                            .unwrap();
                        Notification::new("com.riskycase.autocampnet")
                            .title("Logged out of campnet!")
                            .show()
                            .unwrap();
                    } else {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Unable to logout of campnet!")
                            .show()
                            .unwrap();
                    }
                }
                "reconnect" => {
                    let running_state = app.state::<Arc<Mutex<RunningState>>>();
                    let user_state = app.state::<Arc<Mutex<UserState>>>();
                    let creds = user_state.lock().unwrap().credentials.to_owned();
                    running_state.lock().unwrap().login_guard = Option::None;
                    running_state.lock().unwrap().traffic_guard = Option::None;
                    if (creds.username.len() == 0) | (creds.password.len() == 0) {
                        let window: tauri::Window = app.get_window("main").unwrap();
                        window.show().unwrap();
                    } else {
                        connect_campnet(app.app_handle(), false);
                        networking::traffic::get_remaining_data(app.app_handle(), false);
                    }
                }
                "delete" => {
                    let running_state = app.state::<Arc<Mutex<RunningState>>>();
                    running_state.lock().unwrap().login_guard = Option::None;
                    running_state.lock().unwrap().traffic_guard = Option::None;
                    let user_state = app.state::<Arc<Mutex<UserState>>>();
                    user_state.lock().unwrap().credential_manager.clear();
                    user_state.lock().unwrap().credentials = Credentials {
                        username: "".to_owned(),
                        password: "".to_owned(),
                    };
                    let traffic_state = app.state::<Arc<Mutex<TrafficState>>>();
                    traffic_state.lock().unwrap().traffic = TrafficStats {
                        total: 0.0,
                        last: 0.0,
                        current: 0.0,
                        used: 0.0,
                        remaining: 0.0,
                    };
                    traffic_state.lock().unwrap().traffic_units = TrafficUnits {
                        total: "".to_string(),
                        last: "".to_string(),
                        current: "".to_string(),
                        used: "".to_string(),
                        remaining: "".to_string(),
                    };
                    auto_launch_check(app.app_handle());
                    let window: tauri::Window = app.get_window("main").unwrap();
                    window
                        .emit("credentials", user_state.lock().unwrap().credentials.clone())
                        .unwrap();
                    window
                        .emit("traffic", traffic_state.lock().unwrap().traffic.clone())
                        .unwrap();
                    window
                        .emit(
                            "traffic_units",
                            traffic_state.lock().unwrap().traffic_units.clone(),
                        )
                        .unwrap();
                    window.show().unwrap();
                }
                _ => {}
            },
            tauri::SystemTrayEvent::LeftClick {
                tray_id: _,
                position: _,
                size: _,
                ..
            } => {
                auto_launch_check(app.app_handle());
                let window: tauri::Window = app.get_window("main").unwrap();
                window
                    .emit(
                        "credentials",
                        app.state::<Arc<Mutex<UserState>>>()
                            .lock()
                            .unwrap()
                            .credentials
                            .clone(),
                    )
                    .unwrap();
                window
                    .emit(
                        "traffic",
                        app.state::<Arc<Mutex<TrafficState>>>()
                            .lock()
                            .unwrap()
                            .traffic
                            .clone(),
                    )
                    .unwrap();
                window
                    .emit(
                        "traffic_units",
                        app.state::<Arc<Mutex<TrafficState>>>()
                            .lock()
                            .unwrap()
                            .traffic_units
                            .clone(),
                    )
                    .unwrap();
                window.show().unwrap();
                window.unminimize().unwrap();
                window.set_focus().unwrap();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![networking::user::credential_check])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
