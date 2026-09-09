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
            let _ = std::fs::rename(&path, dir.join("settings.json.bak"));
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
