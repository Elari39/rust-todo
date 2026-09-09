<script setup lang="ts">
import { computed } from "vue";
import type { Task } from "../types";
import { isSameDay, monthMatrix, parseStamp } from "../utils/datetime";

const props = defineProps<{
  now: Date;
  tasks: Task[];
}>();

const MAX_SHOWN = 3;
const weekdays = ["日", "一", "二", "三", "四", "五", "六"];

// 42 个格子放进同一个 7 列网格：各列宽度全局一致，不会因某行的长事件把列撑歪
const days = computed(() => monthMatrix(props.now).flat());

function eventsOn(day: Date) {
  return props.tasks.filter((task) => {
    const due = parseStamp(task.dueAt);
    const start = parseStamp(task.startAt);
    return (due && isSameDay(due, day)) || (start && isSameDay(start, day));
  });
}
</script>

<template>
  <section class="workspace">
    <header class="hero">
      <p class="kicker">日历</p>
      <h1>{{ now.getFullYear() }} 年 {{ now.getMonth() + 1 }} 月</h1>
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
            muted: day.getMonth() !== now.getMonth(),
            today: isSameDay(day, now),
          }"
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
