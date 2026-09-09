import { computed, onMounted, ref } from "vue";
import { api } from "../api";
import type { NewTask, Task, TaskPatch } from "../types";
import { isOverdue, nearestDue, overlapsToday } from "../utils/datetime";

export function useTasks() {
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
  const todayCount = computed(() => todayTasks.value.filter((task) => task.status !== "completed").length);
  const overdueCount = computed(() => pending.value.filter((task) => isOverdue(task)).length);
  const nextDue = computed(() => nearestDue(pending.value));

  async function refresh() {
    loading.value = true;
    error.value = "";
    try {
      tasks.value = await api.listTasks();
      if (selectedId.value && !tasks.value.some((task) => task.id === selectedId.value)) {
        selectedId.value = null;
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
    } finally {
      loading.value = false;
    }
  }

  async function create(input: NewTask) {
    const task = await api.createTask(input);
    await refresh();
    selectedId.value = task.id;
    return task;
  }

  async function update(id: string, patch: TaskPatch) {
    await api.updateTask(id, patch);
    await refresh();
  }

  async function complete(id: string) {
    await api.completeTask(id);
    await refresh();
  }

  async function reopen(id: string) {
    await api.updateTask(id, { status: "pending" });
    await refresh();
  }

  async function remove(id: string) {
    await api.deleteTask(id);
    if (selectedId.value === id) selectedId.value = null;
    await refresh();
  }

  async function markNotified(id: string) {
    await api.markNotified(id);
    await refresh();
  }

  async function reorder(orderedIds: string[]) {
    await api.reorderTasks(orderedIds);
    await refresh();
  }

  function select(id: string | null) {
    selectedId.value = id;
  }

  onMounted(() => {
    void refresh();
  });

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
