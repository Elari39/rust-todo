<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";
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

// 磁贴无边框：系统拖动通道（data-tauri-drag-region / startDragging）在 WebView2 下不可靠，
// 改为按下后手动跟随光标移动窗口——只依赖 setPosition 这类普通窗口调用。
// 按钮和输入框除外；pointer capture 保证光标移出窗口后拖动仍持续。
let drag: {
  pointerId: number;
  startX: number;
  startY: number;
  originX: number;
  originY: number;
} | null = null;

async function onTilePointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  const target = event.target as HTMLElement | null;
  if (target?.closest("button, input")) return;
  const el = event.currentTarget as HTMLElement;
  const scale = window.devicePixelRatio || 1;
  try {
    const origin = await win.outerPosition();
    el.setPointerCapture(event.pointerId);
    drag = {
      pointerId: event.pointerId,
      startX: event.screenX * scale,
      startY: event.screenY * scale,
      originX: origin.x,
      originY: origin.y,
    };
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
}

function onTilePointerMove(event: PointerEvent) {
  if (!drag || event.pointerId !== drag.pointerId) return;
  const scale = window.devicePixelRatio || 1;
  win.setPosition(
    new PhysicalPosition(
      drag.originX + Math.round(event.screenX * scale - drag.startX),
      drag.originY + Math.round(event.screenY * scale - drag.startY),
    ),
  );
}

function onTilePointerUp(event: PointerEvent) {
  if (drag && event.pointerId === drag.pointerId) drag = null;
}

onMounted(() => {
  void refresh();
  poll = window.setInterval(() => {
    void refresh();
  }, 60_000);
});

onUnmounted(() => {
  window.clearInterval(poll);
  for (const timer of fadeTimers.values()) window.clearTimeout(timer);
});

const openCount = computed(
  () => todayTasks.value.filter((task) => task.status !== "completed").length,
);

// 完成条目短暂停留后淡出：watch 到 pending→completed（含主窗口远程完成）开始计时，
// 2 秒后从展示列表移除触发 leave 动画；期间撤销完成则取消计时并恢复显示。
const FADE_DELAY = 2000;
const hiddenIds = ref(new Set<string>());
const fadeTimers = new Map<string, number>();

const visibleTasks = computed(() =>
  todayTasks.value.filter((task) => !hiddenIds.value.has(task.id)),
);

watch(todayTasks, (list) => {
  const statuses = new Map(list.map((task) => [task.id, task.status]));
  for (const [id, timer] of fadeTimers) {
    if (statuses.get(id) !== "completed") {
      window.clearTimeout(timer);
      fadeTimers.delete(id);
      hiddenIds.value.delete(id);
    }
  }
  for (const task of list) {
    if (
      task.status === "completed" &&
      !fadeTimers.has(task.id) &&
      !hiddenIds.value.has(task.id)
    ) {
      fadeTimers.set(
        task.id,
        window.setTimeout(() => {
          fadeTimers.delete(task.id);
          hiddenIds.value.add(task.id);
        }, FADE_DELAY),
      );
    }
  }
  for (const id of [...hiddenIds.value]) {
    if (!statuses.has(id)) hiddenIds.value.delete(id);
  }
});

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

// in-flight 守卫：处理完成前忽略同一任务的重复点击，避免 complete/reopen 竞态来回翻转
const toggling = new Set<string>();

async function toggle(id: string, done: boolean) {
  if (toggling.has(id)) return;
  toggling.add(id);
  try {
    if (done) await reopen(id);
    else await complete(id);
  } catch {
    // 失败已回滚本地状态并写入 error 横幅展示
  } finally {
    toggling.delete(id);
  }
}
</script>

<template>
  <div
    class="tile"
    @pointerdown="onTilePointerDown"
    @pointermove="onTilePointerMove"
    @pointerup="onTilePointerUp"
    @pointercancel="onTilePointerUp"
  >
    <header class="tile-head">
      <b>今日待办</b>
      <span class="tile-count">{{ openCount }}</span>
      <button class="tile-close" type="button" title="关闭磁贴" @click="win.close()">
        <X :size="14" />
      </button>
    </header>

    <p v-if="error" class="tile-error">{{ error }}</p>

    <TransitionGroup tag="ul" name="tile" class="tile-list">
      <li v-if="!visibleTasks.length" key="empty" class="tile-empty">今天没有待办，休息一下。</li>
      <li
        v-for="task in visibleTasks"
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
    </TransitionGroup>

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
  user-select: none;
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
  transition: background-color 0.15s ease, border-color 0.15s ease, color 0.15s ease;
}

.tile-item.done .tile-check {
  background: var(--green);
  border-color: var(--green);
  color: #fff;
}

.tile-item.done .tile-check svg {
  animation: tile-pop 0.3s ease;
}

@keyframes tile-pop {
  0% {
    transform: scale(0.3);
  }
  60% {
    transform: scale(1.3);
  }
  100% {
    transform: scale(1);
  }
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
  transition: color 0.2s ease;
}

/* 列表增删动画：淡出条目并收起高度，剩余条目平滑上移 */
.tile-move,
.tile-enter-active,
.tile-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease, max-height 0.3s ease, padding 0.3s ease;
}

.tile-enter-from,
.tile-leave-to {
  opacity: 0;
  transform: translateX(14px);
}

.tile-leave-active {
  overflow: hidden;
  max-height: 120px;
}

.tile-leave-to {
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
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
