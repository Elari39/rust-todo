use crate::model::Settings;
use std::path::Path;

pub fn load(dir: &Path) -> Result<Settings, String> {
    let text = match std::fs::read_to_string(dir.join("settings.json")) {
        Ok(text) => text,
        Err(_) => return Ok(Settings::default()),
    };
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

pub fn save(dir: &Path, settings: &Settings) -> Result<(), String> {
    let text = serde_json::to_string_pretty(settings).map_err(|err| err.to_string())?;
    std::fs::write(dir.join("settings.json"), text).map_err(|err| err.to_string())
}
