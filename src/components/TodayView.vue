<script setup lang="ts">
import { computed, ref } from "vue";
import { Clock3, Plus, Zap } from "lucide-vue-next";
import type { Task, TaskKind, TaskPatch } from "../types";
import { defaultDue, dueLabel, formatClock, fromInputValue, parseStamp, toInputValue, weekdayName } from "../utils/datetime";
import TaskCard from "./TaskCard.vue";
import TaskDetail from "./TaskDetail.vue";

const props = defineProps<{
  now: Date;
  tasks: Task[];
  selected: Task | null;
  allPending: number;
}>();

const emit = defineEmits<{
  select: [id: string | null];
  create: [payload: { title: string; kind: TaskKind; dueAt: string }];
  complete: [id: string];
  reopen: [id: string];
  remove: [id: string];
  save: [id: string, patch: TaskPatch];
}>();

const draft = ref("");
const dueAt = ref(toInputValue(defaultDue()));

const headline = computed(() => `${props.now.getMonth() + 1} 月 ${props.now.getDate()} 日`);
const nearest = computed(() => {
  const due = props.tasks
    .map((task) => parseStamp(task.dueAt))
    .filter((date): date is Date => Boolean(date))
    .sort((a, b) => a.getTime() - b.getTime())[0];
  return due ? formatClock(due) : "--:--";
});

function submit(kind: TaskKind) {
  const title = draft.value.trim();
  if (!title) return;
  emit("create", { title, kind, dueAt: fromInputValue(dueAt.value) ?? defaultDue() });
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
        <label class="due-chip">
          <Clock3 :size="16" />
          {{ dueLabel(dueAt, now) }}
          <input v-model="dueAt" type="datetime-local" />
        </label>
        <button class="btn btn-green" type="button" @click="submit('detailed')">
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
