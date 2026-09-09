import { ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { api } from "../api";

// 模块级单例：侧边栏与设置页共享磁贴开关状态
const tileOpen = ref(false);
let initialized = false;

async function init() {
  if (initialized) return;
  initialized = true;
  tileOpen.value = await api.tileState().catch(() => false);
  // 磁贴还能从托盘菜单或磁贴自身的关闭按钮开关，窗口聚焦时同步真实状态；
  // 监听随窗口存活，无需退订
  await listen("tauri://focus", async () => {
    tileOpen.value = await api.tileState().catch(() => false);
  });
}

/** 开关磁贴，返回操作后的开关状态；失败时抛出由调用方提示 */
async function toggle(): Promise<boolean> {
  tileOpen.value = await api.toggleTileWindow();
  return tileOpen.value;
}

export function useTile() {
  return { tileOpen, init, toggle };
}
