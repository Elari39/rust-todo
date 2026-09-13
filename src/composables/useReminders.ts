import { onUnmounted, watch, type Ref } from "vue";
import {
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";
import { api } from "../api";
import type { ReminderConfirmation, Task } from "../types";
import { parseStamp } from "../utils/datetime";
import { isDueForNotification } from "../utils/reminders";
import { t, tf } from "../i18n";

const SCAN_INTERVAL_MS = 30_000;

export function useReminders(
  tasks: Ref<Task[]>,
  markNotifiedMany: (confirmations: ReminderConfirmation[]) => Promise<void>,
  leadMinutes: Ref<number>,
) {
  let timer: number | null = null;
  let permissionGranted = false;
  let scanning = false;

  async function ensurePermission(): Promise<boolean> {
    if (permissionGranted) return true;
    try {
      // 已授权直接通过；未授权则尝试申请。false 不缓存：
      // 用户事后在系统设置里放行，下一轮扫描即可恢复，无需重启应用
      permissionGranted =
        (await isPermissionGranted()) || (await requestPermission()) === "granted";
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
      const nowMs = Date.now();
      const leadMs = Math.max(0, leadMinutes.value) * 60_000;
      const confirmations: ReminderConfirmation[] = [];
      for (const task of tasks.value) {
        if (task.status === "completed" || task.notified) continue;
        const due = parseStamp(task.dueAt);
        const dueMs = due?.getTime() ?? null;
        if (due === null || !isDueForNotification(dueMs, nowMs, leadMs)) continue;
        const overdue = due.getTime() < nowMs;
        // 在 await 前保存代次，不能读取发送期间可能已被修改的任务对象。
        const confirmation = { id: task.id, reminderToken: task.reminderToken };
        try {
          await api.sendNotification({
            title: overdue ? t("notify.overdueTitle") : t("notify.dueSoonTitle"),
            body: `${task.title} · ${overdue ? t("notify.overdueBody") : tf("notify.soonBody", { n: leadMinutes.value })}`,
          });
          confirmations.push(confirmation);
        } catch {
          // 单个任务通知失败不中断本轮，等待下一轮扫描重试
        }
      }
      if (confirmations.length) await markNotifiedMany(confirmations);
    } catch {
      // 提醒失败不应产生未处理异常；下一轮扫描会重试
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
