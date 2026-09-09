<script setup lang="ts">
import { computed } from "vue";
import { Check } from "lucide-vue-next";
import { useClock } from "../composables/useClock";
import { useProjects } from "../composables/useProjects";
import type { Task } from "../types";
import { formatShortStamp, isOverdue } from "../utils/datetime";
import { t } from "../i18n";

const props = defineProps<{
  task: Task;
  active: boolean;
}>();

const emit = defineEmits<{
  select: [];
  toggle: [];
}>();

const { projectMap } = useProjects();
const clock = useClock();

// 逾期标记依赖响应式时钟：跨午夜、到期时刻后卡片颜色自动切换
const done = computed(() => props.task.status === "completed");
const started = computed(() => props.task.status === "in_progress");
const overdue = computed(() => !done.value && isOverdue(props.task, clock.value));
const project = computed(() =>
  props.task.projectId ? projectMap.value[props.task.projectId] : undefined,
);

// 标签 pill 用项目色：浅底衬深字，与设计图一致
const pillStyle = computed(() => {
  if (!project.value) return undefined;
  return {
    background: `color-mix(in srgb, ${project.value.color} 12%, #ffffff)`,
    color: project.value.color,
  };
});

const timeLabel = computed(() => formatShortStamp(props.task.dueAt ?? props.task.startAt));
const checkTitle = computed(() => (done.value ? t("detail.reopen") : t("detail.complete")));
</script>

<template>
  <div
    class="task-item"
    :class="{ active, done }"
    role="button"
    tabindex="0"
    @click="emit('select')"
    @keydown.enter.prevent="emit('select')"
  >
    <button
      class="check"
      :class="{ done, active: started }"
      type="button"
      role="checkbox"
      :aria-checked="done"
      :title="checkTitle"
      @click.stop="emit('toggle')"
    >
      <Check v-if="done" :size="12" />
    </button>
    <span class="task-body">
      <span class="task-title">{{ task.title }}</span>
    </span>
    <span class="task-side">
      <span v-if="project" class="pill" :style="pillStyle">
        <i class="pill-dot" />{{ project.name }}
      </span>
      <span v-if="done" class="pill pill-gray">{{ t("task.completed") }}</span>
      <span class="task-time" :class="{ late: overdue }">{{ timeLabel }}</span>
    </span>
  </div>
</template>
