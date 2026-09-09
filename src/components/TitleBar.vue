<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Search, Square, X } from "lucide-vue-next";
import { t } from "../i18n";

const query = defineModel<string>({ default: "" });
const inputRef = ref<HTMLInputElement | null>(null);
const isMaximized = ref(false);

// 无边框窗口：最小化/最大化/关闭都走应用内按钮；
// 关闭与系统标题栏同路径，仍会触发 close-to-tray 拦截
const win = getCurrentWindow();
let unlisten: (() => void) | null = null;

onMounted(async () => {
  isMaximized.value = await win.isMaximized().catch(() => false);
  // 最大化状态随拖拽/双击/系统快捷键变化，统一在 resize 事件里校正
  unlisten = await win.onResized(async () => {
    isMaximized.value = await win.isMaximized().catch(() => isMaximized.value);
  });
});

onUnmounted(() => {
  unlisten?.();
});

function focusSearch() {
  inputRef.value?.focus();
  inputRef.value?.select();
}

defineExpose({ focusSearch });
</script>

<template>
  <header class="titlebar" data-tauri-drag-region>
    <div class="titlebar-search">
      <Search class="lead-ico" :size="14" />
      <input
        ref="inputRef"
        :value="query"
        type="text"
        :placeholder="t('titlebar.search')"
        @input="query = ($event.target as HTMLInputElement).value"
      />
      <button
        v-if="query"
        class="icon-btn clear-btn"
        type="button"
        @click="query = ''"
      >
        <X :size="12" />
      </button>
    </div>
    <div class="titlebar-controls">
      <button class="tbtn" type="button" :title="t('titlebar.minimize')" @click="win.minimize()">
        <Minus :size="15" />
      </button>
      <button
        class="tbtn"
        type="button"
        :title="isMaximized ? t('titlebar.restore') : t('titlebar.maximize')"
        @click="win.toggleMaximize()"
      >
        <Copy v-if="isMaximized" :size="12" />
        <Square v-else :size="12" />
      </button>
      <button class="tbtn close" type="button" :title="t('titlebar.close')" @click="win.close()">
        <X :size="15" />
      </button>
    </div>
  </header>
</template>
