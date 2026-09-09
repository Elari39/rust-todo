<script setup lang="ts">
import { computed, ref } from "vue";
import { ChevronLeft, ChevronRight, Plus } from "lucide-vue-next";
import { useCreateModal } from "../composables/useCreateModal";
import { useProjects } from "../composables/useProjects";
import { useTasks } from "../composables/useTasks";
import type { Task } from "../types";
import { isSameDay, monthMatrix, parseStamp, toStamp } from "../utils/datetime";
import { t, tf } from "../i18n";
import TaskCard from "./TaskCard.vue";

const props = defineProps<{
  now: Date;
  tasks: Task[];
}>();

const MAX_DOTS = 4;
const weekdays = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

const { projectMap } = useProjects();
const { complete, reopen } = useTasks();
const { openCreate } = useCreateModal();

// 本地月份锚点：支持前后翻月查看，初始为当前月
const anchor = ref(new Date(props.now));
const selectedDay = ref(new Date(props.now));

const monthLabel = computed(
  () => `${anchor.value.getFullYear()} 年 ${anchor.value.getMonth() + 1} 月`,
);

function shiftMonth(delta: number) {
  anchor.value = new Date(anchor.value.getFullYear(), anchor.value.getMonth() + delta, 1);
}

function goToday() {
  anchor.value = new Date(props.now);
  selectedDay.value = new Date(props.now);
}

// 42 个格子放进同一个 7 列网格：各列宽度全局一致
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

/** 格子里只画事件色点：项目色，未分组用灰色 */
function dotsFor(day: Date): string[] {
  return eventsOn(day)
    .slice(0, MAX_DOTS)
    .map((task) => (task.projectId ? projectMap.value[task.projectId]?.color : undefined) ?? "#94a3b8");
}

const dayTitle = computed(() => {
  const params = { m: selectedDay.value.getMonth() + 1, d: selectedDay.value.getDate() };
  return isSameDay(selectedDay.value, props.now)
    ? tf("cal.dayHeaderToday", params)
    : tf("cal.dayHeader", params);
});

const dayTasks = computed(() => eventsOn(selectedDay.value));

function createForDay() {
  const date = new Date(selectedDay.value);
  date.setHours(18, 40, 0, 0);
  openCreate({ dueAt: toStamp(date) });
}

function toggleTask(task: Task) {
  if (task.status === "completed") void reopen(task.id);
  else void complete(task.id);
}
</script>

<template>
  <section class="workspace">
    <div class="board solo">
      <div class="page-card">
        <header class="page-head">
          <div class="page-head-main">
            <button class="icon-btn" type="button" :title="t('cal.prev')" @click="shiftMonth(-1)">
              <ChevronLeft :size="16" />
            </button>
            <h1 class="page-title">{{ monthLabel }}</h1>
            <button class="icon-btn" type="button" :title="t('cal.next')" @click="shiftMonth(1)">
              <ChevronRight :size="16" />
            </button>
          </div>
          <button class="btn btn-ghost" type="button" @click="goToday">
            {{ t("cal.todayBtn") }}
          </button>
        </header>

        <div class="cal-layout">
          <div class="cal-main">
            <div class="cal-weekdays">
              <span v-for="day in weekdays" :key="day">{{ day }}</span>
            </div>

            <div class="cal-grid">
              <button
                v-for="day in days"
                :key="day.getTime()"
                class="cal-cell"
                :class="{
                  muted: day.getMonth() !== anchor.getMonth(),
                  today: isSameDay(day, now),
                  selected: isSameDay(day, selectedDay),
                }"
                type="button"
                :aria-label="`${day.getFullYear()}年${day.getMonth() + 1}月${day.getDate()}日，${eventsOn(day).length} 项日程`"
                @click="selectedDay = day"
              >
                <span class="cal-daynum">{{ day.getDate() }}</span>
                <span class="cal-dots">
                  <i
                    v-for="(color, index) in dotsFor(day)"
                    :key="index"
                    class="cal-dot"
                    :style="{ background: color }"
                  />
                </span>
              </button>
            </div>
          </div>

          <aside class="day-panel">
            <div class="day-panel-head">
              <b>{{ dayTitle }}</b>
              <button class="btn btn-primary btn-sm" type="button" @click="createForDay">
                <Plus :size="14" />
                {{ t("common.createTask") }}
              </button>
            </div>
            <div class="day-list">
              <p v-if="!dayTasks.length" class="empty-sm">{{ t("cal.emptyDay") }}</p>
              <TaskCard
                v-for="task in dayTasks"
                :key="task.id"
                :task="task"
                :active="false"
                @toggle="toggleTask(task)"
              />
            </div>
          </aside>
        </div>
      </div>
    </div>
  </section>
</template>
