<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { ArrowDownUp, Plus, X } from "lucide-vue-next";
import { useCreateModal } from "../composables/useCreateModal";
import { useProjects } from "../composables/useProjects";
import type { Task, TaskPatch, TaskStatus } from "../types";
import { parseStamp } from "../utils/datetime";
import { t, tf } from "../i18n";
import TaskCard from "./TaskCard.vue";
import TaskDetail from "./TaskDetail.vue";

const props = defineProps<{
  tasks: Task[];
  selected: Task | null;
  projectId?: string | null;
  /** 搜索词来自标题栏（App.vue），由 prop 驱动 */
  query: string;
}>();

const emit = defineEmits<{
  select: [id: string | null];
  complete: [id: string];
  reopen: [id: string];
  remove: [id: string];
  save: [id: string, patch: TaskPatch];
  status: [status: TaskStatus];
  clearProject: [];
  reorder: [orderedIds: string[]];
}>();

const { projects, load: loadProjects } = useProjects();
const { openCreate } = useCreateModal();
const filter = ref<"all" | "open" | "done">("all");
const sortMode = ref<"due" | "manual">("due");

const filterOptions = computed(() => [
  { value: "all" as const, label: t("all.tabAll") },
  { value: "open" as const, label: t("all.tabOpen") },
  { value: "done" as const, label: t("all.tabDone") },
]);

const project = computed(() =>
  props.projectId ? projects.value.find((item) => item.id === props.projectId) ?? null : null,
);

const visible = computed(() => {
  const needle = props.query.trim().toLowerCase();
  const list = props.tasks.filter((task) => {
    const matchQuery =
      !needle ||
      task.title.toLowerCase().includes(needle) ||
      (task.notes ?? "").toLowerCase().includes(needle);
    const matchProject = !props.projectId || task.projectId === props.projectId;
    const matchStatus =
      filter.value === "all" ||
      (filter.value === "open" && task.status !== "completed") ||
      (filter.value === "done" && task.status === "completed");
    return matchQuery && matchProject && matchStatus;
  });
  if (sortMode.value !== "due") return list;
  return [...list].sort((a, b) => {
    const left = parseStamp(a.dueAt)?.getTime() ?? Number.MAX_SAFE_INTEGER;
    const right = parseStamp(b.dueAt)?.getTime() ?? Number.MAX_SAFE_INTEGER;
    return left - right;
  });
});

/** 拖拽期间用本地副本实时预览顺序，落点后提交并清空 */
const dragId = ref<string | null>(null);
const localOrder = ref<Task[] | null>(null);
const displayList = computed(() => localOrder.value ?? visible.value);
const canDrag = computed(() => sortMode.value === "manual" && !props.query.trim());

function toggleSort() {
  sortMode.value = sortMode.value === "manual" ? "due" : "manual";
}

function onDragStart(id: string) {
  if (!canDrag.value) return;
  dragId.value = id;
}

function onDragOver(event: DragEvent, overId: string) {
  if (!dragId.value || dragId.value === overId) return;
  const list = [...displayList.value];
  const from = list.findIndex((task) => task.id === dragId.value);
  const anchor = list.findIndex((task) => task.id === overId);
  if (from < 0 || anchor < 0) return;
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const after = event.clientY > rect.top + rect.height / 2;
  const [moved] = list.splice(from, 1);
  let target = list.findIndex((task) => task.id === overId);
  if (target < 0) return;
  if (after) target += 1;
  list.splice(target, 0, moved);
  localOrder.value = list;
}

function onDragEnd() {
  if (localOrder.value) {
    emit("reorder", commitMerged(localOrder.value));
  }
  // 本地预览保留到落库刷新（props.tasks 更新）后再撤销，避免列表回闪旧顺序
  dragId.value = null;
}

/** 只重排当前可见任务：按原序遍历，可见任务依次用新顺序替换，隐藏任务原地保留 */
function commitMerged(orderedVisible: Task[]): string[] {
  const visibleIds = new Set(orderedVisible.map((task) => task.id));
  const merged: string[] = [];
  let cursor = 0;
  for (const task of props.tasks) {
    if (visibleIds.has(task.id)) {
      const next = orderedVisible[cursor++];
      if (next) merged.push(next.id);
    } else {
      merged.push(task.id);
    }
  }
  return merged;
}

// 搜索/筛选/排序变化会改变可见集，落库刷新会更新 props.tasks：这些时刻都应撤销本地预览
watch([() => props.query, filter, sortMode, () => props.tasks], () => {
  localOrder.value = null;
});

onMounted(() => {
  // 初次加载失败时进入本视图补拉一次，项目名不再一直显示「未分组」
  if (!projects.value.length) void loadProjects();
});
</script>

<template>
  <section class="workspace">
    <div class="board" :class="{ solo: !selected }">
      <div class="page-card">
        <header class="page-head">
          <div>
            <h1 class="page-title">
              {{ project ? project.name : t("all.title") }}
            </h1>
            <p class="page-sub">
              {{ project ? t("all.projectKicker") : tf("proj.count", { n: tasks.length }) }}
            </p>
          </div>
          <div class="page-actions">
            <button
              v-if="project"
              class="icon-btn"
              type="button"
              :title="t('all.backToAll')"
              @click="emit('clearProject')"
            >
              <X :size="14" />
            </button>
            <div class="tabs">
              <button
                v-for="option in filterOptions"
                :key="option.value"
                class="tab"
                :class="{ on: filter === option.value }"
                type="button"
                @click="filter = option.value"
              >
                {{ option.label }}
              </button>
            </div>
            <button class="btn btn-ghost" type="button" :title="t('all.sortTip')" @click="toggleSort">
              <ArrowDownUp :size="14" />
              {{ sortMode === "manual" ? t("all.sortManual") : t("all.sortDue") }}
            </button>
            <button class="btn btn-primary" type="button" @click="openCreate()">
              <Plus :size="15" />
              {{ t("common.createTask") }}
            </button>
          </div>
        </header>

        <p v-if="query.trim()" class="kicker-hint">
          {{ tf("all.matchCount", { n: visible.length, q: query.trim() }) }}
        </p>
        <p v-else-if="canDrag" class="kicker-hint">{{ t("all.dragHint") }}</p>

        <div class="task-list page-scroll">
          <p v-if="!displayList.length" class="empty">
            {{ query.trim() ? t("all.empty") : t("all.emptyAll") }}
          </p>
          <div
            v-for="task in displayList"
            :key="task.id"
            class="task-slot"
            :class="{ dragging: dragId === task.id }"
            :draggable="canDrag"
            @dragstart="onDragStart(task.id)"
            @dragover.prevent="onDragOver($event, task.id)"
            @dragend="onDragEnd"
            @drop.prevent
          >
            <TaskCard
              :task="task"
              :active="selected?.id === task.id"
              @select="emit('select', task.id)"
              @toggle="
                task.status === 'completed'
                  ? emit('reopen', task.id)
                  : emit('complete', task.id)
              "
            />
          </div>
        </div>
      </div>

      <TaskDetail
        v-if="selected"
        :task="selected"
        @close="emit('select', null)"
        @complete="emit('complete', selected.id)"
        @reopen="emit('reopen', selected.id)"
        @remove="emit('remove', selected.id)"
        @save="(patch) => selected && emit('save', selected.id, patch)"
        @status="(status) => emit('status', status)"
      />
    </div>
  </section>
</template>
