<script setup lang="ts">
import { computed, ref } from "vue";
import { Clock3, Plus, Zap } from "lucide-vue-next";
import type { Task, TaskKind, TaskPatch } from "../types";
import {
  defaultDue,
  dueLabel,
  formatClock,
  fromInputValue,
  parseStamp,
  toInputValue,
  toStamp,
  weekdayName,
} from "../utils/datetime";
import TaskCard from "./TaskCard.vue";
import TaskDetail from "./TaskDetail.vue";

const props = defineProps<{
  now: Date;
  tasks: Task[];
  selected: Task | null;
  allPending: number;
  onCreate: (payload: { title: string; kind: TaskKind; dueAt: string }) => Promise<void>;
}>();

const emit = defineEmits<{
  select: [id: string | null];
  complete: [id: string];
  reopen: [id: string];
  remove: [id: string];
  save: [id: string, patch: TaskPatch];
}>();

const draft = ref("");
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
  { label: "今天 18:40", value: presetInput(0) },
  { label: "明天 18:40", value: presetInput(1) },
  { label: "后天 18:40", value: presetInput(2) },
]);

const headline = computed(() => `${props.now.getMonth() + 1} 月 ${props.now.getDate()} 日`);
const nearest = computed(() => {
  const due = props.tasks
    .map((task) => parseStamp(task.dueAt))
    .filter((date): date is Date => Boolean(date))
    .sort((a, b) => a.getTime() - b.getTime())[0];
  return due ? formatClock(due) : "--:--";
});

async function submit(kind: TaskKind) {
  const title = draft.value.trim();
  if (!title) return;
  try {
    await props.onCreate({
      title,
      kind,
      dueAt: fromInputValue(dueAt.value) ?? defaultDue(),
    });
  } catch {
    // 失败已在错误横幅展示；保留输入让用户直接重试，不必重打
    return;
  }
  draft.value = "";
}
</script>

<template>
  <section class="workspace">
    <header class="hero">
      <p class="kicker">Todo 待办清单 · {{ now.getFullYear() }} 年 · {{ weekdayName(now) }}</p>
      <h1>{{ headline }}, <em>今天的事</em></h1>
      <p>{{ allPending }} 项待办，最近一项 {{ nearest }} 到期。</p>
      <form class="composer" @submit.prevent="submit('quick')">
        <input v-model="draft" type="text" placeholder="输入临时待办，如：买车票、联系张三" />
        <div class="due-wrap">
          <button class="due-chip" type="button" title="选择截止时间" @click="dueOpen = !dueOpen">
            <Clock3 :size="16" />
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
              <span>自定义</span>
              <input v-model="dueAt" type="datetime-local" @change="dueOpen = false" />
            </label>
          </div>
        </div>
        <button
          class="btn btn-green"
          type="button"
          title="详细任务默认标记为「紧急」优先级"
          @click="submit('detailed')"
        >
          <Plus :size="16" /> 新建详细任务
        </button>
        <button class="btn btn-orange" type="submit">
          <Zap :size="16" /> 新建快速任务
        </button>
      </form>
    </header>

    <div class="board" :class="{ solo: !selected }">
      <div class="task-list">
        <p v-if="!tasks.length" class="empty">今天还没有待办，先记一件小事。</p>
        <TaskCard
          v-for="task in tasks"
          :key="task.id"
          :task="task"
          :active="selected?.id === task.id"
          @select="emit('select', task.id)"
        />
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
