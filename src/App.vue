<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Sidebar from "./components/Sidebar.vue";
import TodayView from "./components/TodayView.vue";
import AllTasksView from "./components/AllTasksView.vue";
import CalendarView from "./components/CalendarView.vue";
import GanttView from "./components/GanttView.vue";
import ProjectsView from "./components/ProjectsView.vue";
import SettingsView from "./components/SettingsView.vue";
import TileApp from "./components/TileApp.vue";
import { useClock } from "./composables/useClock";
import { useProjects } from "./composables/useProjects";
import { useReminders } from "./composables/useReminders";
import { useSettings } from "./composables/useSettings";
import { useTasks } from "./composables/useTasks";
import type { AppView, TaskKind } from "./types";
import { toStamp } from "./utils/datetime";

// 置顶磁贴是第二个窗口，加载同一份前端，按窗口 label 切换 UI
const isTile = getCurrentWindow().label === "tile";

const {
  tasks,
  error,
  selected,
  pending,
  todayTasks,
  todayCount,
  create,
  update,
  complete,
  reopen,
  remove,
  markNotifiedMany,
  reorder,
  select,
  refresh,
} = useTasks();

const { projects, load: loadProjects } = useProjects();
const { settings, load: loadSettings } = useSettings();

if (!isTile) {
  useReminders(tasks, markNotifiedMany, computed(() => settings.value.notificationLeadMinutes));
}

const view = ref<AppView>("today");
const activeProjectId = ref<string | null>(null);
const now = useClock();
const allView = ref<InstanceType<typeof AllTasksView> | null>(null);

onMounted(() => {
  window.addEventListener("keydown", onGlobalKeydown);
  if (!isTile) {
    void refresh();
    void loadProjects();
    void loadSettings();
  }
});

onUnmounted(() => {
  window.removeEventListener("keydown", onGlobalKeydown);
});

// Ctrl+K 全局搜索入口：任何视图都能跳到「全部任务」并聚焦搜索框
function onGlobalKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    navigate("all");
    void nextTick(() => allView.value?.focusSearch());
  }
}

async function createQuick(payload: { title: string; kind: TaskKind; dueAt: string }) {
  await create({
    title: payload.title,
    kind: payload.kind,
    priority: payload.kind === "detailed" ? "high" : "normal",
    startAt: toStamp(new Date()),
    dueAt: payload.dueAt,
  });
}

function navigate(next: AppView) {
  view.value = next;
  if (next !== "all") activeProjectId.value = null;
}

function openProject(id: string) {
  activeProjectId.value = id;
  view.value = "all";
}
</script>

<template>
  <TileApp v-if="isTile" />

  <div v-else class="app-shell">
    <Sidebar
      :current="view"
      :today-count="todayCount"
      :all-count="pending.length"
      :project-count="projects.length"
      @navigate="navigate"
    />

    <div class="content-col">
      <div v-if="error" class="error-banner" role="alert">{{ error }}</div>

      <TodayView
        v-if="view === 'today'"
        :now="now"
        :tasks="todayTasks"
        :selected="selected"
        :all-pending="pending.length"
        :on-create="createQuick"
        @select="select"
        @complete="complete"
        @reopen="reopen"
        @remove="remove"
        @save="update"
      />

      <AllTasksView
        v-else-if="view === 'all'"
        ref="allView"
        :tasks="tasks"
        :selected="selected"
        :project-id="activeProjectId"
        @select="select"
        @complete="complete"
        @reopen="reopen"
        @remove="remove"
        @save="update"
        @clear-project="activeProjectId = null"
        @reorder="reorder"
      />

      <CalendarView
        v-else-if="view === 'calendar'"
        :now="now"
        :tasks="tasks"
      />

      <GanttView
        v-else-if="view === 'gantt'"
        :now="now"
        :tasks="tasks"
      />

      <ProjectsView
        v-else-if="view === 'projects'"
        :tasks="tasks"
        @open="openProject"
      />

      <SettingsView v-else />
    </div>
  </div>
</template>
