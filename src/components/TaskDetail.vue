<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { Check, Pencil, Trash2, X } from "lucide-vue-next";
import { useProjects } from "../composables/useProjects";
import type { Task, TaskKind, TaskPriority } from "../types";
import {
  durationHours,
  formatDate,
  fromInputValue,
  isOverdue,
  toInputValue,
} from "../utils/datetime";

const props = defineProps<{
  task: Task;
}>();

const emit = defineEmits<{
  close: [];
  complete: [];
  reopen: [];
  remove: [];
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

// 同时跟踪 updatedAt：任务在别处被更新（磁贴窗口、提醒标记）后表单重新同步，防止旧数据覆盖
watch(
  () => [props.task.id, props.task.updatedAt],
  () => {
    editing.on = false;
    editing.title = props.task.title;
    editing.notes = props.task.notes ?? "";
    editing.priority = props.task.priority;
    editing.kind = props.task.kind;
    editing.startAt = toInputValue(props.task.startAt);
    editing.dueAt = toInputValue(props.task.dueAt);
    editing.projectId = props.task.projectId ?? "";
  },
  { immediate: true },
);

const hours = computed(() => durationHours(props.task.startAt, props.task.dueAt));
const overdue = computed(() => isOverdue(props.task));
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
</script>

<template>
  <aside class="detail">
    <div class="detail-head">
      <strong>任务详情</strong>
      <button class="icon-btn" type="button" @click="emit('close')">
        <X :size="16" />
      </button>
    </div>

    <template v-if="!editing.on">
      <h3 style="margin: 0">{{ task.title }}</h3>
      <div class="chips">
        <span v-if="overdue" class="chip chip-red">已超期</span>
        <span v-else-if="task.status === 'completed'" class="chip chip-green">已完成</span>
        <span class="chip chip-gray">{{ task.priority === "high" ? "紧急" : task.priority === "low" ? "较低" : "一般" }}</span>
        <span class="chip chip-orange">{{ task.kind === "detailed" ? "详细任务" : "快速任务" }}</span>
        <span
          v-if="project"
          class="chip"
          :style="{ background: `${project.color}1a`, color: project.color }"
        >{{ project.name }}</span>
      </div>
      <p v-if="task.notes" style="margin: 0; color: #64748b">{{ task.notes }}</p>
      <div class="kv"><span>开始</span><b>{{ formatDate(task.startAt) }}</b></div>
      <div class="kv"><span>结束</span><b>{{ formatDate(task.dueAt) }}</b></div>
      <div class="kv"><span>时长</span><b>{{ hours == null ? "未计算" : hours === 0 ? "<1 小时" : `${hours} 小时` }}</b></div>
      <div class="kv"><span>项目</span><b>{{ nameOf(task.projectId) }}</b></div>
      <div class="detail-actions">
        <button
          v-if="task.status !== 'completed'"
          class="btn btn-green"
          type="button"
          @click="emit('complete')"
        >
          <Check :size="16" /> 标记完成
        </button>
        <button v-else class="btn btn-ghost" type="button" @click="emit('reopen')">重新打开</button>
        <button class="btn btn-ghost" type="button" @click="editing.on = true">
          <Pencil :size="16" /> 编辑
        </button>
        <button class="icon-btn danger" type="button" @click="emit('remove')">
          <Trash2 :size="16" />
        </button>
      </div>
    </template>

    <form v-else class="form-grid" @submit.prevent="save">
      <input v-model="editing.title" required placeholder="任务标题" />
      <textarea v-model="editing.notes" rows="3" placeholder="备注" />
      <select v-model="editing.projectId">
        <option value="">未分组</option>
        <option v-for="project in projects" :key="project.id" :value="project.id">
          {{ project.name }}
        </option>
      </select>
      <select v-model="editing.priority">
        <option value="low">较低</option>
        <option value="normal">一般</option>
        <option value="high">紧急</option>
      </select>
      <select v-model="editing.kind">
        <option value="quick">快速任务</option>
        <option value="detailed">详细任务</option>
      </select>
      <input v-model="editing.startAt" type="datetime-local" />
      <input v-model="editing.dueAt" type="datetime-local" />
      <div class="detail-actions">
        <button class="btn btn-green" type="submit">保存</button>
        <button class="btn btn-ghost" type="button" @click="editing.on = false">取消</button>
      </div>
    </form>
  </aside>
</template>
