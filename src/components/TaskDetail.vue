<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { Check, Pencil, Play, RotateCcw, Trash2, X } from "lucide-vue-next";
import { useClock } from "../composables/useClock";
import { useProjects } from "../composables/useProjects";
import type { Task, TaskKind, TaskPriority, TaskStatus } from "../types";
import {
  durationHours,
  formatDate,
  fromInputValue,
  isOverdue,
  toInputValue,
} from "../utils/datetime";
import { t, tf } from "../i18n";

const props = defineProps<{
  task: Task;
}>();

const emit = defineEmits<{
  close: [];
  complete: [];
  reopen: [];
  remove: [];
  status: [status: TaskStatus];
  save: [patch: {
    title: string;
    notes: string | null;
    priority: TaskPriority;
    kind: TaskKind;
    startAt: string | null;
    dueAt: string | null;
    projectId: string;
  }];
}>();

const { projects, nameOf } = useProjects();
const clock = useClock();

const editing = reactive({
  on: false,
  title: "",
  notes: "",
  priority: "normal" as TaskPriority,
  kind: "quick" as TaskKind,
  startAt: "",
  dueAt: "",
  projectId: "",
});

function syncForm() {
  editing.title = props.task.title;
  editing.notes = props.task.notes ?? "";
  editing.priority = props.task.priority;
  editing.kind = props.task.kind;
  editing.startAt = toInputValue(props.task.startAt);
  editing.dueAt = toInputValue(props.task.dueAt);
  editing.projectId = props.task.projectId ?? "";
}

// 同时跟踪 updatedAt：任务在别处被更新（磁贴窗口、提醒标记）后表单重新同步，
// 防止旧数据覆盖。正在编辑时跳过重置，保留用户输入不被远程变更清掉；
// 保存完成后 updatedAt 变化会触发同步。
watch(
  () => [props.task.id, props.task.updatedAt],
  () => {
    if (editing.on) return;
    syncForm();
  },
  { immediate: true },
);

function startEdit() {
  syncForm();
  editing.on = true;
}

const hours = computed(() => durationHours(props.task.startAt, props.task.dueAt));
const overdue = computed(() => isOverdue(props.task, clock.value));
const project = computed(() =>
  props.task.projectId ? projects.value.find((p) => p.id === props.task.projectId) : undefined,
);

function save() {
  emit("save", {
    title: editing.title,
    notes: editing.notes.trim() ? editing.notes : null,
    priority: editing.priority,
    kind: editing.kind,
    startAt: fromInputValue(editing.startAt),
    dueAt: fromInputValue(editing.dueAt),
    projectId: editing.projectId,
  });
  editing.on = false;
}

function removeWithConfirm() {
  if (window.confirm(t("detail.confirmDelete"))) emit("remove");
}
</script>

<template>
  <aside class="detail">
    <div class="detail-head">
      <strong>{{ t("detail.title") }}</strong>
      <button class="icon-btn" type="button" @click="emit('close')">
        <X :size="16" />
      </button>
    </div>

    <template v-if="!editing.on">
      <h3>{{ task.title }}</h3>
      <div class="chips">
        <span v-if="overdue" class="chip chip-red">{{ t("task.overdue") }}</span>
        <span v-if="task.status === 'completed'" class="chip chip-green">{{ t("task.completed") }}</span>
        <span v-else-if="task.status === 'in_progress'" class="chip chip-blue">{{ t("task.inProgress") }}</span>
        <span class="chip chip-gray">
          {{ task.priority === "high" ? t("priority.high") : task.priority === "low" ? t("priority.low") : t("priority.normal") }}
        </span>
        <span class="chip chip-orange">{{ task.kind === "detailed" ? t("kind.detailed") : t("kind.quick") }}</span>
        <span
          v-if="project"
          class="chip"
          :style="{ background: `color-mix(in srgb, ${project.color} 12%, #ffffff)`, color: project.color }"
        >{{ project.name }}</span>
      </div>
      <p v-if="task.notes" class="detail-notes">{{ task.notes }}</p>
      <div class="kv"><span>{{ t("detail.start") }}</span><b>{{ formatDate(task.startAt) }}</b></div>
      <div class="kv"><span>{{ t("detail.end") }}</span><b>{{ formatDate(task.dueAt) }}</b></div>
      <div class="kv">
        <span>{{ t("detail.duration") }}</span>
        <b>{{ hours == null ? t("detail.durationUncomputed") : hours === 0 ? t("detail.durationLess") : tf("detail.durationHours", { n: hours }) }}</b>
      </div>
      <div class="kv"><span>{{ t("detail.project") }}</span><b>{{ nameOf(task.projectId) }}</b></div>
      <div class="detail-actions">
        <template v-if="task.status === 'pending'">
          <button class="btn btn-primary" type="button" @click="emit('status', 'in_progress')">
            <Play :size="14" /> {{ t("detail.startTask") }}
          </button>
        </template>
        <template v-else-if="task.status === 'in_progress'">
          <button class="btn btn-ghost" type="button" @click="emit('status', 'pending')">
            <RotateCcw :size="14" /> {{ t("detail.pauseTask") }}
          </button>
        </template>
        <template v-else>
          <button class="btn btn-ghost" type="button" @click="emit('reopen')">{{ t("detail.reopen") }}</button>
        </template>
        <button
          v-if="task.status !== 'completed'"
          class="btn btn-green"
          type="button"
          @click="emit('complete')"
        >
          <Check :size="14" /> {{ t("detail.complete") }}
        </button>
        <button class="btn btn-ghost" type="button" @click="startEdit">
          <Pencil :size="14" /> {{ t("common.edit") }}
        </button>
        <button class="icon-btn danger" type="button" :title="t('detail.deleteTitle')" @click="removeWithConfirm">
          <Trash2 :size="16" />
        </button>
      </div>
    </template>

    <form v-else class="form-grid" @submit.prevent="save">
      <input v-model="editing.title" required :placeholder="t('form.titlePlaceholder')" />
      <textarea v-model="editing.notes" rows="3" :placeholder="t('form.notesPlaceholder')" />
      <select v-model="editing.projectId">
        <option value="">{{ t("common.ungrouped") }}</option>
        <option v-for="project in projects" :key="project.id" :value="project.id">
          {{ project.name }}
        </option>
      </select>
      <select v-model="editing.priority">
        <option value="low">{{ t("priority.low") }}</option>
        <option value="normal">{{ t("priority.normal") }}</option>
        <option value="high">{{ t("priority.high") }}</option>
      </select>
      <select v-model="editing.kind">
        <option value="quick">{{ t("kind.quick") }}</option>
        <option value="detailed">{{ t("kind.detailed") }}</option>
      </select>
      <input v-model="editing.startAt" type="datetime-local" />
      <input v-model="editing.dueAt" type="datetime-local" />
      <div class="detail-actions">
        <button class="btn btn-green" type="submit">{{ t("common.save") }}</button>
        <button class="btn btn-ghost" type="button" @click="editing.on = false">{{ t("common.cancel") }}</button>
      </div>
    </form>
  </aside>
</template>

<style scoped>
.detail-notes {
  margin: 0;
  color: #64748b;
  font-size: 13px;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
</style>
