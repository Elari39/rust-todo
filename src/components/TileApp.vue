<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";
import { Check, Pin, Plus, X } from "lucide-vue-next";
import { useClock } from "../composables/useClock";
import { useTasks } from "../composables/useTasks";
import { t } from "../i18n";
import {
  defaultDue,
  dueLabel,
  fromInputValue,
  isOverdue,
  toInputValue,
  toStamp,
} from "../utils/datetime";

const win = getCurrentWindow();
const { todayTasks, tasks, error, create, complete, reopen, refresh } = useTasks();
const clock = useClock();

const draft = ref("");
const submitting = ref(false);
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
  win
    .setPosition(
      new PhysicalPosition(
        drag.originX + Math.round(event.screenX * scale - drag.startX),
        drag.originY + Math.round(event.screenY * scale - drag.startY),
      ),
    )
    .catch(() => {
      // 窗口销毁瞬间 setPosition 可能被拒，拖动路径上无需打扰用户
    });
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
// 已观察过的任务状态：只对观察到的迁移计时，
// 磁贴启动时首次拉到的历史已完成任务直接视为已知，不进入淡出
const knownStatuses = new Map<string, string>();

// todayTasks 不含已完成任务；淡出期间把它们追加到列表尾部短暂展示
const visibleTasks = computed(() => {
  const fading = tasks.value.filter(
    (task) =>
      task.status === "completed" &&
      fadeTimers.has(task.id) &&
      !hiddenIds.value.has(task.id),
  );
  return [...todayTasks.value, ...fading];
});

watch(tasks, (list) => {
  const statuses = new Map(list.map((task) => [task.id, task.status]));
  for (const task of list) {
    const prev = knownStatuses.get(task.id);
    if (
      task.status === "completed" &&
      prev !== undefined &&
      prev !== "completed" &&
      !fadeTimers.has(task.id)
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
  for (const [id, timer] of fadeTimers) {
    if (statuses.get(id) !== "completed") {
      window.clearTimeout(timer);
      fadeTimers.delete(id);
      hiddenIds.value.delete(id);
    }
  }
  for (const id of [...hiddenIds.value]) {
    // 已完成且淡出已结束的条目不再参与渲染，顺手清理避免集合无限增长
    if (!statuses.has(id) || (!fadeTimers.has(id) && statuses.get(id) === "completed")) {
      hiddenIds.value.delete(id);
    }
  }
  knownStatuses.clear();
  for (const [id, status] of statuses) knownStatuses.set(id, status);
});

async function add() {
  const submittedDraft = draft.value;
  const title = submittedDraft.trim();
  if (!title || submitting.value) return;
  submitting.value = true;
  try {
    await create({
      title,
      kind: "quick",
      // 带上开始时间：否则「截止明天」的任务不满足今日列表的任何过滤条件，
      // 创建后立即从磁贴消失
      startAt: toStamp(new Date()),
      dueAt: fromInputValue(dueAt.value) ?? defaultDue(),
    });
    if (draft.value === submittedDraft) draft.value = "";
  } catch {
    // 失败已在错误横幅展示；保留输入让用户直接重试
  } finally {
    submitting.value = false;
  }
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
      <span class="tile-pin"><Pin :size="13" /></span>
      <b>{{ t("tile.title") }}</b>
      <span class="tile-count">{{ openCount }}</span>
      <button class="tile-close" type="button" :title="t('tile.close')" @click="win.close()">
        <X :size="14" />
      </button>
    </header>

    <p v-if="error" class="tile-error">{{ error }}</p>

    <p class="tile-section">{{ t("tile.section") }}</p>

    <TransitionGroup tag="ul" name="tile" class="tile-list">
      <li v-if="!visibleTasks.length" key="empty" class="tile-empty">{{ t("tile.empty") }}</li>
      <li
        v-for="task in visibleTasks"
        :key="task.id"
        class="tile-item"
        :class="{ done: task.status === 'completed' }"
      >
        <button
          class="tile-check"
          type="button"
          :title="t('tile.check')"
          @click="toggle(task.id, task.status === 'completed')"
        >
          <Check :size="12" />
        </button>
        <div class="tile-body">
          <span class="tile-title">{{ task.title }}</span>
          <span
            class="tile-due"
            :class="{ late: isOverdue(task, clock) }"
          >{{ dueLabel(task.dueAt, clock) }}</span>
        </div>
      </li>
    </TransitionGroup>

    <form class="tile-add" :aria-busy="submitting" @submit.prevent="add">
      <Plus :size="14" />
      <input v-model="draft" type="text" :placeholder="t('tile.addPlaceholder')" />
    </form>
  </div>
</template>

<style scoped>
.tile {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #ffffff;
  border-radius: 18px;
  overflow: hidden;
  box-shadow: 0 12px 32px rgba(46, 74, 134, 0.25);
  user-select: none;
}

.tile-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 14px;
  border-bottom: 1px solid #eef1f9;
  user-select: none;
}

.tile-pin {
  width: 22px;
  height: 22px;
  border-radius: 7px;
  background: var(--blue-soft);
  color: var(--blue);
  display: grid;
  place-items: center;
  flex: 0 0 auto;
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
  background: #fde8e8;
  color: #991b1b;
  font-size: 12px;
}

.tile-section {
  margin: 10px 14px 0;
  color: var(--muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.tile-list {
  list-style: none;
  margin: 0;
  padding: 8px 12px;
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
  background: #f6f8fd;
  border: 1px solid #eef1f9;
  border-radius: 12px;
  padding: 10px;
}

.tile-item.done {
  opacity: 0.75;
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
  border: 2px solid #c2cee6;
  border-radius: 50%;
  background: #fff;
  color: transparent;
  display: grid;
  place-items: center;
  padding: 0;
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
  border: 1px solid #eef1f9;
  background: #f6f8fd;
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
