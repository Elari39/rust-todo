<script setup lang="ts">
import { computed } from "vue";
import { Pencil } from "lucide-vue-next";
import { useProjects } from "../composables/useProjects";
import type { Task } from "../types";
import { formatRange, isOverdue } from "../utils/datetime";

const props = defineProps<{
  task: Task;
  active: boolean;
}>();

const emit = defineEmits<{
  select: [];
}>();

const { projectMap } = useProjects();

const color = computed(() => {
  if (props.task.status === "completed") return "green";
  if (isOverdue(props.task)) return "red";
  if (props.task.priority === "high") return "orange";
  return "cyan";
});

const project = computed(() =>
  props.task.projectId ? projectMap.value[props.task.projectId] : undefined,
);
</script>

<template>
  <button
    class="task-card"
    :class="{ active, done: task.status === 'completed' }"
    type="button"
    @click="emit('select')"
  >
    <span class="dot" :class="color" />
    <div>
      <p class="task-title">{{ task.title }}</p>
      <p class="task-meta">
        <span
          v-if="project"
          class="proj-tag"
          :style="{ color: project.color }"
        ><i class="proj-dot" :style="{ background: project.color }" />{{ project.name }}</span>
        {{ formatRange(task.startAt, task.dueAt) }}
      </p>
    </div>
    <Pencil :size="16" color="#94a3b8" />
  </button>
</template>
