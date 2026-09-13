<script setup lang="ts">
import { computed, ref } from "vue";
import {
  CircleCheck,
  CircleDot,
  Clock3,
  Flame,
  ListTodo,
  Plus,
  Zap,
} from "lucide-vue-next";
import { useCreateModal } from "../composables/useCreateModal";
import type { Task, TaskKind, TaskPatch, TaskStatus } from "../types";
import {
  defaultDue,
  dueLabel,
  formatClock,
  formatFullDate,
  fromInputValue,
  parseStamp,
  toInputValue,
  toStamp,
  weekdayShort,
} from "../utils/datetime";
import { t, tf } from "../i18n";
import TaskCard from "./TaskCard.vue";
import TaskDetail from "./TaskDetail.vue";

const props = defineProps<{
  now: Date;
  tasks: Task[];
  completedTasks: Task[];
  selected: Task | null;
  onCreate: (payload: { title: string; kind: TaskKind; dueAt: string }) => Promise<void>;
  onSave: (id: string, patch: TaskPatch) => Promise<void>;
}>();

const emit = defineEmits<{
  select: [id: string | null];
  complete: [id: string];
  reopen: [id: string];
  remove: [id: string];
  status: [status: TaskStatus];
}>();

const { openCreate } = useCreateModal();

const draft = ref("");
const submitting = ref(false);
const dueAt = ref(toInputValue(defaultDue()));
const dueOpen = ref(false);

// 预设保持默认的 18:40 时刻，只切换日期
function presetInput(offsetDays: number): string {
  const date = new Date(props.now);
  date.setDate(date.getDate() + offsetDays);
  date.setHours(18, 40, 0, 0);
  return toInputValue(toStamp(date));
}

const presets = computed(() => [
  { label: t("preset.today"), value: presetInput(0) },
  { label: t("preset.tomorrow"), value: presetInput(1) },
  { label: t("preset.dayAfter"), value: presetInput(2) },
]);

const dateLabel = computed(() => `${formatFullDate(props.now)} ${weekdayShort(props.now)}`);
const nearest = computed(() => {
  const due = props.tasks
    .filter((task) => task.status !== "completed")
    .map((task) => parseStamp(task.dueAt))
    .filter((date): date is Date => Boolean(date))
    .sort((a, b) => a.getTime() - b.getTime())[0];
  return due ? formatClock(due) : "--:--";
});

const stats = computed(() => ({
  pending: props.tasks.filter((task) => task.status === "pending").length,
  active: props.tasks.filter((task) => task.status === "in_progress").length,
  done: props.completedTasks.length,
}));

// 分组与设计图一致：重要及紧急 → 进行中 → 今天 → 已完成；
// 空组不渲染；紧急组只收未开始的（进行中的紧急任务归入进行中组）
const groups = computed(() => {
  return [
    {
      key: "urgent",
      label: t("today.groupUrgent"),
      icon: Flame,
      tone: "red",
      tasks: props.tasks.filter(
        (task) => task.status === "pending" && task.priority === "high",
      ),
    },
    {
      key: "active",
      label: t("today.groupActive"),
      icon: CircleDot,
      tone: "blue",
      tasks: props.tasks.filter((task) => task.status === "in_progress"),
    },
    {
      key: "today",
      label: t("today.groupToday"),
      icon: Clock3,
      tone: "cyan",
      tasks: props.tasks.filter(
        (task) => task.status === "pending" && task.priority !== "high",
      ),
    },
    {
      key: "done",
      label: t("today.groupDone"),
      icon: CircleCheck,
      tone: "green",
      tasks: props.completedTasks,
    },
  ].filter((group) => group.tasks.length > 0);
});

async function submit(kind: TaskKind) {
  const submittedDraft = draft.value;
  const title = submittedDraft.trim();
  if (!title || submitting.value) return;
  submitting.value = true;
  try {
    await props.onCreate({
      title,
      kind,
      dueAt: fromInputValue(dueAt.value) ?? defaultDue(),
    });
    if (draft.value === submittedDraft) draft.value = "";
  } catch {
    // 失败已在错误横幅展示；保留输入让用户直接重试，不必重打
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <section class="workspace">
    <div class="board" :class="{ solo: !selected }">
      <div class="page-card">
        <header class="page-head">
          <div>
            <h1 class="page-title">{{ t("today.title") }}</h1>
            <p class="page-sub">{{ dateLabel }} · {{ tf("today.nearestDue", { time: nearest }) }}</p>
          </div>
          <button class="btn btn-primary" type="button" @click="openCreate()">
            <Plus :size="16" />
            {{ t("common.createTask") }}
          </button>
        </header>

        <div class="stat-row">
          <div class="stat-card">
            <span class="stat-ico orange"><ListTodo :size="18" /></span>
            <div>
              <b class="stat-num">{{ stats.pending }}</b>
              <span class="stat-label">{{ t("today.statPending") }}</span>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-ico blue"><Clock3 :size="18" /></span>
            <div>
              <b class="stat-num">{{ stats.active }}</b>
              <span class="stat-label">{{ t("today.statActive") }}</span>
            </div>
          </div>
          <div class="stat-card">
            <span class="stat-ico green"><CircleCheck :size="18" /></span>
            <div>
              <b class="stat-num">{{ stats.done }}</b>
              <span class="stat-label">{{ t("today.statDone") }}</span>
            </div>
          </div>
        </div>

        <div class="page-scroll">
          <p v-if="!tasks.length && !completedTasks.length" class="empty">{{ t("today.empty") }}</p>
          <section v-for="group in groups" :key="group.key" class="group">
            <div class="group-head">
              <span class="group-ico" :class="group.tone">
                <component :is="group.icon" :size="13" />
              </span>
              <b>{{ group.label }}</b>
              <span class="group-count">{{ group.tasks.length }}</span>
            </div>
            <TaskCard
              v-for="task in group.tasks"
              :key="task.id"
              :task="task"
              :active="selected?.id === task.id"
              @select="emit('select', task.id)"
              @toggle="
                task.status === 'completed'
                  ? emit('reopen', task.id)
                  : emit('complete', task.id)
              "
            />
          </section>
        </div>

        <form class="composer" :aria-busy="submitting" @submit.prevent="submit('quick')">
          <input v-model="draft" type="text" :placeholder="t('today.composerPlaceholder')" />
          <div class="due-wrap">
            <button class="due-chip" type="button" :title="t('today.composerDue')" @click="dueOpen = !dueOpen">
              <Clock3 :size="15" />
              {{ dueLabel(dueAt, now) }}
            </button>
            <div v-if="dueOpen" class="due-backdrop" @click="dueOpen = false" />
            <div v-if="dueOpen" class="due-pop" @keydown.esc="dueOpen = false">
              <div class="due-presets">
                <button
                  v-for="preset in presets"
                  :key="preset.value"
                  class="due-preset"
                  :class="{ on: dueAt === preset.value }"
                  type="button"
                  @click="
                    dueAt = preset.value;
                    dueOpen = false;
                  "
                >
                  {{ preset.label }}
                </button>
              </div>
              <label class="due-custom">
                <span>{{ t("preset.custom") }}</span>
                <input v-model="dueAt" type="datetime-local" @change="dueOpen = false" />
              </label>
            </div>
          </div>
          <button
            class="btn btn-ghost"
            type="button"
            :disabled="submitting"
            :title="t('today.detailedTip')"
            @click="submit('detailed')"
          >
            <Plus :size="15" /> {{ t("today.detailedCreate") }}
          </button>
          <button class="btn btn-orange" type="submit" :disabled="submitting">
            <Zap :size="15" /> {{ t("today.quickCreate") }}
          </button>
        </form>
      </div>

      <TaskDetail
        v-if="selected"
        :task="selected"
        :on-save="onSave"
        @close="emit('select', null)"
        @complete="emit('complete', selected.id)"
        @reopen="emit('reopen', selected.id)"
        @remove="emit('remove', selected.id)"
        @status="(status) => emit('status', status)"
      />
    </div>
  </section>
</template>
