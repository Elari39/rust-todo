import { onUnmounted, watch, type Ref } from "vue";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import type { Task } from "../types";
import { parseStamp } from "../utils/datetime";

const CATCHUP_MS = 24 * 60 * 60 * 1000;
const SCAN_INTERVAL_MS = 30_000;

export function useReminders(
  tasks: Ref<Task[]>,
  markNotified: (id: string) => Promise<void>,
  leadMinutes: Ref<number>,
) {
  let timer: number | null = null;
  let permissionGranted: boolean | null = null;
  let scanning = false;

  async function ensurePermission(): Promise<boolean> {
    if (permissionGranted !== null) return permissionGranted;
    try {
      if (await isPermissionGranted()) {
        permissionGranted = true;
        return true;
      }
      permissionGranted = (await requestPermission()) === "granted";
      return permissionGranted;
    } catch {
      return false;
    }
  }

  async function scan() {
    // 定时器与 watch 触发可能并发，加互斥防止同一任务重复发通知
    if (scanning) return;
    scanning = true;
    try {
      if (!(await ensurePermission())) return;
      const now = Date.now();
      const leadMs = Math.max(0, leadMinutes.value) * 60_000;
      for (const task of tasks.value) {
        if (task.status === "completed" || task.notified) continue;
        const due = parseStamp(task.dueAt);
        if (!due) continue;
        const dueMs = due.getTime();
        if (dueMs - leadMs <= now && dueMs + CATCHUP_MS >= now) {
          const overdue = dueMs < now;
          await sendNotification({
            title: overdue ? "任务已到期" : "任务即将到期",
            body: `${task.title} · ${overdue ? "已经过截止时间" : `${leadMinutes.value} 分钟内到期`}`,
          });
          await markNotified(task.id);
        }
      }
    } finally {
      scanning = false;
    }
  }

  timer = window.setInterval(() => {
    void scan();
  }, SCAN_INTERVAL_MS);
  void scan();

  onUnmounted(() => {
    if (timer) window.clearInterval(timer);
  });
  watch(tasks, () => {
    void scan();
  });
}
