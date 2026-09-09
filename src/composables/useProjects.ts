import { computed, ref } from "vue";
import { api } from "../api";
import type { Project } from "../types";
import { t } from "../i18n";

// 模块级单例：主窗口内任意组件共享同一份项目数据
const projects = ref<Project[]>([]);
const loading = ref(false);
const error = ref("");
let loaded = false;

export function useProjects() {
  async function load(force = false) {
    // 防重入：多处组件挂载时都可能触发重试，避免并发拉取互相覆盖
    if ((loaded && !force) || loading.value) return;
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
    await load(true);
  }

  /** 变更失败不能静默：写入 error 供横幅展示，并继续抛出让调用方可感知 */
  async function attempt<T>(run: () => Promise<T>): Promise<T> {
    try {
      return await run();
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
      throw err;
    }
  }

  async function create(name: string, color: string) {
    const project = await attempt(() => api.createProject(name, color));
    await refresh();
    return project;
  }

  async function update(id: string, name: string, color: string) {
    await attempt(() => api.updateProject(id, name, color));
    await refresh();
  }

  async function remove(id: string) {
    await attempt(() => api.deleteProject(id));
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
    if (!id) return t("common.ungrouped");
    return projectMap.value[id]?.name ?? t("common.ungrouped");
  }

  return { projects, projectMap, loading, error, load, refresh, create, update, remove, nameOf };
}
