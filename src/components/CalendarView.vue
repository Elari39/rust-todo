<script setup lang="ts">
import { computed, ref } from "vue";
import { ChevronLeft, ChevronRight } from "lucide-vue-next";
import type { Task } from "../types";
import { isSameDay, monthMatrix, parseStamp } from "../utils/datetime";

const props = defineProps<{
  now: Date;
  tasks: Task[];
}>();

const MAX_SHOWN = 3;
const weekdays = ["日", "一", "二", "三", "四", "五", "六"];

// 本地月份锚点：支持前后翻月查看，初始为当前月
const anchor = ref(new Date(props.now));
const monthLabel = computed(
  () => `${anchor.value.getFullYear()} 年 ${anchor.value.getMonth() + 1} 月`,
);

function shiftMonth(delta: number) {
  anchor.value = new Date(anchor.value.getFullYear(), anchor.value.getMonth() + delta, 1);
}

// 42 个格子放进同一个 7 列网格：各列宽度全局一致，不会因某行的长事件把列撑歪
const days = computed(() => monthMatrix(anchor.value).flat());

// 按日索引一次建好：避免每个格子渲染时都对全量任务过滤两次
const eventsByDay = computed(() => {
  const map = new Map<string, Task[]>();
  const push = (task: Task, date: Date) => {
    const key = dayKey(date);
    const bucket = map.get(key);
    if (bucket) {
      // 开始与截止同日的任务只入桶一次
      if (!bucket.includes(task)) bucket.push(task);
    } else {
      map.set(key, [task]);
    }
  };
  for (const task of props.tasks) {
    const start = parseStamp(task.startAt);
    const due = parseStamp(task.dueAt);
    if (start) push(task, start);
    if (due) push(task, due);
  }
  return map;
});

function dayKey(day: Date): string {
  return `${day.getFullYear()}-${day.getMonth()}-${day.getDate()}`;
}

function eventsOn(day: Date): Task[] {
  return eventsByDay.value.get(dayKey(day)) ?? [];
}
</script>

<template>
  <section class="workspace">
    <header class="hero">
      <p class="kicker">日历</p>
      <h1>
        {{ monthLabel }}
        <span class="month-nav">
          <button class="icon-btn" type="button" title="上个月" @click="shiftMonth(-1)">
            <ChevronLeft :size="16" />
          </button>
          <button class="btn btn-ghost" type="button" @click="anchor = new Date(props.now)">
            本月
          </button>
          <button class="icon-btn" type="button" title="下个月" @click="shiftMonth(1)">
            <ChevronRight :size="16" />
          </button>
        </span>
      </h1>
    </header>
    <div class="panel">
      <div class="month-grid month-head">
        <strong v-for="day in weekdays" :key="day">{{ day }}</strong>
      </div>
      <div class="month-grid">
        <div
          v-for="day in days"
          :key="day.getTime()"
          class="month-cell"
          :class="{
            muted: day.getMonth() !== anchor.getMonth(),
            today: isSameDay(day, now),
          }"
          tabindex="0"
          :aria-label="`${day.getFullYear()}年${day.getMonth() + 1}月${day.getDate()}日，${eventsOn(day).length} 项日程`"
        >
          <b>{{ day.getDate() }}</b>
          <div
            v-for="task in eventsOn(day).slice(0, MAX_SHOWN)"
            :key="task.id"
            class="event"
          >
            {{ task.title }}
          </div>
          <span v-if="eventsOn(day).length > MAX_SHOWN" class="month-more">
            +{{ eventsOn(day).length - MAX_SHOWN }} 更多
          </span>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.month-nav {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: 12px;
  vertical-align: middle;
}

.month-cell:focus-visible {
  outline: 2px solid var(--blue);
  outline-offset: -2px;
}
</style>
