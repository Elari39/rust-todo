export type TaskStatus = "pending" | "in_progress" | "completed";
export type TaskPriority = "low" | "normal" | "high";
export type TaskKind = "quick" | "detailed";
export type AppView =
  | "today"
  | "all"
  | "calendar"
  | "gantt"
  | "projects"
  | "settings";

export interface Task {
  id: string;
  title: string;
  notes: string | null;
  status: TaskStatus;
  priority: TaskPriority;
  kind: TaskKind;
  startAt: string | null;
  dueAt: string | null;
  completedAt: string | null;
  projectId: string | null;
  notified: boolean;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface Project {
  id: string;
  name: string;
  color: string;
  createdAt: string;
}

export interface AppSettings {
  notificationLeadMinutes: number;
  closeToTray: boolean;
  locale: string;
}

export interface NewTask {
  title: string;
  notes?: string | null;
  priority?: TaskPriority;
  kind?: TaskKind;
  startAt?: string | null;
  dueAt?: string | null;
  projectId?: string | null;
}

/** 可空字段：省略表示保持原值，null 表示清空。 */
export interface TaskPatch {
  title?: string;
  notes?: string | null;
  priority?: TaskPriority;
  kind?: TaskKind;
  startAt?: string | null;
  dueAt?: string | null;
  status?: TaskStatus;
  /** null 或空字符串表示移出项目 */
  projectId?: string | null;
}
