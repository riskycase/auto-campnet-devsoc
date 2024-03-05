use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

use crate::{RunningState, AutoLaunchManager, TrafficState, UserState};

fn auto_launch_check(app: AppHandle) {
    let window: tauri::Window = app.get_window("main").unwrap();
    window
        .emit(
            "autolaunch",
            app.state::<AutoLaunchManager>().is_enabled().unwrap(),
        )
        .unwrap();
}

pub fn show_window(app: AppHandle) {
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

pub fn reset_running_state(app: AppHandle) {
    let running_state = app.state::<Arc<Mutex<RunningState>>>();
    running_state.lock().unwrap().login_guard = Option::None;
    running_state.lock().unwrap().traffic_guard = Option::None;
}
