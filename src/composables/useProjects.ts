import { computed, ref } from "vue";
import { api } from "../api";
import type { Project } from "../types";

// 模块级单例：主窗口内任意组件共享同一份项目数据
const projects = ref<Project[]>([]);
const loading = ref(false);
const error = ref("");
let loaded = false;

export function useProjects() {
  async function load(force = false) {
    if (loaded && !force) return;
    loading.value = true;
    error.value = "";
    try {
      projects.value = await api.listProjects();
      loaded = true;
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
    } finally {
      loading.value = false;
    }
  }

  async function refresh() {
    loaded = false;
    await load(true);
  }

  async function create(name: string, color: string) {
    const project = await api.createProject(name, color);
    await refresh();
    return project;
  }

  async function update(id: string, name: string, color: string) {
    await api.updateProject(id, name, color);
    await refresh();
  }

  async function remove(id: string) {
    await api.deleteProject(id);
    await refresh();
  }

  const projectMap = computed(() => {
    const map: Record<string, Project> = {};
    for (const project of projects.value) {
      map[project.id] = project;
    }
    return map;
  });

  function nameOf(id: string | null): string {
    if (!id) return "未分组";
    return projectMap.value[id]?.name ?? "未分组";
  }

  return { projects, projectMap, loading, error, load, refresh, create, update, remove, nameOf };
}
