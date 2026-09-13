import { computed, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../api";
import type { NewTask, Task, TaskPatch, TaskStatus } from "../types";
import { useClock } from "./useClock";
import { isOverdue, isTodayStamp, overlapsToday } from "../utils/datetime";

// 模块级单例：同一窗口内共享同一份任务状态（磁贴窗口复用同一前端，也只拉取一次）
const tasks = ref<Task[]>([]);
const loading = ref(true);
const error = ref("");
const selectedId = ref<string | null>(null);
const clock = useClock();

const selected = computed(
  () => tasks.value.find((task) => task.id === selectedId.value) ?? null,
);

const pending = computed(() => tasks.value.filter((task) => task.status !== "completed"));
// 传入响应式时钟：跨午夜、到期时刻后自动重算，今日列表与逾期标记不再停留在旧状态
const todayTasks = computed(() =>
  tasks.value.filter(
    (task) => overlapsToday(task, clock.value) || isOverdue(task, clock.value),
  ),
);
// 完成统计按实际完成日期计算，独立于磁贴和待办列表的时间筛选。
const todayCompletedTasks = computed(() =>
  tasks.value.filter(
    (task) => task.status === "completed" && isTodayStamp(task.completedAt, clock.value),
  ),
);
const todayCount = computed(
  () => todayTasks.value.filter((task) => task.status !== "completed").length,
);

/** 变更失败不能静默：写入 error 供横幅展示，并继续抛出让调用方可感知 */
async function attempt<T>(run: () => Promise<T>): Promise<T> {
  try {
    return await run();
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
    throw err;
  }
}

// 单调序号守卫：磁贴轮询、跨窗口事件、变更操作会并发触发多次刷新，
// IPC 返回顺序不保证与调用顺序一致，过期响应必须丢弃，避免旧数据覆盖新数据
let refreshSeq = 0;

async function refresh() {
  const seq = ++refreshSeq;
  loading.value = true;
  try {
    const data = await api.listTasks();
    if (seq !== refreshSeq) return;
    tasks.value = data;
    error.value = "";
  } catch (err) {
    if (seq === refreshSeq) {
      error.value = err instanceof Error ? err.message : String(err);
    }
  } finally {
    if (seq === refreshSeq) loading.value = false;
  }
}

// 跨窗口同步：磁贴和主窗口各跑一份独立 JS 实例，任一窗口变更任务后，
// 后端广播 tasks-changed（payload 为来源窗口 label）。自己发起的变更已在
// 变更函数里手动刷新过，跳过；其余窗口防抖合并连续事件后重拉。
const myLabel = getCurrentWindow().label;
let syncTimer = 0;
listen<string>("tasks-changed", (event) => {
  if (event.payload === myLabel) return;
  window.clearTimeout(syncTimer);
  syncTimer = window.setTimeout(() => void refresh(), 120);
}).catch((err) => {
  // 监听建立失败意味着跨窗口同步整个失效，必须让用户可见
  error.value = `跨窗口同步不可用: ${err instanceof Error ? err.message : String(err)}`;
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

/** 批量标记已提醒：合并为一次列表刷新，避免每个到期任务各拉一次全量数据 */
async function markNotifiedMany(ids: string[]) {
  for (const id of ids) {
    try {
      await api.markNotified(id);
    } catch (err) {
      // 单个标记失败（如任务已被其他窗口删除）不中断，横幅提示后继续
      error.value = err instanceof Error ? err.message : String(err);
    }
  }
  await refresh();
}

async function reorder(orderedIds: string[]) {
  const snapshot = tasks.value;
  try {
    await attempt(() => api.reorderTasks(orderedIds));
    await refresh();
  } catch {
    // 恢复为数据库真实顺序：换新数组引用触发组件 watch，撤销未落库的本地预览，
    // 否则列表会一直显示拖拽预览顺序。错误横幅已由 attempt 写入。
    tasks.value = [...snapshot];
  }
}

function select(id: string | null) {
  selectedId.value = id;
}

export function useTasks() {
  return {
    tasks,
    error,
    selected,
    selectedId,
    pending,
    todayTasks,
    todayCompletedTasks,
    todayCount,
    refresh,
    create,
    update,
    complete,
    reopen,
    remove,
    markNotifiedMany,
    reorder,
    select,
  };
}
