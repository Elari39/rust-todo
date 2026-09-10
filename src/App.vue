<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Sidebar from "./components/Sidebar.vue";
import TitleBar from "./components/TitleBar.vue";
import TodayView from "./components/TodayView.vue";
import AllTasksView from "./components/AllTasksView.vue";
import CalendarView from "./components/CalendarView.vue";
import GanttView from "./components/GanttView.vue";
import ProjectsView from "./components/ProjectsView.vue";
import SettingsView from "./components/SettingsView.vue";
import TileApp from "./components/TileApp.vue";
import TaskCreateModal from "./components/TaskCreateModal.vue";
import { useClock } from "./composables/useClock";
import { useProjects } from "./composables/useProjects";
import { useReminders } from "./composables/useReminders";
import { useSettings } from "./composables/useSettings";
import { useTasks } from "./composables/useTasks";
import type { AppView, TaskKind, TaskStatus } from "./types";
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
const titleBarRef = ref<InstanceType<typeof TitleBar> | null>(null);

// 标题栏全局搜索：输入即进入「全部任务」并按内容过滤。
// 搜索是全局的——先脱离遗留的项目过滤，否则结果会被悄悄收窄到单个项目
const searchQuery = ref("");
watch(searchQuery, (value) => {
  if (!value.trim()) return;
  activeProjectId.value = null;
  if (view.value !== "all") navigate("all");
});

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

// Ctrl+K 全局搜索入口：任何视图都能跳到「全部任务」并聚焦标题栏搜索框
function onGlobalKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    navigate("all");
    void nextTick(() => titleBarRef.value?.focusSearch());
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
  // 点侧栏「全部任务」或任意导航都算显式离开项目视图；
  // 项目过滤只经 openProject / AllTasksView 的 clear-project 设置
  activeProjectId.value = null;
}

function openProject(id: string) {
  activeProjectId.value = id;
  view.value = "all";
}

/** 任务详情里的状态流转：开始任务 / 标记未开始。
 * 失败已写入全局错误横幅，这里只消掉 rejection 噪音 */
function setStatus(status: TaskStatus) {
  if (selected.value) void update(selected.value.id, { status }).catch(() => {});
}
</script>

<template>
  <TileApp v-if="isTile" />

  <template v-else>
    <TitleBar ref="titleBarRef" v-model="searchQuery" />

    <div class="app-shell">
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
          :on-create="createQuick"
          @select="select"
          @complete="complete"
          @reopen="reopen"
          @remove="remove"
          @save="update"
          @status="setStatus"
        />

        <AllTasksView
          v-else-if="view === 'all'"
          :tasks="tasks"
          :selected="selected"
          :project-id="activeProjectId"
          :query="searchQuery"
          @select="select"
          @complete="complete"
          @reopen="reopen"
          @remove="remove"
          @save="update"
          @status="setStatus"
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

    <TaskCreateModal />
  </template>
</template>
