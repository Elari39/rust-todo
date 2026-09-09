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

  async function save(next: AppSettings) {
    const saved = await api.saveSettings({
      closeToTray: next.closeToTray,
      notificationLeadMinutes: Math.max(0, Math.min(720, Math.round(next.notificationLeadMinutes || 0))),
      locale: next.locale || "zh-CN",
    });
    settings.value = saved;
    setLocale(saved.locale === "zh-CN" ? saved.locale : "zh-CN");
    return saved;
  }

  return { settings, loaded, loadError, load, save };
}
