use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub status: String,
    pub priority: String,
    pub kind: String,
    pub start_at: Option<String>,
    pub due_at: Option<String>,
    pub completed_at: Option<String>,
    pub project_id: Option<String>,
    pub notified: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTask {
    pub title: String,
    pub notes: Option<String>,
    pub priority: Option<String>,
    pub kind: Option<String>,
    pub start_at: Option<String>,
    pub due_at: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPatch {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub priority: Option<String>,
    pub kind: Option<String>,
    pub start_at: Option<String>,
    pub due_at: Option<String>,
    pub status: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub notification_lead_minutes: u32,
    pub close_to_tray: bool,
    pub locale: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            notification_lead_minutes: 15,
            close_to_tray: true,
            locale: "zh-CN".to_string(),
        }
    }
}

/// 备份导入的宽松反序列化结构：字段缺省时由导入流程补默认值，
/// 不与线上 Task/Project 结构体共用，避免给它们加无意义的 serde 默认
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupTask {
    #[serde(default)]
    pub id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub start_at: Option<String>,
    #[serde(default)]
    pub due_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub notified: bool,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupProject {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPayload {
    // 标识备份来源：导入时校验，防止选错 JSON 文件清空现有数据
    #[serde(default)]
    pub app: Option<String>,
    #[serde(default)]
    pub version: Option<i64>,
    #[serde(default)]
    pub tasks: Vec<BackupTask>,
    #[serde(default)]
    pub projects: Vec<BackupProject>,
}
