<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Check, Plus, X } from "lucide-vue-next";
import { useTasks } from "../composables/useTasks";
import {
  defaultDue,
  dueLabel,
  fromInputValue,
  isOverdue,
  toInputValue,
} from "../utils/datetime";

const win = getCurrentWindow();
const { todayTasks, error, create, complete, reopen, refresh } = useTasks();

const draft = ref("");
const dueAt = ref(toInputValue(defaultDue()));
let poll = 0;

onMounted(() => {
  poll = window.setInterval(() => {
    void refresh();
  }, 60_000);
});

onUnmounted(() => {
  window.clearInterval(poll);
});

const openCount = computed(
  () => todayTasks.value.filter((task) => task.status !== "completed").length,
);

async function add() {
  const title = draft.value.trim();
  if (!title) return;
  await create({
    title,
    kind: "quick",
    dueAt: fromInputValue(dueAt.value) ?? defaultDue(),
  });
  draft.value = "";
}

function toggle(id: string, done: boolean) {
  if (done) void reopen(id);
  else void complete(id);
}
</script>

<template>
  <div class="tile">
    <header class="tile-head" data-tauri-drag-region>
      <b>今日待办</b>
      <span class="tile-count">{{ openCount }}</span>
      <button class="tile-close" type="button" title="关闭磁贴" @click="win.close()">
        <X :size="14" />
      </button>
    </header>

    <p v-if="error" class="tile-error">{{ error }}</p>

    <ul class="tile-list">
      <li v-if="!todayTasks.length" class="tile-empty">今天没有待办，休息一下。</li>
      <li
        v-for="task in todayTasks"
        :key="task.id"
        class="tile-item"
        :class="{ done: task.status === 'completed' }"
      >
        <button
          class="tile-check"
          type="button"
          title="切换完成"
          @click="toggle(task.id, task.status === 'completed')"
        >
          <Check :size="12" />
        </button>
        <div class="tile-body">
          <span class="tile-title">{{ task.title }}</span>
          <span
            class="tile-due"
            :class="{ late: isOverdue(task) }"
          >{{ dueLabel(task.dueAt) }}</span>
        </div>
      </li>
    </ul>

    <form class="tile-add" @submit.prevent="add">
      <Plus :size="14" />
      <input v-model="draft" type="text" placeholder="快速记一条，回车添加" />
    </form>
  </div>
</template>

<style scoped>
.tile {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--panel);
  border-radius: 18px;
  overflow: hidden;
  box-shadow: 0 12px 32px rgba(48, 42, 30, 0.18);
}

.tile-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 14px;
  background: var(--card);
  user-select: none;
}

.tile-head b {
  font-size: 13px;
}

.tile-count {
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 999px;
  background: var(--blue);
  color: #fff;
  font-size: 11px;
  display: grid;
  place-items: center;
}

.tile-close {
  margin-left: auto;
  width: 24px;
  height: 24px;
  border: 0;
  border-radius: 8px;
  background: #f1f4f9;
  color: #64748b;
  display: grid;
  place-items: center;
}

.tile-close:hover {
  background: #fee2e2;
  color: #b91c1c;
}

.tile-error {
  margin: 8px 12px 0;
  padding: 6px 10px;
  border-radius: 10px;
  background: #fee2e2;
  color: #991b1b;
  font-size: 12px;
}

.tile-list {
  list-style: none;
  margin: 0;
  padding: 10px 12px;
  flex: 1;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tile-empty {
  color: var(--muted);
  font-size: 12px;
  text-align: center;
  padding: 24px 0;
}

.tile-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  background: var(--card);
  border-radius: 12px;
  padding: 10px;
  box-shadow: var(--shadow);
}

.tile-item.done .tile-title {
  text-decoration: line-through;
  color: var(--muted);
}

.tile-check {
  width: 18px;
  height: 18px;
  flex: 0 0 auto;
  margin-top: 1px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  background: #fff;
  color: transparent;
  display: grid;
  place-items: center;
}

.tile-item.done .tile-check {
  background: var(--green);
  border-color: var(--green);
  color: #fff;
}

.tile-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.tile-title {
  font-size: 12px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tile-due {
  font-size: 11px;
  color: var(--muted);
}

.tile-due.late {
  color: var(--red);
  font-weight: 700;
}

.tile-add {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0 12px 12px;
  padding: 8px 10px;
  border-radius: 12px;
  background: var(--card);
  color: var(--muted);
}

.tile-add input {
  flex: 1;
  border: 0;
  outline: none;
  background: transparent;
  font-size: 12px;
  color: var(--ink);
}
</style>
