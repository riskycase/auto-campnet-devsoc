use super::data_types::Credentials;
use std::io::{Error, Read, Write};
use std::path::PathBuf;
use std::{fs, fs::File};
use tauri::AppHandle;

#[derive(Clone)]
pub struct CredentialManager {
    app_handle: AppHandle,
}

impl CredentialManager {
    pub fn new(app_handle: AppHandle) -> CredentialManager {
        CredentialManager { app_handle }
    }

    fn get_save_file(&self) -> PathBuf {
        self.app_handle
            .path_resolver()
            .app_config_dir()
            .unwrap()
            .join("credentials.json")
    }

    pub fn save(&self, creds: Credentials) -> Result<usize, Error> {
        fs::create_dir_all(self.app_handle.path_resolver().app_config_dir().unwrap()).unwrap();
        let file = File::create(&self.get_save_file());
        File::write(
            &mut file.unwrap(),
            serde_json::to_string(&creds).unwrap().as_bytes(),
        )
    }

    pub fn load(&self) -> Result<Credentials, String> {
        let mut creds_string = String::new();
        let result = File::read_to_string(
            &mut File::open(self.get_save_file()).unwrap(),
            &mut creds_string,
        );
        if result.is_ok() {
            let creds: Credentials = serde_json::from_str(&creds_string).unwrap();
            return Ok(creds);
        } else {
            return Err("Credentials not saved".to_string());
        }
    }

    pub fn clear(&self) -> Result<(), Error> {
        fs::remove_file(self.get_save_file())
    }
}
