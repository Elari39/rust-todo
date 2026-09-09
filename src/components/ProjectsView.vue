<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  BookOpen,
  Briefcase,
  FolderKanban,
  FolderPlus,
  Home,
  Pencil,
  SquareKanban,
  Star,
  Trash2,
} from "lucide-vue-next";
import { useProjects } from "../composables/useProjects";
import type { Project, Task } from "../types";
import { t, tf } from "../i18n";

const props = defineProps<{
  tasks: Task[];
}>();

const emit = defineEmits<{
  open: [projectId: string];
}>();

const { projects, error, create, update, remove, load: loadProjects } = useProjects();

const PALETTE = ["#3D5BDB", "#EC4899", "#22C55E", "#F59E0B", "#8B5CF6", "#22A6C8"];

// 卡片图标按项目顺序循环取用，纯装饰
const ICONS = [BookOpen, Briefcase, Home, SquareKanban, FolderKanban, Star];

const formOpen = ref(false);
const editingId = ref<string | null>(null);
const formName = ref("");
const formColor = ref(PALETTE[0]);

const stats = computed(() => {
  const map: Record<string, { total: number; done: number; active: number }> = {};
  for (const task of props.tasks) {
    if (!task.projectId) continue;
    const entry = (map[task.projectId] ??= { total: 0, done: 0, active: 0 });
    entry.total += 1;
    if (task.status === "completed") entry.done += 1;
    else if (task.status === "in_progress") entry.active += 1;
  }
  return map;
});

function statsOf(project: Project) {
  const entry = stats.value[project.id] ?? { total: 0, done: 0, active: 0 };
  return { ...entry, pending: entry.total - entry.done - entry.active };
}

function pctOf(project: Project): number {
  const entry = statsOf(project);
  return entry.total ? Math.round((entry.done / entry.total) * 100) : 0;
}

function iconFor(index: number) {
  return ICONS[index % ICONS.length];
}

function openCreate() {
  editingId.value = null;
  formName.value = "";
  formColor.value = PALETTE[0];
  formOpen.value = true;
}

function openEdit(project: Project) {
  editingId.value = project.id;
  formName.value = project.name;
  formColor.value = project.color;
  formOpen.value = true;
}

onMounted(() => {
  // 初次加载失败时进入本视图补拉一次
  if (!projects.value.length) void loadProjects();
});

async function submit() {
  const name = formName.value.trim();
  if (!name) return;
  try {
    if (editingId.value) {
      await update(editingId.value, name, formColor.value);
    } else {
      await create(name, formColor.value);
    }
  } catch {
    // 失败已在横幅展示；保留表单让用户重试
    return;
  }
  formOpen.value = false;
  formName.value = "";
}

async function destroy(project: Project) {
  if (!window.confirm(tf("proj.confirmDelete", { name: project.name }))) return;
  try {
    await remove(project.id);
  } catch {
    // 失败已在横幅展示
  }
}
</script>

<template>
  <section class="workspace">
    <div class="board solo">
      <div class="page-card">
        <header class="page-head">
          <div>
            <h1 class="page-title">{{ t("proj.title") }}</h1>
            <p class="page-sub">
              {{ tf("proj.count", { n: projects.length }) }} · {{ t("proj.hint") }}
            </p>
          </div>
          <button class="btn btn-primary" type="button" @click="openCreate">
            <FolderPlus :size="15" />
            {{ t("proj.create") }}
          </button>
        </header>

        <form v-if="formOpen" class="proj-form" @submit.prevent="submit">
          <input v-model="formName" type="text" :placeholder="t('proj.namePlaceholder')" />
          <div class="swatches">
            <button
              v-for="color in PALETTE"
              :key="color"
              type="button"
              class="swatch"
              :class="{ active: formColor === color }"
              :style="{ background: color }"
              @click="formColor = color"
            />
          </div>
          <button class="btn btn-primary" type="submit">
            {{ editingId ? t("proj.saveEdit") : t("proj.createBtn") }}
          </button>
          <button class="btn btn-ghost" type="button" @click="formOpen = false">
            {{ t("common.cancel") }}
          </button>
        </form>

        <div class="page-scroll">
          <p v-if="error" class="error-banner" role="alert">{{ error }}</p>
          <p v-if="!projects.length" class="empty">{{ t("proj.empty") }}</p>
          <div class="projects-grid">
            <div v-for="(project, index) in projects" :key="project.id" class="project-card">
              <div class="project-head">
                <span
                  class="project-ico"
                  :style="{
                    background: `color-mix(in srgb, ${project.color} 14%, #ffffff)`,
                    color: project.color,
                  }"
                >
                  <component :is="iconFor(index)" :size="18" />
                </span>
                <b class="project-name">{{ project.name }}</b>
                <span class="project-actions">
                  <button class="icon-btn" type="button" :title="t('common.edit')" @click="openEdit(project)">
                    <Pencil :size="14" />
                  </button>
                  <button class="icon-btn danger" type="button" :title="t('common.delete')" @click="destroy(project)">
                    <Trash2 :size="14" />
                  </button>
                </span>
              </div>

              <div class="proj-progress">
                <div class="progress">
                  <i :style="{ width: `${pctOf(project)}%`, background: project.color }" />
                </div>
                <span class="proj-pct">{{ pctOf(project) }}%</span>
              </div>

              <div class="proj-stats">
                <div class="proj-stat">
                  <b>{{ statsOf(project).total }}</b>
                  <span>{{ t("proj.statTotal") }}</span>
                </div>
                <div class="proj-stat">
                  <b>{{ statsOf(project).done }}</b>
                  <span>{{ t("proj.statDone") }}</span>
                </div>
                <div class="proj-stat">
                  <b>{{ statsOf(project).active }}</b>
                  <span>{{ t("proj.statActive") }}</span>
                </div>
                <div class="proj-stat">
                  <b>{{ statsOf(project).pending }}</b>
                  <span>{{ t("proj.statPending") }}</span>
                </div>
              </div>

              <button class="btn btn-ghost project-open" type="button" @click="emit('open', project.id)">
                {{ t("proj.viewTasks") }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
