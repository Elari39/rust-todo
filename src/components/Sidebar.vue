<script setup lang="ts">
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

defineProps<{
  current: AppView;
  todayCount: number;
  allCount: number;
  projectCount: number;
  lastSync: string;
}>();

const emit = defineEmits<{
  navigate: [view: AppView];
  pin: [];
}>();

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
      <button class="pin-btn" type="button" @click="emit('pin')">
        <Pin :size="16" />
        <span>置顶磁贴模式</span>
      </button>
      <div class="status-card">
        <CheckSquare :size="18" color="#16a34a" />
        <div>
          <p><span class="status-dot" /> 在线</p>
          <small>最近同步：{{ lastSync }}</small>
        </div>
      </div>
    </div>
  </aside>
</template>
