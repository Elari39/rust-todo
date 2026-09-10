<script setup lang="ts">
import { computed, ref } from "vue";
import { ChevronDown } from "lucide-vue-next";
import { useProjects } from "../composables/useProjects";
import type { Task } from "../types";
import { parseStamp, startOfDay } from "../utils/datetime";
import { t } from "../i18n";

const props = defineProps<{
  now: Date;
  tasks: Task[];
}>();

const mode = ref<"week" | "month">("week");
const horizonDays = computed(() => (mode.value === "week" ? 7 : 30));

interface GanttRow {
  task: Task;
  left: number;
  width: number;
}

interface GanttGroup {
  key: string;
  name: string;
  color: string;
  rows: GanttRow[];
}

const { projects, projectMap } = useProjects();

// 与「未来 horizon 天」窗口有交集的任务按项目分组；
// 组内按条形起点排序，时间线自上而下递进
const groups = computed<GanttGroup[]>(() => {
  const origin = startOfDay(props.now).getTime();
  const end = origin + horizonDays.value * 86400000;
  const span = end - origin;
  const byProject = new Map<string, GanttRow[]>();
  for (const task of props.tasks) {
    if (!task.startAt && !task.dueAt) continue;
    const start = parseStamp(task.startAt)?.getTime() ?? origin;
    let due = parseStamp(task.dueAt)?.getTime() ?? start + 86400000;
    // 截止早于开始的倒置区间按同日处理，避免画出反向条
    if (due < start) due = start;
    if (due <= origin || start >= end) continue;

    const left = Math.max(0, ((start - origin) / span) * 100);
    const right = Math.min(100, ((due - origin) / span) * 100);
    const row: GanttRow = {
      task,
      left,
      width: Math.min(Math.max(2.5, right - left), 100 - left),
    };
    const key = task.projectId ?? "";
    const bucket = byProject.get(key);
    if (bucket) bucket.push(row);
    else byProject.set(key, [row]);
  }

  const result: GanttGroup[] = [];
  for (const [key, rows] of byProject) {
    const project = key ? projectMap.value[key] : undefined;
    result.push({
      key: key || "ungrouped",
      name: project?.name ?? t("common.ungrouped"),
      color: project?.color ?? "#94a3b8",
      rows: rows.sort((a, b) => a.left - b.left || b.width - a.width),
    });
  }
  // 已建项目按建库顺序在前（projects 列表即建库顺序），未分组固定垫底
  const orderOf = (key: string) => {
    const index = projects.value.findIndex((project) => project.id === key);
    return index === -1 ? Number.MAX_SAFE_INTEGER : index;
  };
  result.sort((a, b) => {
    if (a.key === "ungrouped") return 1;
    if (b.key === "ungrouped") return -1;
    return orderOf(a.key) - orderOf(b.key);
  });
  return result;
});

const rangeLabel = computed(() => {
  const start = startOfDay(props.now);
  const end = new Date(start.getFullYear(), start.getMonth(), start.getDate() + horizonDays.value);
  return `${start.getMonth() + 1}月${start.getDate()}日 - ${end.getMonth() + 1}月${end.getDate()}日`;
});

// 日期刻度：周视图每天一档，月视图每 5 天一档
const ticks = computed(() => {
  const origin = startOfDay(props.now);
  const total = horizonDays.value;
  const step = mode.value === "week" ? 1 : 5;
  const list: { pct: number; label: string; edge: string }[] = [];
  for (let offset = 0; offset < total; offset += step) {
    const day = new Date(origin.getFullYear(), origin.getMonth(), origin.getDate() + offset);
    const edge = offset === 0 ? "first" : offset + step >= total ? "last" : "";
    list.push({
      pct: (offset / total) * 100,
      label: `${day.getMonth() + 1}/${day.getDate()}`,
      edge,
    });
  }
  return list;
});

const collapsed = ref(new Set<string>());

function toggleGroup(key: string) {
  const next = new Set(collapsed.value);
  if (next.has(key)) next.delete(key);
  else next.add(key);
  collapsed.value = next;
}

function statusDot(task: Task): string {
  if (task.status === "completed") return "done";
  if (task.status === "in_progress") return "active";
  return "pending";
}
</script>

<template>
  <section class="workspace">
    <div class="board solo">
      <div class="page-card">
        <header class="page-head">
          <div>
            <h1 class="page-title">{{ t("gantt.kicker") }}</h1>
            <p class="page-sub">{{ rangeLabel }}</p>
          </div>
          <div class="seg">
            <button
              class="seg-btn"
              :class="{ on: mode === 'week' }"
              type="button"
              @click="mode = 'week'"
            >
              {{ t("gantt.week") }}
            </button>
            <button
              class="seg-btn"
              :class="{ on: mode === 'month' }"
              type="button"
              @click="mode = 'month'"
            >
              {{ t("gantt.month") }}
            </button>
          </div>
        </header>

        <div class="page-scroll gantt-scroll">
          <p v-if="!groups.length" class="empty">{{ t("gantt.empty") }}</p>
          <div v-else class="gantt-inner">
            <div class="gantt-row gantt-scale-row">
              <span class="gantt-label" />
              <div class="gantt-scale">
                <span
                  v-for="tick in ticks"
                  :key="tick.label"
                  class="gantt-tick"
                  :class="tick.edge"
                  :style="{ left: `${tick.pct}%` }"
                >{{ tick.label }}</span>
              </div>
            </div>

            <template v-for="group in groups" :key="group.key">
              <button
                class="gantt-group-head"
                type="button"
                @click="toggleGroup(group.key)"
              >
                <span class="dot" :style="{ background: group.color }" />
                <b>{{ group.name }}</b>
                <span class="group-count">{{ group.rows.length }}</span>
                <ChevronDown
                  class="chev"
                  :class="{ collapsed: collapsed.has(group.key) }"
                  :size="15"
                />
              </button>
              <template v-if="!collapsed.has(group.key)">
                <div v-for="row in group.rows" :key="row.task.id" class="gantt-row">
                  <span class="gantt-label">
                    <i class="gantt-status-dot" :class="statusDot(row.task)" />
                    <span class="gantt-title">{{ row.task.title }}</span>
                  </span>
                  <div class="gantt-track">
                    <div
                      class="gantt-bar"
                      :style="{
                        left: `${row.left}%`,
                        width: `${row.width}%`,
                        background: group.color,
                      }"
                      :title="row.task.title"
                    />
                  </div>
                </div>
              </template>
            </template>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
