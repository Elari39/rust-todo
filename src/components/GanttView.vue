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
  const span = end - origin;
  const result: { task: Task; left: number; width: number }[] = [];
  for (const task of props.tasks) {
    if (!task.startAt && !task.dueAt) continue;
    const start = parseStamp(task.startAt)?.getTime() ?? origin;
    let due = parseStamp(task.dueAt)?.getTime() ?? start + 86400000;
    // 截止早于开始的倒置区间按同日处理，避免画出反向条
    if (due < start) due = start;
    // 只保留与「未来 horizon 天」窗口有交集的任务，窗口外任务不画
    if (due <= origin || start >= end) continue;

    const left = Math.max(0, ((start - origin) / span) * 100);
    const right = Math.min(100, ((due - origin) / span) * 100);
    result.push({ task, left, width: Math.min(Math.max(4, right - left), 100 - left) });
  }
  // 按条形起点排序输出，时间线自上而下递进，不再跟随数据库顺序
  result.sort((a, b) => a.left - b.left || b.width - a.width);
  return result;
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
