<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { ArrowDownUp, Search, X } from "lucide-vue-next";
import { useProjects } from "../composables/useProjects";
import type { Task, TaskPatch } from "../types";
import { parseStamp } from "../utils/datetime";
import TaskCard from "./TaskCard.vue";
import TaskDetail from "./TaskDetail.vue";

const props = defineProps<{
  tasks: Task[];
  selected: Task | null;
  projectId?: string | null;
}>();

const emit = defineEmits<{
  select: [id: string | null];
  complete: [id: string];
  reopen: [id: string];
  remove: [id: string];
  save: [id: string, patch: TaskPatch];
  clearProject: [];
  reorder: [orderedIds: string[]];
}>();

const { projects } = useProjects();
const query = ref("");
const filter = ref<"all" | "pending" | "completed">("all");
const sortMode = ref<"due" | "manual">("due");
const inputRef = ref<HTMLInputElement | null>(null);

const project = computed(() =>
  props.projectId ? projects.value.find((item) => item.id === props.projectId) ?? null : null,
);

const visible = computed(() => {
  const needle = query.value.trim().toLowerCase();
  const list = props.tasks.filter((task) => {
    const matchQuery =
      !needle ||
      task.title.toLowerCase().includes(needle) ||
      (task.notes ?? "").toLowerCase().includes(needle);
    const matchProject = !props.projectId || task.projectId === props.projectId;
    const matchStatus =
      filter.value === "all" ||
      (filter.value === "pending" && task.status !== "completed") ||
      (filter.value === "completed" && task.status === "completed");
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
const canDrag = computed(() => sortMode.value === "manual" && !query.value.trim());

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
watch([query, filter, sortMode, () => props.tasks], () => {
  localOrder.value = null;
});

function onKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    inputRef.value?.focus();
    inputRef.value?.select();
  }
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <section class="workspace">
    <header class="hero">
      <p class="kicker">{{ project ? "项目跟踪" : "全部任务" }}</p>
      <h1>
        {{ project ? project.name : "所有待办" }}
        <button
          v-if="project"
          class="icon-btn"
          type="button"
          title="返回全部任务"
          @click="emit('clearProject')"
        >
          <X :size="14" />
        </button>
      </h1>
      <form class="composer" @submit.prevent>
        <label class="search-wrap">
          <Search class="lead-ico" :size="14" />
          <input
            ref="inputRef"
            v-model="query"
            type="text"
            placeholder="搜索标题或备注（Ctrl+K）"
          />
          <button
            v-if="query"
            class="icon-btn clear-btn"
            type="button"
            title="清空"
            @click="query = ''"
          >
            <X :size="12" />
          </button>
        </label>
        <button
          class="btn"
          :class="sortMode === 'manual' ? 'btn-green' : 'btn-ghost'"
          type="button"
          title="切换排序方式"
          @click="sortMode = sortMode === 'manual' ? 'due' : 'manual'"
        >
          <ArrowDownUp :size="14" />
          {{ sortMode === "manual" ? "手动排序" : "按截止时间" }}
        </button>
        <button class="btn" :class="filter === 'all' ? 'btn-green' : 'btn-ghost'" type="button" @click="filter = 'all'">全部</button>
        <button class="btn" :class="filter === 'pending' ? 'btn-orange' : 'btn-ghost'" type="button" @click="filter = 'pending'">进行中</button>
        <button class="btn" :class="filter === 'completed' ? 'btn-green' : 'btn-ghost'" type="button" @click="filter = 'completed'">已完成</button>
      </form>
      <p v-if="query.trim()" class="kicker">共 {{ visible.length }} 条匹配「{{ query.trim() }}」</p>
      <p v-else-if="canDrag" class="kicker">拖动卡片调整顺序，切到「按截止时间」恢复时间排序。</p>
    </header>
    <div class="board" :class="{ solo: !selected }">
      <div class="task-list">
        <p v-if="!displayList.length" class="empty">
          {{ query.trim() ? "没有匹配的任务。" : "这里还空着，先加一条。" }}
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
          />
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
      />
    </div>
  </section>
</template>
