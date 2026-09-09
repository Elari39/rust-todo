<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { FolderPlus, Pencil, Trash2 } from "lucide-vue-next";
import { useProjects } from "../composables/useProjects";
import type { Project, Task } from "../types";

const props = defineProps<{
  tasks: Task[];
}>();

const emit = defineEmits<{
  open: [projectId: string];
}>();

const { projects, error, create, update, remove, load: loadProjects } = useProjects();

const PALETTE = ["#3D5BDB", "#16A34A", "#F59E0B", "#EF4444", "#22A6C8", "#8B5CF6"];

const formOpen = ref(false);
const editingId = ref<string | null>(null);
const formName = ref("");
const formColor = ref(PALETTE[0]);

const stats = computed(() => {
  const map: Record<string, { total: number; done: number }> = {};
  for (const task of props.tasks) {
    if (!task.projectId) continue;
    const entry = (map[task.projectId] ??= { total: 0, done: 0 });
    entry.total += 1;
    if (task.status === "completed") entry.done += 1;
  }
  return map;
});

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
  if (!window.confirm(`删除项目「${project.name}」？项目下的任务会变为未分组。`)) return;
  try {
    await remove(project.id);
  } catch {
    // 失败已在横幅展示
  }
}
</script>

<template>
  <section class="workspace">
    <header class="hero">
      <p class="kicker">项目跟踪</p>
      <h1>{{ projects.length }} 个项目进行中</h1>
      <p>点开项目查看任务进度，删除项目不会删除任务。</p>
      <form class="composer" @submit.prevent="submit">
        <template v-if="formOpen">
          <input v-model="formName" type="text" placeholder="项目名称" />
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
          <button class="btn btn-green" type="submit">
            {{ editingId ? "保存修改" : "创建项目" }}
          </button>
          <button class="btn btn-ghost" type="button" @click="formOpen = false">取消</button>
        </template>
        <template v-else>
          <input type="text" placeholder="管理项目从这里开始" disabled />
          <button class="btn btn-green" type="button" @click="openCreate">
            <FolderPlus :size="16" /> 新建项目
          </button>
        </template>
      </form>
    </header>

    <div class="panel">
      <p v-if="error" class="error-banner" role="alert">{{ error }}</p>
      <p v-if="!projects.length" class="empty">还没有项目，先建一个。</p>
      <div class="projects-grid">
        <div v-for="project in projects" :key="project.id" class="project-card">
          <div class="project-head">
            <span class="dot" :style="{ background: project.color }" />
            <b>{{ project.name }}</b>
            <span class="project-actions">
              <button class="icon-btn" type="button" title="编辑" @click="openEdit(project)">
                <Pencil :size="14" />
              </button>
              <button class="icon-btn danger" type="button" title="删除" @click="destroy(project)">
                <Trash2 :size="14" />
              </button>
            </span>
          </div>
          <div class="progress">
            <i
              :style="{
                width: `${stats[project.id]?.total ? (stats[project.id].done / stats[project.id].total) * 100 : 0}%`,
                background: project.color,
              }"
            />
          </div>
          <p class="project-meta">
            {{ stats[project.id]?.done ?? 0 }}/{{ stats[project.id]?.total ?? 0 }} 已完成
          </p>
          <button class="btn btn-ghost project-open" type="button" @click="emit('open', project.id)">
            查看任务
          </button>
        </div>
      </div>
    </div>
  </section>
</template>
