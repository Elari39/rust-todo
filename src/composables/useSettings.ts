import { ref } from "vue";
import { api } from "../api";
import type { AppSettings } from "../types";
import { setLocale } from "../i18n";

// 模块级单例：设置在主窗口内全局共享
const settings = ref<AppSettings>({
  notificationLeadMinutes: 15,
  closeToTray: true,
  locale: "zh-CN",
});
const loaded = ref(false);
const loadError = ref("");
let operationQueue: Promise<void> = Promise.resolve();

function enqueue<T>(run: () => Promise<T>): Promise<T> {
  const pending = operationQueue.then(run);
  // 失败只影响当前调用，不阻塞后续的读取、保存和重试。
  operationQueue = pending.then(() => {}, () => {});
  return pending;
}

async function readSettings() {
  try {
    settings.value = await api.getSettings();
    loaded.value = true;
    loadError.value = "";
    setLocale(settings.value.locale === "zh-CN" ? settings.value.locale : "zh-CN");
  } catch (err) {
    loadError.value = err instanceof Error ? err.message : String(err);
    throw err;
  }
}

export function useSettings() {
  function load(force = false) {
    return enqueue(async () => {
      if (loaded.value && !force) return;
      try {
        await readSettings();
      } catch {
        // 首次读取失败时允许展示默认值，但保存前必须重试并读到真实设置。
      }
    });
  }

  function save(patch: Partial<AppSettings>) {
    const changes = { ...patch };
    // 调用方只提交本次修改；轮到本次写入时再合并最新状态，避免捕获旧快照。
    return enqueue(async () => {
      if (!loaded.value) await readSettings();
      const next = { ...settings.value, ...changes };
      const saved = await api.saveSettings({
        closeToTray: next.closeToTray,
        notificationLeadMinutes: Math.max(0, Math.min(720, Math.round(next.notificationLeadMinutes || 0))),
        locale: next.locale || "zh-CN",
      });
      settings.value = saved;
      loadError.value = "";
      setLocale(saved.locale === "zh-CN" ? saved.locale : "zh-CN");
      return saved;
    });
  }

  return { settings, loaded, loadError, load, save };
}
