use crate::model::{BackupPayload, NewTask, Project, Task, TaskPatch};
use chrono::Local;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  color TEXT NOT NULL DEFAULT '#3D5BDB',
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS tasks (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  notes TEXT,
  status TEXT NOT NULL DEFAULT 'pending',
  priority TEXT NOT NULL DEFAULT 'normal',
  kind TEXT NOT NULL DEFAULT 'quick',
  start_at TEXT,
  due_at TEXT,
  completed_at TEXT,
  project_id TEXT,
  notified INTEGER NOT NULL DEFAULT 0,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_due ON tasks(due_at);
CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id);
"#;

pub fn open(path: &std::path::Path) -> Result<Connection, String> {
    match open_inner(path) {
        Ok(conn) => Ok(conn),
        Err(err) => {
            // 数据库文件损坏时备份现场再重建，绝不让应用完全无法启动
            if err.contains("file is not a database") && path.exists() {
                let file_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("todo.db");
                let backup = path.with_file_name(format!(
                    "{file_name}.corrupt-{}",
                    Local::now().format("%Y%m%d-%H%M%S")
                ));
                let _ = std::fs::rename(path, backup);
                return open_inner(path);
            }
            Err(err)
        }
    }
}

fn open_inner(path: &std::path::Path) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(to_err)?;
    // WAL + busy_timeout：开机自启 + 手动双开等场景下，另一进程持有写锁时
    // 等待重试而不是立即报 SQLITE_BUSY
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(to_err)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(to_err)?;
    let projects_fresh = !table_exists(&conn, "projects")?;
    let tasks_fresh = !table_exists(&conn, "tasks")?;
    conn.execute_batch(SCHEMA).map_err(to_err)?;
    ensure_sort_order(&conn)?;
    // 仅当库中缺 projects/tasks 表时补种示例数据（v0.2 前的旧库没有这两张表，
    // 升级时同样补齐）；老库即使任务被用户清空也不再复活种子
    if projects_fresh || tasks_fresh {
        let tx = conn.unchecked_transaction().map_err(to_err)?;
        if projects_fresh {
            seed_projects(&tx)?;
        }
        if tasks_fresh {
            seed_tasks(&tx)?;
        }
        tx.commit().map_err(to_err)?;
    }
    Ok(conn)
}

/// v0.2 老库没有 sort_order 列：补列并按写入顺序初始化
fn ensure_sort_order(conn: &Connection) -> Result<(), String> {
    if column_exists(conn, "tasks", "sort_order")? {
        return Ok(());
    }
    conn.execute_batch(
        "ALTER TABLE tasks ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
         UPDATE tasks SET sort_order = (SELECT COUNT(*) FROM tasks t2 WHERE t2.rowid <= tasks.rowid);",
    )
    .map_err(to_err)
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(to_err)?;
    let found = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(to_err)?
        .filter_map(Result::ok)
        .any(|name| name == column);
    Ok(found)
}

fn table_exists(conn: &Connection, name: &str) -> Result<bool, String> {
    let found: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [name],
            |row| row.get(0),
        )
        .map_err(to_err)?;
    Ok(found > 0)
}

fn seed_projects(conn: &Connection) -> Result<(), String> {
    for (name, color) in [
        ("工作", "#3D5BDB"),
        ("生活", "#F59E0B"),
        ("学习", "#EC4899"),
    ] {
        insert_project(conn, name, color)?;
    }
    Ok(())
}

fn project_id_by_name(conn: &Connection, name: &str) -> Option<String> {
    conn.query_row("SELECT id FROM projects WHERE name = ?1", [name], |row| {
        row.get(0)
    })
    .ok()
}

fn ensure_project_exists(conn: &Connection, project_id: Option<&str>) -> Result<(), String> {
    let Some(id) = project_id else {
        return Ok(());
    };
    let found: i64 = conn
        .query_row("SELECT COUNT(*) FROM projects WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .map_err(to_err)?;
    if found == 0 {
        return Err("项目不存在".into());
    }
    Ok(())
}

fn seed_tasks(conn: &Connection) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
        .map_err(to_err)?;
    if count > 0 {
        return Ok(());
    }

    let today = Local::now().date_naive();
    let work = project_id_by_name(conn, "工作");
    let life = project_id_by_name(conn, "生活");
    let study = project_id_by_name(conn, "学习");
    let samples = [
        (
            "买火车票，联系张三",
            "quick",
            "high",
            life.clone(),
            Some(format!("{}T09:00:00", today.format("%Y-%m-%d"))),
            Some(format!("{}T18:40:00", today.format("%Y-%m-%d"))),
        ),
        (
            "整理本周工作周报",
            "detailed",
            "normal",
            work.clone(),
            Some(format!(
                "{}T09:30:00",
                (today - chrono::Duration::days(2)).format("%Y-%m-%d")
            )),
            Some(format!("{}T18:00:00", today.format("%Y-%m-%d"))),
        ),
        (
            "完成 Todo 桌面端第二期",
            "detailed",
            "high",
            work.clone(),
            Some(format!("{}T10:00:00", today.format("%Y-%m-%d"))),
            Some(format!(
                "{}T21:00:00",
                (today + chrono::Duration::days(2)).format("%Y-%m-%d")
            )),
        ),
        (
            "回复项目进度同步邮件",
            "quick",
            "normal",
            work.clone(),
            Some(format!(
                "{}T16:00:00",
                (today - chrono::Duration::days(1)).format("%Y-%m-%d")
            )),
            Some(format!("{}T17:30:00", today.format("%Y-%m-%d"))),
        ),
        (
            "复习 Rust 所有权与生命周期",
            "detailed",
            "low",
            study,
            Some(format!("{}T19:00:00", today.format("%Y-%m-%d"))),
            Some(format!(
                "{}T21:00:00",
                (today + chrono::Duration::days(4)).format("%Y-%m-%d")
            )),
        ),
    ];

    for (title, kind, priority, project_id, start_at, due_at) in samples {
        insert(
            conn,
            NewTask {
                title: title.to_string(),
                notes: None,
                priority: Some(priority.to_string()),
                kind: Some(kind.to_string()),
                start_at,
                due_at,
                project_id,
            },
        )?;
    }
    Ok(())
}

fn now_stamp() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// 日期字符串入库前严格校验：list 的字典序排序与前端提醒都依赖
/// `YYYY-MM-DD[T ]HH:MM:SS` 格式成立，脏数据会静默破坏这两处
fn validate_stamp(value: Option<&str>) -> Result<(), String> {
    let Some(text) = value else {
        return Ok(());
    };
    // 空串与 None 同义：表示清除字段
    if text.trim().is_empty() {
        return Ok(());
    }
    for format in ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S"] {
        if chrono::NaiveDateTime::parse_from_str(text, format).is_ok() {
            return Ok(());
        }
    }
    Err(format!("无效的日期格式: {text}"))
}

fn to_err(err: impl ToString) -> String {
    err.to_string()
}

fn map_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        notes: row.get(2)?,
        status: row.get(3)?,
        priority: row.get(4)?,
        kind: row.get(5)?,
        start_at: row.get(6)?,
        due_at: row.get(7)?,
        completed_at: row.get(8)?,
        project_id: row.get(9)?,
        notified: row.get::<_, i64>(10)? != 0,
        sort_order: row.get(13)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

const SELECT: &str = "SELECT id, title, notes, status, priority, kind, start_at, due_at, completed_at, project_id, notified, created_at, updated_at, sort_order FROM tasks";

pub fn list(conn: &Connection) -> Result<Vec<Task>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "{SELECT} ORDER BY CASE WHEN status = 'completed' THEN 1 ELSE 0 END, \
             sort_order ASC, due_at IS NULL, due_at ASC, created_at DESC"
        ))
        .map_err(to_err)?;
    let rows = stmt
        .query_map([], map_task)
        .map_err(to_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(to_err)?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: &str) -> Result<Task, String> {
    conn.query_row(&format!("{SELECT} WHERE id = ?1"), [id], map_task)
        .optional()
        .map_err(to_err)?
        .ok_or_else(|| "任务不存在".to_string())
}

pub fn insert(conn: &Connection, input: NewTask) -> Result<Task, String> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err("标题不能为空".into());
    }
    ensure_project_exists(conn, input.project_id.as_deref())?;
    let start_at = empty_to_none(input.start_at);
    let due_at = empty_to_none(input.due_at);
    validate_stamp(start_at.as_deref())?;
    validate_stamp(due_at.as_deref())?;
    let id = Uuid::new_v4().to_string();
    let stamp = now_stamp();
    let priority = normalize_priority(input.priority.as_deref().unwrap_or("normal"));
    let kind = normalize_kind(input.kind.as_deref().unwrap_or("quick"));
    conn.execute(
        "INSERT INTO tasks (id, title, notes, status, priority, kind, start_at, due_at, completed_at, project_id, notified, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'pending', ?4, ?5, ?6, ?7, NULL, ?9, 0, (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM tasks), ?8, ?8)",
        params![
            id,
            title,
            empty_to_none(input.notes),
            priority,
            kind,
            start_at,
            due_at,
            stamp,
            input.project_id
        ],
    )
    .map_err(to_err)?;
    get(conn, &id)
}

pub fn update(conn: &Connection, id: &str, patch: TaskPatch) -> Result<Task, String> {
    let current = get(conn, id)?;
    let title = patch
        .title
        .map(|value| value.trim().to_string())
        .unwrap_or(current.title);
    if title.is_empty() {
        return Err("标题不能为空".into());
    }
    let notes = match patch.notes {
        Some(value) => {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        None => current.notes,
    };
    let priority = patch
        .priority
        .as_deref()
        .map(normalize_priority)
        .unwrap_or(current.priority);
    let kind = patch
        .kind
        .as_deref()
        .map(normalize_kind)
        .unwrap_or(current.kind);
    validate_stamp(patch.start_at.as_deref())?;
    validate_stamp(patch.due_at.as_deref())?;
    let start_at = match patch.start_at {
        Some(ref value) => empty_to_none(Some(value.clone())),
        None => current.start_at.clone(),
    };
    let due_at = match patch.due_at {
        Some(ref value) => empty_to_none(Some(value.clone())),
        None => current.due_at.clone(),
    };
    // 改期或重开都让任务重新进入提醒流程：已提醒过的任务改了截止时间，
    // 不重置 notified 会导致新提醒永不触发
    let due_changed = patch.due_at.is_some() && due_at != current.due_at;
    let reopened = matches!(patch.status.as_deref(), Some("pending") | Some("in_progress"))
        && current.status == "completed";
    let notified = if due_changed || reopened {
        false
    } else {
        current.notified
    };
    // 只校验显式传入的 project_id；存量数据里的孤儿引用不阻塞后续编辑
    if let Some(value) = patch.project_id.as_deref() {
        if !value.trim().is_empty() {
            ensure_project_exists(conn, Some(value))?;
        }
    }
    let project_id = match patch.project_id {
        Some(value) if value.trim().is_empty() => None,
        Some(value) => Some(value),
        None => current.project_id,
    };
    let (status, completed_at) = match patch.status.as_deref() {
        // 已完成的任务再次标记完成时保留首次完成时间
        Some("completed") if current.status == "completed" => {
            (current.status.clone(), current.completed_at.clone())
        }
        Some("completed") => ("completed".to_string(), Some(now_stamp())),
        // pending / in_progress 都是未完成态，清除完成时间
        Some("pending") | Some("in_progress") => {
            (patch.status.clone().unwrap(), None)
        }
        Some(_) => return Err("无效的任务状态".into()),
        None => (current.status, current.completed_at),
    };
    let stamp = now_stamp();
    conn.execute(
        "UPDATE tasks SET title=?1, notes=?2, priority=?3, kind=?4, start_at=?5, due_at=?6, status=?7, completed_at=?8, project_id=?9, notified=?10, updated_at=?11 WHERE id=?12",
        params![title, notes, priority, kind, start_at, due_at, status, completed_at, project_id, notified, stamp, id],
    )
    .map_err(to_err)?;
    get(conn, id)
}

pub fn complete(conn: &Connection, id: &str) -> Result<Task, String> {
    update(
        conn,
        id,
        TaskPatch {
            title: None,
            notes: None,
            priority: None,
            kind: None,
            start_at: None,
            due_at: None,
            status: Some("completed".into()),
            project_id: None,
        },
    )
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), String> {
    let changed = conn
        .execute("DELETE FROM tasks WHERE id = ?1", [id])
        .map_err(to_err)?;
    if changed == 0 {
        return Err("任务不存在".into());
    }
    Ok(())
}

pub fn mark_notified(conn: &Connection, id: &str) -> Result<Task, String> {
    let changed = conn
        .execute("UPDATE tasks SET notified = 1 WHERE id = ?1", [id])
        .map_err(to_err)?;
    if changed == 0 {
        return Err("任务不存在".into());
    }
    get(conn, id)
}

/// 按传入顺序重写 sort_order；未列出的任务保持原值（排在已列出任务之后）
pub fn reorder(conn: &mut Connection, ordered_ids: &[String]) -> Result<(), String> {
    let tx = conn.transaction().map_err(to_err)?;
    let stamp = now_stamp();
    for (index, id) in ordered_ids.iter().enumerate() {
        tx.execute(
            "UPDATE tasks SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
            params![(index + 1) as i64, stamp, id],
        )
        .map_err(to_err)?;
    }
    tx.commit().map_err(to_err)
}

struct BackupTaskRow {
    id: String,
    title: String,
    notes: Option<String>,
    status: String,
    priority: String,
    kind: String,
    start_at: Option<String>,
    due_at: Option<String>,
    completed_at: Option<String>,
    project_id: Option<String>,
    notified: bool,
    sort_order: i64,
    created_at: String,
    updated_at: String,
}

/// 从备份载荷整体替换任务与项目：任何一条记录不合法都整体拒绝，
/// 不会出现导入一半的中间状态。返回 (任务数, 项目数)。
pub fn import_replace(
    conn: &mut Connection,
    payload: BackupPayload,
) -> Result<(usize, usize), String> {
    if let Some(version) = payload.version {
        if version != 1 {
            return Err(format!("不支持的备份文件版本: {version}"));
        }
    }

    // ---- 预校验并归一化项目 ----
    let mut project_rows: Vec<(String, String, String, String)> = Vec::new();
    let mut project_ids: Vec<String> = Vec::new();
    for item in &payload.projects {
        let name = item.name.trim().to_string();
        if name.is_empty() {
            return Err("备份中的项目名不能为空".into());
        }
        if project_rows.iter().any(|(_, existing, _, _)| *existing == name) {
            return Err(format!("备份中的项目名重复: {name}"));
        }
        let id = match item.id.as_deref() {
            Some(id) if !id.trim().is_empty() => id.trim().to_string(),
            _ => Uuid::new_v4().to_string(),
        };
        if project_ids.contains(&id) {
            return Err("备份中的项目 ID 重复".into());
        }
        let color = item
            .color
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "#3D5BDB".to_string());
        let created_at = match &item.created_at {
            Some(value) => {
                validate_stamp(Some(value.as_str()))?;
                value.clone()
            }
            None => now_stamp(),
        };
        project_ids.push(id.clone());
        project_rows.push((id, name, color, created_at));
    }

    // ---- 预校验并归一化任务 ----
    let mut task_rows: Vec<BackupTaskRow> = Vec::new();
    let mut task_ids: Vec<String> = Vec::new();
    for item in &payload.tasks {
        let title = item.title.trim().to_string();
        if title.is_empty() {
            return Err("备份中的任务标题不能为空".into());
        }
        for stamp in [&item.start_at, &item.due_at, &item.completed_at] {
            validate_stamp(stamp.as_deref())?;
        }
        for stamp in [&item.created_at, &item.updated_at] {
            if let Some(value) = stamp {
                validate_stamp(Some(value.as_str()))?;
            }
        }
        let id = match item.id.as_deref() {
            Some(id) if !id.trim().is_empty() => id.trim().to_string(),
            _ => Uuid::new_v4().to_string(),
        };
        if task_ids.contains(&id) {
            return Err("备份中的任务 ID 重复".into());
        }
        let status = match item.status.as_deref() {
            Some("completed") => "completed".to_string(),
            Some("in_progress") => "in_progress".to_string(),
            _ => "pending".to_string(),
        };
        let completed_at = if status == "completed" {
            item.completed_at
                .clone()
                .or_else(|| item.updated_at.clone())
                .or_else(|| Some(now_stamp()))
        } else {
            None
        };
        let project_id = match item.project_id.as_deref() {
            Some(id) if !id.trim().is_empty() => {
                if !project_ids.contains(&id.trim().to_string()) {
                    return Err(format!(
                        "备份中的任务「{title}」引用了不存在的项目"
                    ));
                }
                Some(id.trim().to_string())
            }
            _ => None,
        };
        let stamp = item.created_at.clone().unwrap_or_else(now_stamp);
        let updated = item.updated_at.clone().unwrap_or_else(|| stamp.clone());
        task_ids.push(id.clone());
        task_rows.push(BackupTaskRow {
            id,
            title,
            notes: empty_to_none(item.notes.clone()),
            status,
            priority: normalize_priority(item.priority.as_deref().unwrap_or("normal")),
            kind: normalize_kind(item.kind.as_deref().unwrap_or("quick")),
            start_at: empty_to_none(item.start_at.clone()),
            due_at: empty_to_none(item.due_at.clone()),
            completed_at,
            project_id,
            notified: item.notified,
            sort_order: item.sort_order.max(0),
            created_at: stamp,
            updated_at: updated,
        });
    }

    // ---- 事务内整体替换 ----
    let tx = conn.transaction().map_err(to_err)?;
    tx.execute("DELETE FROM tasks", []).map_err(to_err)?;
    tx.execute("DELETE FROM projects", []).map_err(to_err)?;
    for (id, name, color, created_at) in &project_rows {
        tx.execute(
            "INSERT INTO projects (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, color, created_at],
        )
        .map_err(to_err)?;
    }
    for row in &task_rows {
        tx.execute(
            "INSERT INTO tasks (id, title, notes, status, priority, kind, start_at, due_at, completed_at, project_id, notified, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                row.id,
                row.title,
                row.notes,
                row.status,
                row.priority,
                row.kind,
                row.start_at,
                row.due_at,
                row.completed_at,
                row.project_id,
                row.notified,
                row.sort_order,
                row.created_at,
                row.updated_at
            ],
        )
        .map_err(to_err)?;
    }
    tx.commit().map_err(to_err)?;
    Ok((task_rows.len(), project_rows.len()))
}

fn map_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        created_at: row.get(3)?,
    })
}

const SELECT_PROJECT: &str = "SELECT id, name, color, created_at FROM projects";

pub fn list_projects(conn: &Connection) -> Result<Vec<Project>, String> {
    let mut stmt = conn
        .prepare(&format!("{SELECT_PROJECT} ORDER BY created_at ASC"))
        .map_err(to_err)?;
    let rows = stmt
        .query_map([], map_project)
        .map_err(to_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(to_err)?;
    Ok(rows)
}

fn get_project(conn: &Connection, id: &str) -> Result<Project, String> {
    conn.query_row(
        &format!("{SELECT_PROJECT} WHERE id = ?1"),
        [id],
        map_project,
    )
    .optional()
    .map_err(to_err)?
    .ok_or_else(|| "项目不存在".to_string())
}

pub fn insert_project(conn: &Connection, name: &str, color: &str) -> Result<Project, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("项目名不能为空".into());
    }
    let duplicated: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM projects WHERE name = ?1",
            [name],
            |row| row.get(0),
        )
        .map_err(to_err)?;
    if duplicated > 0 {
        return Err("同名项目已存在".into());
    }
    let id = Uuid::new_v4().to_string();
    let stamp = now_stamp();
    conn.execute(
        "INSERT INTO projects (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, color, stamp],
    )
    .map_err(to_err)?;
    Ok(Project {
        id,
        name: name.to_string(),
        color: color.to_string(),
        created_at: stamp,
    })
}

pub fn update_project(
    conn: &Connection,
    id: &str,
    name: &str,
    color: &str,
) -> Result<Project, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("项目名不能为空".into());
    }
    let duplicated: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM projects WHERE name = ?1 AND id != ?2",
            params![name, id],
            |row| row.get(0),
        )
        .map_err(to_err)?;
    if duplicated > 0 {
        return Err("同名项目已存在".into());
    }
    let changed = conn
        .execute(
            "UPDATE projects SET name = ?1, color = ?2 WHERE id = ?3",
            params![name, color, id],
        )
        .map_err(to_err)?;
    if changed == 0 {
        return Err("项目不存在".into());
    }
    get_project(conn, id)
}

/// 事务包裹：解绑任务与删除项目要么同时生效，要么都不生效
pub fn delete_project(conn: &mut Connection, id: &str) -> Result<(), String> {
    get_project(conn, id)?;
    let tx = conn.transaction().map_err(to_err)?;
    tx.execute(
        "UPDATE tasks SET project_id = NULL WHERE project_id = ?1",
        [id],
    )
    .map_err(to_err)?;
    tx.execute("DELETE FROM projects WHERE id = ?1", [id])
        .map_err(to_err)?;
    tx.commit().map_err(to_err)
}

fn normalize_priority(value: &str) -> String {
    match value {
        "low" | "high" | "normal" => value.to_string(),
        _ => "normal".into(),
    }
}

fn normalize_kind(value: &str) -> String {
    match value {
        "detailed" => "detailed".into(),
        _ => "quick".into(),
    }
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BackupPayload, BackupProject, BackupTask};

    fn memory() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        conn
    }

    #[test]
    fn create_complete_and_delete_task() {
        let conn = memory();
        let task = insert(
            &conn,
            NewTask {
                title: "买火车票".into(),
                notes: None,
                priority: Some("high".into()),
                kind: Some("quick".into()),
                start_at: Some("2026-09-09T09:00:00".into()),
                due_at: Some("2026-09-09T18:40:00".into()),
                project_id: None,
            },
        )
        .unwrap();

        assert_eq!(list(&conn).unwrap().len(), 1);
        assert_eq!(complete(&conn, &task.id).unwrap().status, "completed");
        delete(&conn, &task.id).unwrap();
        assert!(list(&conn).unwrap().is_empty());
    }

    #[test]
    fn insert_assigns_increasing_sort_order() {
        let mut conn = memory();
        let new = |title: &str| NewTask {
            title: title.into(),
            notes: None,
            priority: None,
            kind: None,
            start_at: None,
            due_at: None,
            project_id: None,
        };
        let first = insert(&conn, new("甲")).unwrap();
        let second = insert(&conn, new("乙")).unwrap();
        assert!(second.sort_order > first.sort_order);

        reorder(&mut conn, &[second.id.clone(), first.id.clone()]).unwrap();
        let reloaded = list(&conn).unwrap();
        assert_eq!(reloaded[0].id, second.id);
    }

    #[test]
    fn reject_empty_title() {
        let conn = memory();
        let err = insert(
            &conn,
            NewTask {
                title: "   ".into(),
                notes: None,
                priority: None,
                kind: None,
                start_at: None,
                due_at: None,
                project_id: None,
            },
        )
        .unwrap_err();
        assert_eq!(err, "标题不能为空");
    }

    #[test]
    fn project_lifecycle_unassigns_tasks() {
        let mut conn = memory();
        let project = insert_project(&conn, "工作", "#3D5BDB").unwrap();
        assert_eq!(list_projects(&conn).unwrap().len(), 1);

        let task = insert(
            &conn,
            NewTask {
                title: "写周报".into(),
                notes: None,
                priority: None,
                kind: None,
                start_at: None,
                due_at: None,
                project_id: Some(project.id.clone()),
            },
        )
        .unwrap();
        assert_eq!(task.project_id.as_deref(), Some(project.id.as_str()));

        assert_eq!(
            insert_project(&conn, "工作", "#3D5BDB").unwrap_err(),
            "同名项目已存在"
        );

        delete_project(&mut conn, &project.id).unwrap();
        assert!(list_projects(&conn).unwrap().is_empty());
        assert_eq!(get(&conn, &task.id).unwrap().project_id, None);
    }

    fn bare_new(title: &str) -> NewTask {
        NewTask {
            title: title.into(),
            notes: None,
            priority: None,
            kind: None,
            start_at: None,
            due_at: None,
            project_id: None,
        }
    }

    #[test]
    fn seeds_only_on_fresh_database() {
        let dir = std::env::temp_dir().join(format!("todo-db-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("todo.db");

        let conn = open(&db_path).unwrap();
        assert!(!list(&conn).unwrap().is_empty());
        for task in list(&conn).unwrap() {
            delete(&conn, &task.id).unwrap();
        }
        assert!(list(&conn).unwrap().is_empty());
        drop(conn);

        // 用户清空任务后重启，示例数据不应复活
        let reopened = open(&db_path).unwrap();
        assert!(list(&reopened).unwrap().is_empty());
        drop(reopened);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn reject_invalid_status() {
        let conn = memory();
        let task = insert(&conn, bare_new("任务")).unwrap();
        let err = update(
            &conn,
            &task.id,
            TaskPatch {
                status: Some("archive".into()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err, "无效的任务状态");
    }

    #[test]
    fn recomplete_preserves_completed_at() {
        let conn = memory();
        let task = insert(&conn, bare_new("任务")).unwrap();
        let first = complete(&conn, &task.id).unwrap();
        let first_at = first.completed_at.clone().unwrap();
        let second = complete(&conn, &task.id).unwrap();
        assert_eq!(second.completed_at.unwrap(), first_at);

        let reopened = update(
            &conn,
            &task.id,
            TaskPatch {
                status: Some("pending".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(reopened.completed_at, None);
    }

    #[test]
    fn reject_unknown_project() {
        let conn = memory();
        let err = insert(
            &conn,
            NewTask {
                project_id: Some("not-a-project".into()),
                ..bare_new("任务")
            },
        )
        .unwrap_err();
        assert_eq!(err, "项目不存在");

        let project = insert_project(&conn, "工作", "#3D5BDB").unwrap();
        let task = insert(&conn, bare_new("任务")).unwrap();
        let err = update(
            &conn,
            &task.id,
            TaskPatch {
                project_id: Some("not-a-project".into()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err, "项目不存在");
        let assigned = update(
            &conn,
            &task.id,
            TaskPatch {
                project_id: Some(project.id.clone()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(assigned.project_id, Some(project.id));
    }

    #[test]
    fn reschedule_and_reopen_rearm_reminders() {
        let conn = memory();
        let task = insert(
            &conn,
            NewTask {
                due_at: Some("2026-09-09T18:40:00".into()),
                ..bare_new("任务")
            },
        )
        .unwrap();
        mark_notified(&conn, &task.id).unwrap();
        assert!(get(&conn, &task.id).unwrap().notified);

        // 改期后重新进入提醒流程
        let rescheduled = update(
            &conn,
            &task.id,
            TaskPatch {
                due_at: Some("2026-12-01T09:00:00".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!rescheduled.notified);

        // 截止时间未变时保留已提醒状态
        mark_notified(&conn, &task.id).unwrap();
        let untouched = update(
            &conn,
            &task.id,
            TaskPatch {
                title: Some("改名".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(untouched.notified);

        // 重开同样重置提醒
        complete(&conn, &task.id).unwrap();
        let reopened = update(
            &conn,
            &task.id,
            TaskPatch {
                status: Some("pending".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!reopened.notified);
    }

    #[test]
    fn reject_invalid_date_format() {
        let conn = memory();
        let err = insert(
            &conn,
            NewTask {
                due_at: Some("明天下午".into()),
                ..bare_new("任务")
            },
        )
        .unwrap_err();
        assert!(err.contains("无效的日期格式"));

        let task = insert(&conn, bare_new("任务")).unwrap();
        let err = update(
            &conn,
            &task.id,
            TaskPatch {
                start_at: Some("2026/09/09 10:00".into()),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(err.contains("无效的日期格式"));

        // 空串（清除字段）与空格分隔的合法格式都应通过
        update(
            &conn,
            &task.id,
            TaskPatch {
                start_at: Some("".into()),
                due_at: Some("2026-09-09 10:00:00".into()),
                ..Default::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn mark_notified_unknown_id_fails() {
        let conn = memory();
        assert_eq!(
            mark_notified(&conn, "no-such-id").unwrap_err(),
            "任务不存在"
        );
    }

    #[test]
    fn in_progress_status_roundtrip() {
        let conn = memory();
        let task = insert(
            &conn,
            NewTask {
                due_at: Some("2026-09-09T18:40:00".into()),
                ..bare_new("任务")
            },
        )
        .unwrap();

        // pending -> in_progress：清除完成时间
        let started = update(
            &conn,
            &task.id,
            TaskPatch {
                status: Some("in_progress".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(started.status, "in_progress");
        assert_eq!(started.completed_at, None);

        // in_progress -> completed
        let done = complete(&conn, &task.id).unwrap();
        assert_eq!(done.status, "completed");
        assert!(done.completed_at.is_some());

        // completed -> in_progress：视为重开，重置提醒
        mark_notified(&conn, &task.id).unwrap();
        let restarted = update(
            &conn,
            &task.id,
            TaskPatch {
                status: Some("in_progress".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(restarted.status, "in_progress");
        assert_eq!(restarted.completed_at, None);
        assert!(!restarted.notified);
    }

    #[test]
    fn migrates_sort_order_for_legacy_db() {
        let dir = std::env::temp_dir().join(format!("todo-legacy-db-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("todo.db");
        {
            // v0.2 之前的 tasks 表：没有 sort_order 列
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE tasks (
                   id TEXT PRIMARY KEY,
                   title TEXT NOT NULL,
                   notes TEXT,
                   status TEXT NOT NULL DEFAULT 'pending',
                   priority TEXT NOT NULL DEFAULT 'normal',
                   kind TEXT NOT NULL DEFAULT 'quick',
                   start_at TEXT,
                   due_at TEXT,
                   completed_at TEXT,
                   project_id TEXT,
                   notified INTEGER NOT NULL DEFAULT 0,
                   created_at TEXT NOT NULL,
                   updated_at TEXT NOT NULL);
                 INSERT INTO tasks (id, title, status, created_at, updated_at) VALUES
                   ('a', '甲', 'pending', '2026-01-01T08:00:00', '2026-01-01T08:00:00'),
                   ('b', '乙', 'pending', '2026-01-01T09:00:00', '2026-01-01T09:00:00'),
                   ('c', '丙', 'pending', '2026-01-01T10:00:00', '2026-01-01T10:00:00');",
            )
            .unwrap();
        }
        let conn = open(&db_path).unwrap();
        let rows = list(&conn).unwrap();
        assert_eq!(rows.len(), 3);
        let order_of = |id: &str| {
            rows.iter()
                .find(|task| task.id == id)
                .map(|task| task.sort_order)
                .unwrap()
        };
        assert!(order_of("a") < order_of("b"));
        assert!(order_of("b") < order_of("c"));
        drop(conn);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn import_replace_replaces_all_and_validates() {
        let mut conn = memory();
        insert_project(&conn, "旧项目", "#123456").unwrap();
        insert(&conn, bare_new("旧任务")).unwrap();

        let project_id = Uuid::new_v4().to_string();
        let payload = BackupPayload {
            version: Some(1),
            projects: vec![BackupProject {
                id: Some(project_id.clone()),
                name: "新项目".into(),
                color: Some("#EC4899".into()),
                created_at: None,
            }],
            tasks: vec![
                BackupTask {
                    id: Some("task-1".into()),
                    title: "进行中任务".into(),
                    notes: None,
                    status: Some("in_progress".into()),
                    priority: Some("high".into()),
                    kind: None,
                    start_at: Some("2026-09-09T09:00:00".into()),
                    due_at: Some("2026-09-10T18:00:00".into()),
                    completed_at: None,
                    project_id: Some(project_id.clone()),
                    notified: false,
                    sort_order: 1,
                    created_at: Some("2026-09-09T08:00:00".into()),
                    updated_at: None,
                },
                BackupTask {
                    id: None,
                    title: "已完成任务".into(),
                    notes: None,
                    status: Some("completed".into()),
                    priority: None,
                    kind: None,
                    start_at: None,
                    due_at: None,
                    completed_at: Some("2026-09-08T10:00:00".into()),
                    project_id: None,
                    notified: true,
                    sort_order: 2,
                    created_at: None,
                    updated_at: None,
                },
            ],
        };

        let (tasks, projects) = import_replace(&mut conn, payload).unwrap();
        assert_eq!((tasks, projects), (2, 1));

        let rows = list(&conn).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(!rows.iter().any(|task| task.title == "旧任务"));
        assert!(!list_projects(&conn).unwrap().iter().any(|p| p.name == "旧项目"));

        let in_progress = rows.iter().find(|task| task.id == "task-1").unwrap();
        assert_eq!(in_progress.status, "in_progress");
        assert_eq!(in_progress.project_id.as_deref(), Some(project_id.as_str()));

        let done = rows.iter().find(|task| task.title == "已完成任务").unwrap();
        assert_eq!(done.status, "completed");
        assert!(done.completed_at.is_some());
        assert_ne!(done.id, ""); // 缺失 ID 时自动补 UUID

        // 非法引用整体拒绝：库里保留原数据
        let bad = BackupPayload {
            version: Some(1),
            projects: vec![],
            tasks: vec![BackupTask {
                id: None,
                title: "孤儿任务".into(),
                notes: None,
                status: None,
                priority: None,
                kind: None,
                start_at: None,
                due_at: None,
                completed_at: None,
                project_id: Some("ghost".into()),
                notified: false,
                sort_order: 0,
                created_at: None,
                updated_at: None,
            }],
        };
        assert!(import_replace(&mut conn, bad).is_err());
        assert_eq!(list(&conn).unwrap().len(), 2);
        assert_eq!(list_projects(&conn).unwrap().len(), 1);

        // 版本号不支持时拒绝
        let future = BackupPayload {
            version: Some(2),
            projects: vec![],
            tasks: vec![],
        };
        assert!(import_replace(&mut conn, future).is_err());
    }
}
