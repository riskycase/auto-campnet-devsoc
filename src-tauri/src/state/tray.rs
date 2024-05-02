use tauri::{AppHandle, Icon, PathResolver, SystemTrayHandle};

#[derive(Clone)]
pub struct TrayManager {
    resolver: PathResolver,
    tray_handle: SystemTrayHandle,
}

impl TrayManager {
    pub fn new(app_handle: AppHandle) -> TrayManager {
        TrayManager {
            resolver: app_handle.path_resolver(),
            tray_handle: app_handle.tray_handle(),
        }
    }

    fn set_tray_icon(&self, path: &str) {
        self.tray_handle
            .set_icon(Icon::File(self.resolver.resolve_resource(path).unwrap()))
            .unwrap();
    }

    pub fn set_active_icon(&self) {
        self.set_tray_icon("resources/icons/active.png")
    }

    pub fn set_used_50_icon(&self) {
        self.set_tray_icon("resources/icons/used_50.png")
    }

    pub fn set_used_90_icon(&self) {
        self.set_tray_icon("resources/icons/used_90.png")
    }

    pub fn set_inactive_icon(&self) {
        self.set_tray_icon("resources/icons/inactive.png")
    }
}
