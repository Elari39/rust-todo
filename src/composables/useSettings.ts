import { ref } from "vue";
import { api } from "../api";
import type { AppSettings } from "../types";
import { setLocale, type Locale } from "../i18n";

// 模块级单例：设置在主窗口内全局共享
const settings = ref<AppSettings>({
  notificationLeadMinutes: 15,
  closeToTray: true,
  locale: "zh-CN",
});
const loaded = ref(false);
const loadError = ref("");
let saveQueue: Promise<void> = Promise.resolve();

export function useSettings() {
  async function load(force = false) {
    if (loaded.value && !force) return;
    try {
      settings.value = await api.getSettings();
      loaded.value = true;
      loadError.value = "";
      if (settings.value.locale === "zh-CN") setLocale(settings.value.locale as Locale);
    } catch (err) {
      // 读不到就用默认值，设置页保存时会重试；但失败原因要让用户可见
      loadError.value = err instanceof Error ? err.message : String(err);
    }
  }

  function save(patch: Partial<AppSettings>) {
    const changes = { ...patch };
    // 调用方只提交本次修改；轮到本次写入时再合并最新状态，避免捕获旧快照。
    const pending = saveQueue.then(async () => {
      const next = { ...settings.value, ...changes };
      const saved = await api.saveSettings({
        closeToTray: next.closeToTray,
        notificationLeadMinutes: Math.max(0, Math.min(720, Math.round(next.notificationLeadMinutes || 0))),
        locale: next.locale || "zh-CN",
      });
      settings.value = saved;
      setLocale(saved.locale === "zh-CN" ? saved.locale : "zh-CN");
      return saved;
    });
    // 失败仍向当前调用方抛出，但不能阻塞队列中后续的设置保存。
    saveQueue = pending.then(() => {}, () => {});
    return pending;
  }

  return { settings, loaded, loadError, load, save };
}
