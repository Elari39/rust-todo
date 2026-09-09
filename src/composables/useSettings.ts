import { ref } from "vue";
import { api } from "../api";
import type { AppSettings } from "../types";

// 模块级单例：设置在主窗口内全局共享
const settings = ref<AppSettings>({ notificationLeadMinutes: 15, closeToTray: true });
const loaded = ref(false);

export function useSettings() {
  async function load(force = false) {
    if (loaded.value && !force) return;
    try {
      settings.value = await api.getSettings();
      loaded.value = true;
    } catch {
      // 读不到就用默认值，设置页保存时会重试
    }
  }

  async function save(next: AppSettings) {
    const saved = await api.saveSettings({
      closeToTray: next.closeToTray,
      notificationLeadMinutes: Math.max(0, Math.min(720, Math.round(next.notificationLeadMinutes || 0))),
    });
    settings.value = saved;
    return saved;
  }

  return { settings, loaded, load, save };
}
