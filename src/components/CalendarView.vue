<script setup lang="ts">
import { computed } from "vue";
import type { Task } from "../types";
import { isSameDay, monthMatrix, parseStamp } from "../utils/datetime";

const props = defineProps<{
  now: Date;
  tasks: Task[];
}>();

const weeks = computed(() => monthMatrix(props.now));
const weekdays = ["日", "一", "二", "三", "四", "五", "六"];

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
      <div class="month-grid" style="margin-bottom: 8px">
        <strong v-for="day in weekdays" :key="day" style="text-align:center;color:#64748b">{{ day }}</strong>
      </div>
      <div v-for="(week, index) in weeks" :key="index" class="month-grid">
        <div
          v-for="day in week"
          :key="day.toISOString()"
          class="month-cell"
          :class="{
            muted: day.getMonth() !== now.getMonth(),
            today: isSameDay(day, now),
          }"
        >
          <b>{{ day.getDate() }}</b>
          <div
            v-for="task in eventsOn(day).slice(0, 3)"
            :key="task.id"
            class="event"
          >
            {{ task.title }}
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
