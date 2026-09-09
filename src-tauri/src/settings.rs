use crate::model::Settings;
use std::path::Path;

const FILE: &str = "settings.json";

/// 任何读取失败都回退默认值：设置文件绝不能阻塞应用启动。
/// 解析失败的文件重命名备份，避免默认值随后覆盖掉仅存的现场。
pub fn load(dir: &Path) -> Settings {
    let path = dir.join(FILE);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => return Settings::default(),
    };
    match serde_json::from_str(&text) {
        Ok(settings) => settings,
        Err(_) => {
            let bak = dir.join("settings.json.bak");
            // 先移除旧备份再改名，明确「替换旧备份」的意图且不依赖 rename 的覆盖语义
            let _ = std::fs::remove_file(&bak);
            let _ = std::fs::rename(&path, bak);
            Settings::default()
        }
    }
}

/// 先写临时文件再原子替换，避免写入中途崩溃留下损坏的 settings.json
pub fn save(dir: &Path, settings: &Settings) -> Result<(), String> {
    let text = serde_json::to_string_pretty(settings).map_err(|err| err.to_string())?;
    let target = dir.join(FILE);
    let tmp = dir.join("settings.json.tmp");
    std::fs::write(&tmp, text).map_err(|err| err.to_string())?;
    std::fs::rename(&tmp, target).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Settings;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let unique = format!(
            "todo-settings-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn save_then_load_roundtrip() {
        let dir = temp_dir("roundtrip");
        let settings = Settings {
            notification_lead_minutes: 90,
            close_to_tray: false,
        };
        save(&dir, &settings).unwrap();
        let loaded = load(&dir);
        assert_eq!(loaded.notification_lead_minutes, 90);
        assert!(!loaded.close_to_tray);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn corrupt_settings_backed_up_and_defaults_loaded() {
        let dir = temp_dir("corrupt");
        std::fs::write(dir.join(FILE), "{not json").unwrap();
        let loaded = load(&dir);
        assert_eq!(loaded.notification_lead_minutes, 15);
        assert!(loaded.close_to_tray);
        // 损坏文件被改名备份，避免默认值随后覆盖掉仅存的现场
        assert!(!dir.join(FILE).exists());
        assert!(dir.join("settings.json.bak").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_settings_file_returns_defaults() {
        let dir = temp_dir("missing");
        let loaded = load(&dir);
        assert_eq!(loaded.notification_lead_minutes, 15);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
