<script setup lang="ts">
import { computed } from "vue";
import type { Task } from "../types";
import { parseStamp, startOfDay } from "../utils/datetime";

const props = defineProps<{
  now: Date;
  tasks: Task[];
}>();

const horizon = 14;

const rows = computed(() => {
  const origin = startOfDay(props.now).getTime();
  const end = origin + horizon * 86400000;
  return props.tasks
    .filter((task) => task.startAt || task.dueAt)
    .map((task) => {
      const start = parseStamp(task.startAt)?.getTime() ?? origin;
      const due = parseStamp(task.dueAt)?.getTime() ?? start + 86400000;
      const left = Math.max(0, ((start - origin) / (end - origin)) * 100);
      const right = Math.min(100, ((due - origin) / (end - origin)) * 100);
      return {
        task,
        left,
        width: Math.max(4, right - left),
      };
    });
});
</script>

<template>
  <section class="workspace">
    <header class="hero">
      <p class="kicker">甘特图</p>
      <h1>未来 14 天</h1>
    </header>
    <div class="panel">
      <p v-if="!rows.length" class="empty">还没有带时间范围的任务。</p>
      <div v-for="row in rows" :key="row.task.id" class="gantt-row">
        <span>{{ row.task.title }}</span>
        <div class="gantt-track">
          <div class="gantt-bar" :style="{ left: `${row.left}%`, width: `${row.width}%` }" />
        </div>
      </div>
    </div>
  </section>
</template>
