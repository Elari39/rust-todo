<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import {
  CalendarDays,
  CheckSquare,
  ChevronLeft,
  GanttChart,
  Pin,
  Settings,
  SquareKanban,
} from "lucide-vue-next";
import type { AppView } from "../types";
import { useTile } from "../composables/useTile";

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
  { id: "today", label: "今日", icon: CalendarDays, count: "today" },
  { id: "all", label: "全部任务", icon: CheckSquare, count: "all" },
  { id: "calendar", label: "日历", icon: CalendarDays },
  { id: "gantt", label: "甘特图", icon: GanttChart },
  { id: "projects", label: "项目跟踪", icon: SquareKanban, count: "projects" },
  { id: "settings", label: "设置", icon: Settings },
];
</script>

<template>
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-mark">
        <CheckSquare :size="16" />
      </div>
      <div class="brand-name">Todo</div>
      <ChevronLeft :size="16" style="margin-left: auto; color: #94a3b8" />
    </div>

    <div class="nav-label">工作台</div>
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
        <component :is="item.icon" :size="16" />
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
        :title="tileOpen ? '关闭置顶磁贴' : '打开置顶磁贴'"
        @click="toggleTile"
      >
        <Pin :size="16" />
        <span>{{ tileOpen ? "关闭置顶磁贴" : "打开置顶磁贴" }}</span>
      </button>
    </div>
  </aside>
</template>
