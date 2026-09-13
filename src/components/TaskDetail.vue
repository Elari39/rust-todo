<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { Check, Pencil, Play, RotateCcw, Trash2, X } from "lucide-vue-next";
import { useClock } from "../composables/useClock";
import { useProjects } from "../composables/useProjects";
import type { Task, TaskKind, TaskPatch, TaskPriority, TaskStatus } from "../types";
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
  onSave: (id: string, patch: TaskPatch) => Promise<void>;
}>();

const emit = defineEmits<{
  close: [];
  complete: [];
  reopen: [];
  remove: [];
  status: [status: TaskStatus];
}>();

const { projects, nameOf } = useProjects();
const clock = useClock();
const saving = ref(false);
let editSession = 0;

const editing = reactive({
  on: false,
  taskId: "",
  title: "",
  notes: "",
  priority: "normal" as TaskPriority,
  kind: "quick" as TaskKind,
  startAt: "",
  dueAt: "",
  projectId: "",
});

function syncForm() {
  editing.taskId = props.task.id;
  editing.title = props.task.title;
  editing.notes = props.task.notes ?? "";
  editing.priority = props.task.priority;
  editing.kind = props.task.kind;
  editing.startAt = toInputValue(props.task.startAt);
  editing.dueAt = toInputValue(props.task.dueAt);
  editing.projectId = props.task.projectId ?? "";
}

// 切换任务时结束编辑并重置表单；同一任务的后台刷新保留正在输入的内容。
watch(
  () => [props.task.id, props.task.updatedAt],
  () => {
    if (editing.taskId !== props.task.id) {
      editSession += 1;
      saving.value = false;
      editing.on = false;
    }
    if (editing.on) return;
    syncForm();
  },
  { immediate: true },
);

function startEdit() {
  editSession += 1;
  syncForm();
  editing.on = true;
}

const hours = computed(() => durationHours(props.task.startAt, props.task.dueAt));
const overdue = computed(() => isOverdue(props.task, clock.value));
const project = computed(() =>
  props.task.projectId ? projects.value.find((p) => p.id === props.task.projectId) : undefined,
);

async function save() {
  // 即使属性刚切换、watch 尚未执行，也不能把旧表单提交给新任务。
  if (!editing.on || editing.taskId !== props.task.id || saving.value) return;
  const session = editSession;
  const taskId = editing.taskId;
  saving.value = true;
  try {
    await props.onSave(taskId, {
      title: editing.title,
      notes: editing.notes.trim() ? editing.notes : null,
      priority: editing.priority,
      kind: editing.kind,
      startAt: fromInputValue(editing.startAt),
      dueAt: fromInputValue(editing.dueAt),
      projectId: editing.projectId,
    });
    // 切换任务后再返回，也属于新的一次编辑，旧请求不能关闭新表单。
    if (session === editSession && props.task.id === taskId) editing.on = false;
  } catch {
    // 保存失败由任务状态模块显示错误；当前表单保持打开，保留输入供重试。
  } finally {
    if (session === editSession) saving.value = false;
  }
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
      <input v-model="editing.title" required :disabled="saving" :placeholder="t('form.titlePlaceholder')" />
      <textarea v-model="editing.notes" rows="3" :disabled="saving" :placeholder="t('form.notesPlaceholder')" />
      <select v-model="editing.projectId" :disabled="saving">
        <option value="">{{ t("common.ungrouped") }}</option>
        <option v-for="project in projects" :key="project.id" :value="project.id">
          {{ project.name }}
        </option>
      </select>
      <select v-model="editing.priority" :disabled="saving">
        <option value="low">{{ t("priority.low") }}</option>
        <option value="normal">{{ t("priority.normal") }}</option>
        <option value="high">{{ t("priority.high") }}</option>
      </select>
      <select v-model="editing.kind" :disabled="saving">
        <option value="quick">{{ t("kind.quick") }}</option>
        <option value="detailed">{{ t("kind.detailed") }}</option>
      </select>
      <input v-model="editing.startAt" type="datetime-local" :disabled="saving" />
      <input v-model="editing.dueAt" type="datetime-local" :disabled="saving" />
      <div class="detail-actions">
        <button class="btn btn-green" type="submit" :disabled="saving">{{ t("common.save") }}</button>
        <button class="btn btn-ghost" type="button" :disabled="saving" @click="editing.on = false">{{ t("common.cancel") }}</button>
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
