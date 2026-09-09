<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import {
  CalendarDays,
  CalendarRange,
  CheckSquare,
  GanttChart,
  Pin,
  Settings,
  SquareKanban,
} from "lucide-vue-next";
import type { AppView } from "../types";
import { useTile } from "../composables/useTile";
import { t } from "../i18n";

defineProps<{
  current: AppView;
  todayCount: number;
  allCount: number;
  projectCount: number;
}>();

const emit = defineEmits<{
  navigate: [view: AppView];
}>();

const { tileOpen, init, toggle } = useTile();
// 磁贴开关失败用本地错误条展示：不清除全局共享的错误横幅
const tileError = ref("");

let bannerTimer = 0;
let pending = false;

onMounted(() => {
  void init();
});

onUnmounted(() => {
  window.clearTimeout(bannerTimer);
});

// 左下角按钮直接开关磁贴；失败几秒后自动清除
async function toggleTile() {
  if (pending) return;
  pending = true;
  try {
    await toggle();
  } catch (err) {
    tileError.value = err instanceof Error ? err.message : String(err);
    window.clearTimeout(bannerTimer);
    bannerTimer = window.setTimeout(() => {
      tileError.value = "";
    }, 4000);
  } finally {
    pending = false;
  }
}

const items: { id: AppView; label: string; icon: typeof CalendarDays; count?: "today" | "all" | "projects" }[] = [
  { id: "today", label: t("nav.today"), icon: CalendarRange, count: "today" },
  { id: "all", label: t("nav.all"), icon: CheckSquare, count: "all" },
  { id: "calendar", label: t("nav.calendar"), icon: CalendarDays },
  { id: "gantt", label: t("nav.gantt"), icon: GanttChart },
  { id: "projects", label: t("nav.projects"), icon: SquareKanban, count: "projects" },
  { id: "settings", label: t("nav.settings"), icon: Settings },
];
</script>

<template>
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-mark">
        <CheckSquare :size="16" />
      </div>
      <div class="brand-name">Todo</div>
    </div>

    <nav class="nav-list">
      <button
        v-for="item in items"
        :key="item.id"
        class="nav-item"
        :class="{ active: current === item.id }"
        :aria-current="current === item.id ? 'page' : undefined"
        type="button"
        @click="emit('navigate', item.id)"
      >
        <component :is="item.icon" :size="17" />
        <span>{{ item.label }}</span>
        <b
          v-if="item.count === 'today' && todayCount"
          class="count"
        >{{ todayCount }}</b>
        <b
          v-else-if="item.count === 'all' && allCount"
          class="count"
        >{{ allCount }}</b>
        <b
          v-else-if="item.count === 'projects' && projectCount"
          class="count"
        >{{ projectCount }}</b>
      </button>
    </nav>

    <div class="sidebar-footer">
      <p v-if="tileError" class="error-banner" role="alert">{{ tileError }}</p>
      <button
        class="pin-btn"
        :class="{ active: tileOpen }"
        type="button"
        :title="tileOpen ? t('nav.tileOn') : t('nav.tileOff')"
        @click="toggleTile"
      >
        <Pin :size="16" />
        <span>{{ tileOpen ? t("nav.tileOn") : t("nav.tileOff") }}</span>
      </button>
    </div>
  </aside>
</template>
