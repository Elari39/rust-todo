import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, NewTask, Project, Task, TaskPatch } from "./types";

export const api = {
  listTasks: () => invoke<Task[]>("list_tasks"),
  createTask: (input: NewTask) => invoke<Task>("create_task", { input }),
  updateTask: (id: string, patch: TaskPatch) =>
    invoke<Task>("update_task", { id, patch }),
  completeTask: (id: string) => invoke<Task>("complete_task", { id }),
  deleteTask: (id: string) => invoke<void>("delete_task", { id }),
  markNotified: (id: string, reminderToken: string) =>
    invoke<Task>("mark_notified", { id, reminderToken }),
  sendNotification: (options: { title: string; body: string }) =>
    invoke<void>("send_notification", options),
  reorderTasks: (orderedIds: string[]) =>
    invoke<void>("reorder_tasks", { orderedIds }),

  listProjects: () => invoke<Project[]>("list_projects"),
  createProject: (name: string, color: string) =>
    invoke<Project>("create_project", { name, color }),
  updateProject: (id: string, name: string, color: string) =>
    invoke<Project>("update_project", { id, name, color }),
  deleteProject: (id: string) => invoke<void>("delete_project", { id }),

  getSettings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) =>
    invoke<AppSettings>("save_settings", { settings }),

  toggleTileWindow: () => invoke<boolean>("toggle_tile_window"),
  tileState: () => invoke<boolean>("tile_state"),
  dataDir: () => invoke<string>("data_dir"),
  exportBackup: () => invoke<string>("export_backup"),
  importBackup: (path: string) => invoke<string>("import_backup", { path }),
};
