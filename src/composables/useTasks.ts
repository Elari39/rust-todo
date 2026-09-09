import { computed, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../api";
import type { NewTask, Task, TaskPatch, TaskStatus } from "../types";
import { isOverdue, nearestDue, overlapsToday } from "../utils/datetime";

// 模块级单例：同一窗口内共享同一份任务状态（磁贴窗口复用同一前端，也只拉取一次）
const tasks = ref<Task[]>([]);
const loading = ref(true);
const error = ref("");
const selectedId = ref<string | null>(null);

const selected = computed(
  () => tasks.value.find((task) => task.id === selectedId.value) ?? null,
);

const pending = computed(() => tasks.value.filter((task) => task.status !== "completed"));
const todayTasks = computed(() =>
  tasks.value.filter((task) => overlapsToday(task) || isOverdue(task)),
);
const todayCount = computed(
  () => todayTasks.value.filter((task) => task.status !== "completed").length,
);
const overdueCount = computed(() => pending.value.filter((task) => isOverdue(task)).length);
const nextDue = computed(() => nearestDue(pending.value));

/** 变更失败不能静默：写入 error 供横幅展示，并继续抛出让调用方可感知 */
async function attempt<T>(run: () => Promise<T>): Promise<T> {
  try {
    return await run();
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
    throw err;
  }
}

async function refresh() {
  loading.value = true;
  try {
    tasks.value = await api.listTasks();
    error.value = "";
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
}

// 跨窗口同步：磁贴和主窗口各跑一份独立 JS 实例，任一窗口变更任务后，
// 后端广播 tasks-changed（payload 为来源窗口 label）。自己发起的变更已在
// 变更函数里手动刷新过，跳过；其余窗口防抖合并连续事件后重拉。
const myLabel = getCurrentWindow().label;
let syncTimer = 0;
void listen<string>("tasks-changed", (event) => {
  if (event.payload === myLabel) return;
  window.clearTimeout(syncTimer);
  syncTimer = window.setTimeout(() => void refresh(), 120);
});

function setLocalStatus(id: string, status: TaskStatus) {
  const task = tasks.value.find((task) => task.id === id);
  if (task) task.status = status;
}

/** 乐观更新：先本地翻转状态让 UI 立即响应，失败则回滚。
 * refresh 只在成功后执行——它成功时会清空 error，失败路径跑会把错误横幅抹掉 */
async function toggleStatus(id: string, status: TaskStatus, run: () => Promise<unknown>) {
  const prev = tasks.value.find((task) => task.id === id)?.status ?? status;
  setLocalStatus(id, status);
  try {
    await attempt(run);
  } catch (err) {
    setLocalStatus(id, prev);
    throw err;
  }
  await refresh();
}

async function create(input: NewTask): Promise<Task> {
  const task = await attempt(() => api.createTask(input));
  await refresh();
  selectedId.value = task.id;
  return task;
}

async function update(id: string, patch: TaskPatch) {
  await attempt(() => api.updateTask(id, patch));
  await refresh();
}

async function complete(id: string) {
  await toggleStatus(id, "completed", () => api.completeTask(id));
}

async function reopen(id: string) {
  await toggleStatus(id, "pending", () => api.updateTask(id, { status: "pending" }));
}

async function remove(id: string) {
  await attempt(() => api.deleteTask(id));
  if (selectedId.value === id) selectedId.value = null;
  await refresh();
}

async function markNotified(id: string) {
  await attempt(() => api.markNotified(id));
  await refresh();
}

async function reorder(orderedIds: string[]) {
  await attempt(() => api.reorderTasks(orderedIds));
  await refresh();
}

function select(id: string | null) {
  selectedId.value = id;
}

export function useTasks() {
  return {
    tasks,
    loading,
    error,
    selected,
    selectedId,
    pending,
    todayTasks,
    todayCount,
    overdueCount,
    nextDue,
    refresh,
    create,
    update,
    complete,
    reopen,
    remove,
    markNotified,
    reorder,
    select,
  };
}
