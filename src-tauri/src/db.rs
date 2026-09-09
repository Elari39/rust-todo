use crate::model::{NewTask, Project, Task, TaskPatch};
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
    let conn = Connection::open(path).map_err(to_err)?;
    let projects_fresh = !table_exists(&conn, "projects")?;
    conn.execute_batch(SCHEMA).map_err(to_err)?;
    ensure_sort_order(&conn)?;
    if projects_fresh {
        seed_projects(&conn)?;
    }
    seed_tasks(&conn)?;
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
    for (name, color) in [("工作", "#3D5BDB"), ("生活", "#16A34A"), ("学习", "#F59E0B")] {
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
            input.notes,
            priority,
            kind,
            empty_to_none(input.start_at),
            empty_to_none(input.due_at),
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
        Some(value) if value.trim().is_empty() => None,
        Some(value) => Some(value),
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
    let start_at = match patch.start_at {
        Some(value) => empty_to_none(Some(value)),
        None => current.start_at,
    };
    let due_at = match patch.due_at {
        Some(value) => empty_to_none(Some(value)),
        None => current.due_at,
    };
    let project_id = match patch.project_id {
        Some(value) if value.trim().is_empty() => None,
        Some(value) => Some(value),
        None => current.project_id,
    };
    let (status, completed_at) = match patch.status.as_deref() {
        Some("completed") => ("completed".to_string(), Some(now_stamp())),
        Some("pending") => ("pending".to_string(), None),
        _ => (current.status, current.completed_at),
    };
    let stamp = now_stamp();
    conn.execute(
        "UPDATE tasks SET title=?1, notes=?2, priority=?3, kind=?4, start_at=?5, due_at=?6, status=?7, completed_at=?8, project_id=?9, updated_at=?10 WHERE id=?11",
        params![title, notes, priority, kind, start_at, due_at, status, completed_at, project_id, stamp, id],
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
    conn.execute("UPDATE tasks SET notified = 1 WHERE id = ?1", [id])
        .map_err(to_err)?;
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

pub fn delete_project(conn: &Connection, id: &str) -> Result<(), String> {
    get_project(conn, id)?;
    conn.execute(
        "UPDATE tasks SET project_id = NULL WHERE project_id = ?1",
        [id],
    )
    .map_err(to_err)?;
    conn.execute("DELETE FROM projects WHERE id = ?1", [id])
        .map_err(to_err)?;
    Ok(())
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
        let conn = memory();
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

        delete_project(&conn, &project.id).unwrap();
        assert!(list_projects(&conn).unwrap().is_empty());
        assert_eq!(get(&conn, &task.id).unwrap().project_id, None);
    }
}
